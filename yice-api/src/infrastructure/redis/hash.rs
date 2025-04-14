/// 哈希表操作
use super::error::Result;
use redis::aio::ConnectionManager;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;


/// 哈希表操作接口
#[async_trait::async_trait]
pub trait HashOps: Send + Sync {
    /// 设置哈希表字段原始字节
    async fn hset_raw(&self, key: &str, field: &str, value: Vec<u8>) -> Result<bool>;

    /// 获取哈希表字段原始字节
    async fn hget_raw(&self, key: &str, field: &str) -> Result<Option<Vec<u8>>>;

    /// 删除哈希表字段
    async fn hdel(&self, key: &str, field: &str) -> Result<bool>;

    /// 检查哈希表字段是否存在
    async fn hexists(&self, key: &str, field: &str) -> Result<bool>;

    /// 获取哈希表所有字段原始字节
    async fn hgetall_raw(&self, key: &str) -> Result<HashMap<String, Vec<u8>>>;

    /// 获取哈希表指定字段原始字节
    async fn hmget_raw(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<Vec<u8>>>>;

    /// 设置多个哈希表字段原始字节
    async fn hmset_raw(&self, key: &str, map: HashMap<String, Vec<u8>>) -> Result<()>;

    /// 获取哈希表字段数量
    async fn hlen(&self, key: &str) -> Result<i64>;

    /// 获取所有字段名
    async fn hkeys(&self, key: &str) -> Result<Vec<String>>;

    /// 对哈希表字段值进行递增
    async fn hincrby(&self, key: &str, field: &str, increment: i64) -> Result<i64>;
}

/// 扩展哈希操作接口 - 提供泛型方法
#[async_trait::async_trait]
pub trait HashOpsExt: HashOps {
    /// 设置哈希表字段
    async fn hset<T: Serialize + Send + Sync>(&self, key: &str, field: &str, value: &T) -> Result<bool> {
        let serialized = serde_json::to_vec(value)?;
        self.hset_raw(key, field, serialized).await
    }

    /// 获取哈希表字段
    async fn hget<T: DeserializeOwned + Send>(&self, key: &str, field: &str) -> Result<Option<T>> {
        match self.hget_raw(key, field).await? {
            Some(data) => {
                let value = serde_json::from_slice(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 获取哈希表所有字段
    async fn hgetall<T: DeserializeOwned + Send>(&self, key: &str) -> Result<HashMap<String, T>> {
        let raw_map = self.hgetall_raw(key).await?;
        let mut result = HashMap::with_capacity(raw_map.len());

        for (field, data) in raw_map {
            let value: T = serde_json::from_slice(&data)?;
            result.insert(field, value);
        }

        Ok(result)
    }

    /// 获取哈希表指定字段
    async fn hmget<T: DeserializeOwned + Send>(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<T>>> {
        let raw_values = self.hmget_raw(key, fields).await?;
        let mut result = Vec::with_capacity(raw_values.len());

        for maybe_data in raw_values {
            let maybe_value = match maybe_data {
                Some(data) => {
                    let value: T = serde_json::from_slice(&data)?;
                    Some(value)
                }
                None => None,
            };
            result.push(maybe_value);
        }

        Ok(result)
    }

    /// 设置多个哈希表字段
    async fn hmset<T: Serialize + Send + Sync>(&self, key: &str, map: &HashMap<String, T>) -> Result<()> {
        let mut raw_map = HashMap::with_capacity(map.len());

        for (field, value) in map {
            let serialized = serde_json::to_vec(value)?;
            raw_map.insert(field.clone(), serialized);
        }

        self.hmset_raw(key, raw_map).await
    }
}

// 为所有 HashOps 实现者自动提供 HashOpsExt 功能
impl<T: HashOps + ?Sized> HashOpsExt for T {}


/// Redis哈希表操作实现
#[derive(Clone)]
pub struct RedisHash {
    connection_manager: ConnectionManager,
    prefix: String,
}

impl RedisHash {
    /// 创建新的Redis哈希表操作
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
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

    async fn hset_raw(&self, key: &str, field: &str, value: Vec<u8>) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result: i32 = redis::cmd("HSET")
            .arg(full_key)
            .arg(field)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    async fn hget_raw(&self, key: &str, field: &str) -> Result<Option<Vec<u8>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("HGET")
            .arg(full_key)
            .arg(field)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn hdel(&self, key: &str, field: &str) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result: i32 = redis::cmd("HDEL")
            .arg(full_key)
            .arg(field)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    async fn hexists(&self, key: &str, field: &str) -> Result<bool> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result: i32 = redis::cmd("HEXISTS")
            .arg(full_key)
            .arg(field)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    async fn hgetall_raw(&self, key: &str) -> Result<HashMap<String, Vec<u8>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("HGETALL")
            .arg(full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn hmget_raw(&self, key: &str, fields: &[&str]) -> Result<Vec<Option<Vec<u8>>>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let mut cmd = redis::cmd("HMGET");
        cmd.arg(full_key);
        for field in fields {
            cmd.arg(*field);
        }
        let result = cmd.query_async(&mut conn).await?;
        Ok(result)
    }

    async fn hmset_raw(&self, key: &str, map: HashMap<String, Vec<u8>>) -> Result<()> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let mut cmd = redis::cmd("HMSET");
        cmd.arg(full_key);
        for (field, value) in map {
            cmd.arg(field).arg(value);
        }
        let _:() = cmd.query_async(&mut conn).await?;
        Ok(())
    }

    async fn hlen(&self, key: &str) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("HLEN")
            .arg(full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn hkeys(&self, key: &str) -> Result<Vec<String>> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("HKEYS")
            .arg(full_key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    async fn hincrby(&self, key: &str, field: &str, increment: i64) -> Result<i64> {
        let full_key = self.get_key(key);
        let mut conn = self.connection_manager.clone();
        let result = redis::cmd("HINCRBY")
            .arg(full_key)
            .arg(field)
            .arg(increment)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }
}
