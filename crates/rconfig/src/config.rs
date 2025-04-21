//! 主配置结构和构建器

use crate::error::{ConfigError, Result};
use crate::presets::*;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// 应用配置，包含所有预设服务配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    
    /// 环境变量
    pub env: Option<String>,
    
    /// 服务器配置
    #[serde(default)]
    pub server: server::ServerConfig,

    /// 主/默认数据库配置 (向后兼容)
    #[serde(default)]
    pub database: database::DatabaseConfig,

    /// 多数据源配置
    #[serde(default)]
    pub databases: database::DatabaseSources,


    /// Redis配置
    #[serde(default)]
    pub redis: redis::RedisConfig,

    /// RabbitMQ配置
    #[serde(default)]
    pub rabbitmq: rabbitmq::RabbitMqConfig,

    /// 日志配置
    #[serde(default)]
    pub logging: logging::LogConfig,

    /// 自定义扩展配置
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl AppConfig {
    /// 创建配置构建器
    pub fn new() -> AppConfigBuilder {
        AppConfigBuilder::new()
    }

    /// 获取服务器配置
    pub fn server(&self) -> &server::ServerConfig {
        &self.server
    }

    /// 获取默认数据库配置（向后兼容）
    pub fn database(&self) -> &database::DatabaseConfig {
        &self.database
    }

    /// 获取指定名称的数据库配置
    /// 如果名称为None或"default"，则返回默认数据库配置
    pub fn get_database(&self, name: Option<&str>) -> Option<&database::DatabaseConfig> {
        match name {
            None => Some(&self.database),
            Some("default") => Some(&self.database),
            Some(name) => self.databases.sources.get(name),
        }
    }

    /// 获取所有数据库源名称
    pub fn database_names(&self) -> Vec<&str> {
        let mut names = vec!["default"];
        names.extend(self.databases.sources.keys().map(|k| k.as_str()));
        names
    }

    /// 获取Redis配置
    pub fn redis(&self) -> &redis::RedisConfig {
        &self.redis
    }

    /// 获取RabbitMQ配置
    pub fn rabbitmq(&self) -> &rabbitmq::RabbitMqConfig {
        &self.rabbitmq
    }

    /// 获取日志配置
    pub fn logging(&self) -> &logging::LogConfig {
        &self.logging
    }

    /// 获取扩展配置
    pub fn get_extension<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self.extensions.get(key)
            .ok_or_else(|| ConfigError::MissingConfig(format!("扩展配置不存在: {}", key)))?;

        serde_json::from_value(value.clone())
            .map_err(ConfigError::from)
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        self.server.validate()?;
        self.database.validate()?;
        self.databases.validate()?;
        self.redis.validate()?;
        self.rabbitmq.validate()?;
        self.logging.validate()?;
        Ok(())
    }
}

/// 配置构建器
pub struct AppConfigBuilder {
    config_builder: config::ConfigBuilder<config::builder::DefaultState>,
}

impl AppConfigBuilder {
    /// 创建新构建器
    pub fn new() -> Self {
        Self {
            config_builder: Config::builder(),
        }
    }

    /// 添加默认配置文件，支持 .json, .toml, .yaml, .hjson, .ini
    pub fn add_default<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path = path.as_ref();
        // 尝试不同扩展名，使用找到的第一个
        for ext in &["json", "toml", "yaml", "hjson", "ini"] {
            let file_path = format!("{}.{}", path.display(), ext);
            if Path::new(&file_path).exists() {
                self.config_builder = self.config_builder
                    .add_source(File::with_name(&file_path).required(false));
                break;
            }
        }
        self
    }

    /// 添加指定环境的配置文件
    pub fn add_environment_file<P: AsRef<Path>>(mut self, env: &str, path: P) -> Self {
        let path = path.as_ref();
        for ext in &["json", "toml", "yaml", "hjson", "ini"] {
            let file_path = format!("{}_{}.{}", path.display(), env, ext);
            if Path::new(&file_path).exists() {
                self.config_builder = self.config_builder
                    .add_source(File::with_name(&file_path).required(false));
                break;
            }
        }
        self
    }

    /// 添加环境变量支持，使用APP_前缀
    pub fn add_environment(mut self) -> Self {
        // 使用APP_前缀，双下划线分隔层级
        self.config_builder = self.config_builder
            .add_source(Environment::with_prefix("APP").separator("__"));
        self
    }

    /// 从特定文件加载配置
    pub fn add_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_builder = self.config_builder
            .add_source(File::with_name(path.as_ref().to_str().unwrap()).required(false));
        self
    }

    /// 从.env文件加载环境变量
    pub fn add_dotenv(self) -> Self {
        // 加载.env文件，忽略错误
        let _ = dotenvy::dotenv();
        self
    }

    /// 构建最终配置
    pub fn build(self) -> Result<AppConfig> {
        let config = self.config_builder.build()?;
        let mut app_config: AppConfig = config.try_deserialize()?;

        // 后处理：如果主数据库已配置但databases.default未配置，则同步
        // 检查default是否为默认值（未配置）
        if app_config.databases.default.username.is_empty() &&
            app_config.databases.default.database.is_empty() &&
            !app_config.database.username.is_empty() {
            // 将主配置同步到多数据源的default配置
            app_config.databases.default = app_config.database.clone();
        }
        
        // 验证配置
        app_config.validate()?;

        Ok(app_config)
    }
}

impl Default for AppConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
