use super::client::RedisClient;
use super::error::Result;
use std::time::{Duration, Instant};
use tracing::{error, info};

/// Redis健康检查工具
#[derive(Clone)]
pub struct HealthCheck {
    client: RedisClient,
    timeout: Duration,
}

impl HealthCheck {
    /// 创建新的健康检查实例
    pub fn new(client: RedisClient) -> Self {
        Self {
            client,
            timeout: Duration::from_secs(5),
        }
    }

    /// 设置健康检查超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 执行健康检查
    pub async fn check(&self) -> Result<HealthStatus> {
        let start = Instant::now();

        let result = tokio::time::timeout(self.timeout, self.client.ping()).await;

        let elapsed = start.elapsed();

        match result {
            Ok(Ok(_)) => {
                info!("Redis health check successful, latency: {:?}", elapsed);
                Ok(HealthStatus {
                    status: Status::Healthy,
                    latency: elapsed,
                })
            }
            Ok(Err(e)) => {
                error!("Redis health check failed: {}", e);
                Ok(HealthStatus {
                    status: Status::Unhealthy(e.to_string()),
                    latency: elapsed,
                })
            }
            Err(_) => {
                error!("Redis health check timed out after {:?}", self.timeout);
                Ok(HealthStatus {
                    status: Status::Unhealthy("Timeout".to_string()),
                    latency: elapsed,
                })
            }
        }
    }
}

/// 健康状态
#[derive(Debug)]
pub struct HealthStatus {
    pub status: Status,
    pub latency: Duration,
}

/// 健康状态枚举
#[derive(Debug)]
pub enum Status {
    Healthy,
    Unhealthy(String),
}
