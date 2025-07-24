use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::{PaymentType, OrderStatus};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentConfig {
    pub id: i64,
    pub tenant_id: i64,
    pub payment_type: i32,
    pub payment_sub_type: i32,
    pub merchant_id: String,
    pub app_id: Option<String>,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub gateway_url: String,
    pub notify_url: String,
    pub return_url: Option<String>,
    pub extra_config: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub tenant_id: i64,
    pub user_id: i64,
    pub payment_type: PaymentType,
    pub amount: i64,
    pub currency: String,
    pub product_name: String,
    pub product_desc: Option<String>,
    pub callback_url: Option<String>,
    pub notify_url: Option<String>,
    pub extra_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentResponse {
    pub order_id: String,
    pub payment_url: Option<String>,
    pub payment_params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub order_id: String,
    pub refund_amount: i64,
    pub refund_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub order_id: String,
    pub refund_amount: i64,
    pub status: String,
    pub third_party_refund_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCallbackRequest {
    pub payment_type: PaymentType,
    pub tenant_id: i64,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payment_request_serialization() {
        let request = CreatePaymentRequest {
            tenant_id: 1,
            user_id: 100,
            payment_type: PaymentType::WxH5,
            amount: 10000,
            currency: "CNY".to_string(),
            product_name: "Test Product".to_string(),
            product_desc: Some("Product description".to_string()),
            callback_url: Some("http://example.com/callback".to_string()),
            notify_url: Some("http://example.com/notify".to_string()),
            extra_data: Some(serde_json::json!({
                "custom": "value"
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreatePaymentRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tenant_id, request.tenant_id);
        assert_eq!(deserialized.payment_type, request.payment_type);
        assert_eq!(deserialized.amount, request.amount);
    }

    #[test]
    fn test_create_payment_response_serialization() {
        let response = CreatePaymentResponse {
            order_id: "order_12345".to_string(),
            payment_url: Some("http://pay.example.com/pay".to_string()),
            payment_params: Some(serde_json::json!({
                "appId": "wx123456",
                "timeStamp": "1619775012",
                "nonceStr": "random_string",
                "package": "prepay_id=123456",
                "signType": "MD5",
                "paySign": "signature"
            })),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: CreatePaymentResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.order_id, response.order_id);
        assert_eq!(deserialized.payment_url, response.payment_url);

        let params = deserialized.payment_params.unwrap();
        assert_eq!(params["appId"], "wx123456");
    }
}