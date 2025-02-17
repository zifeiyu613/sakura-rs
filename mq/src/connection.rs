use crate::mq_config::RabbitMQConfig;
use deadpool_lapin::{Config, Pool, PoolConfig, Runtime};
use lapin::ConnectionProperties;
use once_cell::sync::Lazy;
use std::sync::Arc;

/// **全局 RabbitMQ 连接池**
/// 连接字符串，格式为：amqp://[用户名][:密码@][主机][:端口][/虚拟机]
static RABBITMQ_POOL: Lazy<Arc<Pool>> = Lazy::new(|| {
    let mq_config = RabbitMQConfig::load_config();
    let cfg = Config {
        url: Some(mq_config.rabbit.uri.clone()),
        pool: Some(PoolConfig {
            max_size: mq_config.rabbit.pool_max_size,
            ..PoolConfig::default()
        }),
        connection_properties: ConnectionProperties::default(),
    };
    eprintln!("RABBITMQ_POOL create_pool ...");
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    Arc::new(pool)
});

/// **获取 RabbitMQ 连接**
pub async fn get_rabbitmq_connection() -> deadpool_lapin::Object {
    RABBITMQ_POOL.get().await.expect("Failed to get RabbitMQ connection")
}



// **全局 RabbitMQ 连接**
// static RABBITMQ_CONNECTION: Lazy<Arc<Mutex<Option<Connection>>>> =
//     Lazy::new(|| Arc::new(Mutex::new(None)));

// **初始化 RabbitMQ 连接**
// pub(crate) async fn init_rabbitmq(uri: &str) -> lapin::Result<Arc<Mutex<Option<Connection>>>> {
//     let mut conn_lock = RABBITMQ_CONNECTION.lock();
//     if conn_lock.is_none() {
//         let conn = Connection::connect(uri, ConnectionProperties::default()).await?;
//         *conn_lock = Some(conn);
//     }
//     Ok(RABBITMQ_CONNECTION.clone())
// }

// /// **获取 RabbitMQ 连接**
// pub async fn get_rabbitmq_connection() -> Arc<Mutex<Option<Connection>>> {
//     init_rabbitmq("amqp://guest:guest@localhost:5672")
//         .await
//         .expect("Failed to connect to RabbitMQ")
// }
