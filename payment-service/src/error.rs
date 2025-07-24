use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde_json::json;
use thiserror::Error;
use crate::models::enums::OrderStatus;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("不支持的支付类型: {0}")]
    UnsupportedPaymentType(String),

    #[error("无效的支付类型: {0}")]
    InvalidPaymentType(i32),

    #[error("订单状态错误: 当前 {current}, 需要 {expected:?}")]
    InvalidOrderStatus {
        current: String,
        expected: Vec<String>,
    },

    #[error("状态转换错误: 从 {from:?} 不能应用 {event}")]
    InvalidStateTransition {
        from: OrderStatus,
        event: String,
    },

    #[error("无效的事件: 订单ID {order_id} 与事件订单ID {event_order_id} 不匹配")]
    InvalidEvent {
        order_id: String,
        event_order_id: String
    },

    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("第三方API错误: {code} - {message}")]
    ExternalApi { code: String, message: String },

    #[error("配置错误: {0}")]
    Configuration(String),

    #[error("请求限流")]
    RateLimited,

    #[error("订单不存在: {0}")]
    OrderNotFound(String),
}

impl IntoResponse for PaymentError {
    fn into_response(self) -> Response {
        let (status, error_type, error_message) = match &self {
            PaymentError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DatabaseError",
                format!("数据库操作失败: {}", e)
            ),
            PaymentError::UnsupportedPaymentType(pt) => (
                StatusCode::BAD_REQUEST,
                "UnsupportedPaymentType",
                format!("不支持的支付类型: {}", pt)
            ),
            PaymentError::InvalidPaymentType(code) => (
                StatusCode::BAD_REQUEST,
                "InvalidPaymentType",
                format!("无效的支付类型代码: {}", code)
            ),
            PaymentError::InvalidOrderStatus { current, expected } => (
                StatusCode::CONFLICT,
                "InvalidOrderStatus",
                format!("订单状态错误: 当前 {}, 需要 {:?}", current, expected)
            ),
            PaymentError::InvalidStateTransition { from, event } => (
                StatusCode::CONFLICT,
                "InvalidStateTransition",
                format!("状态转换错误: 从 {:?} 不能应用 {}", from, event)
            ),
            PaymentError::InvalidEvent { order_id, event_order_id } => (
                StatusCode::BAD_REQUEST,
                "InvalidEvent",
                format!("无效的事件: 订单ID {} 与事件订单ID {} 不匹配", order_id, event_order_id)
            ),
            PaymentError::UnsupportedOperation(msg) => (
                StatusCode::BAD_REQUEST,
                "UnsupportedOperation",
                format!("不支持的操作: {}", msg)
            ),
            PaymentError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                format!("内部错误: {}", msg)
            ),
            PaymentError::ExternalApi { code, message } => (
                StatusCode::BAD_GATEWAY,
                "ExternalApiError",
                format!("第三方API错误 {}: {}", code, message)
            ),
            PaymentError::Configuration(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ConfigurationError",
                format!("配置错误: {}", msg)
            ),
            PaymentError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RateLimited",
                "请求被限流，请稍后重试".to_string()
            ),
            PaymentError::OrderNotFound(order_id) => (
                StatusCode::NOT_FOUND,
                "OrderNotFound",
                format!("订单不存在: {}", order_id)
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "type": error_type,
                "message": error_message
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_into_response() {
        // 测试数据库错误响应
        let db_error = PaymentError::Database(sqlx::Error::PoolClosed);
        let response = db_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // 测试无效支付类型错误响应
        let invalid_payment_type = PaymentError::InvalidPaymentType(999);
        let response = invalid_payment_type.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // 测试订单不存在错误响应
        let order_not_found = PaymentError::OrderNotFound("order123".to_string());
        let response = order_not_found.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // 测试限流错误响应
        let rate_limited = PaymentError::RateLimited;
        let response = rate_limited.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}