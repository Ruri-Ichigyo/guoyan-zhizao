use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

mod routes;
mod ws;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (tx, _rx) = broadcast::channel(1000);
    let app_state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/api/result/:job_id", get(routes::get_result))
        .route("/api/results", get(routes::list_results))
        .route("/ws/live", get(ws::websocket_handler))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await?;
    info!("Query service listening on port 3002");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}