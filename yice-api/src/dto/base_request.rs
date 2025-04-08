use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 基础请求字段结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRequest {
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


// 带类型的完整请求
#[derive(Debug, Serialize, Deserialize)]
pub struct TypedRequest<T> {
    #[serde(flatten)]
    pub base: BaseRequest,
    #[serde(flatten)]
    pub dto: T,
}

// 完全动态的请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicRequest {
    #[serde(flatten)]
    pub base: BaseRequest,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// 基础字段的helper方法
impl BaseRequest {
    pub fn uid(&self) -> u64 {
        self.uid
    }

    pub(crate) fn application(&self) -> &str {
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