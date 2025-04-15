use actix_web::web;

mod payment_api;
mod refund_api;

pub use payment_api::*;
pub use refund_api::*;

/// 配置API路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // 支付API
    cfg.service(create_payment)
        .service(query_payment)
        .service(payment_notify);

    // 退款API
    cfg.service(create_refund)
        .service(query_refund);
}