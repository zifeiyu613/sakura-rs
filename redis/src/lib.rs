use crate::redis_config::RedisConfig;
use once_cell::sync::{Lazy, OnceCell};
use r2d2::{Pool};
use redis::Client;
use tracing::info;

pub mod redis_config;
pub mod redis_helper;

// 创建全局静态变量 `DB_POOL`，使用 async fn 初始化
pub static REDIS_POOL: Lazy<OnceCell<Pool<Client>>> = Lazy::new(|| {
    OnceCell::new() // 用 `OnceCell` 来初始化
});

// pub(crate) async fn get_redis_pool() -> &'static Pool {
//     REDIS_POOL.get_or_init(init_redis_pool).await
// }

pub(crate) fn get_redis_conn<'a>() -> &'a Pool<Client> {
    let pool = REDIS_POOL.get_or_init(init_redis_pool);
    pool
}


pub(crate) fn init_redis_pool() -> Pool<Client> {
    // 配置 Deadpool Redis 连接池
    let redis_config = RedisConfig::from_file("redis_config.toml");
    info!("Creating Redis Pool: {:?}", redis_config);

    let client = Client::open(redis_config.uri).unwrap();
    let pool = Pool::builder()
        .max_size(redis_config.pool_max_size as u32)
        .build(client)
        .unwrap();
    pool
}
