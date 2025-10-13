mod ai_client;
mod cache;
mod config;
mod db;
mod errors;
mod models;
mod routes;

use axum::{
    routing::{get, post},
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
        health::health,
        results::{get_result, list_results},
        statistics::statistics,
        uploads::upload_file,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub redis: redis::Client,
    pub ai_client: AiClient,
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

    let state = AppState {
        pool,
        redis,
        ai_client,
        cache_ttl: config.cache_ttl_seconds,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/uploads", post(upload_file))
        .route("/api/v1/results", get(list_results))
        .route("/api/v1/results/:result_id", get(get_result))
        .route("/api/v1/statistics", get(statistics))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    info!("Backend listening on {}", config.bind_address);
    axum::serve(listener, app).await?;

    Ok(())
}
