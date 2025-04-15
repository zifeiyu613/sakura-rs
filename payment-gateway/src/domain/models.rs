use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 支付状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Created,       // 已创建
    Processing,    // 处理中
    Successful,    // 成功
    Failed,        // 失败
    Cancelled,     // 已取消
    Refunded,      // 已退款
    PartiallyRefunded, // 部分退款
}

/// 支付渠道类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentChannelType {
    WechatPay,    // 微信支付
    AliPay,       // 支付宝
    UnionPay,     // 银联
    PayPal,       // PayPal
    Stripe,       // Stripe
    BoostWallet,  // Boost钱包(马来西亚)
}

/// 支付方式类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethodType {
    App,          // APP支付
    H5,           // H5支付
    JsApi,        // JSAPI支付
    Native,       // 原生/扫码支付
    Web,          // 网页支付
    Wallet,       // 钱包支付
    BoostWallet,  // Boost钱包支付
}

/// 支付区域
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentRegion {
    China,        // 中国大陆
    HongKong,     // 香港
    Malaysia,     // 马来西亚
    Singapore,    // 新加坡
    Global,       // 全球
}

/// 支付订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    pub id: String,                            // 系统订单ID
    pub merchant_id: String,                   // 商户ID
    pub order_id: String,                      // 商户订单号
    pub amount: Decimal,                       // 金额
    pub currency: String,                      // 货币代码
    pub status: PaymentStatus,                 // 支付状态
    pub channel: PaymentChannelType,           // 支付渠道
    pub method: PaymentMethodType,             // 支付方式
    pub region: PaymentRegion,                 // 支付区域
    pub subject: String,                       // 订单标题
    pub description: Option<String>,           // 订单描述
    pub metadata: HashMap<String, String>,     // 元数据
    pub created_at: DateTime<Utc>,             // 创建时间
    pub updated_at: DateTime<Utc>,             // 更新时间
    pub expires_at: Option<DateTime<Utc>>,     // 过期时间
    pub callback_url: String,                  // 回调地址
    pub return_url: Option<String>,            // 前端跳转地址
    pub client_ip: Option<String>,             // 客户端IP
}

/// 支付交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub id: String,                            // 系统交易ID
    pub payment_order_id: String,              // 关联的支付订单ID
    pub transaction_id: String,                // 交易编号
    pub channel_transaction_id: Option<String>, // 渠道交易号
    pub amount: Decimal,                        // 交易金额
    pub status: PaymentStatus,                  // 交易状态
    pub created_at: DateTime<Utc>,              // 创建时间
    pub updated_at: DateTime<Utc>,              // 更新时间
    pub metadata: HashMap<String, String>,      // 元数据
    pub error_code: Option<String>,             // 错误码
    pub error_message: Option<String>,          // 错误信息
}

/// 退款订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundOrder {
    pub id: String,                            // 系统退款ID
    pub payment_order_id: String,              // 关联的支付订单ID
    pub transaction_id: String,                // 关联的交易ID
    pub amount: Decimal,                       // 退款金额
    pub reason: String,                        // 退款原因
    pub status: PaymentStatus,                 // 退款状态
    pub refund_id: Option<String>,             // 退款编号
    pub channel_refund_id: Option<String>,     // 渠道退款号
    pub created_at: DateTime<Utc>,             // 创建时间
    pub updated_at: DateTime<Utc>,             // 更新时间
    pub metadata: HashMap<String, String>,     // 元数据
}