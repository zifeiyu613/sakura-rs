//! rlog - 基于 tracing 的日志组件
//!
//! 提供简单易用的日志功能，支持控制台和文件输出，
//! 并支持日志格式化和滚动文件。


use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use futures::TryFutureExt;
use tracing_subscriber::{fmt::{self, format::FmtSpan}, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry};
use tracing_appender::{
    rolling::{RollingFileAppender, Rotation},
    non_blocking::{NonBlocking, WorkerGuard},
};
use tracing_log::LogTracer;
use once_cell::sync::OnceCell;

// 使用您预设的 LogConfig
pub use rconfig::presets::logging::LogConfig;

// 全局日志状态
struct LogState {
    config: LogConfig,
    _guards: Vec<WorkerGuard>, // 保持 guards 存活，确保日志正确写入
}

static LOGGER: OnceCell<Arc<Mutex<LogState>>> = OnceCell::new();

/// 初始化日志系统
///
/// # Arguments
///
/// * `config` - 日志配置对象
///
/// # Returns
///
/// 初始化结果，成功则返回 Ok(())
pub fn init(config: LogConfig) -> Result<(), String> {
    // 防止重复初始化
    if LOGGER.get().is_some() {
        return Err("Logger already initialized".to_string());
    }

    // 将 log crate 的日志转发到 tracing
    if let Err(e) = LogTracer::init() {
        return Err(format!("Failed to initialize LogTracer: {}", e));
    }

    // 设置过滤器
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    // 构建订阅器
    let registry = Registry::default().with(filter);

    // 保存 WorkerGuard 实例
    let mut guards = Vec::new();

    // 控制台输出
    let registry = if config.to_console {
        let console_layer = if config.format.to_lowercase() == "json" {
            fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(std::io::stdout)
                .boxed()
        } else {
            let mut layer = fmt::layer().with_writer(std::io::stdout);

            if config.show_source_location {
                layer = layer
                    .with_file(true)
                    .with_line_number(true)
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
            }

            layer.boxed()
        };

        registry.with(console_layer)
    } else if config.to_file {
        if let Some(file_path) = &config.file_path {
            let dir = file_path.parent().unwrap_or_else(|| std::path::Path::new("."));
            let file_name = file_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "app.log".to_string());

            // 确保目录存在
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| format!("Failed to create log directory: {}", e))?;
            }

            // 设置滚动策略 (基于大小)
            let max_size = config.max_file_size * 1024 * 1024; // MB -> bytes

            let file_appender = RollingFileAppender::builder()
                .rotation(Rotation.)
                .filename_prefix(file_name)
                .max_files(config.max_files as usize)
                .build(dir)
                .map_err(|e| format!("Failed to create log file appender: {}", e))?;

            // 非阻塞写入
            let (non_blocking, guard) = NonBlocking::new(file_appender);
            guards.push(guard);

            let file_layer = if config.format.to_lowercase() == "json" {
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_ansi(false)
                    .with_writer(non_blocking)
                    .boxed()
            } else {
                let mut layer = fmt::layer()
                    .with_ansi(false)
                    .with_writer(non_blocking);

                if config.show_source_location {
                    layer = layer
                        .with_file(true)
                        .with_line_number(true)
                        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
                }

                layer.boxed()
            };

            registry.with(file_layer)
        } else {
            return Err("File path not specified for file logging".to_string());
        }
    };

    // 设置全局订阅器
    registry.init()
        .map_err(|e| format!("Failed to set global subscriber: {}", e))?;

    // 保存配置和 guards
    let log_state = LogState {
        config,
        _guards: guards,
    };

    LOGGER.set(Arc::new(Mutex::new(log_state)))
        .map_err(|_| "Failed to set global logger state".to_string())?;

    Ok(())
}

/// 获取当前日志配置
pub fn get_config() -> Option<LogConfig> {
    LOGGER.get().map(|state| {
        let lock = state.lock().expect("Logger state lock poisoned");
        lock.config.clone()
    })
}

/// 流式 API 构建器
pub struct LoggerBuilder {
    config: LogConfig,
}

impl LoggerBuilder {
    /// 创建新的日志构建器
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
        }
    }

    /// 设置日志级别
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.config.level = level.into();
        self
    }

    /// 设置是否输出到控制台
    pub fn to_console(mut self, enabled: bool) -> Self {
        self.config.to_console = enabled;
        self
    }

    /// 设置是否输出到文件
    pub fn to_file(mut self, enabled: bool) -> Self {
        self.config.to_file = enabled;
        self
    }

    /// 设置日志文件路径
    pub fn file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.file_path = Some(path.into());
        self
    }

    /// 设置日志格式 (json 或 text)
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.config.format = format.into();
        self
    }

    /// 设置是否显示源码位置
    pub fn show_source_location(mut self, show: bool) -> Self {
        self.config.show_source_location = show;
        self
    }

    /// 设置日志文件最大大小(MB)
    pub fn max_file_size(mut self, size_mb: u64) -> Self {
        self.config.max_file_size = size_mb;
        self
    }

    /// 设置保留的日志文件数量
    pub fn max_files(mut self, count: u32) -> Self {
        self.config.max_files = count;
        self
    }

    /// 初始化日志系统
    pub fn init(self) -> Result<(), String> {
        init(self.config)
    }
}

/// 从配置对象初始化
pub fn from_config(config: LogConfig) -> Result<(), String> {
    init(config)
}

// 重新导出 tracing 宏，以便用户可以直接从 rlog 使用
pub use tracing::{
    trace, debug, info, warn, error,
    instrument,    // 用于跟踪函数调用
    span, event,   // 用于更细粒度的跟踪控制
    Level,         // 日志级别类型
};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_logger_init() {
        let config = LogConfig {
            level: "debug".to_string(),
            to_console: true,
            ..Default::default()
        };

        let result = init(config);
        assert!(result.is_ok());

        info!("Test log message");
        debug!("Debug message");
    }

    #[test]
    fn test_file_logging() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempdir()?;
        let log_path = temp.path().join("app.log");

        let config = LogConfig {
            level: "info".to_string(),
            to_console: false,
            to_file: true,
            file_path: Some(log_path.clone()),
            ..Default::default()
        };

        from_config(config)?;

        info!("File log test");

        // 这里可以添加验证文件是否创建的逻辑
        // 由于非阻塞写入，文件内容检查需要在 guard drop 后才准确

        Ok(())
    }
}