use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use config::{Environment, File};
use serde::Deserialize;
use tracing::info;
use crate::errors::ApiError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // pub server: ServerConfig,
    pub mysql: HashMap<String, DatabaseConfig>,
    pub redis: RedisPoolConfig,
    // 其他配置...
}


#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout: u64,
}


/// Redis 连接池配置
#[derive(Debug, Deserialize, Clone)]
pub struct RedisPoolConfig {
    pub uri: String,
    pub max_size: u32,
    // pub min_idle: u32,
    // pub connection_timeout: Duration,
    // pub idle_timeout: Duration,
}


impl Config {
    pub async fn load() -> Result<Self, ApiError> {

        let config_path = dotenvy::var("CONFIG_PATH").unwrap_or_else(|_| {
            format!("{}/config/application.toml", env!("CARGO_MANIFEST_DIR"))
        });

        info!("Loading configuration from {}", &config_path);

        let builder = config::Config::builder()
            .add_source(File::from(Path::new(&config_path)))
            .add_source(Environment::with_prefix("APP").separator("__"));

        let config = builder.build()?;
        let config: Config = config.try_deserialize()?;

        Ok(config)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn should_load_config() {
        let config = Config::load().await.unwrap();

        println!("{:?}", config);

        assert_eq!(config.mysql.len(), 2);
    }

    #[test]
    fn print_cargo_dir() {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rconfig");

        println!("{:?}", assets_dir.join("application.toml"));
    }
}