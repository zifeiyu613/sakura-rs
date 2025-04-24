
mod redis_helper;
mod redis_locker;
mod redis_manager;


pub use redis_helper::RedisHelper;
pub use redis_locker::{RedisLocker, RedisLock, RedisLockGuard};



#[cfg(test)]
mod tests {
    use crate::redis_manager::{init_redis_pool, RedisPoolError};
    use crate::redis_helper::RedisHelper;
    use futures_util::future::join_all;
    use serde_json::Value;
    use std::io::Write;
    use std::time::Duration;

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

        // load_config(Some("/Users/will/RustroverProjects/sakura/sakura-api/rconfig.toml")).unwrap();

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


    async fn example_with_redis_lock() -> Result<(), RedisPoolError> {
        let redis_helper = RedisHelper;
        let locker = redis_helper.locker();

        // 方式1: 直接使用锁
        let lock = locker.try_lock(
            "my_lock_key",
            Duration::from_secs(30),
            3,
            Duration::from_millis(200)
        ).await?;

        // 执行需要锁保护的操作
        // ...业务逻辑...

        // 完成后释放锁
        lock.unlock().await?;

        // 方式2: 使用RAII风格的锁守卫 (推荐)
        {
            let _guard = locker.lock_with_guard(
                "another_lock_key",
                Duration::from_secs(10),
                5,
                Duration::from_millis(100)
            ).await?;

            // 执行需要锁保护的操作
            // ...业务逻辑...

            // 当_guard离开作用域时，锁会自动释放
        }

        Ok(())
    }


}
