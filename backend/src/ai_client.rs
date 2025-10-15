use crate::{errors::AppError, models::AiInferenceResponse};
use reqwest::multipart::{Form, Part};
use uuid::Uuid;

#[derive(Clone)]
pub struct AiClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl AiClient {
    pub fn new(base_url: String, http_client: reqwest::Client) -> Self {
        Self {
            base_url,
            http_client,
        }
    }

    pub async fn infer(
        &self,
        upload_id: Uuid,
        filename: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<AiInferenceResponse, AppError> {
        let mut file_part = Part::bytes(bytes).file_name(filename.to_string());
        if let Some(ct) = content_type {
            file_part = file_part
                .mime_str(ct)
                .map_err(|err| AppError::Unexpected(err.to_string()))?;
        }

        let form = Form::new()
            .text("upload_id", upload_id.to_string())
            .part("file", file_part);

        let response = self
            .http_client
            .post(format!("{}/infer", self.base_url.trim_end_matches('/')))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Unexpected(format!(
                "AI service error ({status}): {body}"
            )));
        }

        let payload = response.json::<AiInferenceResponse>().await?;
        Ok(payload)
    }
}
