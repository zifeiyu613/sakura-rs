// //! HTTP状态码封装
//
// /// HTTP状态码
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum HttpStatusCode {
//     OK = 200,
//     Created = 201,
//     Accepted = 202,
//     NoContent = 204,
//
//     BadRequest = 400,
//     Unauthorized = 401,
//     Forbidden = 403,
//     NotFound = 404,
//     MethodNotAllowed = 405,
//     Conflict = 409,
//
//     InternalServerError = 500,
//     NotImplemented = 501,
//     BadGateway = 502,
//     ServiceUnavailable = 503,
// }
//
// impl HttpStatusCode {
//     /// 获取状态码数值
//     pub fn value(&self) -> u16 {
//         *self as u16
//     }
//
//     /// 获取状态码的标准描述
//     pub fn description(&self) -> &'static str {
//         match self {
//             Self::OK => "OK",
//             Self::Created => "Created",
//             Self::Accepted => "Accepted",
//             Self::NoContent => "No Content",
//
//             Self::BadRequest => "Bad Request",
//             Self::Unauthorized => "Unauthorized",
//             Self::Forbidden => "Forbidden",
//             Self::NotFound => "Not Found",
//             Self::MethodNotAllowed => "Method Not Allowed",
//             Self::Conflict => "Conflict",
//
//             Self::InternalServerError => "Internal Server Error",
//             Self::NotImplemented => "Not Implemented",
//             Self::BadGateway => "Bad Gateway",
//             Self::ServiceUnavailable => "Service Unavailable",
//         }
//     }
// }
//
// impl From<HttpStatusCode> for axum::http::StatusCode {
//     fn from(code: HttpStatusCode) -> Self {
//         axum::http::StatusCode::from_u16(code.value())
//             .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
//     }
// }