use std::collections::HashMap;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dto::response::ApiResponse;
use crate::middleware::decryptor::RequestData;

// 基础请求字段结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRequestFields {
    application: Option<String>,
    channel: Option<String>,
    #[serde(rename = "deviceCode")]
    device_code: Option<String>,
    #[serde(rename = "packageName")]
    package_name: Option<String>,
    source: Option<u8>,
    #[serde(rename = "subChannel")]
    sub_channel: Option<String>,

    uid: u64,

    #[serde(rename = "plainText")]
    plain_text: Option<bool>,
}


// 基础字段的helper方法
impl BaseRequestFields {
    pub fn uid(&self) -> u64 {
        self.uid
    }

    pub fn application(&self) -> &str {
        self.application.as_deref().unwrap_or("")
    }

    pub fn channel(&self) -> &str {
        self.channel.as_deref().unwrap_or("")
    }

    pub fn sub_channel(&self) -> &str {
        &self.sub_channel.as_deref().unwrap_or("")
    }

    pub fn source(&self) -> Option<u8> {
        self.source
    }

    pub fn device_code(&self) -> &str {
        &self.device_code.as_deref().unwrap_or("")
    }

    pub fn package_name(&self) -> &str {
        &self.package_name.as_deref().unwrap_or("")
    }

    pub fn plain_text(&self) -> bool {
        self.plain_text.unwrap_or(false)
    }

}


// 带类型的完整请求
#[derive(Debug, Serialize, Deserialize)]
pub struct TypedRequest<T> {
    #[serde(flatten)]
    pub base: BaseRequestFields,
    #[serde(flatten)]
    pub dto: T,
}

// 完全动态的请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicRequest {
    #[serde(flatten)]
    pub base: BaseRequestFields,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}


// 通用DTO提取器 - 根据字段名自动提取所需DTO
pub struct Dto<T: for<'de> Deserialize<'de>> {
    // 提取出的DTO
    pub inner: Option<T>,
    // 基础请求字段
    pub base: Option<BaseRequestFields>,
}

// 提取器的实现
#[async_trait]
impl<T, S> FromRequestParts<S> for Dto<T>
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
                Json(ApiResponse::error(400, "解密数据不存在"))
            )
        })?;

        let json_value = &extension.json_data;

        // if let Some(value) = json_value {
        //     // 提取基础字段
        //     let base: BaseRequestFields = serde_json::from_value(value.clone()).unwrap_or(None);
        //
        //     // 尝试将整个JSON反序列化为目标DTO类型
        //     let inner: T = serde_json::from_value(value.clone()).unwrap_or(None);
        //     return Ok(Self { inner, Some(base) });
        // }



        Ok(Self { inner, base })
    }
}