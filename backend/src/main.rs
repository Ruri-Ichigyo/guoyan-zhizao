mod ai_client;
mod cache;
mod config;
mod db;
mod errors;
mod models;
mod routes;
mod services;

use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::{
    ai_client::AiClient,
    cache::init_client,
    config::AppConfig,
    db::init_pool,
    routes::{
        chat::{
            create_session, delete_session, get_history, list_sessions, poll_new_messages,
            send_message,
        },
        detections::{create_detection, get_detection, get_detection_snapshot, list_detections},
        health::health,
        records::{delete_record, get_record, get_record_image, list_records},
        results::{get_result, list_results},
        statistics::statistics,
        uploads::upload_file,
    },
    services::{LLMClient, MCPClient},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub redis: redis::Client,
    pub ai_client: AiClient,
    pub llm_client: LLMClient,
    pub mcp_client: MCPClient,
    pub cache_ttl: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .init();

    let config = AppConfig::from_env()?;
    let pool = init_pool(&config.database_url).await?;
    let redis = init_client(&config.redis_url)?;
    let http_client = reqwest::Client::builder().build()?;
    let ai_client = AiClient::new(config.ai_service_url.clone(), http_client);

    // Initialize LLM client
    let llm_api_url = std::env::var("LLM_API_URL")
        .unwrap_or_else(|_| "https://api.siliconflow.cn/v1/messages".to_string());
    let llm_api_key =
        std::env::var("LLM_API_KEY").expect("LLM_API_KEY must be set in environment");
    let llm_model = std::env::var("LLM_MODEL")
        .unwrap_or_else(|_| "tencent/Hunyuan-MT-7B".to_string());
    let llm_client = LLMClient::new(llm_api_url, llm_api_key, llm_model)?;

    // Initialize MCP client with direct database access
    let mcp_allowed_tables = vec!["uploads".to_string(), "results".to_string()];
    let mcp_max_rows = 1000;
    let mcp_client = MCPClient::new(pool.clone(), mcp_allowed_tables, mcp_max_rows);

    let state = AppState {
        pool,
        redis,
        ai_client,
        llm_client,
        mcp_client,
        cache_ttl: config.cache_ttl_seconds,
    };

    let app = Router::new()
        .route("/health", get(health))
        // Upload endpoint
        .route("/api/v1/uploads", post(upload_file))
        // Unified records endpoints (NEW)
        .route("/api/v1/records", get(list_records))
        .route("/api/v1/records/:id", get(get_record).delete(delete_record))
        .route("/api/v1/records/:id/image", get(get_record_image))
        // Legacy endpoints (for backward compatibility)
        .route("/api/v1/results", get(list_results))
        .route("/api/v1/results/:result_id", get(get_result))
        .route("/api/v1/statistics", get(statistics))
        .route("/api/v1/detections", post(create_detection))
        .route("/api/v1/detections", get(list_detections))
        .route("/api/v1/detections/:id", get(get_detection))
        .route("/api/v1/detections/:id/snapshot", get(get_detection_snapshot))
        // Chat endpoints (NEW)
        .route("/api/v1/chat/sessions", post(create_session).get(list_sessions))
        .route("/api/v1/chat/sessions/:id", delete(delete_session))
        .route("/api/v1/chat/message", post(send_message))
        .route("/api/v1/chat/history/:session_id", get(get_history))
        .route("/api/v1/chat/messages/new", get(poll_new_messages))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    info!("Backend listening on {}", config.bind_address);
    axum::serve(listener, app).await?;

    Ok(())
}
