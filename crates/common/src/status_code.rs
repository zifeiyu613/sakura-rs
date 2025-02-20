use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum StatusCode {
    OK {
        error_code: u16,
        message: String,
    },
    BadRequest {
        error_code: u16,
        message: String,
    },
    Unauthorized {
        error_code: u16,
        message: String,
    },
    InternalServerError {
        error_code: u16,
        message: String,
    },
    Custom {
        http_code: u16,
        error_code: u16,
        message: String,
    },


}


impl StatusCode {
    pub fn ok() -> Self {
        StatusCode::OK { error_code: 0, message: "ok".to_string() }
    }

    pub fn bad_request(message: String) -> Self {
        StatusCode::BadRequest { error_code: 400, message }
    }

    pub fn unauthorized(message: String) -> Self {
        StatusCode::Unauthorized { error_code: 401, message }
    }

    pub fn internal_error(message: String) -> Self {
        StatusCode::InternalServerError { error_code: 500, message }
    }
    pub fn custom(http_code: u16, error_code: u16, message: String) -> Self {
        StatusCode::Custom { http_code, error_code, message }
    }

    // 获取 HTTP 状态码
    pub fn http_code(&self) -> u16 {
        match self {
            StatusCode::OK { .. } => 0,
            StatusCode::BadRequest { .. } => 400,
            StatusCode::Unauthorized { .. } => 401,
            StatusCode::InternalServerError { .. } => 500,
            StatusCode::Custom { http_code, .. } => *http_code,
        }
    }

    // 获取业务错误码
    pub fn error_code(&self) -> u16 {
        match self {
            StatusCode::OK { error_code, .. } => *error_code,
            StatusCode::BadRequest { error_code, .. } => *error_code,
            StatusCode::Unauthorized { error_code, .. } => *error_code,
            StatusCode::InternalServerError { error_code, .. } => *error_code,
            StatusCode::Custom { error_code, .. } => *error_code,
        }
    }

    // 获取错误描述
    pub fn message(&self) -> String {
        match self {
            StatusCode::OK { message, .. } => message.clone(),
            StatusCode::BadRequest { message, .. } => message.clone(),
            StatusCode::Unauthorized { message, .. } => message.clone(),
            StatusCode::InternalServerError { message, .. } => message.clone(),
            StatusCode::Custom { message, .. } => message.clone(),
        }
    }

}