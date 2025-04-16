use crate::adapters::{
    PaymentAdapter, PaymentResponse, PaymentStatusResponse,
    RefundResponse, RefundStatusResponse, NotificationResponse
};
use crate::config::AppConfig;
use crate::domain::entities::{PaymentOrder, Refund};
use crate::domain::enums::PaymentMethod;
use crate::utils::errors::AdapterError;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use chrono::Utc;
use tracing::{info, error};
use std::collections::HashMap;

pub mod web;
pub mod h5;
pub mod app;
pub mod mini_program;
pub mod qr_code;

#[derive(Debug, Clone)]
pub struct AlipayAdapter {
    config: AppConfig,
    client: Client,
    app_id: String,
    private_key: String,
    alipay_public_key: String,
    gateway_url: String,
}

impl AlipayAdapter {
    pub fn new(config: AppConfig) -> Self {
        let alipay_config = &config.payment_channels.alipay;
        Self {
            config,
            client: Client::new(),
            app_id: alipay_config.app_id.clone(),
            private_key: alipay_config.private_key.clone(),
            alipay_public_key: alipay_config.public_key.clone(),
            gateway_url: "https://openapi.alipay.com/gateway.do".to_string(),
        }
    }

    // 生成支付宝签名
    fn generate_sign(&self, params: &HashMap<String, String>) -> Result<String, AdapterError> {
        // 按字典序排序参数
        let mut sorted_keys: Vec<&String> = params.keys().collect();
        sorted_keys.sort();

        // 构建待签名字符串
        let mut sign_content = String::new();
        for key in sorted_keys {
            if let Some(value) = params.get(key) {
                if !value.is_empty() && key != "sign" {
                    if !sign_content.is_empty() {
                        sign_content.push('&');
                    }
                    sign_content.push_str(&format!("{}={}", key, value));
                }
            }
        }

        // RSA签名
        let key = openssl::rsa::Rsa::private_key_from_pem(self.private_key.as_bytes())
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        let mut signer = openssl::sign::Signer::new(
            openssl::hash::MessageDigest::sha256(),
            &key
        ).map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        signer.update(sign_content.as_bytes())
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        let signature = signer.sign_to_vec()
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        Ok(base64::encode(signature))
    }

    // 验证支付宝签名
    fn verify_sign(&self, params: &HashMap<String, String>, sign: &str) -> Result<bool, AdapterError> {
        // 按字典序排序参数
        let mut sorted_keys: Vec<&String> = params.keys().collect();
        sorted_keys.sort();

        // 构建待验证字符串
        let mut sign_content = String::new();
        for key in sorted_keys {
            if let Some(value) = params.get(key) {
                if !value.is_empty() && key != "sign" && key != "sign_type" {
                    if !sign_content.is_empty() {
                        sign_content.push('&');
                    }
                    sign_content.push_str(&format!("{}={}", key, value));
                }
            }
        }

        // RSA验证
        let key = openssl::rsa::Rsa::public_key_from_pem(self.alipay_public_key.as_bytes())
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        let key = openssl::pkey::PKey::from_rsa(key)
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        let mut verifier = openssl::sign::Verifier::new(
            openssl::hash::MessageDigest::sha256(),
            &key
        ).map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        verifier.update(sign_content.as_bytes())
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        let sign_bytes = base64::decode(sign)
            .map_err(|e| AdapterError::CryptoError(e.to_string()))?;

        verifier.verify(&sign_bytes)
            .map_err(|e| AdapterError::CryptoError(e.to_string()))
    }

    // 路由到具体的支付方式实现
    async fn route_to_payment_method(
        &self,
        order: &PaymentOrder,
    ) -> Result<PaymentResponse, AdapterError> {
        match order.method {
            PaymentMethod::Web => web::create_payment(self, order).await,
            PaymentMethod::H5 => h5::create_payment(self, order).await,
            PaymentMethod::App => app::create_payment(self, order).await,
            PaymentMethod::MiniProgram => mini_program::create_payment(self, order).await,
            PaymentMethod::QrCode => qr_code::create_payment(self, order).await,
            _ => Err(AdapterError::UnsupportedPaymentMethod(format!(
                "Unsupported payment method: {:?} for Alipay",
                order.method
            ))),
        }
    }

    // 构建公共请求参数
    fn build_common_params(&self, method: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("app_id".to_string(), self.app_id.clone());
        params.insert("method".to_string(), method.to_string());
        params.insert("format".to_string(), "JSON".to_string());
        params.insert("charset".to_string(), "utf-8".to_string());
        params.insert("sign_type".to_string(), "RSA2".to_string());
        params.insert("timestamp".to_string(), Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        params.insert("version".to_string(), "1.0".to_string());

        params
    }

    // 发送请求到支付宝
    async fn send_request(&self, params: HashMap<String, String>) -> Result<serde_json::Value, AdapterError> {
        // 生成签名
        let sign = self.generate_sign(&params)?;
        let mut request_params = params.clone();
        request_params.insert("sign".to_string(), sign);

        // 发送请求
        let response = self.client
            .post(&self.gateway_url)
            .form(&request_params)
            .send()
            .await
            .map_err(|e| AdapterError::NetworkError(e.to_string()))?;

        let response_text = response.text().await
            .map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;

        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;

        Ok(response_json)
    }
}

#[async_trait]
impl PaymentAdapter for AlipayAdapter {
    fn name(&self) -> &'static str {
        "alipay"
    }

    async fn create_payment(&self, order: &PaymentOrder) -> Result<PaymentResponse, AdapterError> {
        info!("Creating Alipay payment for order: {}", order.id);
        self.route_to_payment_method(order).await
    }

    async fn query_payment(&self, order: &PaymentOrder) -> Result<PaymentStatusResponse, AdapterError> {
        info!("Querying Alipay payment status for order: {}", order.id);

        let mut params = self.build_common_params("alipay.trade.query");

        // 设置业务参数
        let biz_content = serde_json::json!({
            "out_trade_no": order.merchant_order_id,
        });

        params.insert("biz_content".to_string(), biz_content.to_string());

        let response_json = self.send_request(params).await?;

        // 解析响应
        let response_key = "alipay_trade_query_response";

        if !response_json.as_object().unwrap().contains_key(response_key) {
            return Err(AdapterError::ResponseParseError(format!("Response does not contain {}", response_key)));
        }

        let response_data = &response_json[response_key];
        let code = response_data["code"].as_str().unwrap_or("");

        if code != "10000" {
            let sub_msg = response_data["sub_msg"].as_str().unwrap_or("Unknown error");
            return Err(AdapterError::ChannelError(sub_msg.to_string()));
        }

        let trade_status = response_data["trade_status"].as_str().unwrap_or("");
        let is_paid = trade_status == "TRADE_SUCCESS" || trade_status == "TRADE_FINISHED";

        let transaction_id = response_data["trade_no"].as_str().map(|s| s.to_string());

        let paid_amount = response_data["total_amount"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|n| rust_decimal::Decimal::from_f64(n).unwrap_or_default());

        let paid_time = response_data["send_pay_date"].as_str().and_then(|t| {
            chrono::DateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        Ok(PaymentStatusResponse {
            is_paid,
            transaction_id,
            paid_amount,
            paid_time,
            raw_response: response_json,
        })
    }

    async fn handle_notification(&self, notification_data: &str) -> Result<NotificationResponse, AdapterError> {
        info!("Handling Alipay payment notification");

        // 将通知数据解析为HashMap
        let mut params: HashMap<String, String> = serde_urlencoded::from_str(notification_data)
            .map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;

        // 验证签名
        let sign = params.get("sign").ok_or_else(|| AdapterError::InvalidSignature("Missing signature".to_string()))?;

        let is_valid = self.verify_sign(&params, sign)?;

        if !is_valid {
            return Err(AdapterError::InvalidSignature("Alipay notification signature validation failed".to_string()));
        }

        let app_id = params.get("app_id").ok_or_else(|| AdapterError::ResponseParseError("Missing app_id".to_string()))?;

        if app_id != &self.app_id {
            return Err(AdapterError::ChannelError("App ID mismatch".to_string()));
        }

        let trade_status = params.get("trade_status").ok_or_else(|| AdapterError::ResponseParseError("Missing trade_status".to_string()))?;

        let is_successful = trade_status == "TRADE_SUCCESS" || trade_status == "TRADE_FINISHED";

        let transaction_id = params.get("trade_no").ok_or_else(|| AdapterError::ResponseParseError("Missing trade_no".to_string()))?.to_string();

        let order_id = params.get("out_trade_no").ok_or_else(|| AdapterError::ResponseParseError("Missing out_trade_no".to_string()))?.to_string();

        let amount = params.get("total_amount").ok_or_else(|| AdapterError::ResponseParseError("Missing total_amount".to_string()))?;
        let amount = amount.parse::<f64>().map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;
        let amount = rust_decimal::Decimal::from_f64(amount).unwrap_or_default();

        let paid_time = params.get("gmt_payment").and_then(|t| {
            chrono::DateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        let raw_data = serde_json::to_value(&params).unwrap_or_default();

        let response_data = "success".to_string();

        Ok(NotificationResponse {
            transaction_id,
            order_id,
            is_successful,
            amount,
            paid_time,
            raw_data,
            response_data,
        })
    }

    async fn create_refund(&self, refund: &Refund, order: &PaymentOrder) -> Result<RefundResponse, AdapterError> {
        info!("Creating Alipay refund for order: {}, refund: {}", order.id, refund.id);

        let mut params = self.build_common_params("alipay.trade.refund");

        // 设置业务参数
        let biz_content = serde_json::json!({
            "out_trade_no": order.merchant_order_id,
            "refund_amount": refund.amount.to_string(),
            "out_request_no": refund.id.to_string(),
            "refund_reason": refund.reason,
        });

        params.insert("biz_content".to_string(), biz_content.to_string());

        let response_json = self.send_request(params).await?;

        // 解析响应
        let response_key = "alipay_trade_refund_response";

        if !response_json.as_object().unwrap().contains_key(response_key) {
            return Err(AdapterError::ResponseParseError(format!("Response does not contain {}", response_key)));
        }

        let response_data = &response_json[response_key];
        let code = response_data["code"].as_str().unwrap_or("");

        if code != "10000" {
            let sub_msg = response_data["sub_msg"].as_str().unwrap_or("Unknown error");
            return Err(AdapterError::ChannelError(sub_msg.to_string()));
        }

        let is_accepted = true;
        let channel_refund_id = response_data["trade_no"].as_str().map(|s| s.to_string());

        Ok(RefundResponse {
            channel_refund_id,
            is_accepted,
            raw_response: response_json,
        })
    }

    async fn query_refund(&self, refund: &Refund, order: &PaymentOrder) -> Result<RefundStatusResponse, AdapterError> {
        info!("Querying Alipay refund status for order: {}, refund: {}", order.id, refund.id);

        let mut params = self.build_common_params("alipay.trade.fastpay.refund.query");

        // 设置业务参数
        let biz_content = serde_json::json!({
            "out_trade_no": order.merchant_order_id,
            "out_request_no": refund.id.to_string(),
        });

        params.insert("biz_content".to_string(), biz_content.to_string());

        let response_json = self.send_request(params).await?;

        // 解析响应
        let response_key = "alipay_trade_fastpay_refund_query_response";

        if !response_json.as_object().unwrap().contains_key(response_key) {
            return Err(AdapterError::ResponseParseError(format!("Response does not contain {}", response_key)));
        }

        let response_data = &response_json[response_key];
        let code = response_data["code"].as_str().unwrap_or("");

        if code != "10000" {
            let sub_msg = response_data["sub_msg"].as_str().unwrap_or("Unknown error");
            return Err(AdapterError::ChannelError(sub_msg.to_string()));
        }

        let refund_status = response_data["refund_status"].as_str().unwrap_or("");
        let is_success = refund_status == "REFUND_SUCCESS";

        let refund_id = response_data["trade_no"].as_str().map(|s| s.to_string());

        let refunded_amount = response_data["refund_amount"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|n| rust_decimal::Decimal::from_f64(n).unwrap_or_default());

        let refund_time = response_data["gmt_refund_pay"].as_str().and_then(|t| {
            chrono::DateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        Ok(RefundStatusResponse {
            is_success,
            refund_id,
            refunded_amount,
            refund_time,
            raw_response: response_json,
        })
    }
}
