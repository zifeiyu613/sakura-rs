use payment_gateway::{
    api::routes,
    app_state::AppState,
    config::AppConfig,
    infrastructure::{
        database::init_database,
        cache::init_redis,
        messaging::init_rabbitmq,
        logging::init_logging,
    },
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化配置
    dotenv::dotenv().ok();
    let config = AppConfig::load()?;

    // 初始化日志
    init_logging(&config);

    info!("Starting payment service...");

    // 初始化数据库连接
    let db_pool = init_database(&config).await?;

    // 初始化Redis客户端
    let redis_manager = init_redis(&config).await?;

    // 初始化RabbitMQ
    let mq_connection = init_rabbitmq(&config).await?;

    // 创建应用状态
    let app_state = Arc::new(AppState::new(
        config.clone(),
        db_pool,
        redis_manager,
        mq_connection,
    ));

    // 初始化路由
    let app = routes::create_router(app_state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
