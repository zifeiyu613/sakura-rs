use axum::{
    Router,
    routing::{get, post},
    Extension,
};
use std::net::SocketAddr;
use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use payment_service::{config, db, handlers, payment, services};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 设置日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "payment_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let settings = config::settings::AppSettings::from_env();

    // 初始化数据库连接池
    let pool = db::create_pool(&settings.database_url).await?;

    // 初始化数据库表
    db::init_db(&pool).await?;

    // 初始化配置缓存
    let config_cache = Arc::new(
        config::cache::ConfigCache::new(
            pool.clone(),
            std::time::Duration::from_secs(settings.cache_ttl_seconds)
        )
    );

    // 初始化支付工厂
    let payment_factory = Arc::new(payment::factory::PaymentFactory::new(config_cache.clone()));

    // 初始化支付服务
    let payment_service = Arc::new(services::payment_service::PaymentService::new(
        pool.clone(),
        payment_factory,
        config_cache,
    ));

    // 构建路由
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/payment/create", post(handlers::create_payment))
        .route("/api/v1/payment/query/:order_id", get(handlers::query_payment))
        .route("/api/v1/payment/callback/:payment_type", post(handlers::payment_callback))
        .route("/api/v1/payment/refund", post(handlers::refund_payment))
        .layer(Extension(payment_service))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server_port));
    tracing::info!("Payment service listening on {}", addr);

    // 处理未定义Paths
    let app= app.fallback(handler_404);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    
    axum::serve::serve(listener, app.into_make_service()).await?;

    Ok(())
}


async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}