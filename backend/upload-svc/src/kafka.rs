use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::json;
use std::time::Duration;
use tracing::{info, error};

pub async fn send_image_stored_event(upload_id: &str, object_key: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    let record = FutureRecord::to("image-events")
        .key(upload_id)
        .payload(&payload.to_string());

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(delivery) => {
            info!("Message sent successfully: {:?}", delivery);
            Ok(())
        }
        Err((e, _)) => {
            error!("Failed to send message: {}", e);
            Err(Box::new(e))
        }
    }
}