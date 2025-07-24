use crate::connection::get_rabbitmq_connection;
use crate::error::MessageQueueError;
use async_trait::async_trait;
use deadpool_lapin::lapin::types::FieldTable;
use futures::StreamExt;
use lapin::options::BasicConsumeOptions;
// use lapin::types::FieldTable;
use lapin::ExchangeKind;

/// **定义通用消费者接口**
#[async_trait]
pub trait RabbitMQConsumer {
    async fn handle_message(&self, message: &[u8]) -> Result<(), MessageQueueError>;
}

/// **启动 RabbitMQ 消费者（支持多个交换机 & 队列）**
pub async fn start_consumer(
    exchange: &str,
    queue: &str,
    binding_key: &str,
    consumer: impl RabbitMQConsumer + Send + Sync + 'static,
) -> Result<(), MessageQueueError> {
    let connection = get_rabbitmq_connection().await;
    let channel = connection.create_channel().await.map_err(|e|MessageQueueError::GenericError(e.to_string()))?;

    // 声明队列
    let queue = channel.queue_declare(
        queue,
        Default::default(),
        FieldTable::default()
    ).await?;

    if !exchange.is_empty() {
        channel
            .exchange_declare(
                exchange,
                ExchangeKind::Direct,
                Default::default(),
                FieldTable::default(),
            )
            .await?;

        // 绑定队列到交换机
        channel
            .queue_bind(
                queue.name().as_str(),
                exchange,
                binding_key,
                Default::default(),
                FieldTable::default(),
            )
            .await?;
    }

    let mut consumer_stream = channel
        .basic_consume(
            queue.name().as_str(),
            "consumer_tag",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tokio::spawn(async move {
        while let Some(delivery) = consumer_stream.next().await {
            match delivery {
                Ok(delivery) => {
                    let data = delivery.data.clone();

                    // 处理消息
                    match consumer.handle_message(&data).await {
                        Ok(_) => {
                            // 处理成功，确认消息
                            if let Err(e) = delivery.ack(Default::default()).await {
                                eprintln!("❌ 确认消息失败: {}", e);
                            }
                        },
                        Err(e) => {
                            eprintln!("❌ 处理消息失败: {}", e);
                            // 处理失败，可能会选择不确认消息
                            // 你可以选择重试，或者将消息重新放回队列
                            if let Err(e) = delivery.nack(Default::default()).await {
                                eprintln!("❌ 重试消息失败: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ RabbitMQ 消息消费错误: {}", e);
                }
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::producer::publish_message;
    use serde::{Deserialize, Serialize};

    #[tokio::test]
    async fn consumer_test() {
        #[derive(Serialize, Deserialize, Debug)]
        struct OrderMessage {
            order_id: u32,
            status: String,
        }

        struct TestMessageConsumer;

   
        #[async_trait]
        impl RabbitMQConsumer for TestMessageConsumer {
            async fn handle_message(
                &self,
                message: &[u8],
            ) -> Result<(), MessageQueueError> {
                let msg: OrderMessage = serde_json::from_slice(message)?;
                println!("消费者 获取消息: {:?}", msg);
                Ok(())
            }
        }

        let data = OrderMessage {
            order_id: 212,
            status: "rust_order_test11".to_string(),
        };

        publish_message("", "rust_order_message", &data).await.unwrap();

        start_consumer(
            "",
            "rust_order_message",
            "rust_order_message",
            TestMessageConsumer,
        ).await.unwrap();


        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
