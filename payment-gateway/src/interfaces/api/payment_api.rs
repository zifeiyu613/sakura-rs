use actix_web::{post, get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::{PaymentChannelType, PaymentMethodType, PaymentRegion};
use crate::domain::service::{
    CreatePaymentRequest, CreateRefundRequest, PaymentService, VerifyPaymentRequest
};
use crate::infrastructure::config::AppState;

/// 创建支付请求
#[derive(Debug, Deserialize)]
pub struct ApiCreatePaymentRequest {
    pub merchant_id: String,
    pub order_id: String,
    pub amount: String,
    pub currency: String,
    pub subject: String,
    pub description: Option<String>,
    pub channel: String,
    pub method: String,
    pub region: String,
    pub callback_url: String,
    pub return_url: Option<String>,
    pub client_ip: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub device_info: Option<HashMap<String, String>>,
    pub user_info: Option<HashMap<String, String>>,
}

/// 创建支付响应
#[derive(Debug, Serialize)]
pub struct ApiCreatePaymentResponse {
    pub code: u32,
    pub message: String,
    pub data: Option<PaymentResponse>,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub order_id: String,
    pub payment_id: String,
    pub redirect_url: Option<String>,
    pub html_form: Option<String>,
    pub qr_code: Option<String>,
    pub sdk_params: Option<HashMap<String, String>>,
}

/// 查询支付请求
#[derive(Debug, Deserialize)]
pub struct ApiQueryPaymentRequest {
    pub order_id: String,
}

/// 查询支付响应
#[derive(Debug, Serialize)]
pub struct ApiQueryPaymentResponse {
    pub code: u32,
    pub message: String,
    pub data: Option<PaymentOrderResponse>,
}

#[derive(Debug, Serialize)]
pub struct PaymentOrderResponse {
    pub id: String,
    pub merchant_id: String,
    pub order_id: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub channel: String,
    pub method: String,
    pub region: String,
    pub subject: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建退款请求
#[derive(Debug, Deserialize)]
pub struct ApiCreateRefundRequest {
    pub merchant_id: String,
    pub order_id: String,
    pub amount: String,
    pub reason: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// 创建退款响应
#[derive(Debug, Serialize)]
pub struct ApiCreateRefundResponse {
    pub code: u32,
    pub message: String,
    pub data: Option<RefundResponse>,
}

#[derive(Debug, Serialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub status: String,
}

/// 查询退款请求
#[derive(Debug, Deserialize)]
pub struct ApiQueryRefundRequest {
    pub refund_id: String,
}

/// 查询退款响应
#[derive(Debug, Serialize)]
pub struct ApiQueryRefundResponse {
    pub code: u32,
    pub message: String,
    pub data: Option<RefundOrderResponse>,
}

#[derive(Debug, Serialize)]
pub struct RefundOrderResponse {
    pub id: String,
    pub payment_order_id: String,
    pub amount: String,
    pub reason: String,
    pub status: String,
    pub refund_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建支付
#[post("/api/v1/payments")]
pub async fn create_payment(
    app_state: web::Data<AppState>,
    req: web::Json<ApiCreatePaymentRequest>,
) -> impl Responder {
    // 解析参数
    let decimal_amount = match rust_decimal::Decimal::from_str_exact(&req.amount) {
        Ok(amount) => amount,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiCreatePaymentResponse {
                code: 400,
                message: "Invalid amount format".to_string(),
                data: None,
            });
        }
    };

    let channel = match parse_payment_channel(&req.channel) {
        Some(channel) => channel,
        None => {
            return HttpResponse::BadRequest().json(ApiCreatePaymentResponse {
                code: 400,
                message: format!("Unsupported payment channel: {}", req.channel),
                data: None,
            });
        }
    };

    let method = match parse_payment_method(&req.method) {
        Some(method) => method,
        None => {
            return HttpResponse::BadRequest().json(ApiCreatePaymentResponse {
                code: 400,
                message: format!("Unsupported payment method: {}", req.method),
                data: None,
            });
        }
    };

    let region = match parse_payment_region(&req.region) {
        Some(region) => region,
        None => {
            return HttpResponse::BadRequest().json(ApiCreatePaymentResponse {
                code: 400,
                message: format!("Unsupported payment region: {}", req.region),
                data: None,
            });
        }
    };

    // 构建请求
    let payment_request = CreatePaymentRequest {
        merchant_id: req.merchant_id.clone(),
        order_id: req.order_id.clone(),
        amount: decimal_amount,
        currency: req.currency.clone(),
        subject: req.subject.clone(),
        description: req.description.clone(),
        channel,
        method,
        region,
        callback_url: req.callback_url.clone(),
        return_url: req.return_url.clone(),
        client_ip: req.client_ip.clone(),
        metadata: req.metadata.clone().unwrap_or_default(),
        device_info: req.device_info.clone(),
        user_info: req.user_info.clone(),
    };

    // 调用支付服务
    match app_state.payment_service.create_payment(payment_request).await {
        Ok(response) => {
            HttpResponse::Ok().json(ApiCreatePaymentResponse {
                code: 200,
                message: "Success".to_string(),
                data: Some(PaymentResponse {
                    order_id: response.order_id,
                    payment_id: response.payment_id,
                    redirect_url: response.redirect_url,
                    html_form: response.html_form,
                    qr_code: response.qr_code,
                    sdk_params: response.sdk_params,
                }),
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiCreatePaymentResponse {
                code: 500,
                message: e,
                data: None,
            })
        }
    }
}

/// 支付通知
#[post("/api/v1/payments/notify/{channel}/{method}")]
pub async fn payment_notify(
    app_state: web::Data<AppState>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    let (channel_str, method_str) = path.into_inner();

    // 解析支付渠道和方式
    let channel = match parse_payment_channel(&channel_str) {
        Some(channel) => channel,
        None => {
            return HttpResponse::BadRequest().body(format!("Unsupported payment channel: {}", channel_str));
        }
    };

    let method = match parse_payment_method(&method_str) {
        Some(method) => method,
        None => {
            return HttpResponse::BadRequest().body(format!("Unsupported payment method: {}", method_str));
        }
    };

    // 获取请求头
    let mut headers = HashMap::new();
    for (key, value) in req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(key.to_string(), value_str.to_string());
        }
    }

    // 获取请求体
    let payload = String::from_utf8_lossy(&body).to_string();

    // 构建验证请求
    let verify_request = VerifyPaymentRequest {
        channel,
        method,
        payload,
        headers,
    };

    // 调用支付服务
    match app_state.payment_service.verify_payment(verify_request).await {
        Ok(_) => {
            // 根据不同的支付渠道返回不同的成功响应
            match channel {
                PaymentChannelType::WechatPay => {
                    HttpResponse::Ok().content_type("text/xml").body(
                        r#"<xml><return_code>SUCCESS</return_code><return_msg>OK</return_msg></xml>"#
                    )
                },
                PaymentChannelType::AliPay => {
                    HttpResponse::Ok().body("success")
                },
                _ => HttpResponse::Ok().body("success"),
            }
        },
        Err(e) => {
            // 根据不同的支付渠道返回不同的失败响应
            match channel {
                PaymentChannelType::WechatPay => {
                    HttpResponse::BadRequest().content_type("text/xml").body(
                        format!(r#"<xml><return_code>FAIL</return_code><return_msg>{}</return_msg></xml>"#, e)
                    )
                },
                PaymentChannelType::AliPay => {
                    HttpResponse::BadRequest().body("fail")
                },
                _ => HttpResponse::BadRequest().body(format!("Error: {}", e)),
            }
        }
    }
}

/// 查询支付
#[get("/api/v1/payments")]
pub async fn query_payment(
    app_state: web::Data<AppState>,
    query: web::Query<ApiQueryPaymentRequest>,
) -> impl Responder {
    // 调用支付服务
    match app_state.payment_service.query_payment(&query.order_id).await {
        Ok(order) => {
            HttpResponse::Ok().json(ApiQueryPaymentResponse {
                code: 200,
                message: "Success".to_string(),
                data: Some(PaymentOrderResponse {
                    id: order.id,
                    merchant_id: order.merchant_id,
                    order_id: order.order_id,
                    amount: order.amount.to_string(),
                    currency: order.currency,
                    status: format!("{:?}", order.status),
                    channel: format!("{:?}", order.channel),
                    method: format!("{:?}", order.method),
                    region: format!("{:?}", order.region),
                    subject: order.subject,
                    created_at: order.created_at.to_rfc3339(),
                    updated_at: order.updated_at.to_rfc3339(),
                }),
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiQueryPaymentResponse {
                code: 500,
                message: e,
                data: None,
            })
        }
    }
}

/// 创建退款
#[post("/api/v1/refunds")]
pub async fn create_refund(
    app_state: web::Data<AppState>,
    req: web::Json<ApiCreateRefundRequest>,
) -> impl Responder {
    // 解析参数
    let decimal_amount = match rust_decimal::Decimal::from_str_exact(&req.amount) {
        Ok(amount) => amount,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiCreateRefundResponse {
                code: 400,
                message: "Invalid amount format".to_string(),
                data: None,
            });
        }
    };

    // 构建请求
    let refund_request = CreateRefundRequest {
        merchant_id: req.merchant_id.clone(),
        payment_order_id: req.order_id.clone(),
        amount: decimal_amount,
        reason: req.reason.clone(),
        metadata: req.metadata.clone().unwrap_or_default(),
    };

    // 调用支付服务
    match app_state.payment_service.create_refund(refund_request).await {
        Ok(response) => {
            HttpResponse::Ok().json(ApiCreateRefundResponse {
                code: 200,
                message: "Success".to_string(),
                data: Some(RefundResponse {
                    refund_id: response.refund_id,
                    status: format!("{:?}", response.status),
                }),
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiCreateRefundResponse {
                code: 500,
                message: e,
                data: None,
            })
        }
    }
}

/// 查询退款
#[get("/api/v1/refunds")]
pub async fn query_refund(
    app_state: web::Data<AppState>,
    query: web::Query<ApiQueryRefundRequest>,
) -> impl Responder {
    // 调用支付服务
    match app_state.payment_service.query_refund(&query.refund_id).await {
        Ok(refund) => {
            HttpResponse::Ok().json(ApiQueryRefundResponse {
                code: 200,
                message: "Success".to_string(),
                data: Some(RefundOrderResponse {
                    id: refund.id,
                    payment_order_id: refund.payment_order_id,
                    amount: refund.amount.to_string(),
                    reason: refund.reason,
                    status: format!("{:?}", refund.status),
                    refund_id: refund.refund_id,
                    created_at: refund.created_at.to_rfc3339(),
                    updated_at: refund.updated_at.to_rfc3339(),
                }),
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiQueryRefundResponse {
                code: 500,
                message: e,
                data: None,
            })
        }
    }
}

// 辅助函数：解析支付渠道
fn parse_payment_channel(channel: &str) -> Option<PaymentChannelType> {
    match channel.to_lowercase().as_str() {
        "wechat" | "wechatpay" => Some(PaymentChannelType::WechatPay),
        "alipay" => Some(PaymentChannelType::AliPay),
        "union" | "unionpay" => Some(PaymentChannelType::UnionPay),
        "paypal" => Some(PaymentChannelType::PayPal),
        "stripe" => Some(PaymentChannelType::Stripe),
        "boost" | "boostwallet" => Some(PaymentChannelType::BoostWallet),
        _ => None,
    }
}

// 辅助函数：解析支付方式
fn parse_payment_method(method: &str) -> Option<PaymentMethodType> {
    match method.to_lowercase().as_str() {
        "app" => Some(PaymentMethodType::App),
        "h5" => Some(PaymentMethodType::H5),
        "jsapi" => Some(PaymentMethodType::JsApi),
        "native" | "qr" => Some(PaymentMethodType::Native),
        "web" => Some(PaymentMethodType::Web),
        "wallet" => Some(PaymentMethodType::Wallet),
        "boost" | "boostwallet" => Some(PaymentMethodType::BoostWallet),
        _ => None,
    }
}

// 辅助函数：解析支付地区
fn parse_payment_region(region: &str) -> Option<PaymentRegion> {
    match region.to_lowercase().as_str() {
        "china" | "cn" => Some(PaymentRegion::China),
        "hongkong" | "hk" => Some(PaymentRegion::HongKong),
        "malaysia" | "my" => Some(PaymentRegion::Malaysia),
        "singapore" | "sg" => Some(PaymentRegion::Singapore),
        "global" => Some(PaymentRegion::Global),
        _ => None,
    }
}

// 配置API路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_payment)
        .service(payment_notify)
        .service(query_payment)
        .service(create_refund)
        .service(query_refund);
}