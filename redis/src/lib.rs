use crate::config::RedisConfig;
use deadpool_redis::{Config, Connection, Pool, PoolConfig, Runtime, Timeouts};
use std::time::Duration;
use once_cell::sync::{Lazy, OnceCell};
use tracing::info;

pub mod config;
pub mod client;

// 创建全局静态变量 `DB_POOL`，使用 async fn 初始化
pub static REDIS_POOL: Lazy<OnceCell<Pool>> = Lazy::new(|| {
    OnceCell::new() // 用 `OnceCell` 来初始化
});

// pub(crate) async fn get_redis_pool() -> &'static Pool {
//     REDIS_POOL.get_or_init(init_redis_pool).await
// }

pub(crate) async fn get_redis_conn() -> Connection {
    let pool = REDIS_POOL.get_or_init(init_redis_pool);
    pool.get().await.unwrap()
}


pub(crate) fn init_redis_pool() -> Pool {
    // 配置 Deadpool Redis 连接池
    let redis_config = RedisConfig::from_file("redis_config.toml");
    info!("Creating Redis Pool: {:?}", redis_config);

    let mut cfg = Config::from_url(redis_config.uri);
    // 配置连接池
    cfg.pool = Some(PoolConfig {
        max_size: redis_config.pool_max_size, // 最大连接数
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(5)),   // 等待连接超时 5 秒
            create: Some(Duration::from_secs(2)), // 创建新连接超时 2 秒
            recycle: None,
        },
        ..Default::default()
    });

    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")

}
