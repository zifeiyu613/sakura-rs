use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use tracing::log::warn;

// 错误日志中间件
pub async fn log_errors(request: Request, next: Next) -> Result<Response, StatusCode> {
    let response = next.run(request).await;

    // 检查是否为错误响应
    if response.status().is_client_error() || response.status().is_server_error() {
        warn!("请求失败: {}", response.status());
    }

    Ok(response)
}
