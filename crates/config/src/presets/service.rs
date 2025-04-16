use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    #[serde(default = "default_service_name")]
    pub name: String,

    #[serde(default = "default_service_host")]
    pub host: String,

    #[serde(default = "default_service_port")]
    pub port: u16,

    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub debug: bool,
}

fn default_service_name() -> String {
    "app".to_string()
}

fn default_service_host() -> String {
    "0.0.0.0".to_string()
}

fn default_service_port() -> u16 {
    8080
}

fn default_environment() -> String {
    "development".to_string()
}

// presets/database.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    #[serde(default)]
    pub ssl_mode: Option<String>,
}

// 类似地实现其他预设配置...
