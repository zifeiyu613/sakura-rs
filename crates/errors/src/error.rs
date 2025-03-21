use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {

    #[error("配置文件加载失败: {0}")]
    LoadError(String),

    #[error("配置解析失败")]
    ParseError,

    #[error("配置未初始化")]
    Uninitialized,
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("数据库连接失败: {0}")]
    ConnectionError(String),
    #[error("数据库查询失败")]
    QueryError,
}