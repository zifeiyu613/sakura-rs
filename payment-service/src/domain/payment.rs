use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::models::enums::{PaymentType, OrderStatus};
use crate::domain::{money::Money, events::{PaymentEvent, apply_event}};
use crate::error::PaymentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    pub id: Option<i64>,
    pub order_id: String,
    pub tenant_id: i64,
    pub user_id: i64,
    pub payment_type: PaymentType,
    pub amount: Money,
    pub status: OrderStatus,
    pub third_party_order_id: Option<String>,
    pub callback_url: Option<String>,
    pub notify_url: Option<String>,
    pub extra_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // 用于跟踪事件
    #[serde(skip)]
    pub(crate) events: Vec<PaymentEvent>,
}

impl PaymentOrder {
    pub fn new(
        tenant_id: i64,
        user_id: i64,
        payment_type: PaymentType,
        amount: Money,
        callback_url: Option<String>,
        notify_url: Option<String>,
        extra_data: Option<serde_json::Value>,
    ) -> Self {
        let now = Utc::now();
        let order_id = Uuid::new_v4().to_string();

        let mut order = Self {
            id: None,
            order_id,
            tenant_id,
            user_id,
            payment_type,
            amount,
            status: OrderStatus::Pending,
            third_party_order_id: None,
            callback_url,
            notify_url,
            extra_data,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        // 添加创建事件
        let _ = order.apply_event(PaymentEvent::OrderCreated {
            order_id: order.order_id.clone(),
            created_at: now,
        });

        order
    }

    pub fn apply_event(&mut self, event: PaymentEvent) -> Result<(), PaymentError> {
        // 确保事件适用于当前订单
        if event.order_id() != self.order_id {
            return Err(PaymentError::InvalidEvent {
                order_id: self.order_id.clone(),
                event_order_id: event.order_id().to_string(),
            });
        }

        // 应用状态转换
        match apply_event(self.status, &event) {
            Ok(new_status) => {
                self.status = new_status;
                self.updated_at = event.event_time();

                // 处理特定事件的额外逻辑
                match &event {
                    PaymentEvent::PaymentCompleted { third_party_order_id, .. } => {
                        self.third_party_order_id = Some(third_party_order_id.clone());
                    }
                    // 其他特定事件处理...
                    _ => {}
                }

                // 保存事件
                self.events.push(event);
                Ok(())
            },
            Err(msg) => Err(PaymentError::InvalidStateTransition {
                from: self.status,
                event: format!("{:?}", event),
            }),
        }
    }

    pub fn initiate_payment(&mut self, payment_url: Option<String>) -> Result<(), PaymentError> {
        self.apply_event(PaymentEvent::PaymentInitiated {
            order_id: self.order_id.clone(),
            payment_url,
            initiated_at: Utc::now(),
        })
    }

    pub fn complete_payment(&mut self, third_party_order_id: String) -> Result<(), PaymentError> {
        self.apply_event(PaymentEvent::PaymentCompleted {
            order_id: self.order_id.clone(),
            third_party_order_id,
            completed_at: Utc::now(),
        })
    }

    pub fn fail_payment(&mut self, reason: String) -> Result<(), PaymentError> {
        self.apply_event(PaymentEvent::PaymentFailed {
            order_id: self.order_id.clone(),
            error_reason: reason,
            failed_at: Utc::now(),
        })
    }

    pub fn request_refund(&mut self, refund_id: String, refund_amount: i64) -> Result<(), PaymentError> {
        self.apply_event(PaymentEvent::RefundRequested {
            order_id: self.order_id.clone(),
            refund_id,
            refund_amount,
            requested_at: Utc::now(),
        })
    }

    pub fn complete_refund(&mut self, refund_id: String) -> Result<(), PaymentError> {
        self.apply_event(PaymentEvent::RefundCompleted {
            order_id: self.order_id.clone(),
            refund_id,
            completed_at: Utc::now(),
        })
    }

    pub fn events(&self) -> &[PaymentEvent] {
        &self.events
    }

    pub fn clear_events(&mut self) -> Vec<PaymentEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::{Money, Currency};

    #[test]
    fn test_create_payment_order() {
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            PaymentType::WxH5,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.tenant_id, 1);
        assert_eq!(order.user_id, 100);
        assert_eq!(order.payment_type, PaymentType::WxH5);
        assert_eq!(order.amount.amount, 10000);
        assert_eq!(order.amount.currency, Currency::CNY);
        assert!(order.events.len() == 1);

        // 检查创建事件
        match &order.events[0] {
            PaymentEvent::OrderCreated { order_id, .. } => {
                assert_eq!(order_id, &order.order_id);
            },
            _ => panic!("Expected OrderCreated event"),
        }
    }

    #[test]
    fn test_payment_flow() {
        let mut order = PaymentOrder::new(
            1,
            100,
            PaymentType::WxH5,
            Money::cny(10000),
            None,
            None,
            None,
        );

        // Initial state
        assert_eq!(order.status, OrderStatus::Pending);

        // Initiate payment
        order.initiate_payment(Some("http://pay.example.com".to_string())).unwrap();
        assert_eq!(order.status, OrderStatus::Processing);

        // Complete payment
        order.complete_payment("third_party_order_123".to_string()).unwrap();
        assert_eq!(order.status, OrderStatus::Success);
        assert_eq!(order.third_party_order_id, Some("third_party_order_123".to_string()));

        // Request refund
        order.request_refund("refund_123".to_string(), 10000).unwrap();
        assert_eq!(order.status, OrderStatus::Refunded);

        // Complete refund
        order.complete_refund("refund_123".to_string()).unwrap();
        assert_eq!(order.status, OrderStatus::Refunded);

        // Check events
        assert_eq!(order.events().len(), 5);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut order = PaymentOrder::new(
            1,
            100,
            PaymentType::WxH5,
            Money::cny(10000),
            None,
            None,
            None,
        );

        // Cannot complete payment directly from Pending
        let result = order.complete_payment("test".to_string());
        assert!(result.is_err());

        // Correct flow
        order.initiate_payment(None).unwrap();
        assert_eq!(order.status, OrderStatus::Processing);

        let result = order.complete_payment("test".to_string());
        assert!(result.is_ok());
        assert_eq!(order.status, OrderStatus::Success);

        // Cannot initiate again after success
        let result = order.initiate_payment(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_order_id_validation() {
        let mut order = PaymentOrder::new(
            1,
            100,
            PaymentType::WxH5,
            Money::cny(10000),
            None,
            None,
            None,
        );

        // Try to apply event with wrong order_id
        let result = order.apply_event(PaymentEvent::PaymentInitiated {
            order_id: "wrong-id".to_string(),
            payment_url: None,
            initiated_at: Utc::now(),
        });

        assert!(result.is_err());
        match result.unwrap_err() {
            PaymentError::InvalidEvent { .. } => {},
            err => panic!("Expected InvalidEvent error, got: {:?}", err),
        }
    }
}