use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error};

use crate::SharedState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultResponse {
    pub job_id: String,
    pub status: String,
    pub results: Vec<DetectionResult>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionResult {
    pub class_name: String,
    pub confidence: f32,
    pub bbox: Vec<f32>, // [x1, y1, x2, y2]
    pub quality_grade: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
}

pub async fn get_result(
    Path(job_id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<ResultResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Getting result for job: {}", job_id);

    // TODO: Query from database
    // For now, return mock data
    let mock_result = ResultResponse {
        job_id: job_id.clone(),
        status: "completed".to_string(),
        results: vec![
            DetectionResult {
                class_name: "apple".to_string(),
                confidence: 0.95,
                bbox: vec![10.0, 20.0, 100.0, 150.0],
                quality_grade: "A".to_string(),
            },
            DetectionResult {
                class_name: "orange".to_string(),
                confidence: 0.89,
                bbox: vec![200.0, 50.0, 300.0, 200.0],
                quality_grade: "B".to_string(),
            },
        ],
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(mock_result))
}

pub async fn list_results(
    Query(params): Query<QueryParams>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    info!("Listing results: page={}, limit={}", page, limit);

    // TODO: Query from database with pagination
    // For now, return mock data
    let mock_results = serde_json::json!({
        "data": [
            {
                "job_id": "test-job-1",
                "status": "completed",
                "total_fruits": 15,
                "grade_a": 8,
                "grade_b": 5,
                "grade_c": 2,
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        ],
        "pagination": {
            "page": page,
            "limit": limit,
            "total": 1,
            "pages": 1
        }
    });

    Ok(Json(mock_results))
}