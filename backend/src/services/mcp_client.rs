use crate::models::MCPQueryResult;
use sqlx::{Column, PgPool, Row};
use uuid::Uuid;

/// MCP (Model Context Protocol) Client for database querying
/// Provides read-only access to detection database through direct SQL queries
#[derive(Clone)]
pub struct MCPClient {
    pool: PgPool,
    allowed_tables: Vec<String>,
    max_rows: usize,
}

#[derive(Debug)]
pub enum MCPError {
    DatabaseError(String),
    InvalidQuery(String),
    Unauthorized,
}

impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCPError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MCPError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            MCPError::Unauthorized => write!(f, "Unauthorized access"),
        }
    }
}

impl std::error::Error for MCPError {}

impl MCPClient {
    /// Create a new MCP client with direct database access
    pub fn new(
        pool: PgPool,
        allowed_tables: Vec<String>,
        max_rows: usize,
    ) -> Self {
        Self {
            pool,
            allowed_tables,
            max_rows,
        }
    }

    /// Execute a database query directly
    /// Note: uploads and results tables don't have user_id column (single-user system)
    pub async fn query(
        &self,
        query: &str,
        _params: Vec<serde_json::Value>,
        _user_id: Uuid,
    ) -> Result<MCPQueryResult, MCPError> {
        // Validate query before executing
        self.validate_query(query)?;

        // Don't inject user_id filter - this is a single-user system
        // Tables (uploads, results) don't have user_id column
        let query_to_execute = query.trim_end_matches(';');

        // Add LIMIT clause
        let limited_query = format!("{} LIMIT {}", query_to_execute, self.max_rows);

        // Execute query
        let rows = sqlx::query(&limited_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MCPError::DatabaseError(e.to_string()))?;

        // Convert rows to JSON
        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();

                // Convert each column to JSON
                for (i, column) in row.columns().iter().enumerate() {
                    let name = column.name();

                    // Try to get value as different types
                    let value = if let Ok(v) = row.try_get::<String, _>(i) {
                        serde_json::Value::String(v)
                    } else if let Ok(v) = row.try_get::<i32, _>(i) {
                        serde_json::Value::Number(v.into())
                    } else if let Ok(v) = row.try_get::<i64, _>(i) {
                        serde_json::Value::Number(v.into())
                    } else if let Ok(v) = row.try_get::<f64, _>(i) {
                        serde_json::Number::from_f64(v)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    } else if let Ok(v) = row.try_get::<f32, _>(i) {
                        serde_json::Number::from_f64(v as f64)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    } else if let Ok(v) = row.try_get::<bool, _>(i) {
                        serde_json::Value::Bool(v)
                    } else if let Ok(v) = row.try_get::<serde_json::Value, _>(i) {
                        v
                    } else if let Ok(v) = row.try_get::<Uuid, _>(i) {
                        serde_json::Value::String(v.to_string())
                    } else if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                        serde_json::Value::String(v.to_rfc3339())
                    } else {
                        serde_json::Value::Null
                    };

                    map.insert(name.to_string(), value);
                }

                serde_json::Value::Object(map)
            })
            .collect();

        Ok(MCPQueryResult {
            row_count: json_rows.len(),
            rows: json_rows,
        })
    }

    /// List available tables
    pub async fn list_tables(&self) -> Result<Vec<String>, MCPError> {
        Ok(self.allowed_tables.clone())
    }

    /// Describe table schema
    pub async fn describe_table(&self, table_name: &str) -> Result<serde_json::Value, MCPError> {
        // Check if table is allowed
        if !self.allowed_tables.contains(&table_name.to_string()) {
            return Err(MCPError::InvalidQuery(format!(
                "Table '{}' is not in allowed list",
                table_name
            )));
        }

        let query = format!(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_name = '{}'
             ORDER BY ordinal_position",
            table_name
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MCPError::DatabaseError(e.to_string()))?;

        let columns: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "name": row.get::<String, _>("column_name"),
                    "type": row.get::<String, _>("data_type"),
                    "nullable": row.get::<String, _>("is_nullable") == "YES"
                })
            })
            .collect();

        Ok(serde_json::json!({
            "table": table_name,
            "columns": columns
        }))
    }

    /// Validate SQL query for safety
    fn validate_query(&self, query: &str) -> Result<(), MCPError> {
        let query_upper = query.to_uppercase();

        // Only allow SELECT statements
        if !query_upper.trim().starts_with("SELECT") {
            return Err(MCPError::InvalidQuery(
                "Only SELECT queries are allowed".to_string(),
            ));
        }

        // Block dangerous keywords
        let dangerous_keywords = [
            "DELETE",
            "UPDATE",
            "INSERT",
            "DROP",
            "CREATE",
            "ALTER",
            "TRUNCATE",
            "EXEC",
            "EXECUTE",
        ];

        for keyword in &dangerous_keywords {
            if query_upper.contains(keyword) {
                return Err(MCPError::InvalidQuery(format!(
                    "Query contains forbidden keyword: {}",
                    keyword
                )));
            }
        }

        // Validate table names
        for table in &self.allowed_tables {
            if query_upper.contains(&table.to_uppercase()) {
                return Ok(());
            }
        }

        Err(MCPError::InvalidQuery(
            "Query does not reference any allowed tables".to_string(),
        ))
    }

    /// Inject user_id filter to query for security
    fn inject_user_filter(&self, query: &str, user_id: Uuid) -> Result<String, MCPError> {
        let query_upper = query.to_uppercase();

        // If query already has WHERE clause, append user_id filter
        if query_upper.contains("WHERE") {
            Ok(format!(
                "{} AND user_id = '{}'",
                query.trim_end_matches(';'),
                user_id
            ))
        } else {
            // If no WHERE clause, add one
            Ok(format!(
                "{} WHERE user_id = '{}'",
                query.trim_end_matches(';'),
                user_id
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_query_select() {
        let pool = PgPool::connect_lazy("postgresql://localhost/test").unwrap();
        let client = MCPClient::new(
            pool,
            vec!["uploads".to_string()],
            1000,
        );

        let result = client.validate_query("SELECT * FROM uploads");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_query_delete() {
        let pool = PgPool::connect_lazy("postgresql://localhost/test").unwrap();
        let client = MCPClient::new(
            pool,
            vec!["uploads".to_string()],
            1000,
        );

        let result = client.validate_query("DELETE FROM uploads");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_query_unknown_table() {
        let pool = PgPool::connect_lazy("postgresql://localhost/test").unwrap();
        let client = MCPClient::new(
            pool,
            vec!["uploads".to_string()],
            1000,
        );

        let result = client.validate_query("SELECT * FROM users");
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_user_filter_no_where() {
        let pool = PgPool::connect_lazy("postgresql://localhost/test").unwrap();
        let client = MCPClient::new(
            pool,
            vec!["uploads".to_string()],
            1000,
        );

        let user_id = Uuid::new_v4();
        let result = client
            .inject_user_filter("SELECT * FROM uploads", user_id)
            .unwrap();
        assert!(result.contains("WHERE user_id"));
        assert!(result.contains(&user_id.to_string()));
    }

    #[test]
    fn test_inject_user_filter_with_where() {
        let pool = PgPool::connect_lazy("postgresql://localhost/test").unwrap();
        let client = MCPClient::new(
            pool,
            vec!["uploads".to_string()],
            1000,
        );

        let user_id = Uuid::new_v4();
        let result = client
            .inject_user_filter("SELECT * FROM uploads WHERE status = 'completed'", user_id)
            .unwrap();
        assert!(result.contains("AND user_id"));
        assert!(result.contains(&user_id.to_string()));
    }
}
