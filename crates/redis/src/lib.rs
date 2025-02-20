use once_cell::sync::{Lazy, OnceCell};
use r2d2::{Error, Pool};
use redis::{Client, ErrorKind, RedisError};
use tracing::{error, info};
use config::app_config::get_config;

pub mod redis_helper;

// 创建全局静态变量 `REDIS_POOL`
pub(crate) static REDIS_POOL: Lazy<OnceCell<Pool<Client>>> = Lazy::new(|| {
    OnceCell::new() // 用 `OnceCell` 来初始化
});

/// 获取 Redis 连接池的引用
pub(crate) fn get_redis_conn() -> Result<Pool<Client>, RedisError> {
    let pool = REDIS_POOL.get_or_init(init_redis_pool);
    Ok(pool.clone())
}

/// 初始化 Redis 连接池
pub(crate) fn init_redis_pool() -> Pool<Client>{
    // 加载配置
    let redis_config = get_config().unwrap().redis;

    // 确保配置中有 Redis 信息
    // let redis_config = match redis {
    //     Some(cfg) => cfg,
    //     None => {
    //         error!("配置中没有 Redis 信息");
    //         panic!("config.redis is None")
    //     }
    // };

    // 不记录敏感信息，如密码等
    let masked_uri = if let Some(uri) = redis_config.uri.strip_prefix("redis://:") {
        "redis://:*****"
    } else {
        redis_config.uri.as_str()
    };

    info!("创建 Redis 连接池，URI: {}", masked_uri);

    // 创建 Redis 客户端
    let client = match Client::open(redis_config.uri) {
        Ok(c) => c,
        Err(e) => {
            error!("创建 Redis 客户端失败: {}", e);
            panic!("无法创建 Redis 客户端")
        }
    };

    // 构建连接池
    let pool = Pool::builder()
        .max_size(redis_config.pool_max_size as u32)
        .build(client)
        .unwrap_or_else(|e| {
            error!("创建 Redis 连接池失败: {}", e);
            panic!("无法创建 Redis 连接池")
        });

    pool
}
