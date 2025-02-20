use r2d2::PooledConnection;
use redis::{Client, Commands, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use std::time::Duration;

/// Redis 命令辅助工具
pub struct RedisHelper;

impl RedisHelper {
    fn get_connection(&self) -> RedisResult<PooledConnection<Client>> {
        let conn = crate::get_redis_conn()?.get().map_err(|err| {
            RedisError::from((
                redis::ErrorKind::IoError,
                "get redis connection err!!!",
                err.to_string(),
            ))
        });
        conn
    }

    /// 设置键值对
    pub fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.get_connection()?; // 从连接池获取连接
        conn.set(key, value)
    }

    pub fn set_ex(&self, key: &str, value: &str, duration: Duration) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.set_ex(key, value, duration.as_secs())
    }

    /// 当不存在 key 时 设置键值对
    pub fn set_nx(&self, key: &str, value: &str) -> RedisResult<bool> {
        let mut conn = self.get_connection()?;
        conn.set_nx(key, value)
    }

    /// 获取键值
    pub fn get<T>(&self, key: &str) -> RedisResult<Option<T>>
    where
        T: FromRedisValue + Sized,
    {
        let mut conn = self.get_connection()?;
        conn.get(key)
    }

    /// 删除键
    pub fn del(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.del(key)
    }

    /// 设置键值对，带过期时间（秒）
    pub fn set_with_expiry(&self, key: &str, value: &str, ttl: u64) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.set_ex(key, value, ttl)
    }

    /// 判断键是否存在
    pub fn exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.get_connection()?;
        conn.exists(key)
    }

    pub fn expire(&self, key: &str, duration: Duration) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.expire(key, duration.as_secs() as i64)
    }

    /// 按给定量增加键的数值。会根据类型发出 INCRBY 或 INCRBYFLOAT
    /// 如果类型不匹配 可能报错
    pub fn incr<T>(&self, key: &str, delta: T) -> RedisResult<T>
    where
        T: FromRedisValue + ToRedisArgs + Sized,
    {
        let mut conn = self.get_connection()?;
        conn.incr(key, delta)
    }

    /// 获取指定区间的数据
    pub fn lrange<T>(&self, key: &str, start: isize, stop: isize) -> RedisResult<Option<Vec<T>>>
    where
    T: FromRedisValue + ToRedisArgs
    {
        let mut conn = self.get_connection()?;
        conn.lrange(key, start, stop)
    }


}

#[cfg(test)]
mod tests {
    use crate::redis_helper::RedisHelper;
    use std::io::Write;
    use config::app_config::load_config;

    #[test]
    fn redis_set_get() {
        // 创建临时文件
        let path =  setup();

        // 前置条件：创建临时文件
        // let mut temp_file = NamedTempFile::new_in("redis_config.toml").expect("Failed to create temp file");
        // let content = r#"
        //     uri="redis://:HuaJian2019testRedis@srv-redis-uat-io.kaiqi.xin:7001/0"
        //     pool_max_size=10
        // "#;
        // writeln!(temp_file, "{}", content).expect("Failed to write to temp file");

        load_config(Some("/Users/will/RustroverProjects/sakura/api/config.toml")).unwrap();


        let tk = "rust:test:key";

        RedisHelper
            .set(tk, "value01")
            .expect("Failed to set value");

        println!("{:?}", RedisHelper.set(tk, "value03"));
        println!("{:?}", RedisHelper.set_nx(tk, "value02"));

        // RedisHelper
        //     .set(tk, "value02")
        //     .expect("Failed to set value");

        let value = RedisHelper
            .get::<String>(tk)
            .expect("Failed to get value");
        println!("Get value: {:?}", value);

        RedisHelper
            .del("rust:test:key")
            .expect("Failed to remove value");

        let key = "rust:test:incr";
        RedisHelper.del(key).expect("Failed to remove value");

        let incr = RedisHelper.incr::<i64>(key,1).unwrap();
        println!("Incr First: {:?}", incr);
        let incr = RedisHelper.incr::<i64>(key,1).unwrap();
        println!("Incr Second: {:?}", incr);

        let incr = RedisHelper.incr::<i64>(key,-1).unwrap();
        assert_eq!(incr, 1);

        let incr = RedisHelper.incr::<f64>(key,1.1).unwrap();
        assert_eq!(incr, 2.1);

        RedisHelper.del(key).expect("Failed to remove value");

        let key1 = "living:room:list:V1:filter:level";

        let exist = RedisHelper.exists(key1).expect("Failed to get value");
        assert!(exist);
        if exist {
            let value = RedisHelper.lrange::<String>(key1, 0, -1).expect("Failed to set value");

            println!("Value: {:?}", value);
            if let Some(value) = value {
                value.iter().for_each(|value| {
                    println!("Value: {:?}", value);
                    println!("Value Json: {:?}", serde_json::from_str::<serde_json::Value>(value).unwrap());
                })
            }
        }

        // 删除文件
        teardown(&path)
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
