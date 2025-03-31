use lapin::{Connection, ConnectionProperties};

use crate::config::Config;
use crate::error::AppError;

pub async fn init_rabbitmq(config: &Config) -> Result<Connection, AppError> {
    tracing::info!("Initializing RabbitMQ connection");

    let conn = Connection::connect(
        &config.rabbitmq.url,
        ConnectionProperties::default(),
    ).await?;

    let channel = conn.create_channel().await?;

    // 创建交换机
    channel.exchange_declare(
        "products",
        lapin::options::ExchangeKind::Topic,
        lapin::options::ExchangeDeclareOptions {
            durable: true,
            ..Default::default()
        },
        Default::default(),
    ).await?;

    // 创建队列
    channel.queue_declare(
        "product_events",
        lapin::options::QueueDeclareOptions {
            durable: true,
            ..Default::default()
        },
        Default::default(),
    ).await?;

    // 绑定队列到交换机
    channel.queue_bind(
        "product_events",
        "products",
        "product.*",
        Default::default(),
        Default::default(),
    ).await?;

    Ok(conn)
}
