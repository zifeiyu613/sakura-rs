pub mod error;
pub mod response;
pub mod business;

pub use error::ApiError;
pub use response::{business_error_response, error_response};

pub use business::BusinessCode;

/// 获取业务状态码对应的HTTP状态码
pub fn get_http_status(business_code: BusinessCode) -> axum::http::StatusCode {
    match business_code {
        BusinessCode::Success => axum::http::StatusCode::OK,
        BusinessCode::ValidationError => axum::http::StatusCode::BAD_REQUEST,
        BusinessCode::Unauthorized => axum::http::StatusCode::UNAUTHORIZED,
        BusinessCode::ResourceNotFound => axum::http::StatusCode::NOT_FOUND,
        BusinessCode::ServiceUnavailable => axum::http::StatusCode::SERVICE_UNAVAILABLE,
        BusinessCode::Forbidden => axum::http::StatusCode::FORBIDDEN,
        BusinessCode::RequestTimeout => axum::http::StatusCode::REQUEST_TIMEOUT,
        // 默认返回500错误
        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}