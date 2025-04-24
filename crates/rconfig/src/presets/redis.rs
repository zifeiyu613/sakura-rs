//! Redis配置

use serde::{Deserialize, Serialize};
use crate::error::{ConfigError, Result};
use super::Validate;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RedisConfig {
    /// Redis主机
    #[serde(default = "default_host")]
    pub host: String,

    /// Redis端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 用户名 (Redis 6.0+)
    #[serde(default)]
    pub username: Option<String>,

    /// 密码
    #[serde(default)]
    pub password: Option<String>,

    /// 数据库索引
    #[serde(default)]
    pub database: u8,

    /// 连接池大小
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// 连接超时(秒)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 连接URL (如果设置，优先使用)
    #[serde(default)]
    pub url: Option<String>,

    /// 是否启用集群模式
    #[serde(default)]
    pub cluster_mode: bool,

    /// 集群节点 (如果cluster_mode=true)
    #[serde(default)]
    pub cluster_nodes: Vec<String>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    6379
}

fn default_pool_size() -> u32 {
    10
}

fn default_timeout() -> u64 {
    5
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            username: None,
            password: None,
            database: 0,
            pool_size: default_pool_size(),
            timeout: default_timeout(),
            url: None,
            cluster_mode: false,
            cluster_nodes: Vec::new(),
        }
    }
}

impl RedisConfig {
    /// 获取Redis连接URL
    pub fn connection_url(&self) -> String {
        if let Some(url) = &self.url {
            return url.clone();
        }

        let mut url = String::from("redis://");

        // 添加认证信息
        if let Some(username) = &self.username {
            url.push_str(username);
            if let Some(password) = &self.password {
                url.push(':');
                url.push_str(password);
            }
            url.push('@');
        } else if let Some(password) = &self.password {
            url.push_str(":");
            url.push_str(password);
            url.push('@');
        }

        // 添加主机和端口
        url.push_str(&format!("{}:{}", self.host, self.port));

        // 添加数据库
        url.push_str(&format!("/{}", self.database));

        url
    }
}

impl Validate for RedisConfig {
    fn validate(&self) -> Result<()> {
        if self.cluster_mode && self.cluster_nodes.is_empty() {
            return Err(ConfigError::ValidationError(
                "Redis集群模式启用时，集群节点列表不能为空".to_string()
            ));
        }
        Ok(())
    }
}
