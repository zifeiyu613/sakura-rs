use crate::domain::entities::PaymentOrder;
use crate::domain::enums::PaymentStatus;
use crate::utils::errors::RepositoryError;
use sqlx::{MySqlPool, MySql, Transaction};
use uuid::Uuid;
use async_trait::async_trait;

#[async_trait]
pub trait OrderRepositoryTrait: Send + Sync {
    async fn create(&self, order: &PaymentOrder) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentOrder>, RepositoryError>;
    async fn find_by_merchant_order_id(
        &self,
        merchant_id: &str,
        merchant_order_id: &str,
    ) -> Result<Option<PaymentOrder>, RepositoryError>;
    async fn update_status(
        &self,
        id: Uuid,
        status: PaymentStatus,
    ) -> Result<(), RepositoryError>;
    async fn update(&self, order: &PaymentOrder) -> Result<(), RepositoryError>;
    async fn transaction(&self) -> Result<Transaction<'static, MySql>, RepositoryError>;
}

pub struct OrderRepository {
    db_pool: MySqlPool,
}

impl OrderRepository {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl OrderRepositoryTrait for OrderRepository {
    async fn create(&self, order: &PaymentOrder) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO payment_orders (
                id, merchant_id, merchant_order_id, amount, currency, status,
                channel, method, subject, description, callback_url, return_url,
                client_ip, metadata, expire_time, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            order.id,
            order.merchant_id,
            order.merchant_order_id,
            order.amount,
            order.currency.to_string(),
            order.status.to_string(),
            order.channel.to_string(),
            order.method.to_string(),
            order.subject,
            order.description,
            order.callback_url,
            order.return_url,
            order.client_ip,
            serde_json::to_string(&order.metadata).unwrap_or_else(|_| "{}".to_string()),
            order.expire_time,
            order.created_at,
            order.updated_at,
        )
            .execute(&self.db_pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentOrder>, RepositoryError> {
        let result = sqlx::query_as!(
            OrderRecord,
            r#"
            SELECT * FROM payment_orders WHERE id = ?
            "#,
            id
        )
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.map(|r| r.into()))
    }

    async fn find_by_merchant_order_id(
        &self,
        merchant_id: &str,
        merchant_order_id: &str,
    ) -> Result<Option<PaymentOrder>, RepositoryError> {
        let result = sqlx::query_as!(
            OrderRecord,
            r#"
            SELECT * FROM payment_orders
            WHERE merchant_id = ? AND merchant_order_id = ?
            "#,
            merchant_id,
            merchant_order_id
        )
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.map(|r| r.into()))
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: PaymentStatus,
    ) -> Result<(), RepositoryError> {
        let now = chrono::Utc::now();

        sqlx::query!(
            r#"
            UPDATE payment_orders
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
            status.to_string(),
            now,
            id
        )
            .execute(&self.db_pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, order: &PaymentOrder) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE payment_orders
            SET status = ?, description = ?, metadata = ?, updated_at = ?
            WHERE id = ?
            "#,
            order.status.to_string(),
            order.description,
            serde_json::to_string(&order.metadata).unwrap_or_else(|_| "{}".to_string()),
            order.updated_at,
            order.id
        )
            .execute(&self.db_pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn transaction(&self) -> Result<Transaction<'static, MySql>, RepositoryError> {
        self.db_pool
            .begin()
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))
    }
}

// 数据库记录结构体，用于sqlx
struct OrderRecord {
    id: Uuid,
    merchant_id: String,
    merchant_order_id: String,
    amount: rust_decimal::Decimal,
    currency: String,
    status: String,
    channel: String,
    method: String,
    subject: String,
    description: Option<String>,
    callback_url: String,
    return_url: Option<String>,
    client_ip: Option<String>,
    metadata: Option<String>,
    expire_time: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<OrderRecord> for PaymentOrder {
    fn from(record: OrderRecord) -> Self {
        use std::str::FromStr;

        Self {
            id: record.id,
            merchant_id: record.merchant_id,
            merchant_order_id: record.merchant_order_id,
            amount: record.amount,
            currency: crate::domain::enums::Currency::from_str(&record.currency).unwrap_or(crate::domain::enums::Currency::CNY),
            status: crate::domain::enums::PaymentStatus::from_str(&record.status).unwrap_or(crate::domain::enums::PaymentStatus::Created),
            channel: crate::domain::enums::PaymentChannel::from_str(&record.channel).unwrap_or(crate::domain::enums::PaymentChannel::Other),
            method: crate::domain::enums::PaymentMethod::from_str(&record.method).unwrap_or(crate::domain::enums::PaymentMethod::Web),
            subject: record.subject,
            description: record.description,
            callback_url: record.callback_url,
            return_url: record.return_url,
            client_ip: record.client_ip,
            metadata: record.metadata.and_then(|s| serde_json::from_str(&s).ok()),
            expire_time: record.expire_time,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
