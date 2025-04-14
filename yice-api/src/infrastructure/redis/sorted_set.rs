/// 有序集合操作

use super::error::Result;
use super::serializer::{JsonSerializer, RedisSerializer};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

/// 有序集合元素
#[derive(Debug, Clone)]
pub struct ScoredValue<T> {
    pub score: f64,
    pub value: T,
}

/// 有序集合操作接口
#[async_trait::async_trait]
pub trait SortedSetOps: Send + Sync {
    /// 添加元素到有序集合
    async fn zadd<T: Serialize + Send + Sync>(&self, key: &str, value: &T, score: f64) -> Result<bool>;

    /// 获取元素的分数
    async fn zscore<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<Option<f64>>;

    /// 获取指定排名范围的元素
    async fn zrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>>;

    /// 获取指定排名范围的元素（带分数）
    async fn zrange_with_scores<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<ScoredValue<T>>>;

    /// 获取指定分数范围的元素
    async fn zrangebyscore<T: DeserializeOwned + Send>(&self, key: &str, min: f64, max: f64) -> Result<Vec<T>>;

    /// 获取指定分数范围的元素（带分数）
    async fn zrangebyscore_with_scores<T: DeserializeOwned + Send>(&self, key: &str, min: f64, max: f64) -> Result<Vec<ScoredValue<T>>>;

    /// 获取有序集合大小
    async fn zcard(&self, key: &str) -> Result<i64>;

    /// 删除元素
    async fn zrem<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<bool>;

    /// 获取元素排名
    async fn zrank<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<Option<i64>>;

    /// 增加元素分数
    async fn zincrby<T: Serialize + Send + Sync>(&self, key: &str, value: &T, increment: f64) -> Result<f64>;

    /// 获取排名范围的元素（按降序）
    async fn zrevrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>>;
}

/// Redis有序集合操作实现
#[derive(Clone)]
pub struct RedisSortedSet {
    connection_manager: ConnectionManager,
    serializer: JsonSerializer,
    prefix: String,
}

impl RedisSortedSet {
    /// 创建新的Redis有序集合操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            serializer: JsonSerializer,
            prefix: "sortedset:".to_string(),
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
impl SortedSetOps for RedisSortedSet {
    async fn zadd<T: Serialize + Send + Sync>(&self, key: &str, value: &T, score: f64) -> Result<bool> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result: i32 = conn.zadd(&full_key, &serialized, score).await?;

        debug!("ZADD {} {} {} -> {}", full_key, score, &serialized, result);
        Ok(result == 1)
    }

    async fn zscore<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<Option<f64>> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result: Option<f64> = conn.zscore(&full_key, &serialized).await?;

        match &result {
            Some(score) => debug!("ZSCORE {} {} -> {}", full_key, &serialized, score),
            None => debug!("ZSCORE {} {} -> None", full_key, &serialized),
        }

        Ok(result)
    }

    async fn zrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<String> = conn.zrange(&full_key, start, stop).await?;

        let mut items = Vec::with_capacity(result.len());
        for data in result {
            items.push(self.serializer.deserialize(&data)?);
        }

        debug!("ZRANGE {} {} {} -> {} items", full_key, start, stop, items.len());
        Ok(items)
    }

    async fn zrange_with_scores<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<ScoredValue<T>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<(String, f64)> = conn.zrange_withscores(&full_key, start, stop).await?;

        let mut items = Vec::with_capacity(result.len());
        for (data, score) in result {
            items.push(ScoredValue {
                score,
                value: self.serializer.deserialize(&data)?,
            });
        }

        debug!("ZRANGE {} {} {} WITHSCORES -> {} items", full_key, start, stop, items.len());
        Ok(items)
    }

    async fn zrangebyscore<T: DeserializeOwned + Send>(&self, key: &str, min: f64, max: f64) -> Result<Vec<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<String> = conn.zrangebyscore(&full_key, min, max).await?;

        let mut items = Vec::with_capacity(result.len());
        for data in result {
            items.push(self.serializer.deserialize(&data)?);
        }

        debug!("ZRANGEBYSCORE {} {} {} -> {} items", full_key, min, max, items.len());
        Ok(items)
    }

    async fn zrangebyscore_with_scores<T: DeserializeOwned + Send>(&self, key: &str, min: f64, max: f64) -> Result<Vec<ScoredValue<T>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<(String, f64)> = conn.zrangebyscore_withscores(&full_key, min, max).await?;

        let mut items = Vec::with_capacity(result.len());
        for (data, score) in result {
            items.push(ScoredValue {
                score,
                value: self.serializer.deserialize(&data)?,
            });
        }

        debug!("ZRANGEBYSCORE {} {} {} WITHSCORES -> {} items", full_key, min, max, items.len());
        Ok(items)
    }

    async fn zcard(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.zcard(&full_key).await?;

        debug!("ZCARD {} -> {}", full_key, result);
        Ok(result)
    }

    async fn zrem<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<bool> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result: i32 = conn.zrem(&full_key, &serialized).await?;

        debug!("ZREM {} {} -> {}", full_key, &serialized, result);
        Ok(result == 1)
    }

    async fn zrank<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<Option<i64>> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result: Option<i64> = conn.zrank(&full_key, &serialized).await?;

        match &result {
            Some(rank) => debug!("ZRANK {} {} -> {}", full_key, &serialized, rank),
            None => debug!("ZRANK {} {} -> None", full_key, &serialized),
        }

        Ok(result)
    }

    async fn zincrby<T: Serialize + Send + Sync>(&self, key: &str, value: &T, increment: f64) -> Result<f64> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result = conn.zincr(&full_key, &serialized, increment).await?;

        debug!("ZINCRBY {} {} {} -> {}", full_key, &serialized, increment, result);
        Ok(result)
    }

    async fn zrevrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<String> = conn.zrevrange(&full_key, start, stop).await?;

        let mut items = Vec::with_capacity(result.len());
        for data in result {
            items.push(self.serializer.deserialize(&data)?);
        }

        debug!("ZREVRANGE {} {} {} -> {} items", full_key, start, stop, items.len());
        Ok(items)
    }
}
