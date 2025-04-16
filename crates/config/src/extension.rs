use serde::{Deserialize, Serialize};
use crate::builder::ConfigBuilder;

pub trait ConfigExtension: serde::Serialize {
    fn key(&self) -> &'static str;
}

// 实现一个示例扩展
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub api_key: String,
    pub api_secret: String,
    pub endpoint: String,
    pub timeout_secs: u64,
}

impl ConfigExtension for PaymentConfig {
    fn key(&self) -> &'static str {
        "payment"
    }
}

// 使扩展可以直接添加到ConfigBuilder
impl ConfigBuilder {
    pub fn with_extension_trait<T: ConfigExtension>(self, extension: T) -> Self {
        self.with_extension(extension.key(), extension)
    }
}
