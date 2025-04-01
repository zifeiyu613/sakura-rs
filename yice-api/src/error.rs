use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YiceError {

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

    #[error("Third party service error: {0}")]
    ThirdParty(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for YiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            YiceError::Auth(_) => (StatusCode::UNAUTHORIZED, "Authentication error"),
            YiceError::Forbidden(_) => (StatusCode::FORBIDDEN, "Authorization error"),
            YiceError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            YiceError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            YiceError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad request"),
            YiceError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            YiceError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cache error"),
            YiceError::RabbitMq(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Messaging error"),
            YiceError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            YiceError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            YiceError::ThirdParty(_) => (StatusCode::BAD_GATEWAY, "External service error"),
            YiceError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
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
