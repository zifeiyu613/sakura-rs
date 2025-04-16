use crate::domain::enums::{PaymentStatus, PaymentChannel, PaymentMethod, Currency};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    pub id: Uuid,
    pub merchant_id: String,
    pub merchant_order_id: String,
    pub amount: Decimal,
    pub currency: Currency,
    pub status: PaymentStatus,
    pub channel: PaymentChannel,
    pub method: PaymentMethod,
    pub subject: String,
    pub description: Option<String>,
    pub callback_url: String,
    pub return_url: Option<String>,
    pub client_ip: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expire_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PaymentOrder {
    pub fn new(
        merchant_id: String,
        merchant_order_id: String,
        amount: Decimal,
        currency: Currency,
        channel: PaymentChannel,
        method: PaymentMethod,
        subject: String,
        callback_url: String,
        return_url: Option<String>,
        client_ip: Option<String>,
        metadata: Option<serde_json::Value>,
        expire_time: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            merchant_id,
            merchant_order_id,
            amount,
            currency,
            status: PaymentStatus::Created,
            channel,
            method,
            subject,
            description: None,
            callback_url,
            return_url,
            client_ip,
            metadata,
            expire_time,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_paid(&self) -> bool {
        matches!(self.status, PaymentStatus::Success)
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.status, PaymentStatus::Processing)
    }

    pub fn is_closed(&self) -> bool {
        matches!(
            self.status,
            PaymentStatus::Closed | PaymentStatus::Failed | PaymentStatus::Expired
        )
    }

    pub fn can_refund(&self) -> bool {
        matches!(self.status, PaymentStatus::Success)
    }

    pub fn update_status(&mut self, status: PaymentStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expire_time) = self.expire_time {
            Utc::now() > expire_time
        } else {
            false
        }
    }
}
