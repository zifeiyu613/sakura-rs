use std::collections::HashMap;
use std::io::Read;
use axum::body::{Body, Bytes};
use axum::BoxError;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;
use serde_json::Value;
use tokio::time::Instant;
use tracing::log::{debug, info};

pub async fn log_request(request: Request, next: Next) -> Result<Response, StatusCode> {
    let start_time = Instant::now();

    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    let headers = request.headers().iter()
        .map(|(name, value)| {
        (
            name.to_string(),
            value.to_str().unwrap_or("<non-utf8>").to_string(),
        )
    }).collect::<Vec<_>>();

    // 捕获查询参数
    let query_params = if let Some(query) = uri.query() {
        query.to_string()
    } else {
        "".to_string()
    };

    // 捕获并克隆请求体 (需要特殊处理)
    let (parts, body) = request.into_parts();
    let bytes = buffer_body(body).await;


    // 尝试将请求体转换为 JSON 用于日志记录
    let body_str = String::from_utf8_lossy(&bytes);
    let body_json: Result<Value, _> = serde_json::from_str(&body_str);
    let body_display = match body_json {
        Ok(json) => serde_json::to_string(&json).unwrap_or_else(|_| body_str.to_string()),
        Err(_) => if body_str.len() > 1024 {
            format!("{} [truncated]", &body_str[..1024])
        } else {
            body_str.to_string()
        }
    };

    // 重新构建请求
    let request = Request::from_parts(parts, Body::from(bytes));

    // 发送请求到下一个处理器
    let response = next.run(request).await;

    // 计算处理时间
    let duration = start_time.elapsed();

    // 记录请求和响应信息
    info!(
        target: "request_logger",
        "REQUEST: {} {} {:?} | Query: {} | Body: {} | Status: {} | Duration: {:?}",
        method,
        uri,
        version,
        query_params,
        body_display,
        response.status().as_u16(),
        duration
    );

    // 可选: 在调试模式下记录请求头
    if cfg!(debug_assertions) {
        debug!(target: "request_logger", "Headers: {:#?}", headers);
    }

    Ok(response)
}

// 辅助函数: 缓冲请求体内容
async fn buffer_body(body: Body) -> Bytes {
    // 在 Axum 0.8 中使用 `axum::body::to_bytes` 或直接收集字节
    axum::body::to_bytes(body, usize::MAX).await.unwrap_or_else(|err| {
        tracing::warn!("Failed to buffer request body: {}", err);
        Bytes::new()
    })
}
