//! # WebSocket Module
//!
//! 查询服务的WebSocket处理模块，实现实时数据推送功能。
//! 支持多客户端连接和广播消息，主要用于实时推送检测结果和系统状态更新。

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use tracing::{info, error};

use crate::SharedState;

/// WebSocket连接处理函数
/// 
/// 处理WebSocket升级请求，将HTTP连接升级为WebSocket连接。
/// 每个新的WebSocket连接都会启动一个独立的处理任务。
/// 
/// # Arguments
/// 
/// * `ws` - WebSocket升级请求
/// * `state` - 应用程序共享状态，包含广播频道
/// 
/// # Returns
/// 
/// 返回HTTP响应，完成WebSocket升级
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

/// WebSocket连接的主处理函数
/// 
/// 处理单个WebSocket连接的生命周期，包括：
/// - 发送欢迎消息
/// - 处理来自客户端的消息
/// - 转发广播消息给客户端
/// - 处理ping/pong心跳检测
/// 
/// # Arguments
/// 
/// * `socket` - WebSocket连接对象
/// * `state` - 应用程序共享状态
async fn websocket_connection(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    info!("New WebSocket connection established");

    // Send initial connection message
    let welcome_msg = json!({
        "type": "connected",
        "message": "Connected to live detection feed",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Err(e) = sender.send(Message::Text(welcome_msg.to_string())).await {
        error!("Failed to send welcome message: {}", e);
        return;
    }

    // Spawn task to handle incoming messages from client
    let state_clone = state.clone();
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!("Received WebSocket message: {}", text);

                    // Handle ping/pong
                    if text == "ping" {
                        // Echo will be handled by broadcast
                        let _ = state_clone.tx.send(json!({
                            "type": "pong",
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }).to_string());
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by client");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle broadcast messages
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg)).await {
                error!("Failed to send broadcast message: {}", e);
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = incoming_task => {
            info!("Incoming message task completed");
        }
        _ = broadcast_task => {
            info!("Broadcast task completed");
        }
    }

    info!("WebSocket connection closed");
}