use crate::connection::get_rabbitmq_connection;
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::BasicProperties;
use serde::{Deserialize, Serialize};

pub async fn publish_message<T: Serialize>(
    exchange: &str,
    routing_key: &str,
    data: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = get_rabbitmq_connection().await;
    match connection.create_channel().await {
        Ok(channel) => {
            let payload_json = serde_json::to_vec(data)?;
            channel
                .basic_publish(
                    exchange,
                    routing_key,
                    BasicPublishOptions::default(),
                    &payload_json,
                    BasicProperties::default(),
                )
                .await?;
        }
        Err(err) => {
            println!("Error creating channel: {}", err);
            return Err(Box::new(err));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct OrderMessage {
        order_id: u32,
        status: String,
    }

    #[tokio::test]
    async fn test() {
        let data = OrderMessage {
            order_id: 100,
            status: "success".to_string(),
        };
        publish_message("", "rust_order_message", &data)
            .await
            .expect("producer error");
    }
}
