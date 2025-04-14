use super::error::{RedisError, Result};
use redis::{aio::ConnectionManager, Client};
use std::sync::Arc;
use std::time::Duration;
use tracing::log::{debug, error, info};

/// Redis客户端封装
#[derive(Clone)]
pub struct RedisClient {
    client: Arc<Client>,
    connection_timeout: Duration,
}

/// Redis客户端构建器
pub struct RedisClientBuilder {
    url: String,
    connection_timeout: Duration,
}

impl RedisClientBuilder {
    /// 创建新的Redis客户端构建器
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            connection_timeout: Duration::from_secs(5),
        }
    }

    /// 设置连接超时时间
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// 构建Redis客户端
    pub fn build(self) -> Result<RedisClient> {
        let client = Client::open(self.url.clone())
            .map_err(|e| RedisError::Connection(format!("Failed to create Redis client: {}", e)))?;

        info!("Redis client created with URL: {}", self.url);

        Ok(RedisClient {
            client: Arc::new(client),
            connection_timeout: self.connection_timeout,
        })
    }
}

impl RedisClient {
    /// 创建新的Redis客户端构建器
    pub fn builder<S: Into<String>>(url: S) -> RedisClientBuilder {
        RedisClientBuilder::new(url)
    }

    /// 获取连接管理器
    pub async fn get_connection_manager(&self) -> Result<ConnectionManager> {
        let client = self.client.clone();
        let conn = tokio::time::timeout(
            self.connection_timeout,
            client.get_connection_manager(),
        )
            .await
            .map_err(|_| RedisError::Connection("Connection timeout".to_string()))?
            .map_err(|e| RedisError::Connection(format!("Failed to get connection: {}", e)))?;

        debug!("Acquired Redis connection manager");
        Ok(conn)
    }

    /// 执行简单ping命令检查连接
    pub async fn ping(&self) -> Result<()> {
        let mut conn = self.get_connection_manager().await?;
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis ping failed: {}", e);
                RedisError::HealthCheck(format!("Failed to ping Redis: {}", e))
            })?;

        if pong != "PONG" {
            error!("Redis ping received unexpected response: {}", pong);
            return Err(RedisError::HealthCheck(format!(
                "Unexpected ping response: {}",
                pong
            )));
        }

        debug!("Redis ping successful");
        Ok(())
    }

    /// 获取原始Redis客户端
    pub fn get_raw_client(&self) -> Arc<Client> {
        self.client.clone()
    }
}
