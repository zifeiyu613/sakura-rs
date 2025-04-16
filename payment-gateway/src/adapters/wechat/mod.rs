use crate::adapters::{
    PaymentAdapter, PaymentResponse, PaymentStatusResponse,
    RefundResponse, RefundStatusResponse, NotificationResponse
};
use crate::config::AppConfig;
use crate::domain::entities::{PaymentOrder, Refund};
use crate::domain::enums::PaymentMethod;
use crate::utils::errors::AdapterError;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use chrono::Utc;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{info, error};

pub mod h5;
pub mod app;
pub mod mini_program;
pub mod qr_code;

#[derive(Debug, Clone)]
pub struct WechatPayAdapter {
    config: AppConfig,
    client: Client,
    mch_id: String,
    app_id: String,
    secret_key: String,
    api_key: String,
}

impl WechatPayAdapter {
    pub fn new(config: AppConfig) -> Self {
        let wechat_config = &config.payment_channels.wechat;
        Self {
            config,
            client: Client::new(),
            mch_id: wechat_config.mch_id.clone(),
            app_id: wechat_config.app_id.clone(),
            secret_key: wechat_config.secret_key.clone(),
            api_key: wechat_config.api_key.clone(),
        }
    }

    // 生成微信支付签名
    fn generate_sign(&self, params: &serde_json::Value) -> String {
        let mut query_string = serde_json::to_string(params)
            .unwrap_or_default();

        query_string.push_str(&self.api_key);

        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");

        mac.update(query_string.as_bytes());

        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        hex::encode(code_bytes)
    }

    // 路由到具体的支付方式实现
    async fn route_to_payment_method(
        &self,
        order: &PaymentOrder,
    ) -> Result<PaymentResponse, AdapterError> {
        match order.method {
            PaymentMethod::H5 => h5::create_payment(self, order).await,
            PaymentMethod::App => app::create_payment(self, order).await,
            PaymentMethod::MiniProgram => mini_program::create_payment(self, order).await,
            PaymentMethod::QrCode => qr_code::create_payment(self, order).await,
            _ => Err(AdapterError::UnsupportedPaymentMethod(format!(
                "Unsupported payment method: {:?} for Wechat Pay",
                order.method
            ))),
        }
    }
}

#[async_trait]
impl PaymentAdapter for WechatPayAdapter {
    fn name(&self) -> &'static str {
        "wechat_pay"
    }

    async fn create_payment(&self, order: &PaymentOrder) -> Result<PaymentResponse, AdapterError> {
        info!("Creating Wechat payment for order: {}", order.id);
        self.route_to_payment_method(order).await
    }

    async fn query_payment(&self, order: &PaymentOrder) -> Result<PaymentStatusResponse, AdapterError> {
        info!("Querying Wechat payment status for order: {}", order.id);

        let api_url = "https://api.mch.weixin.qq.com/pay/orderquery";

        let nonce_str = Uuid::new_v4().simple().to_string();

        let mut request_data = serde_json::json!({
            "appid": self.app_id,
            "mch_id": self.mch_id,
            "out_trade_no": order.merchant_order_id,
            "nonce_str": nonce_str,
        });

        // 生成签名
        let sign = self.generate_sign(&request_data);
        request_data["sign"] = serde_json::Value::String(sign);

        let xml_data = self.json_to_xml(&request_data)?;

        let response = self.client
            .post(api_url)
            .body(xml_data)
            .send()
            .await
            .map_err(|e| AdapterError::NetworkError(e.to_string()))?;

        let response_text = response.text().await
            .map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;

        let response_json = self.xml_to_json(&response_text)?;

        // 解析响应
        let return_code = response_json["return_code"].as_str().unwrap_or("FAIL");
        let result_code = response_json["result_code"].as_str().unwrap_or("FAIL");

        if return_code != "SUCCESS" || result_code != "SUCCESS" {
            let err_msg = response_json["return_msg"].as_str().unwrap_or("Unknown error");
            return Err(AdapterError::ChannelError(err_msg.to_string()));
        }

        let trade_state = response_json["trade_state"].as_str().unwrap_or("");
        let is_paid = trade_state == "SUCCESS";

        let transaction_id = response_json["transaction_id"].as_str().map(|s| s.to_string());

        let paid_amount = response_json["total_fee"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .map(|n| rust_decimal::Decimal::new(n, 2));

        let time_end = response_json["time_end"].as_str();
        let paid_time = time_end.and_then(|t| {
            chrono::NaiveDateTime::parse_from_str(t, "%Y%m%d%H%M%S").ok()
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
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
        info!("Handling Wechat payment notification");

        let notification_json = self.xml_to_json(notification_data)?;

        // 验证签名
        let sign = notification_json["sign"].as_str().unwrap_or("");
        let mut verification_data = notification_json.clone();
        verification_data.as_object_mut().map(|o| o.remove("sign"));

        let calculated_sign = self.generate_sign(&verification_data);

        if sign != calculated_sign {
            return Err(AdapterError::InvalidSignature("Wechat Pay notification signature validation failed".to_string()));
        }

        let return_code = notification_json["return_code"].as_str().unwrap_or("FAIL");
        let result_code = notification_json["result_code"].as_str().unwrap_or("FAIL");

        if return_code != "SUCCESS" || result_code != "SUCCESS" {
            let err_msg = notification_json["return_msg"].as_str().unwrap_or("Unknown error");
            return Err(AdapterError::ChannelError(err_msg.to_string()));
        }

        let transaction_id = notification_json["transaction_id"].as_str()
            .ok_or_else(|| AdapterError::ResponseParseError("Missing transaction_id".to_string()))?
            .to_string();

        let out_trade_no = notification_json["out_trade_no"].as_str()
            .ok_or_else(|| AdapterError::ResponseParseError("Missing out_trade_no".to_string()))?
            .to_string();

        let total_fee = notification_json["total_fee"].as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| AdapterError::ResponseParseError("Invalid total_fee".to_string()))?;

        let amount = rust_decimal::Decimal::new(total_fee, 2);

        let time_end = notification_json["time_end"].as_str();
        let paid_time = time_end.and_then(|t| {
            chrono::NaiveDateTime::parse_from_str(t, "%Y%m%d%H%M%S").ok()
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        });

        let response_data = r#"<xml>
            <return_code><![CDATA[SUCCESS]]></return_code>
            <return_msg><![CDATA[OK]]></return_msg>
        </xml>"#.to_string();

        Ok(NotificationResponse {
            transaction_id,
            order_id: out_trade_no,
            is_successful: true,
            amount,
            paid_time,
            raw_data: notification_json,
            response_data,
        })
    }

    async fn create_refund(&self, refund: &Refund, order: &PaymentOrder) -> Result<RefundResponse, AdapterError> {
        info!("Creating Wechat refund for order: {}, refund: {}", order.id, refund.id);

        let api_url = "https://api.mch.weixin.qq.com/secapi/pay/refund";

        let transaction = sqlx::query!(
            "SELECT channel_transaction_id FROM transactions WHERE id = ?",
            refund.transaction_id
        )
            .fetch_optional(&self.config.db_pool)
            .await
            .map_err(|e| AdapterError::DatabaseError(e.to_string()))?
            .ok_or_else(|| AdapterError::TransactionNotFound(refund.transaction_id.to_string()))?;

        let transaction_id = transaction.channel_transaction_id
            .ok_or_else(|| AdapterError::ChannelTransactionIdNotFound(refund.transaction_id.to_string()))?;

        let nonce_str = Uuid::new_v4().simple().to_string();
        let out_refund_no = refund.id.to_string();

        // 微信支付金额单位是分，需要转换
        let total_fee = (order.amount * rust_decimal::Decimal::from(100)).to_i64().unwrap_or(0);
        let refund_fee = (refund.amount * rust_decimal::Decimal::from(100)).to_i64().unwrap_or(0);

        let mut request_data = serde_json::json!({
            "appid": self.app_id,
            "mch_id": self.mch_id,
            "nonce_str": nonce_str,
            "transaction_id": transaction_id,
            "out_refund_no": out_refund_no,
            "total_fee": total_fee,
            "refund_fee": refund_fee,
            "refund_desc": refund.reason,
        });

        // 生成签名
        let sign = self.generate_sign(&request_data);
        request_data["sign"] = serde_json::Value::String(sign);

        let xml_data = self.json_to_xml(&request_data)?;

        // 使用带证书的客户端，实际环境需要加载证书
        let response = self.client
            .post(api_url)
            .body(xml_data)
            // .cert() // 需要加载证书
            .send()
            .await
            .map_err(|e| AdapterError::NetworkError(e.to_string()))?;

        let response_text = response.text().await
            .map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;

        let response_json = self.xml_to_json(&response_text)?;

        // 解析响应
        let return_code = response_json["return_code"].as_str().unwrap_or("FAIL");
        let result_code = response_json["result_code"].as_str().unwrap_or("FAIL");

        if return_code != "SUCCESS" || result_code != "SUCCESS" {
            let err_msg = response_json["err_code_des"].as_str()
                .or_else(|| response_json["return_msg"].as_str())
                .unwrap_or("Unknown error");

            return Err(AdapterError::ChannelError(err_msg.to_string()));
        }

        let refund_id = response_json["refund_id"].as_str().map(|s| s.to_string());

        Ok(RefundResponse {
            channel_refund_id: refund_id,
            is_accepted: true,
            raw_response: response_json,
        })
    }

    async fn query_refund(&self, refund: &Refund, order: &PaymentOrder) -> Result<RefundStatusResponse, AdapterError> {
        info!("Querying Wechat refund status for order: {}, refund: {}", order.id, refund.id);

        let api_url = "https://api.mch.weixin.qq.com/pay/refundquery";

        let nonce_str = Uuid::new_v4().simple().to_string();
        let out_refund_no = refund.id.to_string();

        let mut request_data = serde_json::json!({
            "appid": self.app_id,
            "mch_id": self.mch_id,
            "nonce_str": nonce_str,
            "out_refund_no": out_refund_no,
        });

        // 生成签名
        let sign = self.generate_sign(&request_data);
        request_data["sign"] = serde_json::Value::String(sign);

        let xml_data = self.json_to_xml(&request_data)?;

        let response = self.client
            .post(api_url)
            .body(xml_data)
            .send()
            .await
            .map_err(|e| AdapterError::NetworkError(e.to_string()))?;

        let response_text = response.text().await
            .map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;

        let response_json = self.xml_to_json(&response_text)?;

        // 解析响应
        let return_code = response_json["return_code"].as_str().unwrap_or("FAIL");
        let result_code = response_json["result_code"].as_str().unwrap_or("FAIL");

        if return_code != "SUCCESS" || result_code != "SUCCESS" {
            let err_msg = response_json["err_code_des"].as_str()
                .or_else(|| response_json["return_msg"].as_str())
                .unwrap_or("Unknown error");

            return Err(AdapterError::ChannelError(err_msg.to_string()));
        }

        // 获取对应的退款状态
        // 获取对应的退款状态
        let refund_count = response_json["refund_count"].as_str()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let mut is_success = false;
        let mut refund_id = None;
        let mut refunded_amount = None;
        let mut refund_time = None;

        for i in 0..refund_count {
            let out_refund_no_key = format!("out_refund_no_{}", i);
            let refund_status_key = format!("refund_status_{}", i);
            let refund_id_key = format!("refund_id_{}", i);
            let refund_fee_key = format!("refund_fee_{}", i);
            let refund_time_key = format!("refund_success_time_{}", i);

            let current_out_refund_no = response_json[out_refund_no_key].as_str().unwrap_or("");

            if current_out_refund_no == out_refund_no {
                let status = response_json[refund_status_key].as_str().unwrap_or("");
                is_success = status == "SUCCESS";

                refund_id = response_json[refund_id_key].as_str().map(|s| s.to_string());

                refunded_amount = response_json[refund_fee_key].as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .map(|n| rust_decimal::Decimal::new(n, 2));

                refund_time = response_json[refund_time_key].as_str().and_then(|t| {
                    chrono::NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S").ok()
                        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                });

                break;
            }
        }

        Ok(RefundStatusResponse {
            is_success,
            refund_id,
            refunded_amount,
            refund_time,
            raw_response: response_json,
        })
    }

    // 辅助方法：将JSON转换为XML
    fn json_to_xml(&self, json: &serde_json::Value) -> Result<String, AdapterError> {
        let mut xml = String::from("<xml>");

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                match value {
                    serde_json::Value::String(s) => {
                        xml.push_str(&format!("<{key}><![CDATA[{s}]]></{key}>"));
                    }
                    _ => {
                        xml.push_str(&format!("<{key}>{value}</{key}>"));
                    }
                }
            }
        }

        xml.push_str("</xml>");
        Ok(xml)
    }

    // 辅助方法：将XML转换为JSON
    fn xml_to_json(&self, xml: &str) -> Result<serde_json::Value, AdapterError> {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.trim_text(true);

        let mut json = serde_json::Map::new();
        let mut buf = Vec::new();
        let mut current_key = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let name = e.name();
                    current_key = String::from_utf8_lossy(name.as_ref()).to_string();
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if !current_key.is_empty() {
                        let text = e.unescape().map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;
                        json.insert(current_key.clone(), serde_json::Value::String(text.to_string()));
                    }
                }
                Ok(quick_xml::events::Event::CData(e)) => {
                    if !current_key.is_empty() {
                        let text = e.escape().map_err(|e| AdapterError::ResponseParseError(e.to_string()))?;
                        json.insert(current_key.clone(), serde_json::Value::String(text.to_string()));
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(AdapterError::ResponseParseError(e.to_string())),
                _ => (),
            }
            buf.clear();
        }

        Ok(serde_json::Value::Object(json))
    }
}

