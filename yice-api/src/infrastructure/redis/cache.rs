use super::error::{RedisError, Result};
use super::serializer::{JsonSerializer, RedisSerializer};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// 通用缓存接口
#[async_trait::async_trait]
pub trait Cache: Send + Sync {

    /// 从缓存获取原始值
    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 设置缓存原始值
    async fn set_raw(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;

    /// 根据键删除缓存
    async fn delete(&self, key: &str) -> Result<bool>;

    /// 检查键是否存在
    async fn exists(&self, key: &str) -> Result<bool>;

    /// 使用模式删除多个键
    async fn delete_by_pattern(&self, pattern: &str) -> Result<u64>;

}

/// 添加默认实现的扩展trait
#[async_trait::async_trait]
pub trait CacheExt: Cache {

    /// 从缓存获取并反序列化值
    async fn get<T: DeserializeOwned + Send + 'static>(&self, key: &str) -> Result<Option<T>> {
        match self.get_raw(key).await? {
            Some(data) => {
                let value: T = serde_json::from_slice(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 序列化并设置缓存值
    async fn set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let serialized = serde_json::to_vec(value)?;
        self.set_raw(key, serialized, ttl).await
    }

    /// 获取或计算缓存值
    async fn get_or_set<T, F, Fut>(&self, key: &str, ttl: Option<Duration>, f: F) -> Result<T>
    where
        T: DeserializeOwned + Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T>> + Send,
    {
        if let Some(value) = self.get::<T>(key).await? {
            return Ok(value);
        }

        let value = f().await?;
        self.set(key, &value, ttl).await?;
        Ok(value)
    }

}

// 自动为所有Cache实现者添加CacheExt功能
impl<T: Cache + ?Sized> CacheExt for T {}

/// Redis缓存实现
#[derive(Clone)]
pub struct RedisCache {
    connection_manager: ConnectionManager,
    serializer: JsonSerializer,
    prefix: String,
}

/// Redis缓存构建器
pub struct CacheBuilder {
    connection_manager: ConnectionManager,
    serializer: JsonSerializer,
    prefix: String,
}

impl CacheBuilder {
    /// 创建新的缓存构建器
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            serializer: JsonSerializer,
            prefix: "cache:".to_string(),
        }
    }


    /// 设置缓存键前缀
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 构建Redis缓存
    pub fn build(self) -> RedisCache {
        RedisCache {
            connection_manager: self.connection_manager,
            serializer: self.serializer,
            prefix: self.prefix,
        }
    }
}

#[async_trait::async_trait]
impl Cache for RedisCache {

    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let mut conn = self.connection_manager.clone();
        let result: Option<Vec<u8>> = redis::cmd("GET").arg(prefixed_key).query_async(&mut conn).await?;
        Ok(result)
    }



    async fn set_raw(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let mut conn = self.connection_manager.clone();
        match ttl {
            Some(ttl) => {
                let _:() = redis::cmd("SETEX")
                        .arg(prefixed_key)
                        .arg(ttl.as_secs())
                        .arg(value)
                        .query_async(&mut conn)
                        .await?;
            }
            None => {
                let _:() = redis::cmd("SET").arg(key).arg(value).query_async(&mut conn).await?;
            }
        };
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let mut conn = self.connection_manager.clone();
        let count: i32 = redis::cmd("DEL").arg(prefixed_key).query_async(&mut conn).await?;
        Ok(count > 0)
    }



    async fn exists(&self, key: &str) -> Result<bool> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let mut conn = self.connection_manager.clone();

        let exists: i32 = redis::cmd("EXISTS").arg(prefixed_key).query_async(&mut conn).await?;
        Ok(exists > 0)
    }

    async fn delete_by_pattern(&self, pattern: &str) -> Result<u64> {
        let prefixed_pattern = format!("{}{}", self.prefix, pattern);
        let mut conn = self.connection_manager.clone();

        // 查找匹配的键
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&prefixed_pattern)
            .query_async(&mut conn)
            .await?;

        if keys.is_empty() {
            debug!("No keys matched pattern: {}", prefixed_pattern);
            return Ok(0);
        }

        // 删除匹配的键
        let count: u64 = redis::cmd("DEL")
            .arg(keys.clone())
            .query_async(&mut conn)
            .await?;

        debug!("Deleted {} keys with pattern: {}", count, prefixed_pattern);

        Ok(count)


    }

}
