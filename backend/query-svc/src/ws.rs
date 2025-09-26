use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use tracing::{info, error, warn};

use crate::SharedState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

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