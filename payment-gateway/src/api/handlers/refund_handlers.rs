use crate::api::extractors::{Json, Path, State};
use crate::api::models::{
    CreateRefundRequest,
    RefundResponse,
    SimulateRefundRequest,
    ApiResponse,
    ApiError,
};
use crate::app_state::AppState;
use crate::services::payment::dto::RefundRequest;
use axum::http::StatusCode;
use uuid::Uuid;
use std::sync::Arc;
use tracing::{info, error};

// 创建退款请求
pub async fn create_refund(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRefundRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RefundResponse>>), ApiError> {
    info!("API: Create refund request received for order: {}", payload.order_id);

    // 验证API签名
    state.auth_service.verify_api_signature(&payload.merchant_id, &payload.sign, &payload)
        .await
        .map_err(|e| ApiError::unauthorized(e.to_string()))?;

    let order_id = Uuid::parse_str(&payload.order_id)
        .map_err(|_| ApiError::bad_request("Invalid order ID format".to_string()))?;

    let amount = payload.amount.parse::<rust_decimal::Decimal>()
        .map_err(|_| ApiError::bad_request("Invalid amount format".to_string()))?;

    // 请求转换
    let request = RefundRequest {
        order_id,
        amount,
        reason: payload.reason,
    };

    // 调用支付服务创建退款
    let result = state.payment_service.create_refund(request).await
        .map_err(|e| {
            error!("Failed to create refund: {}", e);
            ApiError::from(e)
        })?;

    // 转换响应
    let response = RefundResponse {
        refund_id: result.refund_id.to_string(),
        order_id: result.order_id.to_string(),
        amount: result.amount.to_string(),
        status: result.status.to_string(),
        created_at: result.created_at.to_rfc3339(),
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(response))
    ))
}

// 获取退款详情
pub async fn get_refund(
    State(state): State<Arc<AppState>>,
    Path(refund_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<RefundResponse>>), ApiError> {
    let refund_id = Uuid::parse_str(&refund_id)
        .map_err(|_| ApiError::bad_request("Invalid refund ID format".to_string()))?;

    let refund = state.refund_repository.find_by_id(refund_id).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Refund not found".to_string()))?;

    // 转换响应
    let response = RefundResponse {
        refund_id: refund.id.to_string(),
        order_id: refund.order_id.to_string(),
        amount: refund.amount.to_string(),
        status: refund.status.to_string(),
        created_at: refund.created_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}

// 获取退款状态
pub async fn get_refund_status(
    State(state): State<Arc<AppState>>,
    Path(refund_id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<RefundResponse>>), ApiError> {
    let refund_id = Uuid::parse_str(&refund_id)
        .map_err(|_| ApiError::bad_request("Invalid refund ID format".to_string()))?;

    let status = state.payment_service.query_refund_status(refund_id).await
        .map_err(|e| ApiError::from(e))?;

    // 转换响应
    let response = RefundResponse {
        refund_id: status.refund_id.to_string(),
        order_id: status.order_id.to_string(),
        amount: status.amount.to_string(),
        status: status.status.to_string(),
        created_at: status.created_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}

// 模拟退款成功（仅用于测试）
pub async fn simulate_refund_success(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SimulateRefundRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RefundResponse>>), ApiError> {
    // 检查是否为开发环境
    if !state.config.is_development() {
        return Err(ApiError::forbidden("This endpoint is only available in development mode".to_string()));
    }

    let refund_id = Uuid::parse_str(&payload.refund_id)
        .map_err(|_| ApiError::bad_request("Invalid refund ID format".to_string()))?;

    // 获取退款记录
    let refund = state.refund_repository.find_by_id(refund_id).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Refund not found".to_string()))?;

    // 更新退款状态为成功
    let mut updated_refund = refund.clone();
    updated_refund.update_status(crate::domain::enums::RefundStatus::Success);

    state.refund_repository.update(&updated_refund).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    // 获取订单
    let order = state.order_repository.find_by_id(refund.order_id).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Order not found".to_string()))?;

    // 发送通知
    state.notification_service.send_refund_success_notification(&updated_refund, &order).await
        .map_err(|e| ApiError::internal_error(e.to_string()))?;

    // 返回模拟响应
    let response = RefundResponse {
        refund_id: updated_refund.id.to_string(),
        order_id: updated_refund.order_id.to_string(),
        amount: updated_refund.amount.to_string(),
        status: updated_refund.status.to_string(),
        created_at: updated_refund.created_at.to_rfc3339(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}
