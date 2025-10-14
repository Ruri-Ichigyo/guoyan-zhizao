use crate::models::{LLMContent, LLMMessage, LLMRequest, LLMResponse, LLMTool};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

/// LLM Client for SiliconFlow API
/// Implements retry logic with exponential backoff and connection pooling
#[derive(Clone)]
pub struct LLMClient {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
    max_retries: u32,
}

#[derive(Debug)]
pub enum LLMError {
    NetworkError(String),
    ApiError { status: u16, message: String },
    ParseError(String),
    Timeout,
    RateLimitExceeded,
}

impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LLMError::ApiError { status, message } => {
                write!(f, "API error (status {}): {}", status, message)
            }
            LLMError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LLMError::Timeout => write!(f, "Request timeout"),
            LLMError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl std::error::Error for LLMError {}

impl LLMClient {
    /// Create a new LLM client with connection pooling
    pub fn new(api_url: String, api_key: String, model: String) -> Result<Self, LLMError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| LLMError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            api_url,
            api_key,
            model,
            max_retries: 3,
        })
    }

    /// Send a message to the LLM with optional tools (for MCP integration)
    pub async fn send_message(
        &self,
        messages: Vec<LLMMessage>,
        tools: Option<Vec<LLMTool>>,
    ) -> Result<LLMResponse, LLMError> {
        let request = LLMRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 8192,
            tools,
            temperature: Some(0.7),
        };

        self.send_request_with_retry(&request).await
    }

    /// Send a simple text message (convenience method)
    pub async fn send_simple_message(&self, message: &str) -> Result<String, LLMError> {
        let messages = vec![LLMMessage {
            role: "user".to_string(),
            content: message.to_string(),
        }];

        let response = self.send_message(messages, None).await?;
        self.extract_text_from_response(&response)
    }

    /// Send request with exponential backoff retry logic
    async fn send_request_with_retry(&self, request: &LLMRequest) -> Result<LLMResponse, LLMError> {
        let mut retries = 0;
        let mut delay = Duration::from_millis(500);

        loop {
            match self.send_request(request).await {
                Ok(response) => return Ok(response),
                Err(LLMError::RateLimitExceeded) if retries < self.max_retries => {
                    tracing::warn!(
                        "Rate limit exceeded, retrying in {:?} (attempt {}/{})",
                        delay,
                        retries + 1,
                        self.max_retries
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                    retries += 1;
                }
                Err(LLMError::NetworkError(_)) if retries < self.max_retries => {
                    tracing::warn!(
                        "Network error, retrying in {:?} (attempt {}/{})",
                        delay,
                        retries + 1,
                        self.max_retries
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Send HTTP request to LLM API
    async fn send_request(&self, request: &LLMRequest) -> Result<LLMResponse, LLMError> {
        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LLMError::Timeout
                } else {
                    LLMError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();

        // Handle rate limiting
        if status.as_u16() == 429 {
            return Err(LLMError::RateLimitExceeded);
        }

        // Handle error responses
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError::ApiError {
                status: status.as_u16(),
                message: error_body,
            });
        }

        // Parse successful response
        let llm_response: LLMResponse = response
            .json()
            .await
            .map_err(|e| LLMError::ParseError(e.to_string()))?;

        Ok(llm_response)
    }

    /// Extract text content from LLM response
    pub fn extract_text_from_response(&self, response: &LLMResponse) -> Result<String, LLMError> {
        for content in &response.content {
            if content.content_type == "text" {
                if let Some(text) = &content.text {
                    return Ok(text.clone());
                }
            }
        }

        Err(LLMError::ParseError(
            "No text content found in response".to_string(),
        ))
    }

    /// Extract tool calls from LLM response (for MCP integration)
    pub fn extract_tool_calls(&self, response: &LLMResponse) -> Vec<(String, serde_json::Value)> {
        let mut tool_calls = Vec::new();

        for content in &response.content {
            if content.content_type == "tool_use" {
                if let (Some(name), Some(input)) = (&content.name, &content.input) {
                    tool_calls.push((name.clone(), input.clone()));
                }
            }
        }

        tool_calls
    }

    /// Build database query tool definition for LLM
    pub fn build_database_query_tool() -> LLMTool {
        LLMTool {
            name: "query_database".to_string(),
            description: r#"Query the fruit/vegetable detection database to retrieve upload history and detection results.

Available tables (single-user system, no user_id filtering needed):
1. uploads - File upload records
   Columns: id (uuid), filename (text), content_type (text), status (text), created_at (timestamp), updated_at (timestamp)

2. results - Detection results for uploaded images
   Columns: upload_id (uuid), status (text), detections (jsonb), total_fruits (int), grade_a (int), grade_b (int), grade_c (int), avg_confidence (float), created_at (timestamp)

Use this tool when the user asks about:
- Detection history (e.g., "how many detections today?")
- Statistics (e.g., "average confidence score")
- Specific results (e.g., "show latest detection")

IMPORTANT:
- Use ONLY table names 'uploads' and 'results'
- Do NOT filter by user_id (tables don't have this column)
- Do NOT use table names like 'detection_results' or 'upload_results'"#.to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "SQL SELECT query using tables 'uploads' or 'results'. Example: SELECT COUNT(*) FROM results"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Brief explanation of why this query is needed to answer the user's question."
                    }
                },
                "required": ["query", "reason"]
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_client_creation() {
        let client = LLMClient::new(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
            "test-model".to_string(),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_extract_text_from_response() {
        let client = LLMClient::new(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
            "test-model".to_string(),
        )
        .unwrap();

        let response = LLMResponse {
            id: "test-id".to_string(),
            model: "test-model".to_string(),
            content: vec![LLMContent {
                content_type: "text".to_string(),
                text: Some("Hello, world!".to_string()),
                id: None,
                name: None,
                input: None,
            }],
            stop_reason: "end_turn".to_string(),
            usage: crate::models::LLMUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };

        let text = client.extract_text_from_response(&response).unwrap();
        assert_eq!(text, "Hello, world!");
    }

    #[test]
    fn test_extract_tool_calls() {
        let client = LLMClient::new(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
            "test-model".to_string(),
        )
        .unwrap();

        let response = LLMResponse {
            id: "test-id".to_string(),
            model: "test-model".to_string(),
            content: vec![LLMContent {
                content_type: "tool_use".to_string(),
                text: None,
                id: Some("tool-1".to_string()),
                name: Some("query_database".to_string()),
                input: Some(json!({"query": "SELECT * FROM uploads"})),
            }],
            stop_reason: "tool_use".to_string(),
            usage: crate::models::LLMUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };

        let tool_calls = client.extract_tool_calls(&response);
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].0, "query_database");
    }

    #[test]
    fn test_build_database_query_tool() {
        let tool = LLMClient::build_database_query_tool();
        assert_eq!(tool.name, "query_database");
        assert!(tool.description.contains("detection database"));
    }
}
