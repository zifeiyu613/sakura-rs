use async_trait::async_trait;
use sqlx::MySqlPool;
use chrono::Utc;
use crate::domain::payment::PaymentOrder;
use crate::error::PaymentError;
use crate::models::enums::{PaymentType, OrderStatus};
use crate::domain::money::{Money, Currency};

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn save(&self, order: &mut PaymentOrder) -> Result<(), PaymentError>;
    async fn find_by_id(&self, order_id: &str) -> Result<Option<PaymentOrder>, PaymentError>;
    async fn update_status(&self, order_id: &str, status: OrderStatus) -> Result<(), PaymentError>;
    async fn update_third_party_id(&self, order_id: &str, third_party_id: &str) -> Result<(), PaymentError>;
}

pub struct MySqlPaymentRepository {
    pool: MySqlPool,
}

impl MySqlPaymentRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepository for MySqlPaymentRepository {
    async fn save(&self, order: &mut PaymentOrder) -> Result<(), PaymentError> {
        let status_str = match order.status {
            OrderStatus::Pending => "PENDING",
            OrderStatus::Processing => "PROCESSING",
            OrderStatus::Success => "SUCCESS",
            OrderStatus::Failed => "FAILED",
            OrderStatus::Refunded => "REFUNDED",
            OrderStatus::PartialRefunded => "PARTIAL_REFUNDED",
        };

        let currency_str = match order.amount.currency {
            Currency::CNY => "CNY",
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
        };

        // 如果是新订单，则插入
        if order.id.is_none() {
            let result = sqlx::query!(
                r#"
                INSERT INTO payment_orders 
                (order_id, tenant_id, user_id, payment_type, payment_sub_type, 
                 amount, currency, status, third_party_order_id, callback_url, notify_url, extra_data, 
                 created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                order.order_id,
                order.tenant_id,
                order.user_id,
                order.payment_type.type_code(),
                order.payment_type.sub_type_code(),
                order.amount.amount,
                currency_str,
                status_str,
                order.third_party_order_id,
                order.callback_url,
                order.notify_url,
                order.extra_data.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()),
                order.created_at,
                order.updated_at
            )
                .execute(&self.pool)
                .await
                .map_err(PaymentError::Database)?;

            order.id = Some(result.last_insert_id() as i64);
        }
        // 否则更新
        else {
            sqlx::query!(
                r#"
                UPDATE payment_orders 
                SET status = ?, third_party_order_id = ?, updated_at = ?
                WHERE order_id = ?
                "#,
                status_str,
                order.third_party_order_id,
                order.updated_at,
                order.order_id
            )
                .execute(&self.pool)
                .await
                .map_err(PaymentError::Database)?;
        }

        Ok(())
    }

    async fn find_by_id(&self, order_id: &str) -> Result<Option<PaymentOrder>, PaymentError> {
        let row = sqlx::query!(
            r#"
            SELECT * FROM payment_orders WHERE order_id = ?
            "#,
            order_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(PaymentError::Database)?;

        if let Some(row) = row {
            // 转换为领域对象
            let payment_type = PaymentType::from_sub_type(row.payment_sub_type)
                .ok_or_else(|| PaymentError::InvalidPaymentType(row.payment_sub_type))?;

            let currency = match row.currency.as_str() {
                "CNY" => Currency::CNY,
                "USD" => Currency::USD,
                "EUR" => Currency::EUR,
                "GBP" => Currency::GBP,
                "JPY" => Currency::JPY,
                _ => Currency::CNY, // 默认
            };

            let status = match row.status.as_str() {
                "PENDING" => OrderStatus::Pending,
                "PROCESSING" => OrderStatus::Processing,
                "SUCCESS" => OrderStatus::Success,
                "FAILED" => OrderStatus::Failed,
                "REFUNDED" => OrderStatus::Refunded,
                "PARTIAL_REFUNDED" => OrderStatus::PartialRefunded,
                _ => OrderStatus::Pending,
            };

            // 反序列化extra_data
            let extra_data = if let Some(data_str) = &row.extra_data {
                serde_json::from_str(data_str).ok()
            } else {
                None
            };

            // 创建领域对象
            let order = PaymentOrder {
                id: Some(row.id),
                order_id: row.order_id,
                tenant_id: row.tenant_id,
                user_id: row.user_id,
                payment_type,
                amount: Money::new(row.amount, currency),
                status,
                third_party_order_id: row.third_party_order_id,
                callback_url: row.callback_url,
                notify_url: row.notify_url,
                extra_data,
                created_at: row.created_at,
                updated_at: row.updated_at,
                events: Vec::new(),
            };

            Ok(Some(order))
        } else {
            Ok(None)
        }
    }

    async fn update_status(&self, order_id: &str, status: OrderStatus) -> Result<(), PaymentError> {
        let status_str = match status {
            OrderStatus::Pending => "PENDING",
            OrderStatus::Processing => "PROCESSING",
            OrderStatus::Success => "SUCCESS",
            OrderStatus::Failed => "FAILED",
            OrderStatus::Refunded => "REFUNDED",
            OrderStatus::PartialRefunded => "PARTIAL_REFUNDED",
        };

        sqlx::query!(
            r#"
            UPDATE payment_orders 
            SET status = ?, updated_at = ?
            WHERE order_id = ?
            "#,
            status_str,
            Utc::now(),
            order_id
        )
            .execute(&self.pool)
            .await
            .map_err(PaymentError::Database)?;

        Ok(())
    }

    async fn update_third_party_id(&self, order_id: &str, third_party_id: &str) -> Result<(), PaymentError> {
        sqlx::query!(
            r#"
            UPDATE payment_orders 
            SET third_party_order_id = ?, updated_at = ?
            WHERE order_id = ?
            "#,
            third_party_id,
            Utc::now(),
            order_id
        )
            .execute(&self.pool)
            .await
            .map_err(PaymentError::Database)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::mysql::{MySqlPoolOptions, MySqlConnectOptions};
    use sqlx::ConnectOptions;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_payment_repository() -> Result<(), Box<dyn std::error::Error>> {
        let options = MySqlConnectOptions::from_str("mysql://root:password@localhost/payment_service_test")?
            .disable_statement_logging();
        let pool = MySqlPoolOptions::new().connect_with(options).await?;

        // 创建测试表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS payment_orders (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                order_id VARCHAR(64) NOT NULL UNIQUE,
                tenant_id BIGINT NOT NULL,
                user_id BIGINT NOT NULL,
                payment_type INT NOT NULL,
                payment_sub_type INT NOT NULL,
                amount BIGINT NOT NULL,
                currency VARCHAR(10) NOT NULL DEFAULT 'CNY',
                status VARCHAR(20) NOT NULL,
                third_party_order_id VARCHAR(255),
                callback_url VARCHAR(500),
                notify_url VARCHAR(500),
                extra_data JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                INDEX idx_tenant_user (tenant_id, user_id),
                INDEX idx_status (status),
                INDEX idx_created_at (created_at)
            )
            "#
        )
            .execute(&pool)
            .await?;

        // 清理可能存在的测试数据
        sqlx::query("DELETE FROM payment_orders WHERE tenant_id = 999")
            .execute(&pool)
            .await?;

        let repository = MySqlPaymentRepository::new(pool.clone());

        // 创建测试订单
        let mut order = PaymentOrder::new(
            999, // tenant_id
            888, // user_id
            PaymentType::WxH5,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            Some(serde_json::json!({ "test_key": "test_value" })),
        );

        // 保存订单
        repository.save(&mut order).await?;

        // 验证订单ID被设置
        assert!(order.id.is_some());

        // 查询订单
        let retrieved_order = repository.find_by_id(&order.order_id).await?;
        assert!(retrieved_order.is_some());

        let retrieved_order = retrieved_order.unwrap();
        assert_eq!(retrieved_order.tenant_id, 999);
        assert_eq!(retrieved_order.user_id, 888);
        assert_eq!(retrieved_order.payment_type, PaymentType::WxH5);
        assert_eq!(retrieved_order.amount.amount, 10000);
        assert_eq!(retrieved_order.status, OrderStatus::Pending);

        // 测试更新状态
        repository.update_status(&order.order_id, OrderStatus::Processing).await?;

        let updated_order = repository.find_by_id(&order.order_id).await?.unwrap();
        assert_eq!(updated_order.status, OrderStatus::Processing);

        // 测试更新第三方订单ID
        repository.update_third_party_id(&order.order_id, "third_party_123").await?;

        let updated_order = repository.find_by_id(&order.order_id).await?.unwrap();
        assert_eq!(updated_order.third_party_order_id, Some("third_party_123".to_string()));

        // 清理测试数据
        sqlx::query("DELETE FROM payment_orders WHERE tenant_id = 999")
            .execute(&pool)
            .await?;

        Ok(())
    }
}