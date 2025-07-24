use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::models::enums::OrderStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentEvent {
    OrderCreated {
        order_id: String,
        created_at: DateTime<Utc>,
    },
    PaymentInitiated {
        order_id: String,
        payment_url: Option<String>,
        initiated_at: DateTime<Utc>,
    },
    PaymentCompleted {
        order_id: String,
        third_party_order_id: String,
        completed_at: DateTime<Utc>,
    },
    PaymentFailed {
        order_id: String,
        error_reason: String,
        failed_at: DateTime<Utc>,
    },
    RefundRequested {
        order_id: String,
        refund_id: String,
        refund_amount: i64,
        requested_at: DateTime<Utc>,
    },
    RefundCompleted {
        order_id: String,
        refund_id: String,
        completed_at: DateTime<Utc>,
    },
}

impl PaymentEvent {
    pub fn order_id(&self) -> &str {
        match self {
            Self::OrderCreated { order_id, .. } => order_id,
            Self::PaymentInitiated { order_id, .. } => order_id,
            Self::PaymentCompleted { order_id, .. } => order_id,
            Self::PaymentFailed { order_id, .. } => order_id,
            Self::RefundRequested { order_id, .. } => order_id,
            Self::RefundCompleted { order_id, .. } => order_id,
        }
    }

    pub fn event_time(&self) -> DateTime<Utc> {
        match self {
            Self::OrderCreated { created_at, .. } => *created_at,
            Self::PaymentInitiated { initiated_at, .. } => *initiated_at,
            Self::PaymentCompleted { completed_at, .. } => *completed_at,
            Self::PaymentFailed { failed_at, .. } => *failed_at,
            Self::RefundRequested { requested_at, .. } => *requested_at,
            Self::RefundCompleted { completed_at, .. } => *completed_at,
        }
    }
}

pub fn apply_event(current_status: OrderStatus, event: &PaymentEvent) -> Result<OrderStatus, &'static str> {
    match (current_status, event) {
        (OrderStatus::Pending, PaymentEvent::OrderCreated { .. }) => Ok(OrderStatus::Pending),
        (OrderStatus::Pending, PaymentEvent::PaymentInitiated { .. }) => Ok(OrderStatus::Processing),
        (OrderStatus::Processing, PaymentEvent::PaymentCompleted { .. }) => Ok(OrderStatus::Success),
        (OrderStatus::Processing, PaymentEvent::PaymentFailed { .. }) => Ok(OrderStatus::Failed),
        (OrderStatus::Success, PaymentEvent::RefundRequested { .. }) => Ok(OrderStatus::Refunded),
        (OrderStatus::Refunded, PaymentEvent::RefundCompleted { .. }) => Ok(OrderStatus::Refunded),
        _ => Err("Invalid state transition"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_properties() {
        let order_id = "test123".to_string();
        let now = Utc::now();

        let event = PaymentEvent::OrderCreated {
            order_id: order_id.clone(),
            created_at: now,
        };

        assert_eq!(event.order_id(), order_id);
        assert_eq!(event.event_time(), now);
    }

    #[test]
    fn test_valid_state_transitions() {
        // Pending -> Pending (OrderCreated)
        assert_eq!(
            apply_event(
                OrderStatus::Pending,
                &PaymentEvent::OrderCreated {
                    order_id: "test".to_string(),
                    created_at: Utc::now()
                }
            ),
            Ok(OrderStatus::Pending)
        );

        // Pending -> Processing (PaymentInitiated)
        assert_eq!(
            apply_event(
                OrderStatus::Pending,
                &PaymentEvent::PaymentInitiated {
                    order_id: "test".to_string(),
                    payment_url: None,
                    initiated_at: Utc::now()
                }
            ),
            Ok(OrderStatus::Processing)
        );

        // Processing -> Success (PaymentCompleted)
        assert_eq!(
            apply_event(
                OrderStatus::Processing,
                &PaymentEvent::PaymentCompleted {
                    order_id: "test".to_string(),
                    third_party_order_id: "ext123".to_string(),
                    completed_at: Utc::now()
                }
            ),
            Ok(OrderStatus::Success)
        );
    }

    #[test]
    fn test_invalid_state_transitions() {
        // Success -> Processing is invalid
        assert_eq!(
            apply_event(
                OrderStatus::Success,
                &PaymentEvent::PaymentInitiated {
                    order_id: "test".to_string(),
                    payment_url: None,
                    initiated_at: Utc::now()
                }
            ),
            Err("Invalid state transition")
        );

        // Failed -> Refunded is invalid
        assert_eq!(
            apply_event(
                OrderStatus::Failed,
                &PaymentEvent::RefundRequested {
                    order_id: "test".to_string(),
                    refund_id: "refund123".to_string(),
                    refund_amount: 1000,
                    requested_at: Utc::now()
                }
            ),
            Err("Invalid state transition")
        );
    }
}