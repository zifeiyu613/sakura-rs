use std::error::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use sqlx::MySqlPool;
use crate::domain::models::{
    PaymentChannelType, PaymentMethodType, PaymentOrder, PaymentRegion,
    PaymentStatus, PaymentTransaction, RefundOrder
};

// 仓库接口
#[async_trait::async_trait]
pub trait PaymentOrderRepository: Send + Sync {
    async fn save(&self, order: &PaymentOrder) -> Result<(), Box<dyn Error>>;
    async fn update(&self, order: &PaymentOrder) -> Result<(), Box<dyn Error>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<PaymentOrder>, Box<dyn Error>>;
    async fn find_by_order_id(&self, order_id: &str) -> Result<Option<PaymentOrder>, Box<dyn Error>>;
    async fn find_by_merchant_id(&self, merchant_id: &str, limit: i64, offset: i64) -> Result<Vec<PaymentOrder>, Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait PaymentTransactionRepository: Send + Sync {
    async fn save(&self, transaction: &PaymentTransaction) -> Result<(), Box<dyn Error>>;
    async fn update(&self, transaction: &PaymentTransaction) -> Result<(), Box<dyn Error>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<PaymentTransaction>, Box<dyn Error>>;
    async fn find_by_transaction_id(&self, transaction_id: &str) -> Result<Option<PaymentTransaction>, Box<dyn Error>>;
    async fn find_by_channel_transaction_id(&self, channel_transaction_id: &str) -> Result<Option<PaymentTransaction>, Box<dyn Error>>;
    async fn find_by_payment_order_id(&self, payment_order_id: &str) -> Result<Vec<PaymentTransaction>, Box<dyn Error>>;
}

#[async_trait::async_trait]
pub trait RefundOrderRepository: Send + Sync {
    async fn save(&self, refund: &RefundOrder) -> Result<(), Box<dyn Error>>;
    async fn update(&self, refund: &RefundOrder) -> Result<(), Box<dyn Error>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<RefundOrder>, Box<dyn Error>>;
    async fn find_by_refund_id(&self, refund_id: &str) -> Result<Option<RefundOrder>, Box<dyn Error>>;
    async fn find_by_payment_order_id(&self, payment_order_id: &str) -> Result<Vec<RefundOrder>, Box<dyn Error>>;
    async fn find_by_transaction_id(&self, transaction_id: &str) -> Result<Vec<RefundOrder>, Box<dyn Error>>;
}

// 支付订单仓库实现
pub struct PaymentOrderRepositoryImpl {
    pool: MySqlPool,
}

impl PaymentOrderRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PaymentOrderRepository for PaymentOrderRepositoryImpl {
    async fn save(&self, order: &PaymentOrder) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            INSERT INTO payment_orders (
                id, merchant_id, order_id, amount, currency, status, channel,
                method, region, subject, description, metadata, created_at,
                updated_at, expires_at, callback_url, return_url, client_ip
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            "#,
        )
            .bind(&order.id)
            .bind(&order.merchant_id)
            .bind(&order.order_id)
            .bind(order.amount)
            .bind(&order.currency)
            .bind(format!("{:?}", order.status))
            .bind(format!("{:?}", order.channel))
            .bind(format!("{:?}", order.method))
            .bind(format!("{:?}", order.region))
            .bind(&order.subject)
            .bind(&order.description)
            .bind(serde_json::to_value(&order.metadata)?)
            .bind(order.created_at)
            .bind(order.updated_at)
            .bind(order.expires_at)
            .bind(&order.callback_url)
            .bind(&order.return_url)
            .bind(&order.client_ip)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, order: &PaymentOrder) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            UPDATE payment_orders SET
                merchant_id = $2,
                order_id = $3,
                amount = $4,
                currency = $5,
                status = $6,
                channel = $7,
                method = $8,
                region = $9,
                subject = $10,
                description = $11,
                metadata = $12,
                updated_at = $13,
                expires_at = $14,
                callback_url = $15,
                return_url = $16,
                client_ip = $17
            WHERE id = $1
            "#,
        )
            .bind(&order.id)
            .bind(&order.merchant_id)
            .bind(&order.order_id)
            .bind(order.amount)
            .bind(&order.currency)
            .bind(format!("{:?}", order.status))
            .bind(format!("{:?}", order.channel))
            .bind(format!("{:?}", order.method))
            .bind(format!("{:?}", order.region))
            .bind(&order.subject)
            .bind(&order.description)
            .bind(serde_json::to_value(&order.metadata)?)
            .bind(order.updated_at)
            .bind(order.expires_at)
            .bind(&order.callback_url)
            .bind(&order.return_url)
            .bind(&order.client_ip)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<PaymentOrder>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, merchant_id, order_id, amount, currency, status, channel,
                method, region, subject, description, metadata, created_at,
                updated_at, expires_at, callback_url, return_url, client_ip
            FROM payment_orders
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(PaymentOrder {
                    id: r.id,
                    merchant_id: r.merchant_id,
                    order_id: r.order_id,
                    amount: r.amount,
                    currency: r.currency,
                    status: parse_payment_status(&r.status)?,
                    channel: parse_payment_channel_type(&r.channel)?,
                    method: parse_payment_method_type(&r.method)?,
                    region: parse_payment_region(&r.region)?,
                    subject: r.subject,
                    description: r.description,
                    metadata,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    expires_at: r.expires_at,
                    callback_url: r.callback_url,
                    return_url: r.return_url,
                    client_ip: r.client_ip,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_order_id(&self, order_id: &str) -> Result<Option<PaymentOrder>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, merchant_id, order_id, amount, currency, status, channel,
                method, region, subject, description, metadata, created_at,
                updated_at, expires_at, callback_url, return_url, client_ip
            FROM payment_orders
            WHERE order_id = $1
            "#,
            order_id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(PaymentOrder {
                    id: r.id,
                    merchant_id: r.merchant_id,
                    order_id: r.order_id,
                    amount: r.amount,
                    currency: r.currency,
                    status: parse_payment_status(&r.status)?,
                    channel: parse_payment_channel_type(&r.channel)?,
                    method: parse_payment_method_type(&r.method)?,
                    region: parse_payment_region(&r.region)?,
                    subject: r.subject,
                    description: r.description,
                    metadata,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    expires_at: r.expires_at,
                    callback_url: r.callback_url,
                    return_url: r.return_url,
                    client_ip: r.client_ip,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_merchant_id(&self, merchant_id: &str, limit: i64, offset: i64) -> Result<Vec<PaymentOrder>, Box<dyn Error>> {
        let records = sqlx::query!(
            r#"
            SELECT
                id, merchant_id, order_id, amount, currency, status, channel,
                method, region, subject, description, metadata, created_at,
                updated_at, expires_at, callback_url, return_url, client_ip
            FROM payment_orders
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            merchant_id,
            limit,
            offset
        )
            .fetch_all(&self.pool)
            .await?;

        let mut orders = Vec::new();
        for r in records {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

            orders.push(PaymentOrder {
                id: r.id,
                merchant_id: r.merchant_id,
                order_id: r.order_id,
                amount: r.amount,
                currency: r.currency,
                status: parse_payment_status(&r.status)?,
                channel: parse_payment_channel_type(&r.channel)?,
                method: parse_payment_method_type(&r.method)?,
                region: parse_payment_region(&r.region)?,
                subject: r.subject,
                description: r.description,
                metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
                callback_url: r.callback_url,
                return_url: r.return_url,
                client_ip: r.client_ip,
            });
        }

        Ok(orders)
    }
}

// 支付交易仓库实现
pub struct PaymentTransactionRepositoryImpl {
    pool: PgPool,
}

impl PaymentTransactionRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PaymentTransactionRepository for PaymentTransactionRepositoryImpl {
    async fn save(&self, transaction: &PaymentTransaction) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            INSERT INTO payment_transactions (
                id, payment_order_id, transaction_id, channel_transaction_id, amount,
                status, created_at, updated_at, metadata, error_code, error_message
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            )
            "#,
        )
            .bind(&transaction.id)
            .bind(&transaction.payment_order_id)
            .bind(&transaction.transaction_id)
            .bind(&transaction.channel_transaction_id)
            .bind(transaction.amount)
            .bind(format!("{:?}", transaction.status))
            .bind(transaction.created_at)
            .bind(transaction.updated_at)
            .bind(serde_json::to_value(&transaction.metadata)?)
            .bind(&transaction.error_code)
            .bind(&transaction.error_message)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, transaction: &PaymentTransaction) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            UPDATE payment_transactions SET
                payment_order_id = $2,
                transaction_id = $3,
                channel_transaction_id = $4,
                amount = $5,
                status = $6,
                updated_at = $7,
                metadata = $8,
                error_code = $9,
                error_message = $10
            WHERE id = $1
            "#,
        )
            .bind(&transaction.id)
            .bind(&transaction.payment_order_id)
            .bind(&transaction.transaction_id)
            .bind(&transaction.channel_transaction_id)
            .bind(transaction.amount)
            .bind(format!("{:?}", transaction.status))
            .bind(transaction.updated_at)
            .bind(serde_json::to_value(&transaction.metadata)?)
            .bind(&transaction.error_code)
            .bind(&transaction.error_message)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<PaymentTransaction>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, channel_transaction_id, amount,
                status, created_at, updated_at, metadata, error_code, error_message
            FROM payment_transactions
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(PaymentTransaction {
                    id: r.id,
                    payment_order_id: r.payment_order_id,
                    transaction_id: r.transaction_id,
                    channel_transaction_id: r.channel_transaction_id,
                    amount: r.amount,
                    status: parse_payment_status(&r.status)?,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    metadata,
                    error_code: r.error_code,
                    error_message: r.error_message,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_transaction_id(&self, transaction_id: &str) -> Result<Option<PaymentTransaction>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, channel_transaction_id, amount,
                status, created_at, updated_at, metadata, error_code, error_message
            FROM payment_transactions
            WHERE transaction_id = $1
            "#,
            transaction_id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(PaymentTransaction {
                    id: r.id,
                    payment_order_id: r.payment_order_id,
                    transaction_id: r.transaction_id,
                    channel_transaction_id: r.channel_transaction_id,
                    amount: r.amount,
                    status: parse_payment_status(&r.status)?,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    metadata,
                    error_code: r.error_code,
                    error_message: r.error_message,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_channel_transaction_id(&self, channel_transaction_id: &str) -> Result<Option<PaymentTransaction>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, channel_transaction_id, amount,
                status, created_at, updated_at, metadata, error_code, error_message
            FROM payment_transactions
            WHERE channel_transaction_id = $1
            "#,
            channel_transaction_id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(PaymentTransaction {
                    id: r.id,
                    payment_order_id: r.payment_order_id,
                    transaction_id: r.transaction_id,
                    channel_transaction_id: r.channel_transaction_id,
                    amount: r.amount,
                    status: parse_payment_status(&r.status)?,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    metadata,
                    error_code: r.error_code,
                    error_message: r.error_message,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_payment_order_id(&self, payment_order_id: &str) -> Result<Vec<PaymentTransaction>, Box<dyn Error>> {
        let records = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, channel_transaction_id, amount,
                status, created_at, updated_at, metadata, error_code, error_message
            FROM payment_transactions
            WHERE payment_order_id = $1
            ORDER BY created_at DESC
            "#,
            payment_order_id
        )
            .fetch_all(&self.pool)
            .await?;

        let mut transactions = Vec::new();
        for r in records {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

            transactions.push(PaymentTransaction {
                id: r.id,
                payment_order_id: r.payment_order_id,
                transaction_id: r.transaction_id,
                channel_transaction_id: r.channel_transaction_id,
                amount: r.amount,
                status: parse_payment_status(&r.status)?,
                created_at: r.created_at,
                updated_at: r.updated_at,
                metadata,
                error_code: r.error_code,
                error_message: r.error_message,
            });
        }

        Ok(transactions)
    }
}

// 退款订单仓库实现
pub struct RefundOrderRepositoryImpl {
    pool: PgPool,
}

impl RefundOrderRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RefundOrderRepository for RefundOrderRepositoryImpl {
    async fn save(&self, refund: &RefundOrder) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            INSERT INTO refund_orders (
                id, payment_order_id, transaction_id, amount, reason, status,
                refund_id, channel_refund_id, created_at, updated_at, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            )
            "#,
        )
            .bind(&refund.id)
            .bind(&refund.payment_order_id)
            .bind(&refund.transaction_id)
            .bind(refund.amount)
            .bind(&refund.reason)
            .bind(format!("{:?}", refund.status))
            .bind(&refund.refund_id)
            .bind(&refund.channel_refund_id)
            .bind(refund.created_at)
            .bind(refund.updated_at)
            .bind(serde_json::to_value(&refund.metadata)?)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, refund: &RefundOrder) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            r#"
            UPDATE refund_orders SET
                payment_order_id = $2,
                transaction_id = $3,
                amount = $4,
                reason = $5,
                status = $6,
                refund_id = $7,
                channel_refund_id = $8,
                updated_at = $9,
                metadata = $10
            WHERE id = $1
            "#,
        )
            .bind(&refund.id)
            .bind(&refund.payment_order_id)
            .bind(&refund.transaction_id)
            .bind(refund.amount)
            .bind(&refund.reason)
            .bind(format!("{:?}", refund.status))
            .bind(&refund.refund_id)
            .bind(&refund.channel_refund_id)
            .bind(refund.updated_at)
            .bind(serde_json::to_value(&refund.metadata)?)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<RefundOrder>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, amount, reason, status,
                refund_id, channel_refund_id, created_at, updated_at, metadata
            FROM refund_orders
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(RefundOrder {
                    id: r.id,
                    payment_order_id: r.payment_order_id,
                    transaction_id: r.transaction_id,
                    amount: r.amount,
                    reason: r.reason,
                    status: parse_payment_status(&r.status)?,
                    refund_id: r.refund_id,
                    channel_refund_id: r.channel_refund_id,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    metadata,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_refund_id(&self, refund_id: &str) -> Result<Option<RefundOrder>, Box<dyn Error>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, amount, reason, status,
                refund_id, channel_refund_id, created_at, updated_at, metadata
            FROM refund_orders
            WHERE refund_id = $1
            "#,
            refund_id
        )
            .fetch_optional(&self.pool)
            .await?;

        match record {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

                Ok(Some(RefundOrder {
                    id: r.id,
                    payment_order_id: r.payment_order_id,
                    transaction_id: r.transaction_id,
                    amount: r.amount,
                    reason: r.reason,
                    status: parse_payment_status(&r.status)?,
                    refund_id: r.refund_id,
                    channel_refund_id: r.channel_refund_id,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    metadata,
                }))
            },
            None => Ok(None),
        }
    }

    async fn find_by_payment_order_id(&self, payment_order_id: &str) -> Result<Vec<RefundOrder>, Box<dyn Error>> {
        let records = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, amount, reason, status,
                refund_id, channel_refund_id, created_at, updated_at, metadata
            FROM refund_orders
            WHERE payment_order_id = $1
            ORDER BY created_at DESC
            "#,
            payment_order_id
        )
            .fetch_all(&self.pool)
            .await?;

        let mut refunds = Vec::new();
        for r in records {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

            refunds.push(RefundOrder {
                id: r.id,
                payment_order_id: r.payment_order_id,
                transaction_id: r.transaction_id,
                amount: r.amount,
                reason: r.reason,
                status: parse_payment_status(&r.status)?,
                refund_id: r.refund_id,
                channel_refund_id: r.channel_refund_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                metadata,
            });
        }

        Ok(refunds)
    }

    async fn find_by_transaction_id(&self, transaction_id: &str) -> Result<Vec<RefundOrder>, Box<dyn Error>> {
        let records = sqlx::query!(
            r#"
            SELECT
                id, payment_order_id, transaction_id, amount, reason, status,
                refund_id, channel_refund_id, created_at, updated_at, metadata
            FROM refund_orders
            WHERE transaction_id = $1
            ORDER BY created_at DESC
            "#,
            transaction_id
        )
            .fetch_all(&self.pool)
            .await?;

        let mut refunds = Vec::new();
        for r in records {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata.unwrap_or_default())?;

            refunds.push(RefundOrder {
                id: r.id,
                payment_order_id: r.payment_order_id,
                transaction_id: r.transaction_id,
                amount: r.amount,
                reason: r.reason,
                status: parse_payment_status(&r.status)?,
                refund_id: r.refund_id,
                channel_refund_id: r.channel_refund_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                metadata,
            });
        }

        Ok(refunds)
    }
}

// 辅助函数：从字符串解析支付状态
fn parse_payment_status(status: &str) -> Result<PaymentStatus, Box<dyn Error>> {
    match status {
        "Created" => Ok(PaymentStatus::Created),
        "Processing" => Ok(PaymentStatus::Processing),
        "Successful" => Ok(PaymentStatus::Successful),
        "Failed" => Ok(PaymentStatus::Failed),
        "Cancelled" => Ok(PaymentStatus::Cancelled),
        "Refunded" => Ok(PaymentStatus::Refunded),
        "PartiallyRefunded" => Ok(PaymentStatus::PartiallyRefunded),
        _ => Err(format!("Invalid payment status: {}", status).into()),
    }
}

// 辅助函数：从字符串解析支付渠道类型
fn parse_payment_channel_type(channel: &str) -> Result<PaymentChannelType, Box<dyn Error>> {
    match channel {
        "WechatPay" => Ok(PaymentChannelType::WechatPay),
        "AliPay" => Ok(PaymentChannelType::AliPay),
        "UnionPay" => Ok(PaymentChannelType::UnionPay),
        "PayPal" => Ok(PaymentChannelType::PayPal),
        "Stripe" => Ok(PaymentChannelType::Stripe),
        "BoostWallet" => Ok(PaymentChannelType::BoostWallet),
        _ => Err(format!("Invalid payment channel: {}", channel).into()),
    }
}

// 辅助函数：从字符串解析支付方式类型
fn parse_payment_method_type(method: &str) -> Result<PaymentMethodType, Box<dyn Error>> {
    match method {
        "App" => Ok(PaymentMethodType::App),
        "H5" => Ok(PaymentMethodType::H5),
        "JsApi" => Ok(PaymentMethodType::JsApi),
        "Native" => Ok(PaymentMethodType::Native),
        "Web" => Ok(PaymentMethodType::Web),
        "Wallet" => Ok(PaymentMethodType::Wallet),
        "BoostWallet" => Ok(PaymentMethodType::BoostWallet),
        _ => Err(format!("Invalid payment method: {}", method).into()),
    }
}

// 辅助函数：从字符串解析支付地区
fn parse_payment_region(region: &str) -> Result<PaymentRegion, Box<dyn Error>> {
    match region {
        "China" => Ok(PaymentRegion::China),
        "HongKong" => Ok(PaymentRegion::HongKong),
        "Malaysia" => Ok(PaymentRegion::Malaysia),
        "Singapore" => Ok(PaymentRegion::Singapore),
        "Global" => Ok(PaymentRegion::Global),
        _ => Err(format!("Invalid payment region: {}", region).into()),
    }
}