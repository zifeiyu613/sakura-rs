use super::{Cache, CacheBuilder, CounterOps, DistributedLock, HashOps, ListOps, RedisClient, RedisCounter, RedisHash, RedisList, RedisLock, RedisSortedSet, SortedSetOps, error::Result as RedisResult, JsonSerializer};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::Duration;

/// Redis服务管理器，封装所有Redis功能组件
pub struct RedisManager {
    connection: ConnectionManager,
    client: RedisClient,
    pub cache: Arc<dyn Cache>,
    pub lock: Arc<dyn DistributedLock>,
    pub list: Arc<dyn ListOps>,
    pub hash: Arc<dyn HashOps>,
    pub counter: Arc<dyn CounterOps>,
    pub sorted_set: Arc<dyn SortedSetOps>,
}

impl RedisManager {
    /// 创建新的Redis服务管理器
    pub async fn new(redis_url: &str, app_prefix: &str) -> RedisResult<Self> {
        // 创建Redis客户端
        let client = RedisClient::builder(redis_url)
            .connection_timeout(Duration::from_secs(3))
            .build()?;

        // 测试Redis连接
        client.ping().await?;

        // 创建所有Redis组件
        let manager = Self::from_connection(client, app_prefix).await?;

        Ok(manager)
    }

    /// 从已有连接创建管理器
    pub async fn from_connection(client: RedisClient, app_prefix: &str) -> RedisResult<Self> {
        // 获取连接管理器
        let connection = client.get_connection_manager().await?;

        // 确保前缀以冒号结尾
        let prefix = if app_prefix.ends_with(':') {
            app_prefix.to_string()
        } else {
            format!("{}:", app_prefix)
        };

        // 创建各种Redis操作工具
        let cache = CacheBuilder::new(connection.clone())
            .prefix(&format!("{}cache:", prefix))
            .build();

        let lock = RedisLock::new(connection.clone()).with_prefix(&format!("{}lock:", prefix));

        let list = RedisList::new(connection.clone(), JsonSerializer)
            .with_prefix(&format!("{}list:", prefix));

        let hash = RedisHash::new(connection.clone()).with_prefix(&format!("{}hash:", prefix));

        let counter =
            RedisCounter::new(connection.clone()).with_prefix(&format!("{}counter:", prefix));

        let sorted_set =
            RedisSortedSet::new(connection.clone()).with_prefix(&format!("{}zset:", prefix));

        Ok(
            Self {
                connection,
                client,
                cache: Arc::new(cache),
                lock: Arc::new(lock),
                list: Arc::new(list),
                hash: Arc::new(hash),
                counter: Arc::new(counter),
                sorted_set: Arc::new(sorted_set),
            }
        )
    }

    /// 获取原始Redis客户端
    pub fn client(&self) -> &RedisClient {
        &self.client
    }

    /// 健康检查
    pub async fn health_check(&self) -> RedisResult<()> {
        self.client.ping().await
    }
}
