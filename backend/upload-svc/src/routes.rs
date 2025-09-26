use axum::{
    extract::{Multipart, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::Value;
use tracing::{info, error};
use uuid::Uuid;

use crate::{UploadResponse, PresignRequest, kafka, minio};

pub async fn handle_upload(mut multipart: Multipart) -> Result<ResponseJson<UploadResponse>, (StatusCode, ResponseJson<Value>)> {
    let upload_id = Uuid::new_v4().to_string();

    while let Some(field) = multipart.next_field().await.map_err(|_| {
        (StatusCode::BAD_REQUEST, ResponseJson(serde_json::json!({"error": "Invalid multipart data"})))
    })? {
        if let Some(name) = field.name() {
            if name == "file" {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| {
                    (StatusCode::BAD_REQUEST, ResponseJson(serde_json::json!({"error": "Failed to read file data"})))
                })?;

                // Upload to MinIO
                let object_key = format!("uploads/{}/{}", upload_id, filename);
                match minio::upload_file(&object_key, data.to_vec()).await {
                    Ok(_) => {
                        info!("File uploaded successfully: {}", object_key);

                        // Send Kafka message
                        if let Err(e) = kafka::send_image_stored_event(&upload_id, &object_key).await {
                            error!("Failed to send Kafka message: {}", e);
                        }

                        return Ok(ResponseJson(UploadResponse {
                            upload_id,
                            presigned_url: None,
                            status: "uploaded".to_string(),
                        }));
                    }
                    Err(e) => {
                        error!("Failed to upload file: {}", e);
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(serde_json::json!({"error": "Upload failed"}))));
                    }
                }
            }
        }
    }

    Err((StatusCode::BAD_REQUEST, ResponseJson(serde_json::json!({"error": "No file provided"}))))
}

pub async fn handle_presign(Json(request): Json<PresignRequest>) -> Result<ResponseJson<UploadResponse>, (StatusCode, ResponseJson<Value>)> {
    let upload_id = Uuid::new_v4().to_string();
    let object_key = format!("uploads/{}/{}", upload_id, request.filename);

    match minio::generate_presigned_url(&object_key, &request.content_type).await {
        Ok(presigned_url) => {
            info!("Generated presigned URL for: {}", object_key);

            Ok(ResponseJson(UploadResponse {
                upload_id,
                presigned_url: Some(presigned_url),
                status: "presigned".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to generate presigned URL: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(serde_json::json!({"error": "Failed to generate presigned URL"}))))
        }
    }
}