use axum::{
    routing::get,
    Router,
    extract::State,
    response::IntoResponse,
    http::StatusCode,
    middleware,
};

use std::sync::Arc;
use std::time::Duration;

use rconfig::AppConfig;
use rlog::{LogBuilder, LogLevel, LogFormat};
use rdatabase::{DbPool, DatabaseExtension};

// 定义应用状态
#[derive(Clone)]
struct AppState {
    db: DatabaseExtension,
    config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 第一步：加载配置
    let config = AppConfig::new()
        .add_default("config/default")
        .add_environment()
        .build()?;

    // 第二步：初始化日志系统
    mylog::init_from_config(&config)?;

    // 输出启动信息
    tracing::info!("应用启动中...");
    tracing::info!("环境: {}", config.env.as_deref().unwrap_or("development"));

    // 第三步：初始化数据库
    tracing::info!("正在连接数据库...");
    let pool = DbPool::from_config(&config, None).await?;
    tracing::info!("数据库连接成功");

    // 创建应用状态
    let app_state = AppState {
        db: DatabaseExtension::new(pool),
        config: Arc::new(config),
    };

    // 第四步：创建路由
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/health", get(health_handler))
        .route("/api/error", get(error_handler))
        // 添加日志中间件
        .layer(middleware::from_fn(mylog::middleware::log_request_middleware))
        .layer(middleware::from_fn(mylog::middleware::log_error_middleware))
        .with_state(app_state);

    // 获取服务器地址
    let addr = format!(
        "{}:{}",
        app_state.config.server.host,
        app_state.config.server.port
    ).parse()?;

    tracing::info!("服务器运行在 {}", addr);

    // 启动服务器
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// 路由处理函数
async fn root_handler() -> &'static str {
    tracing::debug!("处理根路径请求");
    "欢迎使用我们的API！"
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    // 创建一个跟踪跨度
    let _guard = mylog::span!("health_check", component = "api");

    tracing::info!("执行健康检查");

    // 检查数据库连接
    let db_result = mylog::time_it_async!(
        "database_check",
        state.db.check_connection().await
    );

    match db_result {
        Ok(_) => {
            tracing::info!("健康检查成功");
            (StatusCode::OK, "服务正常运行中")
        },
        Err(e) => {
            tracing::error!(error = %e, "数据库连接检查失败");
            (StatusCode::SERVICE_UNAVAILABLE, "服务暂时不可用")
        }
    }
}

async fn error_handler() -> impl IntoResponse {
    // 模拟错误
    tracing::warn!("即将触发一个模拟错误");

    // 记录详细错误信息
    tracing::error!(
        code = 500,
        reason = "simulation",
        user_id = "anonymous",
        "模拟服务器错误"
    );

    StatusCode::INTERNAL_SERVER_ERROR
}
