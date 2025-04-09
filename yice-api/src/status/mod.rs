// pub mod http;
pub mod business;

// 重新导出，方便使用  
// pub use http::HttpStatusCode;
pub use business::BusinessCode;

// /// 获取业务状态码对应的HTTP状态码
// pub fn get_http_status(business_code: BusinessCode) -> HttpStatusCode {
//     match business_code {
//         BusinessCode::Success => HttpStatusCode::OK,
//         BusinessCode::ValidationError => HttpStatusCode::BadRequest,
//         BusinessCode::Unauthorized => HttpStatusCode::Unauthorized,
//         BusinessCode::ResourceNotFound => HttpStatusCode::NotFound,
//         BusinessCode::ServiceUnavailable => HttpStatusCode::ServiceUnavailable,
//         // 默认返回500错误
//         _ => HttpStatusCode::InternalServerError,
//     }
// }

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