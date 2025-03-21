use crate::redis_locker::RedisLocker;
use crate::redis_manager::{get_redis_pool_manager, RedisPoolError};
use bb8::PooledConnection;
use bb8_redis::{
    redis::AsyncCommands,
    RedisConnectionManager
};
use redis::FromRedisValue;
use redis::ToRedisArgs;
use std::time::Duration;

/// Redis 命令辅助工具
pub struct RedisHelper;

impl RedisHelper {
    pub(crate) async fn get_connection(&self) -> Result<PooledConnection<RedisConnectionManager>, RedisPoolError> {
        let pool = get_redis_pool_manager()?.get_pool();
        let conn = pool.get().await?;
        Ok(conn)
    }

    /// 设置键值对
    pub async fn set<K, V>(&self, key: K, value: V)  -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?; // 从连接池获取连接
        let result = conn.set(key, value).await.map_err(RedisPoolError::from)?;
        Ok(result)
    }

    pub async fn set_ex<K, V>(&self, key: K, value: V, duration: Duration) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.set_ex(key, value, duration.as_secs()).await?;
        Ok(result)
    }

    /// 当不存在 key 时 设置键值对
    pub async fn set_nx<K, V>(&self, key: K, value: V) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.set_nx(key, value).await?;
        Ok(result)
    }

    /// 获取键值
    pub async fn get<K, V>(&self, key: K) -> Result<Option<V>, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.get(key).await?;
        Ok(result)
    }

    /// 删除键
    pub async fn del<K>(&self, key: K) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.del(key).await?;
        Ok(result)
    }

    pub async fn del_keys<K>(&self, key: Vec<K>) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.del(key).await?;
        Ok(result)
    }

    /// 设置键值对，带过期时间（秒）
    pub async fn set_with_expiry<K, V>(&self, key: K, value: V, ttl: u64) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.set_ex(key, value, ttl).await?;
        Ok(result)
    }

    /// 判断键是否存在
    pub async fn exists<K>(&self, key: K) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.exists(key).await?;
        Ok(result)
    }

    pub async fn expire<K>(&self, key: K, duration: Duration) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.expire(key, duration.as_secs() as i64).await?;
        Ok(result)
    }

    /// 按给定量增加键的数值。会根据类型发出 INCRBY 或 INCRBYFLOAT
    /// 如果类型不匹配 可能报错
    pub async fn incr<K, V>(&self, key: K, delta: V) -> Result<V, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync + ToRedisArgs,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.incr(key, delta).await?;
        Ok(result)
    }

    /// 获取指定区间的数据
    pub async fn lrange<K, V>(&self, key: K, start: isize, stop: isize) -> Result<Vec<V>, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync + ToRedisArgs,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.lrange(key, start, stop).await?;
        Ok(result)
    }


    pub async fn llen<K>(&self, key: K) -> Result<usize, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.llen(key).await?;
        Ok(result)
    }



    // 获取 RedisLocker 实例
    pub fn locker(&self) -> RedisLocker {
        RedisLocker::new(self.clone())
    }

}


// 为 RedisHelper 实现 Clone
impl Clone for RedisHelper {
    fn clone(&self) -> Self {
        RedisHelper
    }
}

