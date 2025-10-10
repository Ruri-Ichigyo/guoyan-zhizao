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

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("cache error: {0}")]
    Cache(#[from] redis::RedisError),

    #[error("http service error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("unexpected error: {0}")]
    Unexpected(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unexpected(err.to_string())
    }
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
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
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

impl AppError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        AppError::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        AppError::Forbidden(message.into())
    }
}
