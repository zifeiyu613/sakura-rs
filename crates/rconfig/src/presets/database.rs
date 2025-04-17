use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_secs: u32,
    pub idle_timeout_secs: u32,
}


impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            url: "".into(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout_secs: 30,
            idle_timeout_secs: 60,
        }
    }
}
