use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub host: String,
    pub port: i64,
    pub debug: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            debug: false,
        }
    }
}

