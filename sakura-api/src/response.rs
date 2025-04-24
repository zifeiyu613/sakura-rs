use actix_web::body::BoxBody;
use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Response<T> {
    // #[serde(flatten)]
    pub code: u16,

    #[serde(rename = "msg")]
    pub message: String,

    // 如果为 None，序列化时跳过
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    // 额外字段 可选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,
}

impl<T> Response<T> {

    pub fn ok() -> Self {
        Response {
            code: StatusCode::ok().error_code(),
            message: StatusCode::ok().message(),
            data: None,
            ext: None,
        }
    }

    pub fn success(data: T) -> Self {
        Response {
            code: StatusCode::ok().error_code(),
            message: StatusCode::ok().message(),
            data: Some(data),
            ext: None,
        }
    }

    pub fn error(status_code: StatusCode) -> Self {
        Response {
            code: status_code.error_code(),
            message: status_code.message(),
            data: None,
            ext: None,
        }
    }

    pub fn with_ext(mut self, ext: serde_json::Value) -> Self {
        self.ext = Some(ext);
        self
    }
}

impl<T: Serialize> Responder for Response<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}



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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_all() {
        // 构造成功响应
        let success_response = Response::success(vec![1, 2, 3]);
        println!("{:?}", success_response);
        dbg!("{:?}", serde_json::to_string(&success_response).unwrap());
        println!(
            "{:?}",
            serde_json::to_string_pretty(&success_response).unwrap()
        );

        // 构造错误响应（BadRequest）
        let bad_request_response =
            Response::<()>::error(StatusCode::bad_request("Invalid parameters".to_string()));
        println!("{:?}", bad_request_response);

        // 自定义错误响应
        let custom_error_response = Response::<()>::error(StatusCode::custom(
            422,
            4221,
            "Unprocessable Entity".to_string(),
        ));
        println!("{:?}", custom_error_response);

        // 错误响应带扩展信息
        let error_with_ext = Response::<()>::error(StatusCode::internal_error(
            "Something went wrong".to_string(),
        ))
            .with_ext(serde_json::json!({ "debug_id": 12345 }));
        println!("{:?}", error_with_ext);

        // 获取状态码和错误信息
        // let status_code = bad_request_response.status_code;
        // println!("HTTP Code: {}, Error Code: {}, Message: {}",
        //          status_code.http_code(),
        //          status_code.error_code(),
        //          status_code.message(),
        // );
    }
}

