use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("数据库连接失败: {0}")]
    ConnectionError(String),
    #[error("数据库查询失败")]
    QueryError,
}