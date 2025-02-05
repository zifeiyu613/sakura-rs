// use once_cell::sync::Lazy;
// use sqlx::mysql::MySqlPoolOptions;
// use sqlx::{MySql, Pool};
// use std::time::Duration;
// use tokio::sync::OnceCell;
//
// /// 尝试在一个已经存在的异步运行时（tokio runtime）中使用 tokio::runtime::Runtime::new().block_on()，而这是不允许的。
// /// block_on 是一种同步方法，通常用于从同步代码中运行异步任务。但是，如果你的代码已经在一个异步运行时中运行，
// /// 例如使用了 #[tokio::main] 或者异步环境是由其他代码创建的，那么再创建一个新的运行时并调用 block_on 会导致冲突。
// // 定义一个全局静态变量，连接池
// // pub static DB_POOL: Lazy<Pool<MySql>> = Lazy::new(|| {
// //     // 使用 block_on 初始化异步代码，因为 Lazy 的初始化是同步的
// //     tokio::runtime::Runtime::new()
// //         .unwrap()
// //         .block_on(init_db_pool())
// // });
//
// // 创建全局静态变量 `DB_POOL`，使用 async fn 初始化
// pub static DB_POOL: Lazy<OnceCell<Pool<MySql>>> = Lazy::new(|| {
//     OnceCell::new() // 用 `OnceCell` 来初始化
// });
//
// pub async fn get_db_pool() -> &'static Pool<MySql> {
//     DB_POOL.get_or_init(init_db_pool).await
// }
//
// pub async fn init_db_pool() -> Pool<MySql> {
//     // 通过环境变量或者配置文件读取 database_url
//     // let database_url = std::env::var("DATABASE_URL");
//     let database_url = "mysql://srvjava_uat:Test2018RDBSrvjava1210@rm-m5eu9o0h47352viavio.mysql.rds.aliyuncs.com:3306/phoenix?useUnicode=true&characterEncoding=utf8&allowMultiQueries=true&serverTimezone=GMT%2B8";
//     MySqlPoolOptions::new()
//         .max_connections(5)
//         .acquire_timeout(Duration::from_secs(30))
//         .connect(database_url)
//         .await
//         .expect("Failed to connect to database")
// }
//
// pub async fn init_activity_db_pool() -> Pool<MySql> {
//     // 通过环境变量或者配置文件读取 database_url
//     // let database_url = std::env::var("DATABASE_URL");
//     let database_url = "mysql://srvjava_uat:Test2018RDBSrvjava1210@rm-m5eu9o0h47352viavio.mysql.rds.aliyuncs.com:3306/sakura_huajian_activity?useUnicode=true&characterEncoding=utf8&allowMultiQueries=true&serverTimezone=GMT%2B8";
//     MySqlPoolOptions::new()
//         .max_connections(5)
//         .min_connections(1)
//         .acquire_timeout(Duration::from_secs(30))
//         .connect(database_url)
//         .await
//         .expect("Failed to connect to database")
// }
//
//
//
//
// struct DatabaseManager {
//     connection_pool: Pool<MySql>,
// }
//
// impl DatabaseManager {
//
//     async fn init(database_url: &str) -> Self {
//         let url = if database_url.is_empty() {
//             "mysql://srvjava_uat:Test2018RDBSrvjava1210@rm-m5eu9o0h47352viavio.mysql.rds.aliyuncs.com:3306/phoenix?useUnicode=true&characterEncoding=utf8&allowMultiQueries=true&serverTimezone=GMT%2B8"
//         } else {
//             database_url
//         };
//
//         let connection_pool = MySqlPoolOptions::new()
//             .max_connections(5)
//             .acquire_timeout(Duration::from_secs(5))
//             .connect(url)
//             .await
//             .expect("Failed to connect to database");
//         DatabaseManager { connection_pool }
//     }
//
//
//
//
// }
