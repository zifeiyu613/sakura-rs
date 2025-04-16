use anyhow::{Result, Context};
use redis::{Client as RedisClient, aio::ConnectionManager};
use std::time::Duration;
use tracing::info;
use crate::config::app_config::AppConfig;

pub async fn init_redis(config: &AppConfig) -> Result<ConnectionManager> {
    info!("Initializing Redis connection");

    let client = RedisClient::open(config.redis.url.clone())
        .context("Failed to create Redis client")?;

    // 创建连接管理器
    let manager = ConnectionManager::new(client)
        .await
        .context("Failed to create Redis connection manager")?;

    // 测试连接
    let mut conn = manager.clone();
    redis::cmd("PING")
        .query_async::<_>(&mut conn)
        .await
        .context("Failed to ping Redis")?;

    info!("Redis connection initialized successfully");

    Ok(manager)
}

// Redis缓存助手函数
pub async fn set_with_expiry(
    conn: &mut ConnectionManager,
    key: &str,
    value: &str,
    expiry_seconds: u64,
) -> Result<()> {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(expiry_seconds)
        .query_async(conn)
        .await
        .context("Failed to set Redis key with expiry")?;

    Ok(())
}

pub async fn get(conn: &mut ConnectionManager, key: &str) -> Result<Option<String>> {
    let result: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(conn)
        .await
        .context("Failed to get Redis key")?;

    Ok(result)
}

pub async fn delete(conn: &mut ConnectionManager, key: &str) -> Result<()> {
    redis::cmd("DEL")
        .arg(key)
        .query_async(conn)
        .await
        .context("Failed to delete Redis key")?;

    Ok(())
}

pub async fn increment(conn: &mut ConnectionManager, key: &str) -> Result<i64> {
    let result: i64 = redis::cmd("INCR")
        .arg(key)
        .query_async(conn)
        .await
        .context("Failed to increment Redis key")?;

    Ok(result)
}

pub async fn set_with_nx(
    conn: &mut ConnectionManager,
    key: &str,
    value: &str,
    expiry_seconds: u64,
) -> Result<bool> {
    let result: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("NX")
        .arg("EX")
        .arg(expiry_seconds)
        .query_async(conn)
        .await
        .context("Failed to set Redis key with NX and expiry")?;

    Ok(result.is_some())
}
