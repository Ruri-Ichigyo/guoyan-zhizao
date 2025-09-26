use axum::{
    extract::Multipart,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use uuid::Uuid;

mod routes;
mod kafka;
mod minio;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub upload_id: String,
    pub presigned_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PresignRequest {
    pub filename: String,
    pub content_type: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/upload", post(routes::handle_upload))
        .route("/api/presign", post(routes::handle_presign))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Upload service listening on port 3001");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}