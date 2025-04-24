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
    /// 向列表左侧添加原始字节
    async fn lpush_raw(&self, key: &str, value: Vec<u8>) -> Result<i64>;

    /// 向列表右侧添加原始字节
    async fn rpush_raw(&self, key: &str, value: Vec<u8>) -> Result<i64>;

    /// 从列表左侧弹出原始字节
    async fn lpop_raw(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 从列表右侧弹出原始字节
    async fn rpop_raw(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 获取列表内的原始字节范围
    async fn lrange_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<Vec<u8>>>;

    /// 从列表中移除指定的原始字节值
    async fn lrem_raw(&self, key: &str, count: isize, value: Vec<u8>) -> Result<i64>;

    /// 获取列表长度
    async fn llen(&self, key: &str) -> Result<i64>;

    /// 清空列表
    async fn lclear(&self, key: &str) -> Result<i64>;

    /// 修剪列表
    async fn ltrim(&self, key: &str, start: isize, stop: isize) -> Result<()>;

}

/// 扩展列表操作接口 - 提供泛型方法
#[async_trait::async_trait]
pub trait ListOpsExt: ListOps {
    /// 向列表左侧添加元素
    async fn lpush<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<i64> {
        let serialized = serde_json::to_vec(value)?;
        self.lpush_raw(key, serialized).await
    }

    /// 向列表右侧添加元素
    async fn rpush<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<i64> {
        let serialized = serde_json::to_vec(value)?;
        self.rpush_raw(key, serialized).await
    }

    /// 从列表左侧弹出元素
    async fn lpop<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>> {
        match self.lpop_raw(key).await? {
            Some(data) => {
                let value = serde_json::from_slice(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 从列表右侧弹出元素
    async fn rpop<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>> {
        match self.rpop_raw(key).await? {
            Some(data) => {
                let value = serde_json::from_slice(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 获取列表内的元素范围
    async fn lrange<T: DeserializeOwned + Send>(&self, key: &str, start: isize, stop: isize) -> Result<Vec<T>> {
        let raw_items = self.lrange_raw(key, start, stop).await?;
        let mut items = Vec::with_capacity(raw_items.len());

        for data in raw_items {
            let item: T = serde_json::from_slice(&data)?;
            items.push(item);
        }

        Ok(items)
    }

    /// 从列表中移除指定的值
    async fn lrem<T: Serialize + Send + Sync>(&self, key: &str, count: isize, value: &T) -> Result<i64> {
        let serialized = serde_json::to_vec(value)?;
        self.lrem_raw(key, count, serialized).await
    }
}

// 为所有 ListOps 实现者自动提供 ListOpsExt 功能
impl<T: ListOps + ?Sized> ListOpsExt for T {}

/// Redis列表操作实现
#[derive(Clone)]
pub struct RedisList {
    connection_manager: ConnectionManager,
    prefix: String,
}

impl RedisList {
    /// 创建新的Redis列表操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
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

    async fn lpush_raw(&self, key: &str, value: Vec<u8>) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("LPUSH")
            .arg(&full_key)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        debug!("LPUSH {} -> {}", &full_key, result);
        Ok(result)
    }


    async fn rpush_raw(&self, key: &str, value: Vec<u8>) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("RPUSH")
            .arg(&full_key)
            .arg(value)
            .query_async(&mut conn)
            .await?;

        debug!("RPUSH {} -> {}", full_key, result);
        Ok(result)
    }

    async fn lpop_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("LPOP")
            .arg(&full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }



    async fn rpop_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("RPOP")
            .arg(full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn lrange_raw(&self, key: &str, start: isize, stop: isize) -> Result<Vec<Vec<u8>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("LRANGE")
            .arg(full_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn lrem_raw(&self, key: &str, count: isize, value: Vec<u8>) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("LREM")
            .arg(full_key)
            .arg(count)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn llen(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("LLEN")
            .arg(full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn lclear(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("DEL")
            .arg(full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn ltrim(&self, key: &str, start: isize, stop: isize) -> Result<()> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let _: () = redis::cmd("LTRIM")
            .arg(full_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}
