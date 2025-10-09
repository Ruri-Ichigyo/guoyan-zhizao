//! # Query Service
//!
//! 果蔬检测系统的查询服务，负责处理检测结果的查询和实时数据推送。
//! 该服务提供REST API接口用于查询检测结果，以及WebSocket接口用于实时数据推送。
//!
//! ## 功能特性
//!
//! - 检测结果查询 API
//! - 检测结果列表分页查询  
//! - WebSocket实时数据推送
//! - 健康检查接口
//!
//! ## API 接口
//!
//! - `GET /api/v1/results/:job_id` - 获取指定任务的检测结果
//! - `GET /api/v1/results` - 获取检测结果列表（支持分页）
//! - `GET /api/v1/ws/live` - WebSocket实时数据推送
//! - `GET /health` - 健康检查

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

mod routes;
mod ws;

/// 应用程序共享状态的类型别名
///
/// 使用Arc包装AppState以便在多个异步任务间安全共享状态
pub type SharedState = Arc<AppState>;

/// 应用程序状态结构
///
/// 包含用于WebSocket广播的发送器
pub struct AppState {
    /// 广播频道的发送器，用于向所有连接的WebSocket客户端发送消息
    pub tx: broadcast::Sender<String>,
}

/// 查询服务的主入口函数
///
/// 启动并配置查询服务，包括：
/// - 初始化日志系统
/// - 创建广播频道用于WebSocket通信
/// - 配置路由和中间件
/// - 启动HTTP服务器监听3002端口
///
/// # Errors
///
/// 如果服务器绑定或启动失败，返回错误
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (tx, _rx) = broadcast::channel(1000);
    let app_state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/api/v1/results/:job_id", get(routes::get_result))
        .route("/api/v1/results", get(routes::list_results))
        .route("/api/v1/ws/live", get(ws::websocket_handler))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await?;
    info!("Query service listening on port 3002");

    axum::serve(listener, app).await?;
    Ok(())
}

/// 健康检查接口处理函数
///
/// 返回服务的健康状态，用于负载均衡器和监控系统检查服务可用性
///
/// # Returns
///
/// 返回包含状态信息的JSON响应：`{"status": "ok"}`
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
