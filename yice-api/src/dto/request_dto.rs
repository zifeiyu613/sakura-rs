use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 基础请求字段结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRequest {
    pub(crate) application: String,
    pub(crate) channel: String,
    #[serde(rename = "deviceCode")]
    pub(crate) device_code: String,
    #[serde(rename = "packageName")]
    pub(crate) package_name: String,
    #[serde(rename = "plainText")]
    pub(crate) plain_text: bool,
    pub(crate) source: u32,
    #[serde(rename = "subChannel")]
    pub(crate) sub_channel: String,
    pub(crate) uid: u64,
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

    pub fn application(&self) -> &str {
        &self.application
    }

    pub fn channel(&self) -> &str {
        &self.channel
    }

    pub fn sub_channel(&self) -> &str {
        &self.sub_channel
    }

    pub fn source(&self) -> u32 {
        self.source
    }

    pub fn device_code(&self) -> &str {
        &self.device_code
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn plain_text(&self) -> bool {
        self.plain_text
    }



    // 更多方法...
}