use super::error::{RedisError, Result};
use redis::{aio::ConnectionManager, AsyncCommands};
use std::time::Duration;
use tracing::{debug, error, warn};
use uuid::Uuid;


/// 锁定保护
#[derive(Debug, Clone)]
pub struct LockGuard {
    pub key: String,
    pub value: String,
}

/// 分布式锁基础接口（对象安全版本）
#[async_trait::async_trait]
pub trait DistributedLock: Send + Sync {
    /// 尝试获取锁
    async fn acquire(&self, key: &str, ttl: Duration) -> Result<LockGuard>;

    /// 释放锁
    async fn release(&self, guard: LockGuard) -> Result<bool>;
}


/// 分布式锁扩展接口（包含泛型方法）
#[async_trait::async_trait]
pub trait DistributedLockExt: DistributedLock {
    /// 使用锁执行操作
    async fn with_lock<F, Fut, T, E>(&self, key: &str, ttl: Duration, f: F) -> std::result::Result<T, E>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = std::result::Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: From<RedisError> + Send + 'static,
    {
        // 获取锁
        let guard = self.acquire(key, ttl).await.map_err(E::from)?;

        // 执行临界区代码
        let result = f().await;

        // 释放锁（即使操作失败也尝试释放）
        if let Err(e) = self.release(guard).await {
            warn!("释放锁时发生错误: {}", e);
        }

        result
    }
}


// 自动为所有DistributedLock实现者提供DistributedLockExt功能
impl<T: DistributedLock + ?Sized> DistributedLockExt for T {}



/// Redis分布式锁实现
#[derive(Clone)]
pub struct RedisLock {
    connection_manager: ConnectionManager,
    prefix: String,
}

impl RedisLock {
    /// 创建新的分布式锁
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            prefix: "lock:".to_string(),
        }
    }

    /// 设置锁前缀
    pub fn with_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 获取完整的锁键名
    fn get_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait::async_trait]
impl DistributedLock for RedisLock {

    async fn acquire(&self, key: &str, ttl: Duration) -> Result<LockGuard> {
        let lock_key = self.get_key(key);
        let lock_value = Uuid::new_v4().to_string();
        let mut conn = self.connection_manager.clone();

        // 尝试通过SET NX命令获取锁
        let result: bool = conn.set_nx(&lock_key, &lock_value).await?;

        if result {
            // 成功获取锁，设置过期时间
            let _: () = conn.expire(&lock_key, ttl.as_secs() as i64).await?;
            debug!("获取锁成功: {}", lock_key);

            return Ok(LockGuard {
                key: lock_key,
                value: lock_value,
            });
        }

        // 未能获取锁
        debug!("未能获取锁: {}", lock_key);
        Err(RedisError::LockAcquisitionFailed(format!(
            "Failed to acquire lock: {}",
            key
        )))
    }

    async fn release(&self, guard: LockGuard) -> Result<bool> {
        let mut conn = self.connection_manager.clone();

        // 使用Lua脚本确保只删除自己的锁
        let script = r#"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
        "#;

        let result: i64 = redis::Script::new(script)
            .key(&guard.key)
            .arg(&guard.value)
            .invoke_async(&mut conn)
            .await?;

        let success = result == 1;

        if success {
            debug!("锁已释放: {}", guard.key);
        } else {
            warn!("锁释放失败，可能已过期或被其他进程释放: {}", guard.key);
        }

        Ok(success)
    }

}
