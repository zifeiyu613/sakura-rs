use crate::redis_manager::RedisPoolError;
use crate::redis_helper::RedisHelper;
use bb8_redis::redis::{ToRedisArgs};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use uuid::Uuid;

/// Redis 分布式锁管理器
pub struct RedisLocker {
    redis_helper: RedisHelper,
}

impl RedisLocker {
    pub fn new(redis_helper: RedisHelper) -> Self {
        Self { redis_helper }
    }

    /// 尝试获取分布式锁
    ///
    /// # 参数
    /// * `lock_name` - 锁的名称
    /// * `lease_time` - 锁的租约时间（自动过期时间）
    /// * `retry_times` - 获取锁的重试次数
    /// * `retry_delay` - 重试间隔
    ///
    /// # 返回
    /// 成功获取到锁返回一个 RedisLock 实例，否则返回错误
    pub async fn try_lock<K>(&self,
                             lock_name: K,
                             lease_time: Duration,
                             retry_times: usize,
                             retry_delay: Duration
    ) -> Result<RedisLock, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync + Display + Clone,
    {
        let lock_name_str = format!("redis_lock:{}", lock_name);
        let lock_id = Uuid::new_v4().to_string();

        // 尝试获取锁
        for _ in 0..retry_times + 1 {
            // 使用SET NX EX命令尝试获取锁
            let acquired = self.set_nx_with_expiry(&lock_name_str, &lock_id, lease_time.as_secs()).await?;

            if acquired {
                // 创建锁对象
                let lock = RedisLock::new(
                    self.redis_helper.clone(),
                    lock_name_str,
                    lock_id,
                    lease_time,
                );

                // 启动自动续期任务（如果锁的租约时间 > 0）
                if lease_time.as_secs() > 0 {
                    lock.schedule_expiration_renewal().await;
                }

                return Ok(lock);
            }

            // 如果没有获取到锁且还有重试次数，则等待后重试
            if retry_times > 0 {
                time::sleep(retry_delay).await;
            }
        }

        Err(RedisPoolError::Custom("Failed to acquire lock after retries".into()))
    }

    /// 获取锁并返回锁守卫
    pub async fn lock_with_guard<K>(&self,
                                    lock_name: K,
                                    lease_time: Duration,
                                    retry_times: usize,
                                    retry_delay: Duration
    ) -> Result<RedisLockGuard, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync + Display + Clone,
    {
        let lock = self.try_lock(lock_name, lease_time, retry_times, retry_delay).await?;
        Ok(RedisLockGuard::new(lock))
    }

    /// 设置键值对并设置过期时间（原子操作）
    async fn set_nx_with_expiry<K, V>(&self, key: K, value: V, ttl: u64) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.redis_helper.get_connection().await?;

        // 使用SET命令的NX和EX选项
        let result: bool = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl)
            .query_async(&mut *conn)
            .await?;

        Ok(result)
    }
}

/// Redis分布式锁实现
pub struct RedisLock {
    redis_helper: RedisHelper,
    lock_name: String,
    lock_id: String,
    lease_time: Duration,
    renewal_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl RedisLock {
    /// 创建一个新的Redis锁实例
    fn new(
        redis_helper: RedisHelper,
        lock_name: String,
        lock_id: String,
        lease_time: Duration
    ) -> Self {
        Self {
            redis_helper,
            lock_name,
            lock_id,
            lease_time,
            renewal_task: Arc::new(Mutex::new(None)),
        }
    }

    /// 释放锁
    pub async fn unlock(&self) -> Result<bool, RedisPoolError> {
        // 停止自动续期任务
        if let Some(task) = self.stop_renewal_task().await {
            task.abort();
        }

        // 释放锁
        self.release_lock().await
    }

    /// 使用Lua脚本释放锁（仅当锁被当前实例持有时才释放）
    async fn release_lock(&self) -> Result<bool, RedisPoolError> {
        let mut conn = self.redis_helper.get_connection().await?;

        // Lua脚本确保只有锁的持有者能释放锁
        let script = redis::Script::new(r"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
        ");

        let result: i32 = script
            .key(&self.lock_name)
            .arg(&self.lock_id)
            .invoke_async(&mut *conn)
            .await?;

        Ok(result == 1)
    }

    /// 启动自动续期任务
    async fn schedule_expiration_renewal(&self) {
        let lock_name = self.lock_name.clone();
        let lock_id = self.lock_id.clone();
        let lease_time = self.lease_time;
        let redis_helper = self.redis_helper.clone();
        let renewal_interval = lease_time.mul_f32(0.6); // 在过期时间的60%处更新
        let renewal_task_mutex = self.renewal_task.clone();

        // 创建并启动自动续期任务
        let task = tokio::spawn(async move {
            let mut interval = time::interval(renewal_interval);

            loop {
                interval.tick().await;

                // 尝试更新锁的过期时间
                match update_lock_expiry(&redis_helper, &lock_name, &lock_id, lease_time.as_secs()).await {
                    Ok(true) => {
                        // 成功更新锁的过期时间
                    },
                    Ok(false) => {
                        // 锁不再由此客户端持有，终止续期任务
                        break;
                    },
                    Err(_) => {
                        // 发生错误，稍后重试
                        time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // 存储任务句柄
        let mut renewal_task = renewal_task_mutex.lock().await;
        *renewal_task = Some(task);
    }

    /// 停止自动续期任务
    async fn stop_renewal_task(&self) -> Option<JoinHandle<()>> {
        let mut renewal_task = self.renewal_task.lock().await;
        renewal_task.take()
    }
}

/// 使用Lua脚本更新锁的过期时间（仅当锁被当前实例持有时才更新）
async fn update_lock_expiry<K, V>(
    redis_helper: &RedisHelper,
    key: K,
    expected_value: V,
    ttl: u64
) -> Result<bool, RedisPoolError>
where
    K: ToRedisArgs + Send + Sync,
    V: ToRedisArgs + Send + Sync,
{
    let mut conn = redis_helper.get_connection().await?;

    // Lua脚本确保只有在锁存在且值匹配时才更新过期时间
    let script = redis::Script::new(r"
        if redis.call('get', KEYS[1]) == ARGV[1] then
            return redis.call('expire', KEYS[1], ARGV[2])
        else
            return 0
        end
    ");

    let result: i32 = script
        .key(key)
        .arg(expected_value)
        .arg(ttl)
        .invoke_async(&mut *conn)
        .await?;

    Ok(result == 1)
}

impl Drop for RedisLock {
    fn drop(&mut self) {
        // 创建一个运行时以执行异步释放锁的操作
        let rt = tokio::runtime::Runtime::new().ok().expect("failed to get tokio runtime");

        // 在运行时中执行释放锁的操作
        rt.block_on(async {
            // 尝试释放锁，如果失败也不阻止对象的释放
            let _ = self.unlock().await;
        });
    }
}

// 锁守卫，用于 RAII 风格自动释放锁
pub struct RedisLockGuard {
    lock: RedisLock,
}

impl RedisLockGuard {
    /// 从锁创建守卫
    pub fn new(lock: RedisLock) -> Self {
        Self { lock }
    }

    /// 手动释放锁
    pub async fn unlock(self) -> Result<bool, RedisPoolError> {
        self.lock.unlock().await
    }
}

impl Drop for RedisLockGuard {
    fn drop(&mut self) {
        // 同上，创建运行时以执行异步释放锁的操作
        let rt = tokio::runtime::Runtime::new().ok().expect("failed to get tokio runtime");

        rt.block_on(async {
            let _ = self.lock.unlock().await;
        });
    }
}
