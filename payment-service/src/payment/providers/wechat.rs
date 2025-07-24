use async_trait::async_trait;
use crate::error::PaymentError;
use crate::models::payment::*;
use crate::models::enums::OrderStatus;
use crate::payment::strategy::PaymentStrategy;
use crate::domain::payment::PaymentOrder;

pub struct WechatH5Strategy;

impl WechatH5Strategy {
    pub fn new() -> Self {
        Self
    }

    // 私有方法，用于签名
    fn sign(&self, params: &serde_json::Map<String, serde_json::Value>, api_key: &str) -> String {
        // 实际实现中需要按照微信支付的签名规则进行签名
        // 这里简化处理
        "mocked_signature".to_string()
    }
}

#[async_trait]
impl PaymentStrategy for WechatH5Strategy {
    async fn create_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        // 实现微信H5支付订单创建逻辑
        // 1. 构建请求参数
        let params = serde_json::json!({
            "appid": config.app_id,
            "mch_id": config.merchant_id,
            "nonce_str": uuid::Uuid::new_v4().to_string().replace("-", ""),
            "body": request.product_name,
            "out_trade_no": order.order_id,
            "total_fee": order.amount.amount,
            "spbill_create_ip": "127.0.0.1", // 实际实现中应该从请求中获取
            "notify_url": config.notify_url,
            "trade_type": "MWEB", // H5支付
            "scene_info": serde_json::json!({
                "h5_info": {
                    "type": "Wap",
                    "wap_url": "https://www.example.com",
                    "wap_name": "支付示例"
                }
            })
        });

        // 2. 在实际实现中，这里需要进行签名和调用微信API
        // 这里简化处理，模拟返回一个支付URL

        // 3. 解析响应
        let payment_url = format!("https://wx.tenpay.com/cgi-bin/mmpayweb-bin/checkmweb?prepay_id=wx123456&package=1234567890");

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
        // 在实际实现中，需要调用微信查询订单API
        // 这里简化处理，模拟返回成功状态

        // 模拟API调用
        let api_response = serde_json::json!({
            "return_code": "SUCCESS",
            "result_code": "SUCCESS",
            "trade_state": "SUCCESS",
            "transaction_id": "4200000123456789",
            "out_trade_no": order.order_id,
            "total_fee": order.amount.amount
        });

        // 解析响应
        let trade_state = api_response["trade_state"].as_str().unwrap_or("UNKNOWN");

        match trade_state {
            "SUCCESS" => Ok(OrderStatus::Success),
            "REFUND" => Ok(OrderStatus::Refunded),
            "NOTPAY" | "USERPAYING" => Ok(OrderStatus::Processing),
            "CLOSED" | "REVOKED" | "PAYERROR" => Ok(OrderStatus::Failed),
            _ => Ok(OrderStatus::Processing),
        }
    }

    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError> {
        // 1. 验证签名
        // 实际实现中需要验证微信回调的签名

        // 2. 解析订单号和支付状态
        let order_id = callback_data["out_trade_no"]
            .as_str()
            .ok_or_else(|| PaymentError::Internal("Missing out_trade_no in callback data".to_string()))?
            .to_string();

        let result_code = callback_data["result_code"]
            .as_str()
            .unwrap_or("FAIL");

        let status = if result_code == "SUCCESS" {
            OrderStatus::Success
        } else {
            OrderStatus::Failed
        };

        Ok((order_id, status))
    }

    async fn refund(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        refund_request: &RefundRequest,
    ) -> Result<String, PaymentError> {
        // 实现微信退款逻辑
        // 1. 构建退款请求参数
        let refund_id = uuid::Uuid::new_v4().to_string();
        let params = serde_json::json!({
            "appid": config.app_id,
            "mch_id": config.merchant_id,
            "nonce_str": uuid::Uuid::new_v4().to_string().replace("-", ""),
            "out_trade_no": order.order_id,
            "out_refund_no": &refund_id,
            "total_fee": order.amount.amount,
            "refund_fee": refund_request.refund_amount,
            "refund_desc": refund_request.refund_reason.clone().unwrap_or_else(|| "客户退款".to_string())
        });

        // 2. 在实际实现中，这里需要进行签名和调用微信退款API
        // 这里简化处理，模拟返回一个退款单号

        Ok(refund_id)
    }
}

pub struct WechatSdkStrategy;

impl WechatSdkStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PaymentStrategy for WechatSdkStrategy {
    // 实现微信SDK支付相关方法
    async fn create_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        // 实现微信SDK支付参数生成
        // 这里与H5支付的区别是返回的是APP需要的支付参数，而不是支付URL

        let prepay_id = format!("wx{}", chrono::Utc::now().timestamp());

        let payment_params = serde_json::json!({
            "appid": config.app_id,
            "partnerid": config.merchant_id,
            "prepayid": prepay_id,
            "package": "Sign=WXPay",
            "noncestr": uuid::Uuid::new_v4().to_string().replace("-", ""),
            "timestamp": chrono::Utc::now().timestamp().to_string(),
            "sign": "mocked_signature"
        });

        Ok(CreatePaymentResponse {
            order_id: order.order_id.clone(),
            payment_url: None,
            payment_params: Some(payment_params),
        })
    }

    async fn query_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
    ) -> Result<OrderStatus, PaymentError> {
        // 查询逻辑与H5支付相同
        let wx_h5 = WechatH5Strategy::new();
        wx_h5.query_order(order, config).await
    }

    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError> {
        // 回调处理逻辑与H5支付相同
        let wx_h5 = WechatH5Strategy::new();
        wx_h5.handle_callback(config, callback_data).await
    }

    async fn refund(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        refund_request: &RefundRequest,
    ) -> Result<String, PaymentError> {
        // 退款逻辑与H5支付相同
        let wx_h5 = WechatH5Strategy::new();
        wx_h5.refund(order, config, refund_request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Money;

    #[tokio::test]
    async fn test_wechat_h5_create_order() {
        let strategy = WechatH5Strategy::new();

        // 创建测试订单
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            crate::models::enums::PaymentType::WxH5,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 5,
            payment_sub_type: 5,
            merchant_id: "1234567890".to_string(),
            app_id: Some("wxabcdef1234567890".to_string()),
            private_key: None,
            public_key: None,
            api_key: Some("test_api_key".to_string()),
            api_secret: None,
            gateway_url: "https://api.mch.weixin.qq.com".to_string(),
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
            payment_type: crate::models::enums::PaymentType::WxH5,
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
        assert!(response.payment_url.unwrap().contains("wx.tenpay.com"));
    }

    #[tokio::test]
    async fn test_wechat_sdk_create_order() {
        let strategy = WechatSdkStrategy::new();

        // 创建测试订单
        let order = PaymentOrder::new(
            1, // tenant_id
            100, // user_id
            crate::models::enums::PaymentType::WxSdk,
            Money::cny(10000), // 100元
            Some("http://example.com/callback".to_string()),
            Some("http://example.com/notify".to_string()),
            None,
        );

        // 创建测试配置
        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 2,
            payment_sub_type: 2,
            merchant_id: "1234567890".to_string(),
            app_id: Some("wxabcdef1234567890".to_string()),
            private_key: None,
            public_key: None,
            api_key: Some("test_api_key".to_string()),
            api_secret: None,
            gateway_url: "https://api.mch.weixin.qq.com".to_string(),
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
            payment_type: crate::models::enums::PaymentType::WxSdk,
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
        assert!(params.get("appid").is_some());
        assert!(params.get("prepayid").is_some());
        assert!(params.get("sign").is_some());
    }
}