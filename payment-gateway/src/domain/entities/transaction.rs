use crate::domain::enums::{TransactionStatus, TransactionType};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub order_id: Uuid,
    pub channel_transaction_id: Option<String>,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub gateway_code: Option<String>,
    pub gateway_message: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        order_id: Uuid,
        amount: Decimal,
        transaction_type: TransactionType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            order_id,
            channel_transaction_id: None,
            amount,
            transaction_type,
            status: TransactionStatus::Pending,
            gateway_code: None,
            gateway_message: None,
            gateway_response: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: TransactionStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_channel_transaction_id(&mut self, id: String) {
        self.channel_transaction_id = Some(id);
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
        matches!(self.status, TransactionStatus::Success)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, TransactionStatus::Failed)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, TransactionStatus::Pending)
    }
}
