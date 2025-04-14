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
    /// 从缓存获取值
    async fn get<T: DeserializeOwned + Send + 'static>(&self, key: &str) -> Result<Option<T>>;

    /// 设置缓存值
    async fn set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()>;

    /// 根据键删除缓存
    async fn delete(&self, key: &str) -> Result<bool>;

    /// 检查键是否存在
    async fn exists(&self, key: &str) -> Result<bool>;

    /// 使用模式删除多个键
    async fn delete_by_pattern(&self, pattern: &str) -> Result<u64>;

    /// 获取或计算缓存值
    async fn get_or_set<T, F, Fut>(&self, key: &str, ttl: Option<Duration>, f: F) -> Result<T>
    where
        T: DeserializeOwned + Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T>> + Send;
}

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
    async fn get<T: DeserializeOwned + Send + 'static>(&self, key: &str) -> Result<Option<T>> {
        let prefixed_key = format!("{}{}", self.prefix, key);

        let mut conn = self.connection_manager.clone();
        let result: Option<String> = conn.get(&prefixed_key).await?;

        match result {
            Some(data) => {
                debug!("Cache hit: {}", prefixed_key);
                let value = self.serializer.deserialize(&data)?;
                Ok(Some(value))
            }
            None => {
                debug!("Cache miss: {}", prefixed_key);
                Ok(None)
            }
        }
    }

    async fn set<T: Serialize + Send + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();

        match ttl {
            Some(duration) => {
                let _: () = conn.set_ex(&prefixed_key, serialized, duration.as_secs())
                    .await?;
                debug!("Set cache with TTL: {} ({:?})", prefixed_key, duration);
            }
            None => {
                let _: () = conn.set(&prefixed_key, serialized).await?;
                debug!("Set cache without TTL: {}", prefixed_key);
            }
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let mut conn = self.connection_manager.clone();

        let result: i64 = conn.del(&prefixed_key).await?;

        if result > 0 {
            debug!("Deleted cache key: {}", prefixed_key);
        }

        Ok(result > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        let mut conn = self.connection_manager.clone();

        let result: bool = conn.exists(&prefixed_key).await?;

        Ok(result)
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

    async fn get_or_set<T, F, Fut>(&self, key: &str, ttl: Option<Duration>, f: F) -> Result<T>
    where
        T: DeserializeOwned + Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T>> + Send,
    {
        // 先尝试从缓存获取
        if let Some(value) = self.get::<T>(key).await? {
            return Ok(value);
        }

        // 缓存未命中，执行函数计算值
        let value = f().await?;

        // 缓存计算结果
        if let Err(e) = self.set(key, &value, ttl).await {
            warn!("Failed to cache value for key {}: {}", key, e);
            // 不要因为缓存失败而阻止返回结果
        }

        Ok(value)
    }
}
