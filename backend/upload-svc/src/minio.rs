//! # MinIO Module
//!
//! 上传服务的MinIO对象存储集成模块，实现文件上传和预签名URL生成功能。
//! 使用AWS S3兼容的API与MinIO服务器进行通信，支持文件存储和获取。

use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_config::{BehaviorVersion, Region};
use std::time::Duration;
use tracing::info;

/// 创建并配置MinIO S3客户端
/// 
/// 创建一个配置了MinIO连接参数的AWS S3客户端实例。
/// 使用path-style地址样式和localhost:9000作为默认端点。
/// 
/// # Returns
/// 
/// 成功返回配置好的S3客户端，失败返回错误
/// 
/// # Errors
/// 
/// - AWS配置加载失败
/// - 客户端初始化失败
pub async fn get_client() -> Result<Client, anyhow::Error> {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url("http://localhost:9000")
        .load()
        .await;

    let client = Client::from_conf(
        aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build()
    );

    Ok(client)
}

/// 上传文件到MinIO存储桶
/// 
/// 将文件数据上传到指定的MinIO存储桶中。
/// 使用'fruit-images'作为默认存储桶名称。
/// 
/// # Arguments
/// 
/// * `object_key` - 存储对象的键（路径）
/// * `data` - 要上传的文件二进制数据
/// 
/// # Returns
/// 
/// 成功返回`Ok(())`，失败返回错误
/// 
/// # Errors
/// 
/// - MinIO客户端创建失败
/// - 文件上传操作失败
pub async fn upload_file(object_key: &str, data: Vec<u8>) -> Result<(), anyhow::Error> {
    let client = get_client().await?;

    client
        .put_object()
        .bucket("fruit-images")
        .key(object_key)
        .body(data.into())
        .send()
        .await?;

    info!("Successfully uploaded object: {}", object_key);
    Ok(())
}

/// 生成预签名上传URL
/// 
/// 生成一个允许客户端直接上传文件到MinIO的预签名URL。
/// URL有效期为1小时，超时后将无法使用。
/// 
/// # Arguments
/// 
/// * `object_key` - 存储对象的键（路径）
/// * `content_type` - 文件的MIME类型
/// 
/// # Returns
/// 
/// 成功返回预签名URL字符串，失败返回错误
/// 
/// # Errors
/// 
/// - MinIO客户端创建失败
/// - 预签名配置创建失败
/// - URL生成失败
pub async fn generate_presigned_url(object_key: &str, content_type: &str) -> Result<String, anyhow::Error> {
    let client = get_client().await?;

    let presigning_config = PresigningConfig::expires_in(Duration::from_secs(3600))?;

    let presigned_request = client
        .put_object()
        .bucket("fruit-images")
        .key(object_key)
        .content_type(content_type)
        .presigned(presigning_config)
        .await?;

    Ok(presigned_request.uri().to_string())
}