use crate::domain::enums::RefundStatus;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refund {
    pub id: Uuid,
    pub order_id: Uuid,
    pub transaction_id: Uuid,
    pub channel_refund_id: Option<String>,
    pub amount: Decimal,
    pub reason: String,
    pub status: RefundStatus,
    pub gateway_code: Option<String>,
    pub gateway_message: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Refund {
    pub fn new(
        order_id: Uuid,
        transaction_id: Uuid,
        amount: Decimal,
        reason: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            order_id,
            transaction_id,
            channel_refund_id: None,
            amount,
            reason,
            status: RefundStatus::Pending,
            gateway_code: None,
            gateway_message: None,
            gateway_response: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: RefundStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_channel_refund_id(&mut self, id: String) {
        self.channel_refund_id = Some(id);
        self.updated_at = Utc::now();
    }

    pub fn set_gateway_response(
        &mut self,
        code: Option<String>,
        message: Option<String>,
        response: Option<serde_json::Value>,
    ) {
        self.gateway_code = code;
        self.gateway_message = message;
        self.gateway_response = response;
        self.updated_at = Utc::now();
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.status, RefundStatus::Success)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, RefundStatus::Failed)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, RefundStatus::Pending)
    }
}
