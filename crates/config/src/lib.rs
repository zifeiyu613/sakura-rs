use once_cell::sync::Lazy;
use std::sync::RwLock;
use config::{Config, File, Environment};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub mysql: MysqlConfig,
    pub redis: RedisConfig,
    pub mq: MqConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MysqlConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub uri: String,
    pub pool_max_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MqConfig {
    pub uri: String,
    pub exchange: String,
}

// **全局唯一的 `CONFIG_MANAGER`**
pub static CONFIG_MANAGER: Lazy<RwLock<Option<AppConfig>>> = Lazy::new(|| RwLock::new(None));

/// **初始化配置**
pub fn load_config(config_path: Option<&str>) -> Result<()> {
    let config_path = config_path.unwrap_or("config.toml");

    let mut config_builder = Config::builder()
        .add_source(File::with_name(config_path))
        .add_source(Environment::with_prefix("APP")); // 支持环境变量覆盖

    let config: AppConfig = config_builder
        .build()
        .context("无法加载配置文件")?
        .try_deserialize()
        .context("配置解析失败")?;

    let mut config_lock = CONFIG_MANAGER.write().unwrap();
    *config_lock = Some(config);

    println!("✅ 配置文件已成功加载: {}", config_path);

    Ok(())
}

/// **获取配置**
pub fn get_config() -> Result<AppConfig> {
    CONFIG_MANAGER.read().unwrap()
        .clone()
        .ok_or_else(|| anyhow::anyhow!("配置未初始化"))
}
