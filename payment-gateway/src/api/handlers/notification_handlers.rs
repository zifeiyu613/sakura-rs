use crate::api::extractors::{RawBody, Path, State};
use crate::api::models::ApiError;
use crate::app_state::AppState;
use crate::domain::enums::PaymentChannel;
use axum::body::Bytes;
use axum::http::StatusCode;
use std::sync::Arc;
use tracing::{info, error};

// 处理微信支付通知
pub async fn handle_wechat_notification(
    State(state): State<Arc<AppState>>,
    RawBody(body): RawBody,
) -> Result<(StatusCode, String), ApiError> {
    info!("Received WeChat payment notification");

    let notification_data = String::from_utf8(body.to_vec())
        .map_err(|_| ApiError::bad_request("Invalid notification data encoding".to_string()))?;

    // 处理支付通知
    let response = state.payment_service.handle_payment_notification(
        PaymentChannel::Wechat,
        &notification_data
    ).await
        .map_err(|e| {
            error!("Failed to process WeChat notification: {}", e);
            ApiError::from(e)
        })?;

    Ok((StatusCode::OK, response))
}

// 处理支付宝通知
pub async fn handle_alipay_notification(
    State(state): State<Arc<AppState>>,
    RawBody(body): RawBody,
) -> Result<(StatusCode, String), ApiError> {
    info!("Received Alipay payment notification");

    let notification_data = String::from_utf8(body.to_vec())
        .map_err(|_| ApiError::bad_request("Invalid notification data encoding".to_string()))?;

    // 处理支付通知
    let response = state.payment_service.handle_payment_notification(
        PaymentChannel::Alipay,
        &notification_data
    ).await
        .map_err(|e| {
            error!("Failed to process Alipay notification: {}", e);
            ApiError::from(e)
        })?;

    Ok((StatusCode::OK, response))
}

// 处理银联支付通知
pub async fn handle_unionpay_notification(
    State(state): State<Arc<AppState>>,
    RawBody(body): RawBody,
) -> Result<(StatusCode, String), ApiError> {
    info!("Received UnionPay payment notification");

    let notification_data = String::from_utf8(body.to_vec())
        .map_err(|_| ApiError::bad_request("Invalid notification data encoding".to_string()))?;

    // 处理支付通知
    let response = state.payment_service.handle_payment_notification(
        PaymentChannel::UnionPay,
        &notification_data
    ).await
        .map_err(|e| {
            error!("Failed to process UnionPay notification: {}", e);
            ApiError::from(e)
        })?;

    Ok((StatusCode::OK, response))
}

// 处理国际支付通知
pub async fn handle_international_notification(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    RawBody(body): RawBody,
) -> Result<(StatusCode, String), ApiError> {
    info!("Received international payment notification from provider: {}", provider);

    let notification_data = String::from_utf8(body.to_vec())
        .map_err(|_| ApiError::bad_request("Invalid notification data encoding".to_string()))?;

    // 根据提供商名称确定支付渠道
    let channel = match provider.as_str() {
        "paypal" => PaymentChannel::PayPal,
        "stripe" => PaymentChannel::Stripe,
        "adyen" => PaymentChannel::Adyen,
        "global_pay" => PaymentChannel::GlobalPay,
        "boost" => PaymentChannel::Boost,
        "grabpay" => PaymentChannel::GrabPay,
        "touchngo" => PaymentChannel::TouchNGo,
        _ => return Err(ApiError::bad_request(format!("Unsupported payment provider: {}", provider))),
    };

    // 处理支付通知
    let response = state.payment_service.handle_payment_notification(
        channel,
        &notification_data
    ).await
        .map_err(|e| {
            error!("Failed to process {} notification: {}", provider, e);
            ApiError::from(e)
        })?;

    Ok((StatusCode::OK, response))
}
