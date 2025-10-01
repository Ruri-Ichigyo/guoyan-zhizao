//! # Kafka Module
//!
//! 上传服务的Kafka集成模块，负责发送文件存储事件通知。
//! 当文件成功上传到MinIO后，会发送事件消息到Kafka主题，以触发后续的处理流程。

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::json;
use std::time::Duration;
use tracing::{info, error};

/// 发送图像存储事件到Kafka
/// 
/// 当图像文件成功上传到MinIO后，调用此函数发送事件通知。
/// 该事件将触发后续的AI检测处理流程。
/// 
/// # Arguments
/// 
/// * `upload_id` - 上传任务的唯一标识符
/// * `object_key` - MinIO中的对象键（文件路径）
/// 
/// # Returns
/// 
/// 成功返回`Ok(())`，失败返回包含错误信息的`Err`
/// 
/// # Errors
/// 
/// - Kafka配置或连接失败
/// - 消息序列化失败
/// - 消息发送超时或失败
pub async fn send_image_stored_event(upload_id: &str, object_key: &str) -> Result<(), anyhow::Error> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("message.timeout.ms", "5000")
        .create()?;

    let payload = json!({
        "event_type": "image.stored",
        "upload_id": upload_id,
        "object_key": object_key,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let payload_string = payload.to_string();
    let record = FutureRecord::to("image-events")
        .key(upload_id)
        .payload(&payload_string);

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(delivery) => {
            info!("Message sent successfully: {:?}", delivery);
            Ok(())
        }
        Err((e, _)) => {
            error!("Failed to send message: {}", e);
            Err(anyhow::anyhow!("Kafka send failed: {}", e))
        }
    }
}