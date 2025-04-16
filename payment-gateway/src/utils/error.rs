use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::fmt;
use tracing::error;

// API响应结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            code: "0".to_string(),
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            success: false,
            code: code.to_string(),
            message: message.to_string(),
            data: None,
        }
    }
}

// 业务错误代码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 系统错误 (1xxx)
    InternalServerError = 1000,
    DatabaseError = 1001,
    CacheError = 1002,
    ConfigError = 1003,
    NetworkError = 1004,
    TaskExecutionError = 1005,

    // 认证错误 (2xxx)
    Unauthorized = 2000,
    InvalidCredentials = 2001,
    TokenExpired = 2002,
    InsufficientPermissions = 2003,
    AccountDisabled = 2004,

    // 请求错误 (3xxx)
    BadRequest = 3000,
    InvalidParameters = 3001,
    ResourceNotFound = 3002,
    MethodNotAllowed = 3003,
    RequestTimeout = 3004,
    RateLimitExceeded = 3005,

    // 业务错误 (4xxx)
    ValidationFailed = 4000,
    InvalidSignature = 4001,
    OrderNotFound = 4002,
    PaymentFailed = 4003,
    RefundFailed = 4004,
    DuplicateOrder = 4005,
    InsufficientFunds = 4006,
    ChannelUnavailable = 4007,
    TransactionExpired = 4008,
    UnsupportedCurrency = 4009,

    // 第三方服务错误 (5xxx)
    ChannelApiError = 5000,
    ChannelTimeout = 5001,
    ChannelRejected = 5002,
    ChannelInvalidResponse = 5003,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InternalServerError => "Internal server error",
            Self::DatabaseError => "Database error",
            Self::CacheError => "Cache error",
            Self::ConfigError => "Configuration error",
            Self::NetworkError => "Network error",
            Self::TaskExecutionError => "Task execution error",

            Self::Unauthorized => "Unauthorized",
            Self::InvalidCredentials => "Invalid credentials",
            Self::TokenExpired => "Token expired",
            Self::InsufficientPermissions => "Insufficient permissions",
            Self::AccountDisabled => "Account disabled",

            Self::BadRequest => "Bad request",
            Self::InvalidParameters => "Invalid parameters",
            Self::ResourceNotFound => "Resource not found",
            Self::MethodNotAllowed => "Method not allowed",
            Self::RequestTimeout => "Request timeout",
            Self::RateLimitExceeded => "Rate limit exceeded",

            Self::ValidationFailed => "Validation failed",
            Self::InvalidSignature => "Invalid signature",
            Self::OrderNotFound => "Order not found",
            Self::PaymentFailed => "Payment failed",
            Self::RefundFailed => "Refund failed",
            Self::DuplicateOrder => "Duplicate order",
            Self::InsufficientFunds => "Insufficient funds",
            Self::ChannelUnavailable => "Channel unavailable",
            Self::TransactionExpired => "Transaction expired",
            Self::UnsupportedCurrency => "Unsupported currency",

            Self::ChannelApiError => "Channel API error",
            Self::ChannelTimeout => "Channel timeout",
            Self::ChannelRejected => "Channel rejected",
            Self::ChannelInvalidResponse => "Channel invalid response",
        }
    }

    pub fn as_u16(&self) -> u16 {
        *self as u16
    }

    pub fn as_status_code(&self) -> StatusCode {
        match self {
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CacheError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ConfigError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NetworkError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TaskExecutionError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::AccountDisabled => StatusCode::FORBIDDEN,

            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::InvalidParameters => StatusCode::BAD_REQUEST,
            Self::ResourceNotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,

            Self::ValidationFailed => StatusCode::BAD_REQUEST,
            Self::InvalidSignature => StatusCode::BAD_REQUEST,
            Self::OrderNotFound => StatusCode::NOT_FOUND,
            Self::PaymentFailed => StatusCode::BAD_REQUEST,
            Self::RefundFailed => StatusCode::BAD_REQUEST,
            Self::DuplicateOrder => StatusCode::CONFLICT,
            Self::InsufficientFunds => StatusCode::BAD_REQUEST,
            Self::ChannelUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::TransactionExpired => StatusCode::BAD_REQUEST,
            Self::UnsupportedCurrency => StatusCode::BAD_REQUEST,

            Self::ChannelApiError => StatusCode::BAD_GATEWAY,
            Self::ChannelTimeout => StatusCode::GATEWAY_TIMEOUT,
            Self::ChannelRejected => StatusCode::BAD_GATEWAY,
            Self::ChannelInvalidResponse => StatusCode::BAD_GATEWAY,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Application error: {0}")]
    Application(ErrorCode, String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External service error: {code}, {message}")]
    ExternalService { code: String, message: String },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Internal server error: {0}")]
    InternalServer(anyhow::Error),
}

impl AppError {
    pub fn application(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Application(code, message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn external_service(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::TooManyRequests(message.into())
    }

    pub fn internal_server(err: impl Into<anyhow::Error>) -> Self {
        Self::InternalServer(err.into())
    }

    // 获取错误对应的HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Application(code, _) => code.as_status_code(),
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    // 获取错误代码
    pub fn error_code(&self) -> String {
        match self {
            Self::Application(code, _) => code.as_u16().to_string(),
            Self::Database(_) => ErrorCode::DatabaseError.as_u16().to_string(),
            Self::Redis(_) => ErrorCode::CacheError.as_u16().to_string(),
            Self::Validation(_) => ErrorCode::ValidationFailed.as_u16().to_string(),
            Self::ExternalService { code, .. } => code.clone(),
            Self::NotFound(_) => ErrorCode::ResourceNotFound.as_u16().to_string(),
            Self::Unauthorized(_) => ErrorCode::Unauthorized.as_u16().to_string(),
            Self::Forbidden(_) => ErrorCode::InsufficientPermissions.as_u16().to_string(),
            Self::Conflict(_) => ErrorCode::DuplicateOrder.as_u16().to_string(),
            Self::TooManyRequests(_) => ErrorCode::RateLimitExceeded.as_u16().to_string(),
            Self::InternalServer(_) => ErrorCode::InternalServerError.as_u16().to_string(),
        }
    }

    // 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            Self::Application(_, msg) => msg.clone(),
            Self::Database(err) => {
                error!("Database error: {}", err);
                "A database error occurred. Please try again later.".to_string()
            }
            Self::Redis(err) => {
                error!("Redis error: {}", err);
                "A caching error occurred. Please try again later.".to_string()
            }
            Self::Validation(msg) => msg.clone(),
            Self::ExternalService { message, .. } => message.clone(),
            Self::NotFound(msg) => msg.clone(),
            Self::Unauthorized(msg) => msg.clone(),
            Self::Forbidden(msg) => msg.clone(),
            Self::Conflict(msg) => msg.clone(),
            Self::TooManyRequests(msg) => msg.clone(),
            Self::InternalServer(err) => {
                error!("Internal server error: {:#}", err);
                "An internal error occurred. Please try again later.".to_string()
            }
        }
    }
}

// 将AppError转换为Axum响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let message = self.user_message();

        let response = ApiResponse::<()>::error(&code, &message);

        // 记录5xx错误
        if status.is_server_error() {
            error!(
                status_code = %status.as_u16(),
                error_code = %code,
                error_message = %message,
                "Server error occurred"
            );
        }

        (status, Json(response)).into_response()
    }
}

// 处理anyhow::Error的转换
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalServer(err)
    }
}

// 从应用错误获取API响应
pub fn api_error<T: Serialize>(code: &str, message: &str) -> ApiResponse<T> {
    ApiResponse {
        success: false,
        code: code.to_string(),
        message: message.to_string(),
        data: None,
    }
}

// 创建成功响应
pub fn api_success<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse {
        success: true,
        code: "0".to_string(),
        message: "Success".to_string(),
        data: Some(data),
    }
}

// 通用错误处理器
pub async fn handle_error(err: anyhow::Error) -> AppError {
    AppError::InternalServer(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let success = ApiResponse::success("test data");
        assert_eq!(success.success, true);
        assert_eq!(success.code, "0");
        assert_eq!(success.message, "Success");
        assert_eq!(success.data, Some("test data"));

        let error = ApiResponse::<String>::error("1001", "Test error");
        assert_eq!(error.success, false);
        assert_eq!(error.code, "1001");
        assert_eq!(error.message, "Test error");
        assert_eq!(error.data, None);
    }

    #[test]
    fn test_error_code() {
        assert_eq!(ErrorCode::InvalidSignature.as_str(), "Invalid signature");
        assert_eq!(ErrorCode::InvalidSignature.as_u16(), 4001);
        assert_eq!(ErrorCode::InvalidSignature.as_status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_app_error() {
        let app_error = AppError::application(ErrorCode::PaymentFailed, "Payment failed due to insufficient funds");
        assert_eq!(app_error.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.error_code(), "4003");

        let validation_error = AppError::validation("Invalid card number");
        assert_eq!(validation_error.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(validation_error.error_code(), "4000");

        let not_found_error = AppError::not_found("Order not found");
        assert_eq!(not_found_error.status_code(), StatusCode::NOT_FOUND);
    }
}
