use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {

    #[error("ConfigError error: {0}")]
    ConfigError(#[from] rconfig::ConfigError),
    
    
    #[error("Database error: {0}")]
    Database(#[from] rdatabase::DbError),

    // #[error("Redis error: {0}")]
    // Redis(#[from] redis::RedisError),
    // 
    // #[error("RabbitMQ error: {0}")]
    // RabbitMQ(#[from] lapin::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
