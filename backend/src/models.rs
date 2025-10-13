use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
