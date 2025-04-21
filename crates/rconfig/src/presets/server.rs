//! 服务器配置

use serde::{Deserialize, Serialize};
use crate::error::Result;
use super::Validate;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    /// 服务器主机名或IP
    #[serde(default = "default_host")]
    pub host: String,

    /// 服务端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 工作线程数，默认为CPU核心数
    #[serde(default = "default_workers")]
    pub workers: usize,

    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// 超时设置（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// 是否开启HTTPS
    #[serde(default)]
    pub use_tls: bool,

    /// 证书文件路径（如果use_tls=true）
    #[serde(default)]
    pub cert_path: Option<String>,

    /// 密钥文件路径（如果use_tls=true）
    #[serde(default)]
    pub key_path: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_workers() -> usize {
    num_cpus::get()
}

fn default_max_connections() -> usize {
    1000
}

fn default_timeout() -> u64 {
    30
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: default_workers(),
            max_connections: default_max_connections(),
            timeout: default_timeout(),
            use_tls: false,
            cert_path: None,
            key_path: None,
        }
    }
}

impl Validate for ServerConfig {
    fn validate(&self) -> Result<()> {
        // 验证TLS配置
        if self.use_tls {
            if self.cert_path.is_none() {
                return Err(crate::error::ConfigError::ValidationError(
                    "TLS启用时证书路径不能为空".to_string()
                ));
            }
            if self.key_path.is_none() {
                return Err(crate::error::ConfigError::ValidationError(
                    "TLS启用时密钥路径不能为空".to_string()
                ));
            }
        }
        Ok(())
    }
}


// impl Default for ServiceConfig {
//     fn default() -> Self {
//         Self {
//             name: env!("CARGO_PKG_NAME").to_string(),
//             version: env!("CARGO_PKG_VERSION").to_string(),
//             environment: "development".to_string(),
//             host: "0.0.0.0".to_string(),
//             port: 8080,
//             debug: false,
//         }
//     }
// }

