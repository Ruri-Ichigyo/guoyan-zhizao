# MCP Server Contract

**Service**: Model Context Protocol Database Query Server
**Version**: 0.1.0
**Protocol**: JSON-RPC 2.0 over HTTP

## Overview

The MCP server provides read-only database access to the LLM for intelligent query generation. It implements the Model Context Protocol specification, exposing database schema and query execution capabilities.

---

## Configuration

### Environment Variables

```bash
# PostgreSQL Connection
DATABASE_URL=postgresql://mcp_readonly:password@localhost:5432/gyz

# Security Settings
ALLOWED_TABLES=uploads,results
MAX_ROWS_PER_QUERY=1000
QUERY_TIMEOUT_SECONDS=5

# Server Settings
MCP_HOST=0.0.0.0
MCP_PORT=3000
```

### Docker Deployment

```dockerfile
FROM node:20-alpine

WORKDIR /app

# Install MCP server
RUN npm install -g @modelcontextprotocol/server-postgres

# Copy configuration
COPY mcp-config.json /app/config.json

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3000/health', (r) => { process.exit(r.statusCode === 200 ? 0 : 1); })"

# Start server
CMD ["npx", "-y", "@modelcontextprotocol/server-postgres"]
```

---

## JSON-RPC Endpoints

All endpoints use POST method with JSON-RPC 2.0 format:

```
POST http://localhost:3000/rpc
Content-Type: application/json
```

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": "unique-request-id",
  "method": "method_name",
  "params": {
    "param1": "value1"
  }
}
```

### Response Format (Success)

```json
{
  "jsonrpc": "2.0",
  "id": "unique-request-id",
  "result": {
    // Method-specific result
  }
}
```

### Response Format (Error)

```json
{
  "jsonrpc": "2.0",
  "id": "unique-request-id",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "details": "Additional error information"
    }
  }
}
```

---

## Available Methods

### 1. list_tables

**Description**: Returns list of accessible tables (filtered by whitelist).

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "list_tables",
  "params": {}
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "result": {
    "tables": [
      {
        "name": "uploads",
        "schema": "public",
        "row_count_estimate": 1523
      },
      {
        "name": "results",
        "schema": "public",
        "row_count_estimate": 1498
      }
    ]
  }
}
```

---

### 2. describe_table

**Description**: Returns detailed schema for a specific table.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "describe_table",
  "params": {
    "table_name": "results"
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "2",
  "result": {
    "table_name": "results",
    "schema": "public",
    "columns": [
      {
        "name": "upload_id",
        "type": "uuid",
        "nullable": false,
        "is_primary_key": true,
        "default": null
      },
      {
        "name": "status",
        "type": "text",
        "nullable": false,
        "is_primary_key": false,
        "default": null
      },
      {
        "name": "total_fruits",
        "type": "integer",
        "nullable": true,
        "is_primary_key": false,
        "default": null
      },
      {
        "name": "grade_a",
        "type": "integer",
        "nullable": true,
        "is_primary_key": false,
        "default": "0"
      },
      {
        "name": "grade_b",
        "type": "integer",
        "nullable": true,
        "is_primary_key": false,
        "default": "0"
      },
      {
        "name": "grade_c",
        "type": "integer",
        "nullable": true,
        "is_primary_key": false,
        "default": "0"
      },
      {
        "name": "avg_confidence",
        "type": "real",
        "nullable": true,
        "is_primary_key": false,
        "default": null
      },
      {
        "name": "created_at",
        "type": "timestamp with time zone",
        "nullable": false,
        "is_primary_key": false,
        "default": "now()"
      }
    ],
    "indexes": [
      {
        "name": "results_pkey",
        "columns": ["upload_id"],
        "is_unique": true
      }
    ],
    "foreign_keys": [
      {
        "name": "results_upload_id_fkey",
        "columns": ["upload_id"],
        "references_table": "uploads",
        "references_columns": ["id"]
      }
    ]
  }
}
```

---

### 3. query

**Description**: Executes a read-only SQL query with parameterization.

**Security Features**:
- Only SELECT statements allowed
- Parameterized queries prevent SQL injection
- Row limit enforced (max 1000 rows)
- Query timeout enforced (5 seconds)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "3",
  "method": "query",
  "params": {
    "sql": "SELECT COUNT(*) as count FROM results WHERE grade_a > 0 AND created_at > NOW() - INTERVAL '7 days'",
    "parameters": []
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "3",
  "result": {
    "columns": ["count"],
    "rows": [
      [42]
    ],
    "row_count": 1,
    "execution_time_ms": 45
  }
}
```

**Request with Parameters** (Recommended):
```json
{
  "jsonrpc": "2.0",
  "id": "4",
  "method": "query",
  "params": {
    "sql": "SELECT upload_id, total_fruits, grade_a, grade_b, grade_c FROM results WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2",
    "parameters": ["2025-10-07T00:00:00Z", 10]
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "4",
  "result": {
    "columns": ["upload_id", "total_fruits", "grade_a", "grade_b", "grade_c"],
    "rows": [
      ["550e8400-e29b-41d4-a716-446655440000", 150, 80, 50, 20],
      ["660e8400-e29b-41d4-a716-446655440001", 120, 60, 40, 20]
    ],
    "row_count": 2,
    "execution_time_ms": 23
  }
}
```

**Error Response (Forbidden Query)**:
```json
{
  "jsonrpc": "2.0",
  "id": "5",
  "error": {
    "code": -32001,
    "message": "Query not allowed",
    "data": {
      "reason": "Only SELECT statements are permitted",
      "attempted_query": "DELETE FROM results WHERE id = '...'"
    }
  }
}
```

**Error Response (Row Limit Exceeded)**:
```json
{
  "jsonrpc": "2.0",
  "id": "6",
  "error": {
    "code": -32002,
    "message": "Row limit exceeded",
    "data": {
      "max_rows": 1000,
      "query_would_return": 1523,
      "suggestion": "Add LIMIT clause to query"
    }
  }
}
```

---

### 4. list_columns

**Description**: Returns column details for a specific table.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "7",
  "method": "list_columns",
  "params": {
    "table_name": "uploads"
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "7",
  "result": {
    "table_name": "uploads",
    "columns": [
      {
        "name": "id",
        "type": "uuid",
        "nullable": false,
        "description": "Primary key"
      },
      {
        "name": "filename",
        "type": "text",
        "nullable": false,
        "description": "Original file name"
      },
      {
        "name": "content_type",
        "type": "text",
        "nullable": false,
        "description": "MIME type"
      },
      {
        "name": "status",
        "type": "text",
        "nullable": false,
        "description": "Processing status: processing, completed, failed"
      },
      {
        "name": "created_at",
        "type": "timestamp with time zone",
        "nullable": false,
        "description": "Upload timestamp"
      }
    ]
  }
}
```

---

## Error Codes

Standard JSON-RPC 2.0 error codes plus custom codes:

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | JSON-RPC format error |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Server error |
| -32001 | Query not allowed | Non-SELECT query attempted |
| -32002 | Row limit exceeded | Query would return > MAX_ROWS |
| -32003 | Query timeout | Query exceeded timeout limit |
| -32004 | Table not allowed | Table not in whitelist |
| -32005 | Syntax error | Invalid SQL syntax |

---

## Integration with Rust Backend

### HTTP Client Example

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

pub struct MCPClient {
    client: Client,
    base_url: String,
}

impl MCPClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create MCP client");

        Self { client, base_url }
    }

    pub async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: "query".to_string(),
            params: serde_json::json!({
                "sql": sql,
                "parameters": params
            }),
        };

        let response = self.client
            .post(&format!("{}/rpc", self.base_url))
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(format!("MCP Error {}: {}", error.code, error.message).into());
        }

        let result: QueryResult = serde_json::from_value(rpc_response.result.unwrap())?;
        Ok(result)
    }

    pub async fn describe_table(&self, table_name: &str) -> Result<TableSchema, Box<dyn std::error::Error>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: "describe_table".to_string(),
            params: serde_json::json!({
                "table_name": table_name
            }),
        };

        let response = self.client
            .post(&format!("{}/rpc", self.base_url))
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(format!("MCP Error {}: {}", error.code, error.message).into());
        }

        let result: TableSchema = serde_json::from_value(rpc_response.result.unwrap())?;
        Ok(result)
    }
}

#[derive(Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}

#[derive(Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub schema: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub nullable: bool,
}
```

---

## LLM System Prompt for MCP Tools

When sending requests to the LLM, include this in the system prompt:

```markdown
You have access to the following database query tools:

1. **list_tables**: Get available tables
2. **describe_table(table_name)**: Get schema for a table
3. **query(sql, parameters)**: Execute read-only SQL query

Available tables:
- **uploads**: User-uploaded fruit/vegetable images
  - id (UUID), filename (TEXT), status (TEXT), created_at (TIMESTAMPTZ)

- **results**: AI detection results
  - upload_id (UUID), total_fruits (INT), grade_a (INT), grade_b (INT), grade_c (INT),
    avg_confidence (REAL), created_at (TIMESTAMPTZ)

When user asks database questions:
1. Use describe_table to understand schema if needed
2. Generate SQL query with $1, $2 placeholders for parameters
3. Use query tool with SQL and parameters array
4. Synthesize natural language response from results

Always filter by user context: WHERE user_id = $user_id (injected by backend)

Example flow for "上周有多少个A级水果?":
1. query("SELECT COUNT(*) FROM results WHERE grade_a > 0 AND created_at > NOW() - INTERVAL '7 days'", [])
2. Get result: [[42]]
3. Respond: "根据数据库查询,上周您识别了42个A级水果。"
```

---

## Health Check

### Endpoint: GET /health

**Response (Healthy)**:
```json
{
  "status": "healthy",
  "database": "connected",
  "uptime_seconds": 3600,
  "queries_executed": 1523
}
```

**Response (Unhealthy)**:
```json
{
  "status": "unhealthy",
  "database": "disconnected",
  "error": "connection timeout"
}
```

---

## Security Considerations

1. **Read-Only Access**: MCP uses dedicated PostgreSQL role with SELECT-only permissions
2. **Table Whitelist**: Only `uploads` and `results` accessible
3. **Parameterized Queries**: Prevents SQL injection
4. **Row Limits**: Maximum 1000 rows per query
5. **Query Timeout**: 5-second execution limit
6. **No Schema Modification**: CREATE/ALTER/DROP blocked
7. **User Context Injection**: Backend adds user_id filter to all queries

---

## Testing

### Unit Tests

```bash
# Test list_tables
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"list_tables","params":{}}'

# Test query
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"2","method":"query","params":{"sql":"SELECT COUNT(*) FROM results","parameters":[]}}'

# Test forbidden query (should fail)
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"3","method":"query","params":{"sql":"DELETE FROM results","parameters":[]}}'
```

---

**Next**: Create quickstart.md for integration scenarios
