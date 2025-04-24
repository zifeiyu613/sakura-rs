//! 错误响应处理

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;
use serde_json::Value;

use crate::errors::{ApiError, BusinessCode};

/// API响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            code: BusinessCode::Success.value(),
            message: BusinessCode::Success.default_message().to_string(),
            data: Some(data),
        }
    }

    /// 创建错误响应
    pub fn error(code: BusinessCode, message: Option<String>) -> Self {
        Self {
            code: code.value(),
            message: message.unwrap_or_else(|| code.default_message().to_string()),
            data: None,
        }
    }

}

// 专门针对 Value 类型的实现
impl ApiResponse<Value> {
    /// 创建带详细信息的错误响应
    pub fn error_with_details(code: BusinessCode, message: Option<String>, details: Value) -> Self {
        Self {
            code: code.value(),
            message: message.unwrap_or_else(|| code.default_message().to_string()),
            data: Some(details),
        }
    }
}

/// 从YiceError创建标准错误响应
pub fn error_response(err: &ApiError) -> Response {
    let code = err.business_code();
    let status = err.status_code();
    let message = err.to_string();

    // 检查是否有详情
    if let Some(details) = err.details() {
        (
            status,
            Json(ApiResponse::<Value>::error_with_details(
                code,
                Some(message),
                details,
            ))
        ).into_response()
    } else {
        (
            status,
            Json(ApiResponse::<Value>::error(
                code,
                Some(message),
            ))
        ).into_response()
    }
}

/// 根据业务码创建API错误响应
pub fn business_error_response(
    code: BusinessCode,
    message: Option<String>
) -> Response {
    let status = crate::errors::get_http_status(code);

    (
        status,
        Json(ApiResponse::<Value>::error(code, message))
    ).into_response()
}


impl<T> IntoResponse for ApiResponse<T>
where T: Serialize {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(json) => {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(json))
                    .unwrap()
                    .into_response()
            }
            Err(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}