use argon2::{Argon2, PasswordHash};
use argon2::password_hash::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Authorization error: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("RabbitMQ error: {0}")]
    RabbitMq(#[from] lapin::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("IO error: {0}")]
    Argon2(#[from] Error),

    #[error("Third party service error: {0}")]
    ThirdParty(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, "Authentication error"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "Authorization error"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad request"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cache error"),
            AppError::RabbitMq(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Messaging error"),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            AppError::Argon2(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Argon2 error"),
            AppError::ThirdParty(_) => (StatusCode::BAD_GATEWAY, "External service error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "details": self.to_string()
            }
        }));

        (status, body).into_response()
    }
}
