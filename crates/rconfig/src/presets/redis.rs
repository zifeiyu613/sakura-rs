use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {

    pub uri: String,

    #[serde(default = "default_max_size")]
    pub max_size: u32,
    #[serde(default = "default_min_idle")]
    pub min_idle: u32,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: Duration,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: Duration,

}

fn default_connection_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_idle_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_max_size() -> u32 {
    10
}

fn default_min_idle() -> u32 {
    1
}