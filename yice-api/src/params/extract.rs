use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use crate::errors::response::ApiResponse;
use crate::params::{BaseRequestFields, RequestDto};
use crate::middleware::decryptor::RequestData;
use crate::status::BusinessCode;

// 提取器的实现
impl<T, S> FromRequestParts<S> for RequestDto<T>
where
    T: for<'de> Deserialize<'de> + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ApiResponse<Value>>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从request extension中获取解密后的JSON
        let extension = parts.extensions.get::<RequestData>().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(BusinessCode::BadRequest, Some("解密数据不存在".to_string()))),
            )
        })?;

        let json_value = &extension.json_data;

        // 初始化结果变量
        let mut inner: Option<T> = None;
        let mut base: Option<BaseRequestFields> = None;

        // 只有当JSON数据存在时才尝试解析
        if let Some(json) = json_value {
            // 尝试解析整个JSON为T类型
            inner = match serde_json::from_value::<T>(json.clone()) {
                Ok(data) => Some(data),
                Err(e) => {
                    // 记录错误但不返回错误响应，允许空DTO
                    tracing::debug!("解析DTO失败: {}, 将返回None", e);
                    None
                }
            };

            // 尝试解析基础字段
            base = match serde_json::from_value::<BaseRequestFields>(json.clone()) {
                Ok(fields) => Some(fields),
                Err(e) => {
                    // 记录错误但不返回错误响应
                    tracing::debug!("解析基础字段失败: {}, 将返回None", e);
                    None
                }
            };
        } else {
            // JSON数据不存在的情况
            tracing::debug!("请求中没有JSON数据");
        }

        Ok(Self { inner, base })
    }
}