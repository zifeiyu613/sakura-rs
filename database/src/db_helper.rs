use std::time::Duration;
use once_cell::sync::{Lazy, OnceCell};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, MySqlPool, Pool};



pub struct DbHelper {
    phoenix_pool: MySqlPool,
    activity_pool: MySqlPool,
}


impl DbHelper {


}

// 创建全局静态变量 `DB_POOL`，使用 async fn 初始化
pub static DB_POOL: Lazy<OnceCell<Pool<MySql>>> = Lazy::new(|| {
    OnceCell::new() // 用 `OnceCell` 来初始化
});


pub async fn init_phoenix_pool() -> Pool<MySql> {
    // 通过环境变量或者配置文件读取 database_url
    // let database_url = std::env::var("DATABASE_URL");
    // let database_url = "mysql://srvjava_uat:Test2018RDBSrvjava1210@rm-m5eu9o0h47352viavio.mysql.rds.aliyuncs.com:3306/phoenix?useUnicode=true&characterEncoding=utf8&allowMultiQueries=true&serverTimezone=GMT%2B8";
    let database_config = crate::db_config::DatabaseConfig::from_file("mysql_config.toml");
    MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_config.phoenix_config.database_url.as_str())
        .await
        .expect("Failed to connect to database")
}

pub async fn init_activity_pool() -> Pool<MySql> {
    // 通过环境变量或者配置文件读取 database_url
    // let database_url = std::env::var("DATABASE_URL");
    let database_url = "mysql://srvjava_uat:Test2018RDBSrvjava1210@rm-m5eu9o0h47352viavio.mysql.rds.aliyuncs.com:3306/sakura_huajian_activity?useUnicode=true&characterEncoding=utf8&allowMultiQueries=true&serverTimezone=GMT%2B8";
    MySqlPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
        .expect("Failed to connect to database")
}