//! 数据库错误类型定义

use thiserror::Error;

/// 数据库操作错误
#[derive(Error, Debug)]
pub enum DbError {
    /// 数据库连接错误
    #[error("数据库连接错误: {0}")]
    ConnectionError(String),

    /// 数据库查询错误
    #[error("数据库查询错误: {0}")]
    QueryError(#[from] sqlx::Error),

    /// 数据库配置错误
    #[error("数据库配置错误: {0}")]
    ConfigError(String),

    /// 数据库池错误
    #[error("数据库池错误: {0}")]
    PoolError(String),

    /// 不支持的数据库类型
    #[error("不支持的数据库类型: {0}")]
    UnsupportedDbType(String),

    // 数据库URL解析错误
    // #[error("数据库URL解析错误: {0}")]
    // UrlParseError(#[from] url::ParseError),

    /// 数据源不存在
    #[error("数据源不存在: {0}")]
    SourceNotFound(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 数据库操作结果类型
pub type Result<T> = std::result::Result<T, DbError>;

impl From<&str> for DbError {
    fn from(message: &str) -> Self {
        DbError::Other(message.to_string())
    }
}

impl From<String> for DbError {
    fn from(message: String) -> Self {
        DbError::Other(message)
    }
}

impl From<rconfig::error::ConfigError> for DbError {
    fn from(err: rconfig::error::ConfigError) -> Self {
        DbError::ConfigError(err.to_string())
    }
}
