use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {

    level: String,

}


impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "INFO".to_string(),
        }
    }
}