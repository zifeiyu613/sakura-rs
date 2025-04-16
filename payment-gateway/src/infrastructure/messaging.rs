use crate::config::AppConfig;
use anyhow::{Result, Context, anyhow};
use lapin::{
    Connection, ConnectionProperties, Channel,
    options::{QueueDeclareOptions, BasicPublishOptions, BasicConsumeOptions, BasicAckOptions},
    types::FieldTable,
    message::DeliveryResult,
};
use tokio::sync::Mutex;
use futures_lite::stream::StreamExt;
use std::sync::Arc;
use tracing::{info, error, warn};
use std::time::Duration;

// 消息处理器类型
pub type MessageHandler = Arc<dyn Fn(Vec<u8>) -> Result<()> + Send + Sync>;

pub struct MessageBroker {
    channel: Channel,
    connection: Connection,
    config: AppConfig,
}

impl MessageBroker {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        info!("Initializing RabbitMQ connection");

        // 创建连接
        let connection = Self::create_connection(&config.rabbitmq.url).await?;

        // 创建通道
        let channel = connection.create_channel().await
            .context("Failed to create RabbitMQ channel")?;

        info!("RabbitMQ connection initialized successfully");

        Ok(Self {
            channel,
            connection,
            config: config.clone(),
        })
    }

    async fn create_connection(url: &str) -> Result<Connection> {
        let mut retry_count = 0;
        let max_retries = 5;
        let retry_interval = Duration::from_secs(2);

        loop {
            match Connection::connect(url, ConnectionProperties::default()).await {
                Ok(conn) => return Ok(conn),
                Err(err) => {
                    retry_count += 1;
                    if retry_count > max_retries {
                        return Err(anyhow!("Failed to connect to RabbitMQ after {} retries: {}", max_retries, err));
                    }

                    warn!("Failed to connect to RabbitMQ (attempt {}/{}): {}. Retrying in {:?}...",
                          retry_count, max_retries, err, retry_interval);
                    tokio::time::sleep(retry_interval).await;
                }
            }
        }
    }

    // 声明队列
    pub async fn declare_queue(&self, queue_name: &str) -> Result<()> {
        self.channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await
            .context(format!("Failed to declare queue: {}", queue_name))?;

        info!("Declared queue: {}", queue_name);
        Ok(())
    }

    // 发布消息
    pub async fn publish(&self, queue_name: &str, payload: &[u8]) -> Result<()> {
        // 确保队列存在
        self.declare_queue(queue_name).await?;

        self.channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                payload,
                Default::default(),
            )
            .await
            .context(format!("Failed to publish message to queue: {}", queue_name))?;

        Ok(())
    }

    // 消费消息
    pub async fn consume(
        &self,
        queue_name: &str,
        handler: MessageHandler,
    ) -> Result<()> {
        // 确保队列存在
        self.declare_queue(queue_name).await?;

        let consumer = self.channel
            .basic_consume(
                queue_name,
                &format!("consumer-{}", queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .context(format!("Failed to start consuming from queue: {}", queue_name))?;

        info!("Started consuming from queue: {}", queue_name);

        let channel = self.channel.clone();

        // 在新任务中处理消息
        tokio::spawn(async move {
            consumer.for_each(move |delivery| {
                let channel = channel.clone();
                let handler = handler.clone();

                async move {
                    match delivery {
                        Ok(delivery) => {
                            let payload = delivery.data.clone();

                            match handler(payload) {
                                Ok(_) => {
                                    // 确认消息已处理
                                    if let Err(err) = channel
                                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                                        .await
                                    {
                                        error!("Failed to acknowledge message: {}", err);
                                    }
                                }
                                Err(err) => {
                                    error!("Error processing message: {}", err);
                                    // 消息处理失败，拒绝消息并重新入队
                                    if let Err(e) = channel
                                        .basic_reject(delivery.delivery_tag, lapin::options::BasicRejectOptions { requeue: true })
                                        .await
                                    {
                                        error!("Failed to reject message: {}", e);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("Failed to receive message: {}", err);
                        }
                    }
                }
            })
                .await;
        });

        Ok(())
    }

    // 关闭连接
    pub async fn close(&self) -> Result<()> {
        self.connection.close(0, "Closing connection").await
            .context("Failed to close RabbitMQ connection")?;

        Ok(())
    }
}

// 初始化消息队列
pub async fn init_rabbitmq(config: &AppConfig) -> Result<MessageBroker> {
    MessageBroker::new(config).await
}
