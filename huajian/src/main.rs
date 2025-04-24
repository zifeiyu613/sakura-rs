use axum::{Router, middleware};
use huajian::{AppResult, DbName};
use huajian::{AppState, modules};
use huajian::{error::AppError, middleware::decrypted::decrypt};
use rconfig::AppConfig;
use rdatabase::DbPool;
use rlog::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    let path = format!(
        "{}/config/application-{}",
        env!("CARGO_MANIFEST_DIR"),
        "dev"
    );
    println!("Loading configuration from {}", &path);
    let app_config = AppConfig::new()
        .add_default(&path)
        .add_environment()
        .build()?;

    if let Some(log) = &app_config.log {
        rlog::init(log).expect("Failed to initialize logger");
    }

    // 连接数据库
    info!("Connecting to database...");
    let db_pool = DbPool::load_all_sources(&app_config).await?;
    let phoenix_pool = db_pool.get_pool(DbName::Phoenix.as_str()).await.unwrap();
    let activity_pool = db_pool.get_pool(DbName::Activity.as_str()).await.unwrap();

    db_pool.check_connection().await?;

    // 连接 Redis
    // info!("Connecting to Redis...");
    // let redis_client = db::redis::create_client(&config.redis_url)?;

    // 连接 RabbitMQ
    // info!("Connecting to RabbitMQ...");
    // let rabbit_connection = mq::rabbitmq::create_connection(&config.rabbitmq_url).await?;

    // 创建应用状态
    let state = AppState {
        db: db_pool,
        // redis: std::sync::Arc::new(redis_client),
        // rabbit: std::sync::Arc::new(rabbit_connection),
        app_config: Arc::new(app_config.clone()),
    };
    // 启动服务器
    let addr = format!("{}:{}", &app_config.server.host, &app_config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 创建路由
    let routers = create_router(state).await;

    info!("🚀 WebServer is running on: {}", &addr);

    axum::serve(listener, routers.into_make_service())
        // .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

async fn create_router(state: AppState) -> Router {
    Router::new()
        // 健康检查
        .route("/health", axum::routing::get(|| async { "OK" }))
        // API 版本前缀
        .nest("/api/v1", api_v1_routes())
        .with_state(state)
        .layer(middleware::from_fn(decrypt))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive())
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", modules::users::routes())
        .nest("/accounts", modules::accounts::routes())
        .nest("/activities", modules::activities::routes())
        .nest("/config", modules::configuration::routes())
}

async fn shutdown_signal() {
    // ...
}
