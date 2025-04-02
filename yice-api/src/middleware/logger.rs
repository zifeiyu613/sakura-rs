use crate::middleware::decryptor::RequestData;
use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, info};

pub async fn log_request(request: Request, next: Next) -> Result<Response, StatusCode> {
    let start_time = Instant::now();

    // 提取请求信息
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    // 获取查询参数
    // 获取查询参数
    let query_params = uri.query().unwrap_or("").to_string();

    // 尝试从扩展中获取请求数据（由解密中间件添加）
    let request_data = request.extensions().get::<RequestData>().cloned();

    // 准备请求体显示
    let body_display = if let Some(data) = request_data.as_ref() {
        // 优先使用处理后的数据
        if let Some(processed) = &data.processed_body {
            if data.is_decrypted {
                format!("[DECRYPTED] {}", format_content(processed))
            } else {
                format!("[PLAINTEXT] {}", format_content(processed))
            }
        } else {
            // 使用原始数据
            format_content_bytes(&data.original_body)
        }
    } else {
        // 如果不存在请求数据，使用空字符串
        "<no body>".to_string()
    };

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

    Ok(response)
}

// 格式化字符串内容 (限制长度)
fn format_content(content: &str) -> String {
    if content.len() > 1024 {
        format!("{} [truncated]", &content[..1024])
    } else {
        content.to_string()
    }
}

// 格式化字节内容 (转换为字符串并尝试美化 JSON)
fn format_content_bytes(bytes: &Bytes) -> String {
    let body_str = String::from_utf8_lossy(bytes);

    // 尝试解析为 JSON 以更美观地显示
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
        let formatted = serde_json::to_string(&json).unwrap_or(body_str.to_string());
        format_content(&formatted)
    } else {
        format_content(&body_str)
    }
}

// 格式化处理后的内容
fn processed_content_display(content: &str) -> String {
    if content.len() > 1024 {
        format!("{} [truncated]", &content[..1024])
    } else {
        content.to_string()
    }
}

// 格式化原始内容
fn raw_content_display(bytes: &Bytes) -> String {
    let body_str = String::from_utf8_lossy(bytes);

    // 尝试解析为 JSON 以更美观地显示
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
        let formatted = serde_json::to_string(&json).unwrap_or(body_str.to_string());
        if formatted.len() > 1024 {
            format!("{} [truncated]", &formatted[..1024])
        } else {
            formatted
        }
    } else if body_str.len() > 1024 {
        format!("{} [truncated]", &body_str[..1024])
    } else {
        body_str.to_string()
    }
}
