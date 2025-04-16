use crate::api::extractors::{Json, Path, Query, State};
use crate::api::models::{
    CreatePaymentRequest,
    PaymentResponse,
    PaymentStatusResponse,
    SimulatePaymentRequest,
    ApiResponse,
    ApiError,
};
use crate::app_state::AppState;
use crate::domain::enums::{Currency, PaymentChannel, PaymentMethod};
use crate::services::payment::dto::CreateOrderRequest;
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use tracing::{info, error};

// 创建支付订单
pub async fn create_payment(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PaymentResponse>>), ApiError> {
    info!("API: Create payment request received for merchant: {}", payload.merchant_id);

    // 验证API签名
    state.auth_service.verify_api_signature(&payload.merchant_id, &payload.sign, &payload)
        .await
        .map_err(|e| ApiError::unauthorized(e.to_string()))?;

    // 请求转换
    let request = CreateOrderRequest {
        merchant_id: payload.merchant_id,
        merchant_order_id: payload.merchant_order_id,
        amount: payload.amount,
        currency: Currency::from_str(&payload.currency)
            .map_err(|_| ApiError::bad_request(format!("Invalid currency: {}", payload.currency)))?,
        channel: PaymentChannel::from_str(&payload.channel)
            .map_err(|_| ApiError::bad_request(format!("Invalid payment channel: {}", payload.channel)))?,
        method: PaymentMethod::from_str(&payload.method)
            .map_err(|_| ApiError::bad_request(format!("Invalid payment method: {}", payload.method)))?,
        subject: payload.subject,
        callback_url: payload.callback_url,
        return_url: payload.return_url,
        client_ip: payload.client_ip,
        metadata: payload.metadata,
        expire_time: payload.expire_time,
    };

    // 调用支付服务创建订单
    let result = state.payment_service.create_order(request).await
        .map_err(|e| {
            error!("Failed to create payment: {}", e);
            ApiError::from(e)
        })?;

    // 转换响应
    let response = PaymentResponse {
        order_id: result.order_id.to_string(),
        merchant_order_id: result.merchant_order_id,
        amount: result.amount.to_string(),
        currency: result.currency.to_string(),
        status: result.status.to_string(),
        payment_url: result.payment_url,
        qr_code: result.qr_code,
        html_form: result.html_form,
        app_parameters: result.app_parameters,
        expire_time: result.expire_time.map(|t| t.to_rfc3339()),
        created_at: result.created_at.to_rfc3339(),
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(response))
    ))
}

// 获取支付订单详情
pub async fn get_payment(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<PaymentResponse>>), ApiError> {
    let order_id = Uuid::parse_str(&order_id)
        .map_err(|_| ApiError::bad_request("Invalid order ID format".to_string()))?;

    let order = state.order_repository.find_by_id(order_id).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Order not found".to_string()))?;

    // 转换响应
    let response = PaymentResponse {
        order_id: order.id.to_string(),
        merchant_order_id: order.merchant_order_id,
        amount: order.amount.to_string(),
        currency: order.currency.to_string(),
        status: order.status.to_string(),
        payment_url: None,
        qr_code: None,
        html_form: None,
        app_parameters: None,
        expire_time: order.expire_time.map(|t| t.to_rfc3339()),
        created_at: order.created_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}

// 获取支付订单状态
pub async fn get_payment_status(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<PaymentStatusResponse>>), ApiError> {
    let order_id = Uuid::parse_str(&order_id)
        .map_err(|_| ApiError::bad_request("Invalid order ID format".to_string()))?;

    let status = state.payment_service.query_order_status(order_id).await
        .map_err(|e| ApiError::from(e))?;

    // 转换响应
    let response = PaymentStatusResponse {
        order_id: status.order_id.to_string(),
        merchant_order_id: status.merchant_order_id,
        status: status.status.to_string(),
        paid_amount: status.paid_amount.map(|a| a.to_string()),
        paid_time: status.paid_time.map(|t| t.to_rfc3339()),
        channel_transaction_id: status.channel_transaction_id,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}

// 模拟支付成功（仅用于测试）
pub async fn simulate_payment_success(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SimulatePaymentRequest>,
) -> Result<(StatusCode, Json<ApiResponse<PaymentStatusResponse>>), ApiError> {
    // 检查是否为开发环境
    if !state.config.is_development() {
        return Err(ApiError::forbidden("This endpoint is only available in development mode".to_string()));
    }

    let order_id = Uuid::parse_str(&payload.order_id)
        .map_err(|_| ApiError::bad_request("Invalid order ID format".to_string()))?;

    // 获取订单
    let order = state.order_repository.find_by_id(order_id).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Order not found".to_string()))?;

    // 更新订单状态为成功
    let mut updated_order = order.clone();
    updated_order.update_status(crate::domain::enums::PaymentStatus::Success);

    state.order_repository.update(&updated_order).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    // 创建成功交易记录
    let transactions = state.transaction_repository.find_by_order_id(order_id).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    if let Some(mut transaction) = transactions.into_iter().next() {
        transaction.update_status(crate::domain::enums::TransactionStatus::Success);
        transaction.set_channel_transaction_id(format!("test_{}", Uuid::new_v4()));

        state.transaction_repository.update(&transaction).await
            .map_err(|e| ApiError::internal_error(e.to_string()))?;

        // 发送通知
        state.notification_service.send_payment_success_notification(&updated_order, &transaction).await
            .map_err(|e| ApiError::internal_error(e.to_string()))?;
    }

    // 返回模拟响应
    let response = PaymentStatusResponse {
        order_id: order_id.to_string(),
        merchant_order_id: updated_order.merchant_order_id,
        status: updated_order.status.to_string(),
        paid_amount: Some(updated_order.amount.to_string()),
        paid_time: Some(chrono::Utc::now().to_rfc3339()),
        channel_transaction_id: Some(format!("test_{}", Uuid::new_v4())),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}
