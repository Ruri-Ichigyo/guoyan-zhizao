//! # Routes Module
//!
//! 上传服务的路由处理模块，定义了文件上传相关的API接口。
//! 支持直接上传和预签名URL上传两种方式，并集成MinIO对象存储和Kafka消息队列。

use axum::{
    extract::{Multipart, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::Value;
use tracing::{info, error};
use uuid::Uuid;

use crate::{UploadResponse, PresignRequest, kafka, minio};

/// 处理直接文件上传的函数
/// 
/// 接收multipart/form-data格式的文件上传请求，将文件保存到MinIO并发送Kafka通知。
/// 
/// # 处理流程
/// 
/// 1. 解析multipart请求，提取文件数据
/// 2. 生成唯一的上传ID和对象键
/// 3. 将文件上传到MinIO对象存储
/// 4. 发送文件存储事件到Kafka
/// 
/// # Arguments
/// 
/// * `multipart` - 包含文件数据的multipart请求
/// 
/// # Returns
/// 
/// 返回上传结果，包含上传ID和状态，或者错误信息
#[axum::debug_handler]
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

/// 处理预签名URL生成的函数
/// 
/// 生成一个预签名的MinIO上传URL，允许客户端直接上传文件到对象存储。
/// 这种方式可以减少服务器的负载，提高上传效率。
/// 
/// # Arguments
/// 
/// * `request` - 预签名请求参数，包含文件名和内容类型
/// 
/// # Returns
/// 
/// 返回预签名URL和上传ID，或者错误信息
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