use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::models::{
    PaymentMethodType, PaymentOrder, PaymentRegion, PaymentStatus, RefundOrder
};

/// 默认请求超时时间（秒）
pub const DEFAULT_TIMEOUT: u64 = 30;

/// 支付配置
#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub app_id: String,           // 应用ID
    pub api_key: String,          // API密钥
    pub private_key: Option<String>, // 私钥
    pub public_key: Option<String>,  // 公钥
    pub merchant_id: String,      // 商户ID
    pub api_url: String,          // API地址
    pub timeout: Duration,        // 超时时间
}

/// 支付结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub payment_id: String,       // 支付ID
    pub order_id: String,         // 订单ID
    pub redirect_url: Option<String>, // 重定向URL
    pub html_form: Option<String>,    // HTML表单(表单提交方式)
    pub qr_code: Option<String>,      // 二维码内容
    pub sdk_params: Option<HashMap<String, String>>, // SDK参数(APP支付等)
}

/// 退款结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResult {
    pub refund_id: String,       // 退款ID
    pub status: PaymentStatus,   // 退款状态
}

/// 支付处理器接口
#[async_trait]
pub trait PaymentProcessor: Send + Sync {
    /// 创建支付
    async fn create_payment(&self, order: &PaymentOrder) -> Result<PaymentResult, String>;

    /// 查询支付状态
    async fn query_payment(&self, payment_id: &str) -> Result<PaymentOrder, String>;

    /// 验证支付通知
    async fn verify_payment(&self, payload: &str, headers: &HashMap<String, String>) -> Result<PaymentOrder, String>;

    /// 创建退款
    async fn create_refund(&self, refund: &RefundOrder) -> Result<RefundResult, String>;

    /// 查询退款状态
    async fn query_refund(&self, refund_id: &str) -> Result<RefundOrder, String>;
}

/// 支付处理器工厂
pub struct PaymentProcessorFactory {
    processors: HashMap<String, Arc<dyn PaymentProcessor>>,
}

impl PaymentProcessorFactory {
    /// 创建新的支付处理器工厂
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
        }
    }

    /// 注册支付处理器
    pub fn register(
        &mut self,
        method: PaymentMethodType,
        region: PaymentRegion,
        merchant_id: &str,
        processor: Arc<dyn PaymentProcessor>,
    ) {
        let key = format!("{}:{}:{}", Self::method_to_string(&method), Self::region_to_string(&region), merchant_id);
        self.processors.insert(key, processor);
    }

    /// 获取支付处理器
    pub fn get_processor(
        &self,
        method: PaymentMethodType,
        region: PaymentRegion,
        merchant_id: &str,
    ) -> Option<Arc<dyn PaymentProcessor>> {
        let key = format!("{}:{}:{}", Self::method_to_string(&method), Self::region_to_string(&region), merchant_id);
        // 先尝试获取指定商户的处理器
        if let Some(processor) = self.processors.get(&key) {
            return Some(Arc::clone(processor));
        }

        // 如果没有找到特定商户的处理器，尝试获取默认处理器
        let default_key = format!("{}:{}:default_merchant", Self::method_to_string(&method), Self::region_to_string(&region));
        self.processors.get(&default_key).map(Arc::clone)
    }

    // 辅助方法：将支付方式转换为字符串
    fn method_to_string(method: &PaymentMethodType) -> String {
        match method {
            PaymentMethodType::App => "App".to_string(),
            PaymentMethodType::H5 => "H5".to_string(),
            PaymentMethodType::JsApi => "JsApi".to_string(),
            PaymentMethodType::Native => "Native".to_string(),
            PaymentMethodType::Web => "Web".to_string(),
            PaymentMethodType::Wallet => "Wallet".to_string(),
            PaymentMethodType::BoostWallet => "BoostWallet".to_string(),
        }
    }

    // 辅助方法：将支付区域转换为字符串
    fn region_to_string(region: &PaymentRegion) -> String {
        match region {
            PaymentRegion::China => "China".to_string(),
            PaymentRegion::HongKong => "HongKong".to_string(),
            PaymentRegion::Malaysia => "Malaysia".to_string(),
            PaymentRegion::Singapore => "Singapore".to_string(),
            PaymentRegion::Global => "Global".to_string(),
        }
    }
}