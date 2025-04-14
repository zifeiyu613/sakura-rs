/// 序列化工具

use super::error::{RedisError, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Redis数据序列化和反序列化接口
pub trait RedisSerializer {
    /// 将值序列化为字符串
    fn serialize<T: Serialize>(&self, value: &T) -> Result<String>;

    /// 从字符串反序列化为指定类型
    fn deserialize<T: DeserializeOwned>(&self, data: &str) -> Result<T>;
}

/// JSON序列化器实现
#[derive(Debug, Clone, Default)]
pub struct JsonSerializer;

impl RedisSerializer for JsonSerializer {
    fn serialize<T: Serialize>(&self, value: &T) -> Result<String> {
        serde_json::to_string(value).map_err(|e| {
            RedisError::Serialization(format!("Failed to serialize to JSON: {}", e))
        })
    }

    fn deserialize<T: DeserializeOwned>(&self, data: &str) -> Result<T> {
        serde_json::from_str(data).map_err(|e| {
            RedisError::Deserialization(format!("Failed to deserialize from JSON: {}", e))
        })
    }
}
