//! 应用错误类型定义

use chrono::ParseError;
use thiserror::Error;
use crate::status::BusinessCode;
use serde_json::Value;


#[derive(Debug, Error)]
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

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Date parse error: {0}")]
    DateParseError(#[from] ParseError),

    #[error("Data parse error: {0}")]
    DataParseError(#[from] serde_json::error::Error),

    #[error("Urlencoded parse error: {0}")]
    UrlencodedParseError(#[from] serde_urlencoded::ser::Error),


    // 扩展: 添加业务错误类型，集成BusinessCode
    #[error("{message}")]
    Business {
        code: BusinessCode,
        message: String,
        details: Option<Value>,
    },
}

impl YiceError {
    /// 创建业务错误
    pub fn business(code: BusinessCode) -> Self {
        Self::Business {
            code,
            message: code.default_message().to_string(),
            details: None,
        }
    }

    /// 创建带消息的业务错误
    pub fn business_with_message(code: BusinessCode, message: impl Into<String>) -> Self {
        Self::Business {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// 创建带详情的业务错误
    pub fn business_with_details(
        code: BusinessCode,
        message: impl Into<String>,
        details: Value
    ) -> Self {
        Self::Business {
            code,
            message: message.into(),
            details: Some(details),
        }
    }

    /// 获取错误的业务码
    pub fn business_code(&self) -> BusinessCode {
        match self {
            Self::Business { code, .. } => *code,
            Self::Auth(_) => BusinessCode::Unauthorized,
            Self::Forbidden(_) => BusinessCode::Forbidden,
            Self::NotFound(_) => BusinessCode::ResourceNotFound,
            Self::Validation(_) => BusinessCode::ValidationError,
            Self::Database(_) => BusinessCode::DatabaseError,
            Self::Redis(_) => BusinessCode::RedisError,
            Self::RabbitMq(_) => BusinessCode::MessageQueueError,
            Self::Config(_) => BusinessCode::ConfigError,
            Self::Io(_) => BusinessCode::IOError,
            Self::ThirdParty(_) => BusinessCode::ThirdPartyServiceError,
            Self::Internal(_) => BusinessCode::InternalError,
            Self::BadRequest(_) => BusinessCode::BadRequest,
            Self::HttpError(_) => BusinessCode::ExternalApiError,
            Self::UrlParseError(_) | Self::DateParseError(_) |
            Self::DataParseError(_) | Self::UrlencodedParseError(_) => BusinessCode::ParseError,
        }
    }

    /// 获取错误详情
    pub fn details(&self) -> Option<Value> {
        match self {
            Self::Business { details, .. } => details.clone(),
            _ => None,
        }
    }

    /// 获取HTTP状态码
    pub fn status_code(&self) -> axum::http::StatusCode {
        crate::status::get_http_status(self.business_code())
    }
}

// impl IntoResponse for YiceError {
//     fn into_response(self) -> Response {
//         let (status, error_message) = match self {
//             YiceError::Auth(_) => (StatusCode::UNAUTHORIZED, "Authentication error"),
//             YiceError::Forbidden(_) => (StatusCode::FORBIDDEN, "Authorization error"),
//             YiceError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found"),
//             YiceError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
//             YiceError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad request"),
//             YiceError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
//             YiceError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cache error"),
//             YiceError::RabbitMq(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Messaging error"),
//             YiceError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
//             YiceError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
//             YiceError::ThirdParty(_) => (StatusCode::BAD_GATEWAY, "External service error"),
//             YiceError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
//             YiceError::HmacError => (StatusCode::INTERNAL_SERVER_ERROR, "HMAC error"),
//             YiceError::CustomError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
//             YiceError::HttpError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Http error"),
//             _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
//
//         };
//
//         let body = Json(ApiResponse::<Value>::error(
//             status.as_u16() as i32,
//             &format!("{}", error_message)
//         ));
//
//         (status, body).into_response()
//     }
// }
