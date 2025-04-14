use super::error::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use std::time::Duration;
use tracing::debug;

/// 计数器操作接口
#[async_trait::async_trait]
pub trait CounterOps: Send + Sync {
    /// 增加计数器值
    async fn incr(&self, key: &str) -> Result<i64>;

    /// 增加计数器值，指定增量
    async fn incrby(&self, key: &str, increment: i64) -> Result<i64>;

    /// 减少计数器值
    async fn decr(&self, key: &str) -> Result<i64>;

    /// 减少计数器值，指定减量
    async fn decrby(&self, key: &str, decrement: i64) -> Result<i64>;

    /// 获取计数器值
    async fn get(&self, key: &str) -> Result<i64>;

    /// 设置计数器值
    async fn set(&self, key: &str, value: i64) -> Result<()>;

    /// 设置计数器值，同时设置过期时间
    async fn set_with_ttl(&self, key: &str, value: i64, ttl: Duration) -> Result<()>;

    /// 设置过期时间
    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool>;

    /// 重置计数器（删除）
    async fn reset(&self, key: &str) -> Result<bool>;
}

/// Redis计数器操作实现
#[derive(Clone)]
pub struct RedisCounter {
    connection_manager: ConnectionManager,
    prefix: String,
}

impl RedisCounter {
    /// 创建新的Redis计数器操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            prefix: "counter:".to_string(),
        }
    }

    /// 设置键前缀
    pub fn with_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 获取完整的键名
    fn get_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait::async_trait]
impl CounterOps for RedisCounter {
    async fn incr(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.incr(&full_key, 1).await?;

        debug!("INCR {} -> {}", full_key, result);
        Ok(result)
    }

    async fn incrby(&self, key: &str, increment: i64) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.incr(&full_key, increment).await?;

        debug!("INCRBY {} {} -> {}", full_key, increment, result);
        Ok(result)
    }

    async fn decr(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.decr(&full_key, 1).await?;

        debug!("DECR {} -> {}", full_key, result);
        Ok(result)
    }

    async fn decrby(&self, key: &str, decrement: i64) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.decr(&full_key, decrement).await?;

        debug!("DECRBY {} {} -> {}", full_key, decrement, result);
        Ok(result)
    }

    async fn get(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Option<i64> = conn.get(&full_key).await?;

        match result {
            Some(value) => {
                debug!("GET {} -> {}", full_key, value);
                Ok(value)
            }
            None => {
                debug!("GET {} -> 0 (not found)", full_key);
                Ok(0)
            }
        }
    }

    async fn set(&self, key: &str, value: i64) -> Result<()> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let _: () = conn.set(&full_key, value).await?;

        debug!("SET {} {}", full_key, value);
        Ok(())
    }

    async fn set_with_ttl(&self, key: &str, value: i64, ttl: Duration) -> Result<()> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let _: () = conn.set_ex(&full_key, value, ttl.as_secs()).await?;

        debug!("SETEX {} {} {:?}", full_key, value, ttl);
        Ok(())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.expire(&full_key, ttl.as_secs() as i64).await?;

        debug!("EXPIRE {} {:?} -> {}", full_key, ttl, result);
        Ok(result)
    }

    async fn reset(&self, key: &str) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: i64 = conn.del(&full_key).await?;

        debug!("DEL {} (counter reset) -> {}", full_key, result);
        Ok(result > 0)
    }
}
