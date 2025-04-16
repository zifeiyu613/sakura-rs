use crate::api::extractors::{Query, State};
use crate::api::models::{
    PaymentChannelResponse,
    ApiResponse,
    ApiError,
};
use crate::app_state::AppState;
use crate::domain::enums::Currency;
use axum::http::StatusCode;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ChannelQuery {
    merchant_id: String,
    currency: String,
    amount: String,
}

// 获取可用支付渠道
pub async fn get_available_channels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ChannelQuery>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<PaymentChannelResponse>>>), ApiError> {
    // 转换查询参数
    let currency = Currency::from_str(&query.currency)
        .map_err(|_| ApiError::bad_request(format!("Invalid currency: {}", query.currency)))?;

    let amount = query.amount.parse::<rust_decimal::Decimal>()
        .map_err(|_| ApiError::bad_request("Invalid amount format".to_string()))?;

    // 调用支付服务获取可用渠道
    let channels = state.payment_service.get_available_payment_channels(
        &query.merchant_id,
        currency,
        amount,
    ).await
        .map_err(|e| ApiError::from(e))?;

    // 转换响应
    let response = channels.into_iter()
        .map(|c| PaymentChannelResponse {
            channel: c.channel.to_string(),
            methods: c.methods.into_iter().map(|m| m.to_string()).collect(),
            display_name: c.display_name,
            logo_url: c.logo_url,
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(response))
    ))
}
