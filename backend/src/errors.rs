use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("resource not found")]
    NotFound,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("cache error: {0}")]
    Cache(#[from] redis::RedisError),

    #[error("http service error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Database(_) | AppError::Cache(_) | AppError::HttpClient(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = ErrorResponse {
            message: self.to_string(),
        };

        (status, Json(payload)).into_response()
    }
}
