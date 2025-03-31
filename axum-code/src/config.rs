use serde::Deserialize;
use std::env;
use std::path::Path;

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rabbitmq: RabbitMqConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub third_party: ThirdPartyConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMqConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThirdPartyConfig {
    pub base_url: String,
    pub api_key: String,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config".to_string());

        let builder = config::Config::builder()
            .add_source(config::File::from(Path::new(&config_path).join("default")))
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        let config = builder.build()?;
        let config: Config = config.try_deserialize()?;

        Ok(config)
    }
}
