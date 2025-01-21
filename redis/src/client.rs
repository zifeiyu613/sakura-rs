use redis::{AsyncCommands, Commands};
use crate::{get_redis_conn};

pub async fn set(key: &str, value: &str) {
    let mut conn = get_redis_conn().await;
    conn.set(key, value).await.expect("TODO: panic message");
}


pub async fn get<T>(key: &str)  -> T {
    let mut conn = get_redis_conn().await;
    let result = conn.get(key).await;
    match result {
        Ok(value) => value,
        Err(e) => println!("{:?}", e),
    }
}

pub async fn del(key: &str) {
    let mut conn = get_redis_conn().await;
    conn.del(key).await.expect("TODO: panic message");
}