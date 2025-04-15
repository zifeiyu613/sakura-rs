use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

use crate::domain::payment::{
    PaymentConfig, PaymentMethodType, PaymentProcessor, PaymentRegion,
    PaymentRequest, PaymentResponse, PaymentResult, RefundRequest, RefundResponse
};
use crate::domain::models::{PaymentStatus, PaymentTransaction, RefundOrder};
use crate::infrastructure::utils::crypto::generate_hmac_sha256;

pub struct BoostWalletProcessor {
    config: PaymentConfig,
    client: Client,
}

impl BoostWalletProcessor {
    pub fn new(config: PaymentConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    // 生成请求认证头
    fn generate_auth_headers(&self, payload: &str) -> HashMap<String, String> {
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let nonce = Uuid::new_v4().to_string().replace("-", "");

        // 生成签名内容: timestamp + nonce + payload
        let sign_content = format!("{}{}{}", timestamp, nonce, payload);
        let signature = generate_hmac_sha256(&sign_content, &self.config.api_key);

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.config.app_id));
        headers.insert("X-Timestamp".to_string(), timestamp);
        headers.insert("X-Nonce".to_string(), nonce);
        headers.insert("X-Signature".to_string(), signature);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        headers
    }
}

#[async_trait]
impl PaymentProcessor for BoostWalletProcessor {
    fn get_supported_methods(&self) -> Vec<PaymentMethodType> {
        vec![PaymentMethodType::BoostWallet]
    }

    fn get_supported_regions(&self) -> Vec<PaymentRegion> {
        vec![PaymentRegion::Malaysia]
    }

    async fn create_payment(&self, request: PaymentRequest) -> PaymentResult<PaymentResponse> {
        // 创建Boost支付请求
        let payload = json!({
            "order": {
                "id": request.order.order_id,
                "title": request.order.subject,
                "description": request.order.description,
                "amount": request.order.amount.to_string(),
                "currency": request.order.currency
            },
            "redirectUrl": request.order.return_url,
            "callbackUrl": request.order.callback_url,
            "type": "WEB"
        });

        let payload_str = payload.to_string();
        let headers = self.generate_auth_headers(&payload_str);

        // 准备请求头
        let mut request_headers = reqwest::header::HeaderMap::new();
        for (key, value) in &headers {
            request_headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value).unwrap()
            );
        }

        // 发送请求
        let response = self.client
            .post(&format!("{}/payments", self.config.api_url))
            .headers(request_headers)
            .body(payload_str)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(format!("Boost payment error: {} - {}", status, response_text).into());
        }

        // 解析响应
        let response_data: serde_json::Value = serde_json::from_str(&response_text)?;

        // 检查响应状态
        if let Some(error) = response_data.get("error") {
            return Err(format!("Boost payment error: {}", error).into());
        }

        // 获取支付URL和交易ID
        let payment_url = response_data.get("paymentUrl")
            .and_then(|v| v.as_str())
            .ok_or("Missing payment URL")?
            .to_string();

        let transaction_id = response_data.get("transactionId")
            .and_then(|v| v.as_str())
            .ok_or("Missing transaction ID")?
            .to_string();

        // 创建交易记录
        let transaction = PaymentTransaction {
            id: Uuid::new_v4().to_string(),
            payment_order_id: request.order.id.clone(),
            transaction_id: Uuid::new_v4().to_string(), // 内部交易ID
            channel_transaction_id: Some(transaction_id),
            amount: request.order.amount,
            status: PaymentStatus::Processing,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
            error_code: None,
            error_message: None,
        };

        Ok(PaymentResponse {
            transaction,
            redirect_url: Some(payment_url),
            html_form: None,
            qr_code: None,
            sdk_params: None,
        })
    }

    async fn verify_payment(&self, payload: String, headers: HashMap<String, String>) -> PaymentResult<PaymentTransaction> {
        // 验证签名
        let timestamp = headers.get("X-Timestamp").ok_or("Missing timestamp")?;
        let nonce = headers.get("X-Nonce").ok_or("Missing nonce")?;
        let signature = headers.get("X-Signature").ok_or("Missing signature")?;

        // 生成签名内容: timestamp + nonce + payload
        let sign_content = format!("{}{}{}", timestamp, nonce, payload);
        let expected_signature = generate_hmac_sha256(&sign_content, &self.config.api_key);

        if signature != &expected_signature {
            return Err("Invalid signature".into());
        }

        // 解析通知数据
        let notification_data: serde_json::Value = serde_json::from_str(&payload)?;

        // 检查交易状态
        let status_code = notification_data.get("status")
            .and_then(|v| v.as_str())
            .ok_or("Missing status")?;

        let payment_status = match status_code {
            "COMPLETED" => PaymentStatus::Successful,
            "FAILED" => PaymentStatus::Failed,
            "CANCELLED" => PaymentStatus::Cancelled,
            _ => PaymentStatus::Processing,
        };

        // 获取关键信息
        let transaction_id = notification_data.get("transactionId")
            .and_then(|v| v.as_str())
            .ok_or("Missing transactionId")?
            .to_string();

        let order_id = notification_data.get("orderId")
            .and_then(|v| v.as_str())
            .ok_or("Missing orderId")?
            .to_string();

        let amount_str = notification_data.get("amount")
            .and_then(|v| v.as_str())
            .ok_or("Missing amount")?;

        let amount = rust_decimal::Decimal::from_str_exact(amount_str)?;

        // 构建交易信息
        let mut metadata = HashMap::new();
        if let Some(obj) = notification_data.as_object() {
            for (key, value) in obj {
                metadata.insert(key.clone(), value.to_string());
            }
        }

        let transaction = PaymentTransaction {
            id: Uuid::new_v4().to_string(), // 这里应该是查询数据库获取原交易ID
            payment_order_id: order_id,
            transaction_id: Uuid::new_v4().to_string(), // 内部交易ID，应该是从数据库查询
            channel_transaction_id: Some(transaction_id),
            amount,
            status: payment_status,
            created_at: Utc::now(), // 应该从数据库获取原始时间
            updated_at: Utc::now(),
            metadata,
            error_code: None,
            error_message: None,
        };

        Ok(transaction)
    }

    async fn query_payment(&self, order_id: &str) -> PaymentResult<PaymentTransaction> {
        // 构建查询请求
        let headers = self.generate_auth_headers("");

        // 准备请求头
        let mut request_headers = reqwest::header::HeaderMap::new();
        for (key, value) in &headers {
            request_headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value).unwrap()
            );
        }

        // 发送请求
        let response = self.client
            .get(&format!("{}/payments/order/{}", self.config.api_url, order_id))
            .headers(request_headers)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(format!("Boost payment query error: {} - {}", status, response_text).into());
        }

        // 解析响应
        let response_data: serde_json::Value = serde_json::from_str(&response_text)?;

        // 检查响应状态
        if let Some(error) = response_data.get("error") {
            return Err(format!("Boost payment query error: {}", error).into());
        }

        // 解析交易状态
        let status_code = response_data.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("PENDING");

        let payment_status = match status_code {
            "COMPLETED" => PaymentStatus::Successful,
            "FAILED" => PaymentStatus::Failed,
            "CANCELLED" => PaymentStatus::Cancelled,
            _ => PaymentStatus::Processing,
        };

        // 获取交易信息
        let transaction_id = response_data.get("transactionId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let amount_str = response_data.get("amount")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let amount = rust_decimal::Decimal::from_str_exact(amount_str).unwrap_or_default();

        // 构建交易信息
        let mut metadata = HashMap::new();
        if let Some(obj) = response_data.as_object() {
            for (key, value) in obj {
                metadata.insert(key.clone(), value.to_string());
            }
        }

        let transaction = PaymentTransaction {
            id: Uuid::new_v4().to_string(), // 这里应该是查询数据库获取原交易ID
            payment_order_id: order_id.to_string(),
            transaction_id: Uuid::new_v4().to_string(), // 内部交易ID，应该是从数据库查询
            channel_transaction_id: Some(transaction_id),
            amount,
            status: payment_status,
            created_at: Utc::now(), // 应该从数据库获取原始时间
            updated_at: Utc::now(),
            metadata,
            error_code: None,
            error_message: None,
        };

        Ok(transaction)
    }

    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResponse> {
        // 创建退款请求
        let refund_id = Uuid::new_v4().to_string();
        let payload = json!({
            "refundId": refund_id,
            "orderId": request.payment_order_id,
            "amount": request.amount.to_string(),
            "reason": request.reason
        });

        let payload_str = payload.to_string();
        let headers = self.generate_auth_headers(&payload_str);

        // 准备请求头
        let mut request_headers = reqwest::header::HeaderMap::new();
        for (key, value) in &headers {
            request_headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value).unwrap()
            );
        }

        // 发送请求
        let response = self.client
            .post(&format!("{}/payments/refund", self.config.api_url))
            .headers(request_headers)
            .body(payload_str)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(format!("Boost refund error: {} - {}", status, response_text).into());
        }

        // 解析响应
        let response_data: serde_json::Value = serde_json::from_str(&response_text)?;

        // 检查响应状态
        if let Some(error) = response_data.get("error") {
            return Err(format!("Boost refund error: {}", error).into());
        }

        // 获取退款状态
        let status_code = response_data.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("PROCESSING");

        let refund_status = match status_code {
            "COMPLETED" => PaymentStatus::Refunded,
            "FAILED" => PaymentStatus::Failed,
            _ => PaymentStatus::Processing,
        };

        // 构建退款订单
        let refund_order = RefundOrder {
            id: Uuid::new_v4().to_string(),
            payment_order_id: request.payment_order_id,
            transaction_id: request.transaction_id,
            amount: request.amount,
            reason: request.reason,
            status: refund_status,
            refund_id: Some(refund_id),
            channel_refund_id: response_data.get("refundId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: request.metadata,
        };

        Ok(RefundResponse { refund_order })
    }

    async fn query_refund(&self, refund_id: &str) -> PaymentResult<RefundOrder> {
        // 构建查询请求
        let headers = self.generate_auth_headers("");

        // 准备请求头
        let mut request_headers = reqwest::header::HeaderMap::new();
        for (key, value) in &headers {
            request_headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value).unwrap()
            );
        }

        // 发送请求
        let response = self.client
            .get(&format!("{}/payments/refund/{}", self.config.api_url, refund_id))
            .headers(request_headers)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(format!("Boost refund query error: {} - {}", status, response_text).into());
        }

        // 解析响应
        let response_data: serde_json::Value = serde_json::from_str(&response_text)?;

        // 检查响应状态
        if let Some(error) = response_data.get("error") {
            return Err(format!("Boost refund query error: {}", error).into());
        }

        // 解析退款状态
        let status_code = response_data.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("PROCESSING");

        let refund_status = match status_code {
            "COMPLETED" => PaymentStatus::Refunded,
            "FAILED" => PaymentStatus::Failed,
            _ => PaymentStatus::Processing,
        };

        // 获取退款信息
        let order_id = response_data.get("orderId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let amount_str = response_data.get("amount")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let amount = rust_decimal::Decimal::from_str_exact(amount_str).unwrap_or_default();

        let reason = response_data.get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 构建退款订单
        let mut metadata = HashMap::new();
        if let Some(obj) = response_data.as_object() {
            for (key, value) in obj {
                metadata.insert(key.clone(), value.to_string());
            }
        }

        let refund_order = RefundOrder {
            id: Uuid::new_v4().to_string(), // 这里应该是查询数据库获取原退款订单ID
            payment_order_id: order_id,
            transaction_id: "".to_string(), // 应该从数据库查询
            amount,
            reason,
            status: refund_status,
            refund_id: Some(refund_id.to_string()),
            channel_refund_id: response_data.get("refundId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            created_at: Utc::now(), // 应该从数据库获取原始时间
            updated_at: Utc::now(),
            metadata,
        };

        Ok(refund_order)
    }
}