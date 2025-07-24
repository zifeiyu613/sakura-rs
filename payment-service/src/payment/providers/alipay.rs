use async_trait::async_trait;
use crate::error::PaymentError;
use crate::models::payment::*;
use crate::models::enums::OrderStatus;
use crate::payment::strategy::PaymentStrategy;
use crate::domain::payment::PaymentOrder;

pub struct AlipayH5Strategy;

impl AlipayH5Strategy {
    pub fn new() -> Self {
        Self
    }

    // 私有方法，用于签名
    fn sign(&self, params: &serde_json::Map<String, serde_json::Value>, private_key: &str) -> String {
        // 实际实现中需要按照支付宝的签名规则进行签名
        // 这里简化处理
        "mocked_signature".to_string()
    }
}

#[async_trait]
impl PaymentStrategy for AlipayH5Strategy {
    async fn create_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        // 实现支付宝H5支付订单创建逻辑
        // 1. 构建请求参数
        let biz_content = serde_json::json!({
            "out_trade_no": order.order_id,
            "total_amount": (order.amount.amount as f64 / 100.0).to_string(), // 转换为元
            "subject": request.product_name,
            "product_code": "QUICK_WAP_WAY",
            "body": request.product_desc.clone().unwrap_or_default()
        });

        let params = serde_json::json!({
            "app_id": config.app_id,
            "method": "alipay.trade.wap.pay",
            "charset": "utf-8",
            "sign_type": "RSA2",
            "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "version": "1.0",
            "notify_url": config.notify_url,
            "return_url": config.return_url,
            "biz_content": biz_content.to_string()
        });

        // 2. 在实际实现中，这里需要进行签名
        // 这里简化处理，直接拼接URL

        // 3. 生成支付URL
        let mut query_string = String::new();
        if let serde_json::Value::Object(map) = params {
            for (key, value) in map {
                if let Some(value_str) = value.as_str() {
                    if !query_string.is_empty() {
                        query_string.push('&');
                    }
                    query_string.push_str(&format!("{}={}", key, urlencoding::encode(value_str)));
                }
            }
        }

        let payment_url = format!("{}?{}&sign={}", config.gateway_url, query_string, "mocked_signature");

        Ok(CreatePaymentResponse {
            order_id: order.order_id.clone(),
            payment_url: Some(payment_url),
            payment_params: None,
        })
    }

    async fn query_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
    ) -> Result<OrderStatus, PaymentError> {
        // 在实际实现中，需要调用支付宝查询订单API
        // 这里简化处理，模拟返回成功状态

        // 模拟API调用
        let api_response = serde_json::json!({
            "alipay_trade_query_response": {
                "code": "10000",
                "msg": "Success",
                "trade_no": "2021123112345678",
                "out_trade_no": order.order_id,
                "trade_status": "TRADE_SUCCESS",
                "total_amount": (order.amount.amount as f64 / 100.0).to_string()
            }
        });

        // 解析响应
        let trade_status = api_response["alipay_trade_query_response"]["trade_status"].as_str().unwrap_or("WAIT_BUYER_PAY");

        match trade_status {
            "TRADE_SUCCESS" | "TRADE_FINISHED" => Ok(OrderStatus::Success),
            "TRADE_CLOSED" => Ok(OrderStatus::Refunded),
            "WAIT_BUYER_PAY" => Ok(OrderStatus::Processing),
            _ => Ok(OrderStatus::Processing),
        }
    }

    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError> {
        // 1. 验证签名
        // 实际实现中需要验证支付宝回调的签名

        // 2. 解析订单号和支付状态
        let order_id = callback_data["out_trade_no"]
            .as_str()
            .ok_or_else(|| PaymentError::Internal("Missing out_trade_no in callback data".to_string()))?
            .to_string();

        let trade_status = callback_data["trade_status"]
            .as_str()
            .unwrap_or("WAIT_BUYER_PAY");

        let status = match trade_status {
            "TRADE_SUCCESS" | "TRADE_FINISHED" => OrderStatus::Success,
            "TRADE_CLOSED" => OrderStatus::Refunded,
            "WAIT_BUYER_PAY" => OrderStatus::Processing,
            _ => OrderStatus::Processing,
        };

        Ok((order_id, status))
    }

    async fn refund(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        refund_request: &RefundRequest,
    ) -> Result<String, PaymentError> {
        // 实现支付宝退款逻辑
        // 1. 构建退款请求参数
        let refund_id = uuid::Uuid::new_v4().to_string();
        let biz_content = serde_json::json!({
            "out_trade_no": order.order_id,
            "refund_amount": (refund_request.refund_amount as f64 / 100.0).to_string(),
            "out_request_no": refund_id,
            "refund_reason": refund_request.refund_reason.clone().unwrap_or_else(|| "客户退款".to_string())
        });

        let params = serde_json::json!({
            "app_id": config.app_id,
            "method": "alipay.trade.refund",
            "charset": "utf-8",
            "sign_type": "RSA2",
            "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "version": "1.0",
            "biz_content": biz_content.to_string()
        });

        // 2. 在实际实现中，这里需要进行签名和调用支付宝退款API
        // 这里简化处理，模拟返回一个退款单号

        Ok(refund_id)
    }
}

pub struct AlipaySdkStrategy;

impl AlipaySdkStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PaymentStrategy for AlipaySdkStrategy {
    // 实现支付宝SDK支付相关方法
    async fn create_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        // 实现支付宝SDK支付参数生成
        // 这里与H5支付的区别是返回的是APP需要的支付参数，而不是支付URL

        let biz_content = serde_json::json!({
            "out_trade_no": order.order_id,
            "total_amount": (order.amount.amount as f64 / 100.0).to_string(), // 转换为元
            "subject": request.product_name,
            "product_code": "QUICK_MSECURITY_PAY",
            "body": request.product_desc.clone().unwrap_or_default()
        });

        let params = serde_json::json!({
            "app_id": config.app_id,
            "method": "alipay.trade.app.pay",
            "charset": "utf-8",
            "sign_type": "RSA2",
            "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "version": "1.0",
            "notify_url": config.notify_url,
            "biz_content": biz_content.to_string()
        });

        // 在实际实现中，这里需要进行签名
        // 这里简化处理，直接拼接字符串

        let mut order_string = String::new();
        if let serde_json::Value::Object(map) = params {
            for (key, value) in map {
                if let Some(value_str) = value.as_str() {
                    if !order_string.is_empty() {
                        order_string.push('&');
                    }
                    order_string.push_str(&format!("{}={}", key, urlencoding::encode(value_str)));
                }
            }
        }

        order_string.push_str("&sign=mocked_signature");

        Ok(CreatePaymentResponse {
            order_id: order.order_id.clone(),
            payment_url: None,
            payment_params: Some(serde_json::json!({
                "orderString": order_string
            })),
        })
    }

    async fn query_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
    ) -> Result<OrderStatus, PaymentError> {
        // 查询逻辑与H5支付相同
        let alipay_h5 = AlipayH5Strategy::new();
        alipay_h5.query_order(order, config).await
    }

    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError> {
        // 回调处理逻辑与H5支付相同
        let alipay_h5 = AlipayH5Strategy::new();
        alipay_h5.handle_callback(config, callback_data).await
    }

    async fn refund(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        refund_request: &RefundRequest,
    ) -> Result<String, PaymentError> {
        // 退款逻辑与H5支付相同
        let alipay_h5 = AlipayH5Strategy::new();
        alipay_h5.refund(order, config, refund_request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Money;

    #[tokio::test]
    async fn test_alipay_h5_create_order() {
        let strategy = AlipayH5Strategy::new();

        // 创建测试订单
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            crate::models::enums::PaymentType::ZfbH5,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 6,
            payment_sub_type: 6,
            merchant_id: "2088123456789012".to_string(),
            app_id: Some("2021000123456789".to_string()),
            private_key: Some("-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgw\n-----END PRIVATE KEY-----".to_string()),
            public_key: Some("-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A\n-----END PUBLIC KEY-----".to_string()),
            api_key: None,
            api_secret: None,
            gateway_url: "https://openapi.alipay.com/gateway.do".to_string(),
            notify_url: "https://www.example.com/notify".to_string(),
            return_url: Some("https://www.example.com/return".to_string()),
            extra_config: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 创建支付请求
        let request = CreatePaymentRequest {
            tenant_id: 1,
            user_id: 100,
            payment_type: crate::models::enums::PaymentType::ZfbH5,
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
        assert!(response.payment_url.is_some());
        assert!(response.payment_url.unwrap().contains("openapi.alipay.com"));
    }

    #[tokio::test]
    async fn test_alipay_sdk_create_order() {
        let strategy = AlipaySdkStrategy::new();

        // 创建测试订单
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            crate::models::enums::PaymentType::ZfbSdk,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 3,
            payment_sub_type: 3,
            merchant_id: "2088123456789012".to_string(),
            app_id: Some("2021000123456789".to_string()),
            private_key: Some("-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgw\n-----END PRIVATE KEY-----".to_string()),
            public_key: Some("-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A\n-----END PUBLIC KEY-----".to_string()),
            api_key: None,
            api_secret: None,
            gateway_url: "https://openapi.alipay.com/gateway.do".to_string(),
            notify_url: "https://www.example.com/notify".to_string(),
            return_url: None,
            extra_config: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // 创建支付请求
        let request = CreatePaymentRequest {
            tenant_id: 1,
            user_id: 100,
            payment_type: crate::models::enums::PaymentType::ZfbSdk,
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
        assert!(response.payment_params.is_some());

        let params = response.payment_params.unwrap();
        assert!(params.get("orderString").is_some());
    }
}