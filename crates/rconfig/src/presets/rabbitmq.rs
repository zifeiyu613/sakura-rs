//! RabbitMQ配置

use serde::{Deserialize, Serialize};
use crate::error::{ConfigError, Result};
use super::Validate;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RabbitMqConfig {
    /// 主机名
    #[serde(default = "default_host")]
    pub host: String,

    /// 端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 用户名
    #[serde(default = "default_username")]
    pub username: String,

    /// 密码
    #[serde(default)]
    pub password: String,

    /// 虚拟主机
    #[serde(default = "default_vhost")]
    pub vhost: String,

    /// 连接超时(秒)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 连接URL (如果设置，优先使用)
    #[serde(default)]
    pub url: Option<String>,

    /// 是否启用TLS
    #[serde(default)]
    pub use_tls: bool,

    /// 是否自动重连
    #[serde(default = "default_auto_reconnect")]
    pub auto_reconnect: bool,

    /// 重连尝试次数
    #[serde(default = "default_reconnect_attempts")]
    pub reconnect_attempts: u32,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    5672
}

fn default_username() -> String {
    "guest".to_string()
}

fn default_vhost() -> String {
    "/".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_auto_reconnect() -> bool {
    true
}

fn default_reconnect_attempts() -> u32 {
    5
}

impl Default for RabbitMqConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            username: default_username(),
            password: String::new(),
            vhost: default_vhost(),
            timeout: default_timeout(),
            url: None,
            use_tls: false,
            auto_reconnect: default_auto_reconnect(),
            reconnect_attempts: default_reconnect_attempts(),
        }
    }
}

impl RabbitMqConfig {
    /// 获取RabbitMQ连接URL
    pub fn connection_url(&self) -> String {
        if let Some(url) = &self.url {
            return url.clone();
        }

        let protocol = if self.use_tls { "amqps" } else { "amqp" };

        format!(
            "{}://{}:{}@{}:{}/{}",
            protocol, self.username, self.password, self.host, self.port, self.vhost
        )
    }
}

impl Validate for RabbitMqConfig {
    fn validate(&self) -> Result<()> {
        if self.username.is_empty() {
            return Err(ConfigError::ValidationError(
                "RabbitMQ用户名不能为空".to_string()
            ));
        }

        if self.password.is_empty() {
            return Err(ConfigError::ValidationError(
                "RabbitMQ密码不能为空".to_string()
            ));
        }

        Ok(())
    }
}
