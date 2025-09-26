use aws_sdk_s3::{Client, Config, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use tracing::{info, error};

pub async fn get_client() -> Result<Client, Box<dyn std::error::Error>> {
    let credentials = Credentials::new(
        "minio",
        "minio123456",
        None,
        None,
        "minio-provider"
    );

    let config = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(credentials)
        .endpoint_url("http://localhost:9000")
        .force_path_style(true)
        .build();

    Ok(Client::from_conf(config))
}

pub async fn upload_file(object_key: &str, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn generate_presigned_url(object_key: &str, content_type: &str) -> Result<String, Box<dyn std::error::Error>> {
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