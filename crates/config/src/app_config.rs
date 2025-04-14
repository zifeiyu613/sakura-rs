use config::{Config, Environment, File};
use errors::error::ConfigError;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::log_config::LogConfig;

/// APP_CONFIG
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub mysql: HashMap<String, MysqlConfig>,
    pub redis: RedisConfig,
    pub rabbit: MqConfig,

    pub log: LogConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MysqlConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout: u64,
}


#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    /// The URI for connecting to the Redis server. For example:
    /// <redis://127.0.0.1/>
    pub uri: String,

    pub pool_max_size: usize,

    /// The new connection will time out operations after `response_timeout` has passed.
    pub response_timeout: Option<u64>,

    /// Each connection attempt to the server will time out after `connection_timeout`.
    pub connection_timeout: Option<u64>,

    /// number_of_retries times, with an exponentially increasing delay
    pub number_of_retries: Option<usize>,

    /// The resulting duration is calculated by taking the base to the `n`-th power,
    /// where `n` denotes the number of past attempts.
    pub exponent_base: Option<u64>,

    /// A multiplicative factor that will be applied to the retry delay.
    /// For example, using a factor of `1000` will make each delay in units of seconds.
    pub factor: Option<u64>,

    /// Apply a maximum delay between connection attempts. The delay between attempts won't be longer than max_delay milliseconds.
    pub max_delay: Option<u64>,
}


/// ** RabbitMQ **
#[derive(Debug, Deserialize, Clone)]
pub struct MqConfig {
    pub uri: String,
    pub pool_max_size: usize,
    pub exchange: Option<String>,
}

/// **全局唯一的 `CONFIG_MANAGER`**
///
/// 在 Rust 中，RwLockReadGuard 是一种非 Send 类型。RwLock 确保在多线程中只有一个线程可以修改数据，
/// 但它默认不允许 ReadGuard 类型在线程间传递。这是因为，读取锁在释放之前对数据的持有是不可移动的，也不允许传递。

static CONFIG_MANAGER: Lazy<Arc<RwLock<Option<AppConfig>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));


/// **初始化配置**
pub fn load_config(config_path: Option<&str>) -> Result<(), ConfigError> {

    let config_path: &str = config_path.unwrap_or("config.toml");

    // let config_path: &str = config_path.unwrap_or_else(|| {
    //     env::var("APP_CONFIG_PATH").unwrap_or("config.toml".to_string()).as_str()
    // });

    let config_builder = Config::builder()
        .add_source(File::with_name(config_path))
        .add_source(Environment::with_prefix("APP")); // 支持环境变量覆盖

    // 尝试加载配置文件
    let config: AppConfig = config_builder
        .build()
        .map_err(|e| ConfigError::LoadError(e.to_string()))?  // 使用 `ConfigError::LoadError` 错误
        .try_deserialize()
        .map_err(|_| ConfigError::ParseError)?;  // 使用 `ConfigError::ParseError` 错误

    // 锁定并写入配置
    let mut config_lock = CONFIG_MANAGER.write().unwrap();
    *config_lock = Some(config);

    println!("✅ 配置文件已成功加载: {}", config_path);

    Ok(())
}


/// **获取配置**
pub fn get_config() -> Result<AppConfig, ConfigError> {
    // 获取配置的读锁
    let config_lock = CONFIG_MANAGER.read().unwrap();
    config_lock
        .clone()
        .ok_or(ConfigError::Uninitialized)  // 如果未初始化，返回 `ConfigError::Uninitialized`
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_config() {
        let path = "/Users/will/RustroverProjects/sakura/sakura-api/config.toml";
        load_config(Some(path)).unwrap();
        let app_config = get_config().unwrap();
        println!("{:?}", &app_config);
        let app_config = get_config().unwrap();
        println!("{:?}", &app_config.mysql);
        println!("{:?}", &app_config.redis);
    }
}