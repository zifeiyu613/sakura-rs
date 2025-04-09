use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

// 使用泛型的API响应包装器
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
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
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