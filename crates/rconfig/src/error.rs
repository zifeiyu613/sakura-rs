use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlParse(String),

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Failed to deserialize configuration: {0}")]
    Deserialization(#[from] config::ConfigError),

    #[error("Failed to parse environment variables: {0}")]
    EnvParse(String),

    #[error("Missing required configuration key: {0}")]
    MissingKey(String),

    #[error("Invalid configuration value for key '{key}': {message}")]
    InvalidValue {
        key: String,
        message: String,
    },

    #[error("Configuration source not found: {0}")]
    SourceNotFound(PathBuf),

    #[error("Conflicting configuration values: {0}")]
    ConflictingValues(String),

    #[error("Failed to merge configuration sources: {0}")]
    MergeFailure(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Failed to load CLI args: {0}")]
    CliArgsError(String),

    #[error("Unknown configuration error: {0}")]
    Other(String),
}

impl From<&str> for ConfigError {
    fn from(msg: &str) -> Self {
        ConfigError::Other(msg.to_string())
    }
}

impl From<String> for ConfigError {
    fn from(msg: String) -> Self {
        ConfigError::Other(msg)
    }
}

impl From<yaml_rust::ScanError> for ConfigError {
    fn from(err: yaml_rust::ScanError) -> Self {
        ConfigError::YamlParse(err.to_string())
    }
}

// 实用函数，用于在配置上下文中处理错误
pub fn invalid_value<T>(key: &str, message: &str) -> Result<T, ConfigError> {
    Err(ConfigError::InvalidValue {
        key: key.to_string(),
        message: message.to_string(),
    })
}

pub fn missing_key<T>(key: &str) -> Result<T, ConfigError> {
    Err(ConfigError::MissingKey(key.to_string()))
}
