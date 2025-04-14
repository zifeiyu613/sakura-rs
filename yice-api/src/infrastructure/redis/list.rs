/// 列表操作

use super::error::{RedisError, Result};
use super::serializer::{JsonSerializer, RedisSerializer};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

/// 列表操作接口
#[async_trait::async_trait]
pub trait ListOps: Send + Sync {
    /// 向列表左侧添加元素
    async fn lpush<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<i64>;

    /// 向列表右侧添加元素
    async fn rpush<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<i64>;

    /// 从列表左侧弹出元素
    async fn lpop<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>>;

    /// 从列表右侧弹出元素
    async fn rpop<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>>;

    /// 获取列表内的元素范围
    async fn lrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>>;

    /// 获取列表长度
    async fn llen(&self, key: &str) -> Result<i64>;

    /// 清空列表
    async fn lclear(&self, key: &str) -> Result<i64>;

    /// 从列表中移除指定的值
    async fn lrem<T: Serialize + Send + Sync>(&self, key: &str, count: isize, value: &T) -> Result<i64>;

    /// 修剪列表
    async fn ltrim(&self, key: &str, start: isize, stop: isize) -> Result<()>;
}

/// Redis列表操作实现
#[derive(Clone)]
pub struct RedisList {
    connection_manager: ConnectionManager,
    serializer: JsonSerializer,
    prefix: String,
}

impl RedisList {
    /// 创建新的Redis列表操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            serializer: JsonSerializer,
            prefix: "list:".to_string(),
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
impl ListOps for RedisList {
    async fn lpush<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<i64> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result = conn.lpush(&full_key, serialized).await?;

        debug!("LPUSH {} -> {}", full_key, result);
        Ok(result)
    }

    async fn rpush<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<i64> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result = conn.rpush(&full_key, serialized).await?;

        debug!("RPUSH {} -> {}", full_key, result);
        Ok(result)
    }

    async fn lpop<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Option<String> = conn.lpop(&full_key, None).await?;

        match result {
            Some(data) => {
                debug!("LPOP {} -> data", full_key);
                let value = self.serializer.deserialize(&data)?;
                Ok(Some(value))
            }
            None => {
                debug!("LPOP {} -> None", full_key);
                Ok(None)
            }
        }
    }

    async fn rpop<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Option<String> = conn.rpop(&full_key, None).await?;

        match result {
            Some(data) => {
                debug!("RPOP {} -> data", full_key);
                let value = self.serializer.deserialize(&data)?;
                Ok(Some(value))
            }
            None => {
                debug!("RPOP {} -> None", full_key);
                Ok(None)
            }
        }
    }

    async fn lrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result: Vec<String> = conn.lrange(&full_key, start, stop).await?;

        let mut items = Vec::with_capacity(result.len());
        for data in result {
            items.push(self.serializer.deserialize(&data)?);
        }

        debug!("LRANGE {} {} {} -> {} items", full_key, start, stop, items.len());
        Ok(items)
    }

    async fn llen(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.llen(&full_key).await?;

        debug!("LLEN {} -> {}", full_key, result);
        Ok(result)
    }

    async fn lclear(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let result = conn.del(&full_key).await?;

        debug!("DEL {} (list clear) -> {}", full_key, result);
        Ok(result)
    }

    async fn lrem<T: Serialize + Send + Sync>(&self, key: &str, count: isize, value: &T) -> Result<i64> {
        let full_key = self.get_key(key);
        let serialized = self.serializer.serialize(value)?;

        let mut conn = self.connection_manager.clone();
        let result = conn.lrem(&full_key, count, serialized).await?;

        debug!("LREM {} {} -> {}", full_key, count, result);
        Ok(result)
    }

    async fn ltrim(&self, key: &str, start: isize, stop: isize) -> Result<()> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();

        let _: () = conn.ltrim(&full_key, start, stop).await?;

        debug!("LTRIM {} {} {}", full_key, start, stop);
        Ok(())
    }
}
