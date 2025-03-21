use std::time::Duration;
use bb8::{Pool, RunError};
use bb8_redis::RedisConnectionManager;
use once_cell::sync::OnceCell;
use tracing::info;
use config::app_config::get_config;

/// Redis 连接池错误类型
#[derive(Debug, thiserror::Error)]
pub enum RedisPoolError {

    #[error("Failed to initialize Redis pool: {0}")]
    InitializationError(String),

    #[error("Pool timed out")]
    PoolTimeout,

    #[error("Pool error: {0}")]
    PoolError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Redis operator error: {0}")]
    OperatorError(#[from] bb8_redis::redis::RedisError),

    #[error("User code error: {0}")]
    UserError(String),

    #[error("Custom error: {0}")]
    Custom(String),

}



// 实现从 RunError 到 RedisPoolError 的转换
impl From<RunError<redis::RedisError>> for RedisPoolError {

    fn from(err: RunError<redis::RedisError>) -> Self {
        match err {
            RunError::User(user_err) => RedisPoolError::UserError(user_err.to_string()),
            RunError::TimedOut => RedisPoolError::PoolTimeout
        }
    }
}

/// Redis 连接池配置
#[derive(Debug)]
pub struct RedisPoolConfig {
    pub uri: String,
    pub max_size: u32,
    pub min_idle: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}


/// Redis 连接池管理器
#[derive(Clone)]
pub struct RedisPoolManager {
    pool: Pool<RedisConnectionManager>,
}

impl RedisPoolManager {
    /// 创建新的连接池管理器实例
    async fn new() -> Result<Self, RedisPoolError> {
        let config = Self::get_pool_config()?;

        // 打印掩码后的URI
        let masked_uri = if let Some(_) = config.uri.strip_prefix("redis://:") {
            "redis://:*****".to_string()
        } else {
            config.uri.clone()
        };
        info!("Initializing Redis connection pool with URI: {}", masked_uri);

        let manager = RedisConnectionManager::new(&*config.uri)
            .map_err(|e| RedisPoolError::InitializationError(e.to_string()))?;

        let pool = Pool::builder()
            .max_size(config.max_size)
            .min_idle(Some(config.min_idle))
            .connection_timeout(config.connection_timeout)
            .idle_timeout(Some(config.idle_timeout))
            .build(manager)
            .await
            .map_err(|e| RedisPoolError::InitializationError(e.to_string()))?;

        Ok(Self { pool })
    }

    /// 获取连接池配置
    fn get_pool_config() -> Result<RedisPoolConfig, RedisPoolError> {
        let config = get_config().map_err(|e| RedisPoolError::InitializationError(e.to_string()))?;

        Ok(RedisPoolConfig {
            uri: config.redis.uri,
            max_size: config.redis.pool_max_size as u32,
            min_idle: 5,
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
        })
    }

    /// 获取连接池引用
    pub fn get_pool(&self) -> &Pool<RedisConnectionManager> {
        &self.pool
    }

}

// 全局静态连接池
pub static REDIS_POOL: OnceCell<RedisPoolManager> = OnceCell::new();

// 初始化函数
pub async fn init_redis_pool() -> Result<(), RedisPoolError> {
    if REDIS_POOL.get().is_some() {
        return Ok(());
    }

    let manager = RedisPoolManager::new().await?;
    REDIS_POOL
        .set(manager)
        .map_err(|_| RedisPoolError::InitializationError("Pool already initialized".into()))?;
    Ok(())
}

// 获取连接池
pub fn get_redis_pool_manager() -> Result<&'static RedisPoolManager, RedisPoolError> {
    REDIS_POOL
        .get()
        .ok_or_else(|| RedisPoolError::InitializationError("Redis pool not initialized".into()))
}
