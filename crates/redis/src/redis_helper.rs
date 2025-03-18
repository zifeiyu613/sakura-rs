use crate::RedisPoolError;
use bb8::{ PooledConnection};
use bb8_redis::{
    RedisConnectionManager,
    redis::{cmd, AsyncCommands}
};
use redis::ToRedisArgs;
use redis::FromRedisValue;
use std::time::Duration;

/// Redis 命令辅助工具
pub struct RedisHelper;

impl RedisHelper {
    async fn get_connection(&self) -> Result<PooledConnection<RedisConnectionManager>, RedisPoolError> {
        let pool = crate::get_redis_pool_manager()?.get_pool();
        let conn = pool.get().await?;
        Ok(conn)
    }

    // async fn using_connection_pool_extractor(&self) -> Result<String, RedisPoolError> {
    //     // let mut conn = pool.get().await.map_err(internal_error)?;
    //     let mut conn = self.get_connection().await?; // 从连接池获取连接
    //     let result: String = conn.get("foo").await.map_err(RedisPoolError::RuntimeError("".to_string()))?;
    //     Ok(result)
    // }

    /// 设置键值对
    pub async fn set<K, V>(&self, key: K, value: V)  -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?; // 从连接池获取连接
        let result = conn.set(key, value).await.map_err(RedisPoolError::from)?;
        Ok(result)
    }

    pub async fn set_ex<K, V>(&self, key: K, value: V, duration: Duration) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.set_ex(key, value, duration.as_secs()).await?;
        Ok(result)
    }

    /// 当不存在 key 时 设置键值对
    pub async fn set_nx<K, V>(&self, key: K, value: V) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.set_nx(key, value).await?;
        Ok(result)
    }

    /// 获取键值
    pub async fn get<K, V>(&self, key: K) -> Result<Option<V>, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.get(key).await?;
        Ok(result)
    }

    /// 删除键
    pub async fn del<K>(&self, key: K) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.del(key).await?;
        Ok(result)
    }

    pub async fn del_keys<K>(&self, key: Vec<K>) -> Result<bool, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.del(key).await?;
        Ok(result)
    }

    /// 设置键值对，带过期时间（秒）
    pub async fn set_with_expiry<K, V>(&self, key: K, value: V, ttl: u64) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.set_ex(key, value, ttl).await?;
        Ok(result)
    }

    /// 判断键是否存在
    pub async fn exists<K>(&self, key: K) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.exists(key).await?;
        Ok(result)
    }

    pub async fn expire<K>(&self, key: K, duration: Duration) -> Result<bool, RedisPoolError>
    where
    K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.expire(key, duration.as_secs() as i64).await?;
        Ok(result)
    }

    /// 按给定量增加键的数值。会根据类型发出 INCRBY 或 INCRBYFLOAT
    /// 如果类型不匹配 可能报错
    pub async fn incr<K, V>(&self, key: K, delta: V) -> Result<V, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync + ToRedisArgs,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.incr(key, delta).await?;
        Ok(result)
    }

    /// 获取指定区间的数据
    pub async fn lrange<K, V>(&self, key: K, start: isize, stop: isize) -> Result<Vec<V>, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync + ToRedisArgs,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.lrange(key, start, stop).await?;
        Ok(result)
    }


    pub async fn llen<K>(&self, key: K) -> Result<usize, RedisPoolError>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result = conn.llen(key).await?;
        Ok(result)
    }


}

#[cfg(test)]
mod tests {
    use crate::redis_helper::RedisHelper;
    use config::app_config::load_config;
    use std::io::Write;
    use futures_util::future::join_all;
    use serde_json::Value;
    use crate::init_redis_pool;

    #[tokio::test]
    async fn redis_set_get() {
        // 创建临时文件
        // let path =  setup();

        // 前置条件：创建临时文件
        // let mut temp_file = NamedTempFile::new_in("redis_config.toml").expect("Failed to create temp file");
        // let content = r#"
        //     uri="redis://:HuaJian2019testRedis@srv-redis-uat-io.kaiqi.xin:7001/0"
        //     pool_max_size=10
        // "#;
        // writeln!(temp_file, "{}", content).expect("Failed to write to temp file");

        load_config(Some("/Users/will/RustroverProjects/sakura/sakura-api/config.toml")).unwrap();

        init_redis_pool().await.unwrap();

        let tk = "rust:test:key";

        RedisHelper
            .set(tk, "value01").await
            .expect("Failed to set value");

        println!("{:?}", RedisHelper.set(tk, "value03").await.unwrap());
        println!("{:?}", RedisHelper.set_nx(tk, "value02").await.unwrap());

        // RedisHelper
        //     .set(tk, "value02")
        //     .expect("Failed to set value");

        let value = RedisHelper
            .get::<&str, String>(tk).await
            .expect("Failed to get value");
        println!("Get value: {:?}", value);

        // let result = RedisHelper
        //     .del("rust:test:key").await
        //     .expect("Failed to remove value");
        // println!("Remove result: {:?}", result);

        RedisHelper.set("rust:test:key", "value04").await.unwrap();
        RedisHelper.set("rust:test:key1", "value04").await.unwrap();
        // RedisHelper.set("rust:test:key2", "value04").await.unwrap();

        let result = RedisHelper
            .del_keys(vec!["rust:test:key", "rust:test:key1", "rust:test:key2"]).await
            .expect("Failed to remove value");
        println!("Remove keys result: {:?}", result);

        let key = "rust:test:incr";
        RedisHelper.del(key).await.expect("Failed to remove value");

        let incr = RedisHelper.incr::<&str, u32>(key,1).await.unwrap();
        println!("Incr First: {:?}", incr);
        let incr = RedisHelper.incr::<&str, u32>(key,1).await.unwrap();
        println!("Incr Second: {:?}", incr);

        let incr = RedisHelper.incr::<&str, i32>(key,-1).await.unwrap();
        assert_eq!(incr, 1);

        let incr = RedisHelper.incr::<&str, f32>(key,1.1).await.unwrap();
        assert_eq!(incr, 2.1);

        let mut handles = vec![];
        for i in 0..10 {
            handles.push(tokio::spawn(async move {
                let incr_v = RedisHelper.incr("rust:test:key", 1).await.unwrap();
                println!("NO: {:?}, {}", i, incr_v);
            }))
        }
        join_all(handles).await;

        RedisHelper.del(key).await.expect("Failed to remove value");

        let key1 = "living:room:list:env:TEST";
        let exist = RedisHelper.exists(key1).await.expect("Failed to get value");
        let room_list = RedisHelper.get::<_, String>(key1).await.unwrap();
        println!("{:?}, Exist {:?}, {:?}", key1, exist, room_list);

        let key2 = "living:room:list:V1:filter:level";
        let exist = RedisHelper.exists(key2).await.expect("Failed to get value");
        println!("{:?}, Exist {:?}", key2, exist);
        // assert!(exist);
        if exist {
            let list = RedisHelper.lrange::<_, String>(key2, 0, -1).await.expect("Failed to get value");

            println!("list: {:?}", list);
            list.into_iter().for_each(|item| {
                println!("item: {:?}", item);
                println!("item Json: {:?}", serde_json::from_str::<Value>(&item).unwrap());
            })
        }

        // 删除文件
        // teardown(&path)
    }


    fn setup() -> String {
        // 创建临时文件，返回文件路径
        let file_path = "redis_config.toml".to_string();
        let mut file = std::fs::File::create(&file_path).expect("Failed to create test file");
        let content = r#"
         [redis]
         uri="redis://:HuaJian2019testRedis@srv-redis-uat-io.kaiqi.xin:7001/0"
         pool_max_size=10
        "#;
        writeln!(file, "{}", content).expect("Failed to write to test file");
        file_path
    }

    fn teardown(file_path: &str) {
        // 删除文件
        std::fs::remove_file(file_path).expect("Failed to delete test file");
    }

}
