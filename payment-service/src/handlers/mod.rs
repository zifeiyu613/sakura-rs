use axum::{
    extract::{Path, Json, State, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use std::sync::Arc;
use axum::response::Response;
use serde_json::json;
use serde::Deserialize;

use crate::models::payment::{CreatePaymentRequest, RefundRequest};
use crate::models::enums::PaymentType;
use crate::services::payment_service::PaymentService;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}

pub async fn create_payment(
    Extension(service): Extension<Arc<PaymentService>>,
    Json(request): Json<CreatePaymentRequest>,
) -> Response {
    match service.create_payment(request).await {
        Ok(response) => (StatusCode::OK, Json(json!({ "success": true, "data": response }))).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn query_payment(
    Extension(service): Extension<Arc<PaymentService>>,
    Path(order_id): Path<String>,
) -> Response {
    match service.query_payment(&order_id).await {
        Ok(status) => (StatusCode::OK, Json(json!({ "success": true, "status": status }))).into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    tenant_id: Option<i64>,
}

pub async fn payment_callback(
    Extension(service): Extension<Arc<PaymentService>>,
    Path(payment_type_str): Path<String>,
    Query(query): Query<CallbackQuery>,
    Json(callback_data): Json<serde_json::Value>,
) -> Response {
    // 从请求中提取 tenant_id
    let tenant_id = query.tenant_id
        .or_else(|| callback_data.get("tenant_id").and_then(|v| v.as_i64()))
        .unwrap_or(1);

    // 解析支付类型
    let payment_type = match payment_type_str.parse::<PaymentType>() {
        Ok(pt) => pt,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": {
                        "type": "InvalidPaymentType",
                        "message": format!("Invalid payment type: {}", payment_type_str)
                    }
                })),
            ).into_response();
        }
    };

    match service.handle_callback(payment_type, tenant_id, callback_data).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn refund_payment(
    Extension(service): Extension<Arc<PaymentService>>,
    Json(request): Json<RefundRequest>,
) -> Response {
    match service.refund_payment(request).await {
        Ok(refund_id) => (
            StatusCode::OK,
            Json(json!({ "success": true, "refund_id": refund_id })),
        ).into_response(),
        Err(e) => e.into_response(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::payment_service::PaymentService;
    use crate::config::cache::ConfigCache;
    use crate::payment::factory::PaymentFactory;
    use axum::http::Request;
    use axum::body::Body;
    use tower::ServiceExt;
    use axum::routing::{post, get};
    use axum::Router;
    use sqlx::mysql::MySqlPoolOptions;
    use std::sync::Arc;
    use chrono::Utc;

    async fn setup_test_db() -> anyhow::Result<sqlx::MySqlPool> {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/payment_service_test".to_string());

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        // 初始化测试数据库表
        crate::db::init_db(&pool).await?;

        Ok(pool)
    }

    async fn setup_test_data(pool: &sqlx::MySqlPool) -> anyhow::Result<()> {
        // 清理可能存在的测试数据
        sqlx::query("DELETE FROM payment_configs WHERE tenant_id = 999")
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM payment_orders WHERE tenant_id = 999")
            .execute(pool)
            .await?;

        // 插入测试支付配置
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO payment_configs 
            (tenant_id, payment_type, payment_sub_type, merchant_id, app_id, gateway_url, notify_url, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
            .bind(999i64)  // 使用特殊的 tenant_id 用于测试
            .bind(5i32)    // WX_H5 type_code
            .bind(5i32)    // WX_H5 sub_type_code
            .bind("test_merchant_id")
            .bind("test_app_id")
            .bind("https://api.test.com")
            .bind("https://notify.test.com")
            .bind(true)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn cleanup_test_data(pool: &sqlx::MySqlPool) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM payment_configs WHERE tenant_id = 999")
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM payment_orders WHERE tenant_id = 999")
            .execute(pool)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_create_payment_handler() -> anyhow::Result<()> {
        // 设置测试数据库
        let pool = setup_test_db().await?;
        setup_test_data(&pool).await?;

        // 创建真实的服务实例
        let config_cache = Arc::new(ConfigCache::new(
            pool.clone(),
            std::time::Duration::from_secs(60)
        ));

        let payment_factory = Arc::new(PaymentFactory::new(config_cache.clone()));
        let payment_service = Arc::new(PaymentService::new(
            pool.clone(),
            payment_factory,
            config_cache,
        ));

        // 创建测试app
        let app = Router::new()
            .route("/api/v1/payment/create", post(create_payment))
            .layer(Extension(payment_service));

        // 创建测试请求 - 使用测试 tenant_id
        let request_body = json!({
            "tenant_id": 999,  // 使用测试 tenant_id
            "user_id": 100,
            "payment_type": "WX_H5",
            "amount": 10000,
            "currency": "CNY",
            "product_name": "测试商品",
            "product_desc": "商品描述",
            "callback_url": "http://example.com/callback",
            "notify_url": "http://example.com/notify"
        });

        let request = Request::builder()
            .uri("/api/v1/payment/create")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_body)?))
            .unwrap();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态
        assert_eq!(response.status(), StatusCode::OK);

        // 解析响应体
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;

        // 验证响应内容
        assert_eq!(body["success"], true);
        assert!(body["data"]["order_id"].is_string());

        // 验证订单确实被创建到数据库中
        let order_id = body["data"]["order_id"].as_str().unwrap();
        let order_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM payment_orders WHERE order_id = ? AND tenant_id = 999"
        )
            .bind(order_id)
            .fetch_one(&pool)
            .await?;

        assert_eq!(order_exists, 1);

        // 清理测试数据
        cleanup_test_data(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_query_payment_handler() -> anyhow::Result<()> {
        // 设置测试数据库
        let pool = setup_test_db().await?;
        setup_test_data(&pool).await?;

        // 创建真实的服务实例
        let config_cache = Arc::new(ConfigCache::new(
            pool.clone(),
            std::time::Duration::from_secs(60)
        ));

        let payment_factory = Arc::new(PaymentFactory::new(config_cache.clone()));
        let payment_service = Arc::new(PaymentService::new(
            pool.clone(),
            payment_factory,
            config_cache,
        ));

        // 先创建一个订单
        let create_request = crate::models::payment::CreatePaymentRequest {
            tenant_id: 999,
            user_id: 100,
            payment_type: crate::models::enums::PaymentType::WxH5,
            amount: 10000,
            currency: "CNY".to_string(),
            product_name: "测试商品".to_string(),
            product_desc: Some("商品描述".to_string()),
            callback_url: Some("http://example.com/callback".to_string()),
            notify_url: Some("http://example.com/notify".to_string()),
            extra_data: None,
        };

        let create_response = payment_service.create_payment(create_request).await?;
        let order_id = create_response.order_id;

        // 创建测试app
        let app = Router::new()
            .route("/api/v1/payment/query/:order_id", get(query_payment))
            .layer(Extension(payment_service));

        // 创建查询请求
        let request = Request::builder()
            .uri(&format!("/api/v1/payment/query/{}", order_id))
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;

        assert_eq!(body["success"], true);
        // 由于这是集成测试，支付状态可能是 PENDING 或 PROCESSING
        assert!(body["status"].is_string());

        // 清理测试数据
        cleanup_test_data(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_create_payment_invalid_tenant() -> anyhow::Result<()> {
        // 设置测试数据库（不插入配置数据）
        let pool = setup_test_db().await?;

        // 创建真实的服务实例
        let config_cache = Arc::new(ConfigCache::new(
            pool.clone(),
            std::time::Duration::from_secs(60)
        ));

        let payment_factory = Arc::new(PaymentFactory::new(config_cache.clone()));
        let payment_service = Arc::new(PaymentService::new(
            pool.clone(),
            payment_factory,
            config_cache,
        ));

        // 创建测试app
        let app = Router::new()
            .route("/api/v1/payment/create", post(create_payment))
            .layer(Extension(payment_service));

        // 创建测试请求 - 使用不存在的 tenant_id
        let request_body = json!({
            "tenant_id": 888,  // 不存在的 tenant_id
            "user_id": 100,
            "payment_type": "WX_H5",
            "amount": 10000,
            "currency": "CNY",
            "product_name": "测试商品",
            "product_desc": "商品描述",
            "callback_url": "http://example.com/callback",
            "notify_url": "http://example.com/notify"
        });

        let request = Request::builder()
            .uri("/api/v1/payment/create")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_body)?))
            .unwrap();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应 - 应该返回错误
        assert_ne!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;

        assert_eq!(body["success"], false);
        assert!(body["error"].is_object());

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// 
//     // 简化的测试助手
//     struct TestContext {
//         service: Arc<PaymentService>,
//         pool: sqlx::MySqlPool,
//     }
// 
//     impl TestContext {
//         async fn new() -> anyhow::Result<Self> {
//             let pool = setup_test_db().await?;
//             setup_test_data(&pool).await?;
// 
//             let config_cache = Arc::new(ConfigCache::new(
//                 pool.clone(),
//                 std::time::Duration::from_secs(60)
//             ));
// 
//             let payment_factory = Arc::new(PaymentFactory::new(config_cache.clone()));
//             let service = Arc::new(PaymentService::new(
//                 pool.clone(),
//                 payment_factory,
//                 config_cache,
//             ));
// 
//             Ok(Self { service, pool })
//         }
// 
//         fn app(&self) -> Router {
//             Router::new()
//                 .route("/api/v1/payment/create", post(create_payment))
//                 .route("/api/v1/payment/query/:order_id", get(query_payment))
//                 .layer(Extension(self.service.clone()))
//         }
// 
//         async fn cleanup(&self) -> anyhow::Result<()> {
//             cleanup_test_data(&self.pool).await
//         }
//     }
// 
//     #[tokio::test]
//     async fn test_payment_handlers() -> anyhow::Result<()> {
//         let ctx = TestContext::new().await?;
//         let app = ctx.app();
// 
//         // 测试创建支付
//         let request_body = json!({
//             "tenant_id": 999,
//             "user_id": 100,
//             "payment_type": "WX_H5",
//             "amount": 10000,
//             "currency": "CNY",
//             "product_name": "测试商品"
//         });
// 
//         let response = app.clone()
//             .oneshot(
//                 Request::builder()
//                     .uri("/api/v1/payment/create")
//                     .method("POST")
//                     .header("Content-Type", "application/json")
//                     .body(Body::from(serde_json::to_vec(&request_body)?))
//                     .unwrap()
//             )
//             .await
//             .unwrap();
// 
//         assert_eq!(response.status(), StatusCode::OK);
// 
//         ctx.cleanup().await?;
//         Ok(())
//     }
// }