mod api;
mod config;
mod domain;
mod error;
mod server;

mod infrastructure;
mod logging;
mod middleware;
mod utils;

use std::sync::Arc;
use crate::config::Config;
use crate::error::AppError;
use crate::infrastructure::database::{mysql::init_mysql, redis::init_redis};
use crate::infrastructure::messaging::rabbitmq::init_rabbitmq;
use crate::logging::init_logging;
use crate::server::create_app;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 加载配置
    let config = Config::load()?;

    // 初始化日志
    init_logging(&config)?;

    tracing::info!("Starting API service");

    // 初始化数据库连接
    let db_pool = init_mysql(&config).await?;
    let redis = init_redis(&config).await?;
    let amqp = init_rabbitmq(&config).await?;
    let amqp = Arc::new(amqp);

    // 创建应用状态
    let app_state = server::AppState {
        config: config.clone(),
        db: db_pool,
        redis,
        amqp,
    };

    // 创建并启动服务器
    let app = create_app(app_state).await?;
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", &addr);

    axum::serve(listener, app).await?;
    Ok(())
}
