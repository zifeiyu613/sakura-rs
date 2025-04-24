//! 数据库配置 - 支持多数据源

use serde::{Deserialize, Serialize};
use url::Url;
use std::collections::HashMap;
use crate::error::{ConfigError, Result};
use super::Validate;

/// 单个数据库配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// 数据库类型: mysql, postgres等
    #[serde(default = "default_db_type")]
    pub db_type: String,

    /// 主机名
    #[serde(default = "default_host")]
    pub host: String,

    /// 端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 用户名
    #[serde(default)]
    pub username: String,

    /// 密码
    #[serde(default)]
    pub password: String,

    /// 数据库名
    #[serde(default)]
    pub database: String,

    /// 连接池最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// 连接池最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// 连接超时(秒)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 连接URL (如果设置，优先使用)
    #[serde(default)]
    pub url: Option<String>,

    /// 额外参数
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// 多数据源配置，管理多个命名的数据库连接
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DatabaseSources {
    /// 默认/主数据库配置
    #[serde(default)]
    pub default: DatabaseConfig,

    /// 命名的额外数据库源
    #[serde(default)]
    pub sources: HashMap<String, DatabaseConfig>,
}

fn default_db_type() -> String {
    "mysql".to_string()
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    3306
}

fn default_min_connections() -> u32 {
    5
}

fn default_max_connections() -> u32 {
    20
}

fn default_timeout() -> u64 {
    30
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: default_db_type(),
            host: default_host(),
            port: default_port(),
            username: String::new(),
            password: String::new(),
            database: String::new(),
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            timeout: default_timeout(),
            url: None,
            options: HashMap::new(),
        }
    }
}

impl DatabaseConfig {
    /// 获取完整连接URL
    pub fn connection_url(&self) -> Result<String> {
        if let Some(url) = &self.url {
            // 验证URL格式
            let _ = Url::parse(url)?;
            return Ok(url.clone());
        }

        // 根据数据库类型构建URL
        match self.db_type.as_str() {
            "mysql" => {
                Ok(format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                ))
            },
            "postgres" => {
                Ok(format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                ))
            },
            "sqlite" => {
                Ok(format!("sqlite://{}", self.database))
            },
            _ => Err(ConfigError::ValidationError(
                format!("不支持的数据库类型: {}", self.db_type)
            )),
        }
    }
}

impl DatabaseSources {
    /// 获取指定名称的数据库配置
    ///
    /// # Arguments
    /// * `name` - 数据源名称，如果为None则返回默认数据源
    ///
    /// # Returns
    /// * `Some(DatabaseConfig)` - 找到的数据库配置
    /// * `None` - 未找到指定名称的数据源
    pub fn get(&self, name: Option<&str>) -> Option<&DatabaseConfig> {
        match name {
            None => Some(&self.default),
            Some("default") => Some(&self.default),
            Some(name) => self.sources.get(name),
        }
    }

    /// 获取所有数据源名称
    pub fn source_names(&self) -> Vec<&str> {
        let mut names = vec!["default"];
        names.extend(self.sources.keys().map(|k| k.as_str()));
        names
    }

    /// 迭代所有数据源
    pub fn iter(&self) -> impl Iterator<Item = (&str, &DatabaseConfig)> {
        std::iter::once(("default", &self.default))
            .chain(self.sources.iter().map(|(k, v)| (k.as_str(), v)))
    }
}

impl Validate for DatabaseConfig {
    fn validate(&self) -> Result<()> {
        // 如果URL存在，验证URL格式
        if let Some(url) = &self.url {
            let _ = Url::parse(url)?;
            return Ok(());
        }

        // 否则验证必填字段
        if self.username.is_empty() {
            return Err(ConfigError::ValidationError("数据库用户名不能为空".to_string()));
        }

        if self.database.is_empty() && self.db_type != "sqlite" {
            return Err(ConfigError::ValidationError("数据库名不能为空".to_string()));
        }

        Ok(())
    }
}

impl Validate for DatabaseSources {
    fn validate(&self) -> Result<()> {
        // 验证默认数据库配置
        // self.default.validate()?;

        // 验证所有额外数据源
        for (name, config) in &self.sources {
            config.validate().map_err(|e| {
                ConfigError::ValidationError(format!("数据源'{}'验证失败: {}", name, e))
            })?;
        }

        Ok(())
    }
}
