/// 哈希表操作
use super::error::{RedisError, Result};
use super::serializer::{JsonSerializer, RedisSerializer};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// 哈希表操作接口
#[async_trait::async_trait]
pub trait HashOps: Send + Sync {
    /// 设置哈希表字段
    async fn hset<T: Serialize + Send + Sync>(&self, key: &str, field: &str, value: &T) -> Result<bool>;

    /// 获取哈希表字段
    async fn hget<T: DeserializeOwned + Send>(&self, key: &str, field: &str) -> Result<Option<T>>;

    /// 删除哈希表字段
    async fn hdel(&self, key: &str, field: &str) -> Result<bool>;

    /// 检查哈希表字段是否存在
    async fn hexists(&self, key: &str, field: &str) -> Result<bool>;

    /// 获取哈希表所有字段
    async fn hgetall<T: DeserializeOwned + Send>(&self, key: &str) -> Result<HashMap<String, T>>;

    /// 获取哈希表指定字段
    async fn hmget<T: DeserializeOwned + Send>(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<T>>>;

    /// 设置多个哈希表字段
    async fn hmset<T: Serialize + Send + Sync>(&self, key: &str, map: &HashMap<String, T>) -> Result<()>;

    /// 获取哈希表字段数量
    async fn hlen(&self, key: &str) -> Result<i64>;

    /// 获取所有字段名
    async fn hkeys(&self, key: &str) -> Result<Vec<String>>;

    /// 对哈希表字段值进行递增
    async fn hincrby(&self, key: &str, field: &str, increment: i64) -> Result<i64>;
}

/// Redis哈希表操作实现
#[derive(Clone)]
pub struct RedisHash {
    connection_manager: ConnectionManager,
    serializer: JsonSerializer,
    prefix: String,
}

impl RedisHash {
    /// 创建新的Redis哈希表操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            serializer: JsonSerializer,
            prefix: "hash:".to_string(),
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
impl HashOps for RedisHash {

    async fn hset<T: Serialize + Send + Sync>(&self, key: &str, field: &str, value: &T) -> Result<bool> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result: i32 = conn.hset(&full_key, field, serialized).await?;

        debug!("HSET {} {} -> {}", full_key, field, result);
        Ok(result == 1)
    }

    async fn hget<T: DeserializeOwned + Send>(&self, key: &str, field: &str) -> Result<Option<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Option<String> = conn.hget(&full_key, field).await?;

        match result {
            Some(data) => {
                debug!("HGET {} {} -> data", full_key, field);
                let value = self.serializer.deserialize(&data)?;
                Ok(Some(value))
            }
            None => {
                debug!("HGET {} {} -> None", full_key, field);
                Ok(None)
            }
        }
    }

    async fn hdel(&self, key: &str, field: &str) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: i32 = conn.hdel(&full_key, field).await?;

        debug!("HDEL {} {} -> {}", full_key, field, result);
        Ok(result == 1)
    }

    async fn hexists(&self, key: &str, field: &str) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.hexists(&full_key, field).await?;

        debug!("HEXISTS {} {} -> {}", full_key, field, result);
        Ok(result)
    }

    async fn hgetall<T: DeserializeOwned + Send>(&self, key: &str) -> Result<HashMap<String, T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: HashMap<String, String> = conn.hgetall(&full_key).await?;
        let mut map = HashMap::with_capacity(result.len());

        for (field, data) in result {
            map.insert(field.clone(), self.serializer.deserialize(&data)?);
        }

        debug!("HGETALL {} -> {} fields", full_key, map.len());
        Ok(map)
    }

    async fn hmget<T: DeserializeOwned + Send>(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<T>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<Option<String>> = conn.hget(&full_key, fields).await?;
        let mut values = Vec::with_capacity(result.len());

        for item in result {
            match item {
                Some(data) => values.push(Some(self.serializer.deserialize(&data)?)),
                None => values.push(None),
            }
        }

        debug!("HMGET {} ({} fields) -> {} results", full_key, fields.len(), values.len());
        Ok(values)
    }

    async fn hmset<T: Serialize + Send + Sync>(&self, key: &str, map: &HashMap<String, T>) -> Result<()> {
        if map.is_empty() {
            return Ok(());
        }

        let full_key = self.get_key(key);
        let mut serialized_map = Vec::with_capacity(map.len());

        for (field, value) in map {
            serialized_map.push((field.clone(), self.serializer.serialize(value)?));
        }

        let mut conn = self.connection_manager.clone();
        let _: () = conn.hset_multiple(&full_key, &serialized_map).await?;

        debug!("HMSET {} ({} fields)", full_key, map.len());
        Ok(())
    }

    async fn hlen(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.hlen(&full_key).await?;

        debug!("HLEN {} -> {}", full_key, result);
        Ok(result)
    }

    async fn hkeys(&self, key: &str) -> Result<Vec<String>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.hkeys(&full_key).await?;

        // debug!("HKEYS {} -> {} keys", full_key, result.len());
        Ok(result)
    }

    async fn hincrby(&self, key: &str, field: &str, increment: i64) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.hincr(&full_key, field, increment).await?;

        debug!("HINCRBY {} {} {} -> {}", full_key, field, increment, result);
        Ok(result)
    }
}
