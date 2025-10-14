use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// User Module Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: Option<String>,
    pub role: String, // 'admin', 'user', 'operator'
    pub status: String, // 'active', 'inactive', 'suspended'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Upload & Detection Module Models
// ============================================================================

// Custom deserializer for DateTime<Utc> that accepts naive datetime strings
fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // Try parsing with timezone first
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // Fall back to naive datetime and assume UTC
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
        .map(|naive| naive.and_utc())
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRecord {
    pub class_name: String,
    pub confidence: f32,
    pub bbox: Vec<f32>,
    pub quality_grade: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInferenceResponse {
    pub upload_id: Uuid,
    pub status: String,
    pub results: Vec<DetectionRecord>,
    pub total_fruits: usize,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponsePayload {
    pub upload_id: Uuid,
    pub filename: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub total_fruits: i32,
    pub grade_a: i32,
    pub grade_b: i32,
    pub grade_c: i32,
    pub avg_confidence: f32,
    pub detections: Vec<DetectionRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationPayload {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub pages: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResultSummary {
    pub upload_id: Uuid,
    pub filename: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub total_fruits: Option<i32>,
    pub grade_a: Option<i32>,
    pub grade_b: Option<i32>,
    pub grade_c: Option<i32>,
    pub avg_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResults {
    pub data: Vec<ResultSummary>,
    pub pagination: PaginationPayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatisticsResponse {
    pub totals: HashMap<String, i64>,
    pub averages: HashMap<String, f32>,
}

#[derive(Debug, FromRow)]
pub struct UploadRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub filename: String,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct ResultRow {
    pub upload_id: Uuid,
    pub status: String,
    pub detections: serde_json::Value,
    pub total_fruits: i32,
    pub grade_a: i32,
    pub grade_b: i32,
    pub grade_c: i32,
    pub avg_confidence: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct SummaryRow {
    pub id: Uuid,
    pub filename: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub total_fruits: Option<i32>,
    pub grade_a: Option<i32>,
    pub grade_b: Option<i32>,
    pub grade_c: Option<i32>,
    pub avg_confidence: Option<f32>,
}

#[derive(Debug, FromRow)]
pub struct DetailedRow {
    pub id: Uuid,
    pub filename: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub result_status: Option<String>,
    pub detections: Option<serde_json::Value>,
    pub total_fruits: Option<i32>,
    pub grade_a: Option<i32>,
    pub grade_b: Option<i32>,
    pub grade_c: Option<i32>,
    pub avg_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LiveDetection {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub snapshot: Vec<u8>,
    pub labels: Vec<String>,
    pub confidence_scores: Vec<f32>,
    pub detected_classes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDetectionSummary {
    pub id: Uuid,
    pub labels: Vec<String>,
    pub confidence_scores: Vec<f32>,
    pub detected_classes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLiveDetectionRequest {
    pub snapshot: String, // base64 encoded image
    pub labels: Vec<String>,
    pub confidence_scores: Vec<f32>,
    pub detected_classes: Vec<String>,
}

// ============================================================================
// Chat Module Models (T010)
// ============================================================================

// Database models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub status: String, // 'active', 'inactive', 'archived'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String, // 'user', 'assistant', 'system'
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// API Request/Response models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryResponse {
    pub messages: Vec<ChatMessage>,
    pub session: ChatSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<ChatSession>,
    pub total: i64,
}

// LLM API models (SiliconFlow API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<LLMMessage>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<LLMTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMContent {
    #[serde(rename = "type")]
    pub content_type: String, // "text" or "tool_use"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<LLMContent>,
    pub stop_reason: String,
    pub usage: LLMUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// MCP (Model Context Protocol) models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPQuery {
    pub query: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPQueryResult {
    pub rows: Vec<serde_json::Value>,
    pub row_count: usize,
}
