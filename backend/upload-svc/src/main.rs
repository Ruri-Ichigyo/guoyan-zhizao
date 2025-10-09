//! # Upload Service
//!
//! 果蔬检测系统的文件上传服务，负责处理图像文件的上传和存储。
//! 该服务支持直接上传和预签名URL上传两种方式，并集成MinIO对象存储和Kafka消息队列。
//!
//! ## 功能特性
//! 
//! - 直接文件上传到MinIO
//! - 预签名URL生成用于客户端直传
//! - Kafka事件通知集成
//! - 健康检查接口
//!
//! ## API 接口
//!
//! - `POST /api/v1/uploads` - 直接上传文件
//! - `POST /api/v1/uploads/presign` - 生成预签名URL
//! - `GET /health` - 健康检查

use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::info;

mod routes;
mod kafka;
mod minio;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    /// 上传任务的唯一标识符
    pub upload_id: String,
    /// 预签名URL（仅在预签名请求中返回）
    pub presigned_url: Option<String>,
    /// 上传状态："uploaded", "presigned"
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PresignRequest {
    /// 文件名
    pub filename: String,
    /// 文件MIME类型
    pub content_type: String,
}

/// 上传服务的主入口函数
/// 
/// 启动并配置上传服务，包括：
/// - 初始化日志系统
/// - 配置路由和中间件
/// - 启动HTTP服务器监听3001端口
/// 
/// # Errors
/// 
/// 如果服务器绑定或启动失败，返回错误
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/v1/uploads", post(routes::handle_upload))
        .route("/api/v1/uploads/presign", post(routes::handle_presign))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Upload service listening on port 3001");

    axum::serve(listener, app).await?;
    Ok(())
}

/// 健康检查接口处理函数
/// 
/// 返回服务的健康状态，用于负载均衡器和监控系统检查服务可用性
/// 
/// # Returns
/// 
/// 返回HTTP 200状态码和包含状态信息的JSON响应：`{"status": "ok"}`
async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}
