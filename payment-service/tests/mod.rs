#[cfg(test)]
mod payment_tests {
    use payment_service::domain::money::{Money, Currency};
    use payment_service::domain::payment::PaymentOrder;
    use payment_service::models::enums::{PaymentType, OrderStatus};
    use payment_service::domain::events::PaymentEvent;
    use chrono::Utc;

    #[test]
    fn test_payment_state_machine() {
        // 创建订单
        let mut order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            PaymentType::WxH5,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 测试初始状态
        assert_eq!(order.status, OrderStatus::Pending);

        // 测试发起支付
        order.initiate_payment(Some("http://pay.example.com".to_string())).unwrap();
        assert_eq!(order.status, OrderStatus::Processing);

        // 测试支付成功
        order.complete_payment("third_party_order_123".to_string()).unwrap();
        assert_eq!(order.status, OrderStatus::Success);
        assert_eq!(order.third_party_order_id, Some("third_party_order_123".to_string()));

        // 测试退款
        order.request_refund("refund_123".to_string(), 10000).unwrap();
        assert_eq!(order.status, OrderStatus::Refunded);

        // 验证事件数量
        assert_eq!(order.events().len(), 4);
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut order = PaymentOrder::new(
            1,
            100,
            PaymentType::WxH5,
            Money::cny(10000),
            None,
            None,
            None,
        );

        // 尝试直接从Pending到Success，应该失败
        let result = order.complete_payment("test".to_string());
        assert!(result.is_err());

        // 正确流程：Pending -> Processing -> Success
        order.initiate_payment(None).unwrap();
        let result = order.complete_payment("test".to_string());
        assert!(result.is_ok());
    }
}