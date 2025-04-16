use crate::domain::entities::{PaymentOrder, Transaction, Refund};
use crate::config::AppConfig;
use crate::utils::errors::AdapterError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub mod wechat;
pub mod alipay;
pub mod unionpay;
pub mod international;
mod wechat;
mod alipay;

/// 支付适配器公共接口
#[async_trait]
pub trait PaymentAdapter: Send + Sync {
    /// 获取支付适配器名称
    fn name(&self) -> &'static str;

    /// 创建支付交易
    async fn create_payment(&self, order: &PaymentOrder) -> Result<PaymentResponse, AdapterError>;

    /// 查询交易状态
    async fn query_payment(&self, order: &PaymentOrder) -> Result<PaymentStatusResponse, AdapterError>;

    /// 处理支付回调通知
    async fn handle_notification(&self, notification_data: &str) -> Result<NotificationResponse, AdapterError>;

    /// 发起退款
    async fn create_refund(&self, refund: &Refund, order: &PaymentOrder) -> Result<RefundResponse, AdapterError>;

    /// 查询退款状态
    async fn query_refund(&self, refund: &Refund, order: &PaymentOrder) -> Result<RefundStatusResponse, AdapterError>;
}

/// 支付结果响应
#[derive(Debug, Clone)]
pub struct PaymentResponse {
    pub channel_transaction_id: Option<String>,
    pub payment_url: Option<String>,
    pub qr_code: Option<String>,
    pub html_form: Option<String>,
    pub app_parameters: Option<serde_json::Value>,
    pub raw_response: serde_json::Value,
}

/// 支付状态响应
#[derive(Debug, Clone)]
pub struct PaymentStatusResponse {
    pub is_paid: bool,
    pub transaction_id: Option<String>,
    pub paid_amount: Option<rust_decimal::Decimal>,
    pub paid_time: Option<chrono::DateTime<chrono::Utc>>,
    pub raw_response: serde_json::Value,
}

/// 退款响应
#[derive(Debug, Clone)]
pub struct RefundResponse {
    pub channel_refund_id: Option<String>,
    pub is_accepted: bool,
    pub raw_response: serde_json::Value,
}

/// 退款状态响应
#[derive(Debug, Clone)]
pub struct RefundStatusResponse {
    pub is_success: bool,
    pub refund_id: Option<String>,
    pub refunded_amount: Option<rust_decimal::Decimal>,
    pub refund_time: Option<chrono::DateTime<chrono::Utc>>,
    pub raw_response: serde_json::Value,
}

/// 通知响应
#[derive(Debug, Clone)]
pub struct NotificationResponse {
    pub transaction_id: String,
    pub order_id: String,
    pub is_successful: bool,
    pub amount: rust_decimal::Decimal,
    pub paid_time: Option<chrono::DateTime<chrono::Utc>>,
    pub raw_data: serde_json::Value,
    pub response_data: String, // 返回给支付网关的确认数据
}

/// 支付适配器注册表
pub struct PaymentAdapterRegistry {
    adapters: HashMap<crate::domain::enums::PaymentChannel, Arc<dyn PaymentAdapter>>,
}

impl PaymentAdapterRegistry {
    pub fn new(config: &AppConfig) -> Self {
        let mut adapters = HashMap::new();

        // 注册微信支付适配器
        let wechat_adapter = Arc::new(wechat::WechatPayAdapter::new(config.clone()));
        adapters.insert(crate::domain::enums::PaymentChannel::Wechat, wechat_adapter);

        // 注册支付宝适配器
        let alipay_adapter = Arc::new(alipay::AlipayAdapter::new(config.clone()));
        adapters.insert(crate::domain::enums::PaymentChannel::Alipay, alipay_adapter);

        // 注册云闪付适配器
        let unionpay_adapter = Arc::new(unionpay::UnionPayAdapter::new(config.clone()));
        adapters.insert(crate::domain::enums::PaymentChannel::UnionPay, unionpay_adapter);

        // 注册国际支付适配器
        let paypal_adapter = Arc::new(international::paypal::PayPalAdapter::new(config.clone()));
        adapters.insert(crate::domain::enums::PaymentChannel::PayPal, paypal_adapter);

        // 注册其他支付适配器
        // ...

        Self { adapters }
    }

    pub fn get(&self, channel: crate::domain::enums::PaymentChannel) -> Option<Arc<dyn PaymentAdapter>> {
        self.adapters.get(&channel).cloned()
    }
}
