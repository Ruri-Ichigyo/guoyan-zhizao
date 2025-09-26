use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SortCommand {
    pub job_id: String,
    pub action: String, // "start", "stop", "pause", "resume"
    pub target_grade: Option<String>, // "A", "B", "C"
    pub conveyor_speed: Option<u32>, // 0-100
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortStatus {
    pub job_id: String,
    pub status: String, // "idle", "running", "paused", "error"
    pub conveyor_speed: u32,
    pub sorted_count: u32,
    pub error_count: u32,
    pub uptime: u64, // seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortResponse {
    pub success: bool,
    pub message: String,
    pub status: Option<SortStatus>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/sort/command", post(handle_sort_command))
        .route("/api/sort/status", get(get_sort_status))
        .route("/api/sort/config", put(update_sort_config))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await?;
    info!("Sort service listening on port 3003");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({"status": "ok"}))
}

async fn handle_sort_command(
    Json(command): Json<SortCommand>,
) -> Result<ResponseJson<SortResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    info!("Received sort command: {:?}", command);

    // TODO: Implement actual sorting machine control
    match command.action.as_str() {
        "start" => {
            info!("Starting sorting process for job: {}", command.job_id);
            Ok(ResponseJson(SortResponse {
                success: true,
                message: "Sorting started successfully".to_string(),
                status: Some(SortStatus {
                    job_id: command.job_id,
                    status: "running".to_string(),
                    conveyor_speed: command.conveyor_speed.unwrap_or(50),
                    sorted_count: 0,
                    error_count: 0,
                    uptime: 0,
                }),
            }))
        }
        "stop" => {
            info!("Stopping sorting process for job: {}", command.job_id);
            Ok(ResponseJson(SortResponse {
                success: true,
                message: "Sorting stopped successfully".to_string(),
                status: Some(SortStatus {
                    job_id: command.job_id,
                    status: "idle".to_string(),
                    conveyor_speed: 0,
                    sorted_count: 0,
                    error_count: 0,
                    uptime: 0,
                }),
            }))
        }
        "pause" => {
            info!("Pausing sorting process for job: {}", command.job_id);
            Ok(ResponseJson(SortResponse {
                success: true,
                message: "Sorting paused successfully".to_string(),
                status: None,
            }))
        }
        "resume" => {
            info!("Resuming sorting process for job: {}", command.job_id);
            Ok(ResponseJson(SortResponse {
                success: true,
                message: "Sorting resumed successfully".to_string(),
                status: None,
            }))
        }
        _ => {
            error!("Unknown sort command: {}", command.action);
            Err((
                StatusCode::BAD_REQUEST,
                ResponseJson(serde_json::json!({"error": "Unknown command"})),
            ))
        }
    }
}

async fn get_sort_status() -> ResponseJson<SortStatus> {
    // TODO: Get actual status from sorting machine
    ResponseJson(SortStatus {
        job_id: "current-job".to_string(),
        status: "idle".to_string(),
        conveyor_speed: 0,
        sorted_count: 0,
        error_count: 0,
        uptime: 0,
    })
}

async fn update_sort_config(
    Json(config): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    info!("Updating sort configuration: {:?}", config);

    // TODO: Validate and apply configuration
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "Configuration updated successfully"
    })))
}