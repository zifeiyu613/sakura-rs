use std::collections::HashMap;
use std::path::Path;
use config::{Environment, File};
use serde::Deserialize;
use tracing::info;
use crate::errors::ApiError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // pub server: ServerConfig,
    pub mysql: HashMap<String, DatabaseConfig>,
    // pub redis: RedisConfig,
    // 其他配置...
}


#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout: u64,
}

impl Config {
    pub async fn load() -> Result<Self, ApiError> {

        let config_path = dotenvy::var("CONFIG_PATH").unwrap_or_else(|_| "./yice-api/config/".to_string());

        info!("Loading configuration from {}", &config_path);

        let builder = config::Config::builder()
            .add_source(File::from(Path::new(&config_path).join("application.toml")))
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
}