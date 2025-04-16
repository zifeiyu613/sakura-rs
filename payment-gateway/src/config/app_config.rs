use std::path::Path;
use serde::Deserialize;
use anyhow::Result;
use config::{Config, Environment, File};
use tracing::info;
use crate::utils::error::AppError;

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64, // 秒
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64, // 秒
}

#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: u64, // 秒
    pub cache_ttl: u64,          // 秒
}

#[derive(Clone, Debug, Deserialize)]
pub struct RabbitMQConfig {
    pub url: String,
    pub retry_interval: u64, // 秒
    pub max_retries: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SecurityConfig {
    pub api_key_header: String,
    pub jwt_secret: String,
    pub jwt_expiry: u64, // 分钟
    pub api_signing_timeout: u64, // 秒
}

#[derive(Clone, Debug, Deserialize)]
pub struct PaymentConfig {
    pub transaction_timeout: u64, // 分钟
    pub refund_timeout: u64,      // 分钟
    pub webhook_retry_count: u32,
    pub webhook_retry_interval: u64, // 秒
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub json_format: bool,
    pub file_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rabbitmq: RabbitMQConfig,
    pub security: SecurityConfig,
    pub payment: PaymentConfig,
    pub logging: LoggingConfig,
    pub environment: String,
    pub service_name: String,
}

impl AppConfig {
    pub fn load() -> std::result::Result<Self, AppError> {

        let config_path = dotenvy::var("CONFIG_PATH").unwrap_or_else(|_| {
            format!("{}/config/application.toml", env!("CARGO_MANIFEST_DIR"))
        });

        info!("Loading configuration from {}", &config_path);

        let builder = Config::builder()
            .add_source(File::from(Path::new(&config_path)))
            .add_source(Environment::with_prefix("APP").separator("__"));

        let config = builder.build()?;
        let config: AppConfig = config.try_deserialize()?;

        Ok(config)
    }

    pub fn is_development(&self) -> bool {
        self.environment.to_lowercase() == "development"
    }

    pub fn is_production(&self) -> bool {
        self.environment.to_lowercase() == "production"
    }

    pub fn is_testing(&self) -> bool {
        self.environment.to_lowercase() == "testing"
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
                request_timeout: 30,
            },
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/payment".to_string(),
                max_connections: 10,
                connection_timeout: 5,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                connection_timeout: 5,
                cache_ttl: 3600,
            },
            rabbitmq: RabbitMQConfig {
                url: "amqp://guest:guest@localhost:5672".to_string(),
                retry_interval: 5,
                max_retries: 3,
            },
            security: SecurityConfig {
                api_key_header: "X-API-Key".to_string(),
                jwt_secret: "default_secret_please_change_in_production".to_string(),
                jwt_expiry: 60,
                api_signing_timeout: 300,
            },
            payment: PaymentConfig {
                transaction_timeout: 30,
                refund_timeout: 60,
                webhook_retry_count: 3,
                webhook_retry_interval: 60,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                json_format: false,
                file_path: None,
            },
            environment: "development".to_string(),
            service_name: "payment-gateway".to_string(),
        }
    }
}
