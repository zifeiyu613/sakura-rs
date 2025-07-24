use async_trait::async_trait;
use crate::error::PaymentError;
use crate::models::payment::*;
use crate::models::enums::OrderStatus;
use crate::payment::strategy::PaymentStrategy;
use crate::domain::payment::PaymentOrder;

pub struct AppleIapStrategy;

impl AppleIapStrategy {
    pub fn new() -> Self {
        Self
    }

    // 验证 Apple 收据  
    async fn verify_receipt(&self, receipt_data: &str, is_sandbox: bool) -> Result<serde_json::Value, PaymentError> {
        // 实际实现中需要调用 Apple 的验证接口  
        // 这里简化处理，返回模拟的验证结果  

        Ok(serde_json::json!({  
            "status": 0,  
            "receipt": {  
                "in_app": [  
                    {  
                        "quantity": "1",  
                        "product_id": "com.example.product",  
                        "transaction_id": "1000000123456789",  
                        "original_transaction_id": "1000000123456789",  
                        "purchase_date": "2023-05-20 12:34:56 Etc/GMT",  
                        "purchase_date_ms": "1684581296000",  
                        "original_purchase_date": "2023-05-20 12:34:56 Etc/GMT",  
                        "original_purchase_date_ms": "1684581296000",  
                        "expires_date": "2023-06-20 12:34:56 Etc/GMT",  
                        "expires_date_ms": "1687173296000",  
                        "is_trial_period": "false",  
                        "is_in_intro_offer_period": "false"  
                    }  
                ]  
            }  
        }))
    }
}

#[async_trait]
impl PaymentStrategy for AppleIapStrategy {
    async fn create_order(
        &self,
        order: &PaymentOrder,
        _config: &PaymentConfig,
        _request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        // Apple IAP 通常在客户端完成，这里只需要记录订单  
        Ok(CreatePaymentResponse {
            order_id: order.order_id.clone(),
            payment_url: None,
            payment_params: None,
        })
    }

    async fn query_order(
        &self,
        order: &PaymentOrder,
        _config: &PaymentConfig,
    ) -> Result<OrderStatus, PaymentError> {
        // 对于 Apple IAP，通常不会主动查询订单状态  
        // 而是依赖客户端回调或收据验证  

        // 如果订单状态不是 Pending，则直接返回当前状态  
        if order.status != OrderStatus::Pending {
            return Ok(order.status);
        }

        // 否则返回处理中状态  
        Ok(OrderStatus::Processing)
    }

    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError> {
        // 从回调数据中获取订单ID  
        let order_id = callback_data["order_id"]
            .as_str()
            .ok_or_else(|| PaymentError::Internal("Missing order_id in callback data".to_string()))?
            .to_string();

        // 从回调数据中获取收据数据  
        let receipt_data = callback_data["receipt-data"]
            .as_str()
            .ok_or_else(|| PaymentError::Internal("Missing receipt-data in callback data".to_string()))?;

        // 检查是否是沙箱环境  
        let is_sandbox = config.extra_config
            .as_ref()
            .and_then(|c| c.get("is_sandbox"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // 验证收据  
        let verification_response = self.verify_receipt(receipt_data, is_sandbox).await?;

        // 解析验证结果  
        let status = verification_response["status"].as_i64().unwrap_or(1);

        if status == 0 {
            // 验证成功  
            Ok((order_id, OrderStatus::Success))
        } else {
            // 验证失败  
            let error_message = format!("Apple receipt verification failed with status: {}", status);
            Err(PaymentError::ExternalApi {
                code: status.to_string(),
                message: error_message,
            })
        }
    }

    async fn refund(
        &self,
        _order: &PaymentOrder,
        _config: &PaymentConfig,
        _refund_request: &RefundRequest,
    ) -> Result<String, PaymentError> {
        // Apple IAP 退款需要用户自己在 App Store 操作，不提供API退款
        // 或者开发者通过 App Store Connect 手动处理

        return Err(PaymentError::UnsupportedOperation(
            "Apple IAP does not support API refund. User must request refund via App Store.".to_string()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Money;

    #[tokio::test]
    async fn test_apple_iap_create_order() {
        let strategy = AppleIapStrategy::new();

        // 创建测试订单
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            crate::models::enums::PaymentType::AppleIap,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 1,
            payment_sub_type: 1,
            merchant_id: "com.example.app".to_string(),
            app_id: Some("com.example.app".to_string()),
            private_key: None,
            public_key: None,
            api_key: None,
            api_secret: None,
            gateway_url: "https://buy.itunes.apple.com/verifyReceipt".to_string(),
            notify_url: "https://www.example.com/notify".to_string(),
            return_url: None,
            extra_config: Some(serde_json::json!({
                "is_sandbox": true
            })),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 创建支付请求
        let request = CreatePaymentRequest {
            tenant_id: 1,
            user_id: 100,
            payment_type: crate::models::enums::PaymentType::AppleIap,
            amount: 10000,
            currency: "CNY".to_string(),
            product_name: "测试商品".to_string(),
            product_desc: Some("商品描述".to_string()),
            callback_url: Some("http://example.com/callback".to_string()),
            notify_url: Some("http://example.com/notify".to_string()),
            extra_data: None,
        };

        // 测试创建订单
        let result = strategy.create_order(&order, &config, &request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.order_id, order.order_id);
        assert!(response.payment_url.is_none());
        assert!(response.payment_params.is_none());
    }

    #[tokio::test]
    async fn test_apple_iap_handle_callback() {
        let strategy = AppleIapStrategy::new();

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 1,
            payment_sub_type: 1,
            merchant_id: "com.example.app".to_string(),
            app_id: Some("com.example.app".to_string()),
            private_key: None,
            public_key: None,
            api_key: None,
            api_secret: None,
            gateway_url: "https://buy.itunes.apple.com/verifyReceipt".to_string(),
            notify_url: "https://www.example.com/notify".to_string(),
            return_url: None,
            extra_config: Some(serde_json::json!({
                "is_sandbox": true
            })),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 创建回调数据
        let callback_data = serde_json::json!({
            "order_id": "test_order_123",
            "receipt-data": "BASE64_ENCODED_RECEIPT_DATA",
            "transaction_id": "1000000123456789",
            "product_id": "com.example.product"
        });

        // 测试处理回调
        let result = strategy.handle_callback(&config, &callback_data).await;
        assert!(result.is_ok());

        let (order_id, status) = result.unwrap();
        assert_eq!(order_id, "test_order_123");
        assert_eq!(status, OrderStatus::Success);
    }

    #[tokio::test]
    async fn test_apple_iap_refund() {
        let strategy = AppleIapStrategy::new();

        // 创建测试订单
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            crate::models::enums::PaymentType::AppleIap,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 1,
            payment_sub_type: 1,
            merchant_id: "com.example.app".to_string(),
            app_id: Some("com.example.app".to_string()),
            private_key: None,
            public_key: None,
            api_key: None,
            api_secret: None,
            gateway_url: "https://buy.itunes.apple.com/verifyReceipt".to_string(),
            notify_url: "https://www.example.com/notify".to_string(),
            return_url: None,
            extra_config: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 创建退款请求
        let refund_request = RefundRequest {
            order_id: order.order_id.clone(),
            refund_amount: 10000,
            refund_reason: Some("测试退款".to_string()),
        };

        // 测试退款
        let result = strategy.refund(&order, &config, &refund_request).await;
        assert!(result.is_err());

        match result {
            Err(PaymentError::UnsupportedOperation(_)) => {
                // 预期的错误类型
            },
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }
}