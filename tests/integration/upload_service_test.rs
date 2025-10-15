use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UploadResponse {
    upload_id: Uuid,
    status: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ResultSummary {
    upload_id: Uuid,
    filename: String,
    status: String,
    total_fruits: i32,
    avg_confidence: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PaginatedResults {
    data: Vec<ResultSummary>,
    pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Pagination {
    page: i64,
    limit: i64,
    total: i64,
    pages: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Statistics {
    totals: std::collections::HashMap<String, i64>,
    averages: std::collections::HashMap<String, f32>,
}

async fn upload_handler() -> Result<Json<UploadResponse>, StatusCode> {
    Ok(Json(UploadResponse {
        upload_id: Uuid::new_v4(),
        status: "completed".to_string(),
    }))
}

async fn results_handler() -> Result<Json<PaginatedResults>, StatusCode> {
    let payload = PaginatedResults {
        data: vec![ResultSummary {
            upload_id: Uuid::new_v4(),
            filename: "demo.jpg".to_string(),
            status: "completed".to_string(),
            total_fruits: 3,
            avg_confidence: 0.92,
        }],
        pagination: Pagination {
            page: 1,
            limit: 10,
            total: 1,
            pages: 1,
        },
    };

    Ok(Json(payload))
}

async fn statistics_handler() -> Result<Json<Statistics>, StatusCode> {
    let mut totals = std::collections::HashMap::new();
    totals.insert("total".into(), 1);
    totals.insert("completed".into(), 1);
    totals.insert("processing".into(), 0);
    totals.insert("failed".into(), 0);

    let mut averages = std::collections::HashMap::new();
    averages.insert("avg_confidence".into(), 0.92);

    Ok(Json(Statistics { totals, averages }))
}

fn build_router() -> Router {
    Router::new()
        .route("/api/v1/uploads", post(upload_handler))
        .route("/api/v1/results", get(results_handler))
        .route("/api/v1/statistics", get(statistics_handler))
}

#[tokio::test]
async fn upload_endpoint_returns_completed_status() {
    let app = build_router();
    let server = axum_test::TestServer::new(app).unwrap();

    let response = server.post("/api/v1/uploads").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body: UploadResponse = response.json();
    assert_eq!(body.status, "completed");
}

#[tokio::test]
async fn results_endpoint_returns_paginated_payload() {
    let app = build_router();
    let server = axum_test::TestServer::new(app).unwrap();

    let response = server.get("/api/v1/results").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body: PaginatedResults = response.json();
    assert_eq!(body.pagination.total, 1);
    assert_eq!(body.data.len(), 1);
    assert_eq!(body.data[0].status, "completed");
}

#[tokio::test]
async fn statistics_endpoint_returns_totals_and_averages() {
    let app = build_router();
    let server = axum_test::TestServer::new(app).unwrap();

    let response = server.get("/api/v1/statistics").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Statistics = response.json();
    assert_eq!(body.totals.get("total"), Some(&1));
    assert_eq!(body.averages.get("avg_confidence"), Some(&0.92));
}
