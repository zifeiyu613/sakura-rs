use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::{ServiceConfig, DatabaseConfig, RedisConfig, RabbitMQConfig, LoggingConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    // 预设配置
    pub service: ServiceConfig,
    pub database: HashMap<String, DatabaseConfig>,
    pub redis: Option<RedisConfig>,
    pub rabbitmq: Option<RabbitMQConfig>,
    pub logging: LoggingConfig,

    // 自定义配置存储
    #[serde(flatten)]
    custom: HashMap<String, serde_json::Value>,
}

impl AppConfig {
    // 获取自定义配置
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.custom.get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    // 检查是否包含指定配置
    pub fn contains(&self, key: &str) -> bool {
        self.custom.contains_key(key)
    }

    // 获取主数据库配置
    pub fn main_database(&self) -> Option<&DatabaseConfig> {
        self.database.get("main")
    }

    // 简便的访问器方法
    pub fn service_name(&self) -> &str {
        &self.service.name
    }

    pub fn is_development(&self) -> bool {
        self.service.environment == "development"
    }

    pub fn is_production(&self) -> bool {
        self.service.environment == "production"
    }
}
