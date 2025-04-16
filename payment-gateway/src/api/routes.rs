use crate::api::handlers::{
    payment_handlers,
    refund_handlers,
    notification_handlers,
    channel_handlers,
};
use crate::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // 健康检查
        .route("/health", get(|| async { "OK" }))

        // 支付相关接口
        .route("/api/v1/payments", post(payment_handlers::create_payment))
        .route("/api/v1/payments/:order_id", get(payment_handlers::get_payment))
        .route("/api/v1/payments/:order_id/status", get(payment_handlers::get_payment_status))

        // 退款相关接口
        .route("/api/v1/refunds", post(refund_handlers::create_refund))
        .route("/api/v1/refunds/:refund_id", get(refund_handlers::get_refund))
        .route("/api/v1/refunds/:refund_id/status", get(refund_handlers::get_refund_status))

        // 通知接口
        .route("/api/v1/notifications/wechat", post(notification_handlers::handle_wechat_notification))
        .route("/api/v1/notifications/alipay", post(notification_handlers::handle_alipay_notification))
        .route("/api/v1/notifications/unionpay", post(notification_handlers::handle_unionpay_notification))
        .route("/api/v1/notifications/international/:provider", post(notification_handlers::handle_international_notification))

        // 支付渠道接口
        .route("/api/v1/payment_channels", get(channel_handlers::get_available_channels))

        // 内部测试接口（仅在开发环境可用）
        .route("/api/internal/test/simulate_payment", post(payment_handlers::simulate_payment_success))
        .route("/api/internal/test/simulate_refund", post(refund_handlers::simulate_refund_success))

        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
