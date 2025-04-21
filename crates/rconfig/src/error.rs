//! 配置错误类型

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置加载错误: {0}")]
    LoadError(#[from] config::ConfigError),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("缺少必要配置: {0}")]
    MissingConfig(String),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("URL解析错误: {0}")]
    UrlParseError(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
