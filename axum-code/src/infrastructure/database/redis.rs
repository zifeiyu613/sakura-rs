use redis::Client;

use crate::config::Config;
use crate::error::AppError;

pub async fn init_redis(config: &Config) -> Result<Client, AppError> {
    tracing::info!("Initializing Redis connection");

    let client = redis::Client::open(config.redis.url.as_str())?;

    // 测试连接
    let mut conn = client.get_connection().await?;
    redis::cmd("PING").query(&mut conn).await?;

    Ok(client)
}
