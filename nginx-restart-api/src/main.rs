use axum::{
    routing::{get, post},
    extract::State,
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{env, process::Command, sync::Arc, time::Duration};
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing::{info, error, warn};

// 应用状态
struct AppState {
    restart_key: String,
}

// 请求模型
#[derive(Deserialize)]
struct RestartRequest {
    key: String,
}

// 响应模型
#[derive(Serialize)]
struct ApiResponse {
    message: String,
    success: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 从环境变量获取密钥，默认值为更安全的随机值
    let restart_key = env::var("NGINX_RESTART_KEY")
        .unwrap_or_else(|_| {
            let key = uuid::Uuid::new_v4().to_string();
            info!("No NGINX_RESTART_KEY provided, using generated key: {}", key);
            key
        });

    // 获取端口，默认为9999
    let port = env::var("PORT").unwrap_or_else(|_| "9999".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("Starting Nginx restart API server on {}", addr);

    // 创建应用状态
    let state = Arc::new(AppState { restart_key });

    // 配置路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/restart-nginx", post(restart_nginx))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// 健康检查处理器
async fn health_check() -> (StatusCode, Json<ApiResponse>) {
    info!("Health check requested");

    (
        StatusCode::OK,
        Json(ApiResponse {
            message: "Service is healthy".to_string(),
            success: true,
        }),
    )
}

// Nginx重启处理器
async fn restart_nginx(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RestartRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    // 验证密钥
    if payload.key != state.restart_key {
        warn!("Invalid restart key provided");
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                message: "Invalid authentication key".to_string(),
                success: false,
            }),
        );
    }

    info!("Valid restart request received, executing restart script");

    // 使用异步任务执行重启命令
    tokio::spawn(async {
        // 小延迟确保响应先返回
        sleep(Duration::from_millis(100)).await;

        // 执行重启脚本
        match Command::new("/usr/local/bin/restart-nginx.sh").output() {
            Ok(output) => {
                if output.status.success() {
                    info!("Nginx restart successful");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Nginx restart failed: {}", stderr);
                }
            },
            Err(e) => {
                error!("Failed to execute restart script: {}", e);
            }
        }
    });

    (
        StatusCode::OK,
        Json(ApiResponse {
            message: "Nginx restart initiated".to_string(),
            success: true,
        }),
    )
}
