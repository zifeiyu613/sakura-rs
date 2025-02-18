use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use tracing::instrument::WithSubscriber;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::fmt::{self, writer::MakeWriterExt};
use tracing_subscriber::{filter, prelude::*, registry::Registry};
use tracing_subscriber::filter::Filtered;

#[derive(Deserialize)]
struct Config {
    logging: LoggingConfig,
    audit: AuditConfig,
}

#[derive(Deserialize)]
struct LoggingConfig {
    log_level: String,
    log_path: String,
    file_rotation: String,
    log_format: String,
}

#[derive(Deserialize)]
struct AuditConfig {
    enabled: bool,
    audit_log_level: String,
}

/// 加载 TOML 配置文件
fn load_config<P: AsRef<Path>>(path: P) -> Config {
    // let file = File::open(path).expect("无法打开配置文件");
    let config_content = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "❌ 读取 `{}` 失败，请确保配置文件存在",
            &path.as_ref().display()
        )
    });

    let config: Config = toml::de::from_str(&config_content).expect("无法解析配置文件");
    config
}

/// 初始化日志系统：设置常规日志和审计日志的输出目标
pub fn init_logging() {
    let config = load_config(Path::new("config.toml"));
    // 解析日志级别
    let log_level = match config.logging.log_level.as_str() {
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "ERROR" => Level::ERROR,
        _ => Level::INFO, // 默认 INFO
    };

    let stderr = std::io::stderr.with_max_level(Level::DEBUG);

    let std_layer = fmt::layer()
        .with_writer(stderr)
        .with_span_events(fmt::format::FmtSpan::CLOSE) // 记录 span 关闭时的事件
        .with_timer(ChronoLocal::rfc_3339())
        .with_thread_ids(false)
        .with_line_number(false)
        .with_target(true) // 可根据需要选择是否记录 target 信息
        .json(); // 以 JSON 格式输出

    // 设置日志时间格式（上海时区）
    // let timer = ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());

    // 创建每日滚动的文件 appender，用于常规日志
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, &config.logging.log_path, "app.log");
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        // .map_writer(move |w| file_stderr.or_else(w))
        .with_span_events(fmt::format::FmtSpan::CLOSE) // 记录 span 关闭时的事件
        .with_timer(ChronoLocal::rfc_3339())
        .with_thread_ids(true)
        .with_line_number(true)
        .with_target(false) // 可根据需要选择是否记录 target 信息
        .json() // 以 JSON 格式输出
        .with_filter(LevelFilter::from_level(Level::DEBUG));


    // 创建每日滚动的文件 appender，用于审计日志
    let audit_appender =
        RollingFileAppender::new(Rotation::DAILY, &config.logging.log_path, "audit.log");
    let audit_layer = fmt::layer()
        .with_writer(audit_appender)
        .json()
        .with_timer(ChronoLocal::rfc_3339())
        .with_thread_ids(false)
        .with_line_number(true)
        // 仅过滤 target 中包含 "audit" 的事件
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target().contains("audit")
        }));

    // 构建全局 subscriber，并设置为全局默认
    let subscriber = Registry::default()
        .with(std_layer)
        .with(file_layer).with(audit_layer);

    tracing::subscriber::set_global_default(subscriber).expect("无法设置全局 subscriber");
}
