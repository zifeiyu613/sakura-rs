use crate::config::AppConfig;
use anyhow::{Result, Context};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::Path;
use std::str::FromStr;

pub fn init_logging(config: &AppConfig) -> Result<()> {
    let env_filter = EnvFilter::from_str(&format!("{}={}", config.service_name, config.logging.level))
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = if config.logging.json_format {
        fmt::layer().json().with_span_events(FmtSpan::CLOSE)
    } else {
        fmt::layer().with_span_events(FmtSpan::CLOSE)
    };

    // 基本订阅者
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    // 如果配置了日志文件路径，添加文件输出
    if let Some(file_path) = &config.logging.file_path {
        let path = Path::new(file_path);
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            dir,
            path.file_name().unwrap_or_default(),
        );

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking);

        if config.logging.json_format {
            let json_file_layer = file_layer.json();
            subscriber.with(json_file_layer).init();
        } else {
            subscriber.with(file_layer).init();
        }

        // 保留 _guard 变量，确保在程序退出前不会被删除
        // 但我们需要让它保持活动状态，所以使用 Box::leak
        Box::leak(Box::new(_guard));
    } else {
        subscriber.init();
    }

    tracing::info!("Logging initialized with level: {}", config.logging.level);

    Ok(())
}

// 为请求添加跟踪ID的中间件
pub fn request_tracing_middleware() -> impl tower::ServiceComponent {
    use tower_http::trace::{self, TraceLayer};
    use axum::http::Request;
    use uuid::Uuid;

    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|id| id.to_str().ok())
                .unwrap_or_else(|| {
                    let id = Uuid::new_v4().to_string();
                    // 我们不能在这里修改请求，所以只返回生成的 ID
                    id.as_str()
                });

            tracing::info_span!(
                "request",
                request_id = %request_id,
                method = %request.method(),
                uri = %request.uri(),
            )
        })
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO))
}
