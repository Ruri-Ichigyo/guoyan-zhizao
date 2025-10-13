use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{CreateLiveDetectionRequest, LiveDetectionSummary},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct PaginatedDetections {
    pub data: Vec<LiveDetectionSummary>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub pages: i64,
}

/// Create a new live detection record
pub async fn create_detection(
    State(state): State<AppState>,
    Json(payload): Json<CreateLiveDetectionRequest>,
) -> Result<Json<LiveDetectionSummary>, AppError> {
    // Decode base64 snapshot
    let snapshot_bytes = general_purpose::STANDARD.decode(&payload.snapshot)
        .map_err(|e| AppError::ValidationError(format!("Invalid base64 snapshot: {}", e)))?;

    // Insert into database
    let record = sqlx::query_as::<_, (Uuid, Vec<String>, Vec<f32>, Vec<String>, chrono::DateTime<chrono::Utc>, String)>(
        r#"
        INSERT INTO detections (snapshot, labels, confidence_scores, detected_classes, source)
        VALUES ($1, $2, $3, $4, 'live_camera')
        RETURNING id, labels, confidence_scores, detected_classes, created_at, source
        "#,
    )
    .bind(&snapshot_bytes)
    .bind(&payload.labels)
    .bind(&payload.confidence_scores)
    .bind(&payload.detected_classes)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert detection: {}", e)))?;

    let summary = LiveDetectionSummary {
        id: record.0,
        labels: record.1,
        confidence_scores: record.2,
        detected_classes: record.3,
        created_at: record.4,
        source: record.5,
    };

    Ok(Json(summary))
}

/// List all live detection records with pagination
pub async fn list_detections(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedDetections>, AppError> {
    let offset = (query.page - 1) * query.limit;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM detections")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to count detections: {}", e)))?;

    // Fetch records
    let records = sqlx::query_as::<_, (Uuid, Vec<String>, Vec<f32>, Vec<String>, chrono::DateTime<chrono::Utc>, String)>(
        r#"
        SELECT id, labels, confidence_scores, detected_classes, created_at, source
        FROM detections
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(query.limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to fetch detections: {}", e)))?;

    let data: Vec<LiveDetectionSummary> = records
        .into_iter()
        .map(|r| LiveDetectionSummary {
            id: r.0,
            labels: r.1,
            confidence_scores: r.2,
            detected_classes: r.3,
            created_at: r.4,
            source: r.5,
        })
        .collect();

    let pages = (total + query.limit - 1) / query.limit;

    Ok(Json(PaginatedDetections {
        data,
        pagination: PaginationInfo {
            page: query.page,
            limit: query.limit,
            total,
            pages,
        },
    }))
}

/// Get a specific detection record by ID
pub async fn get_detection(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<LiveDetectionSummary>, AppError> {
    let record = sqlx::query_as::<_, (Uuid, Vec<String>, Vec<f32>, Vec<String>, chrono::DateTime<chrono::Utc>, String)>(
        r#"
        SELECT id, labels, confidence_scores, detected_classes, created_at, source
        FROM detections
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to fetch detection: {}", e)))?
    .ok_or(AppError::NotFound(format!("Detection {} not found", id)))?;

    let summary = LiveDetectionSummary {
        id: record.0,
        labels: record.1,
        confidence_scores: record.2,
        detected_classes: record.3,
        created_at: record.4,
        source: record.5,
    };

    Ok(Json(summary))
}

/// Get detection snapshot image by ID
pub async fn get_detection_snapshot(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<(StatusCode, Vec<u8>), AppError> {
    let snapshot: Vec<u8> = sqlx::query_scalar(
        "SELECT snapshot FROM detections WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to fetch snapshot: {}", e)))?
    .ok_or(AppError::NotFound(format!("Detection {} not found", id)))?;

    Ok((StatusCode::OK, snapshot))
}
