//! rlog - 基于 tracing 的日志组件
//!
//! 提供简单易用的日志功能，支持控制台和文件输出，
//! 并支持日志格式化和滚动文件。

use std::collections::HashMap;
use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::{self}, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry};

// 使用预设的 LogConfig
pub use rconfig::presets::logging::LogConfig;

// 全局日志状态
struct LogState {
    config: LogConfig,
    _guards: Vec<WorkerGuard>, // 保持 guards 存活，确保日志正确写入
}

static LOGGER: OnceCell<Arc<Mutex<LogState>>> = OnceCell::new();


/// 自定义时间格式化
#[derive(Debug, Clone)]
struct CustomTime;

impl fmt::time::FormatTime for CustomTime {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        // 使用 RFC3339 格式
        let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        write!(w, "{}", now.format(&time::format_description::well_known::Rfc3339).unwrap())
    }
}


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

    // 构建基本过滤器
    let mut filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => {
            // 解析全局日志级别
            let level_str = config.level.to_lowercase();
            let level = Level::from_str(&level_str)
                .map_err(|_| format!("Invalid log level: {}", level_str))?;
            EnvFilter::new(format!("{}", level))
        }
    };

    // 添加模块级别过滤器
    for (module, level) in &config.module_filters {
        let directive = format!("{}={}", module, level);
        match directive.parse() {
            Ok(directive) => filter = filter.add_directive(directive),
            Err(e) => return Err(format!("Invalid filter directive '{}': {}", directive, e)),
        }
    }
    

    // 存储 WorkerGuard 实例，防止过早丢弃
    let mut guards = Vec::new();
    
    // 构建订阅器
    let mut registry = Registry::default()
        .with(filter).with(console_layer());

    // 自定义时间格式化器
    let timer = CustomTime;
    
    let console_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(std::io::stdout)
        .with_timer(timer)
        .with_ansi(config.use_ansi_colors)
        .with_file(config.show_source_location)
        .with_line_number(config.show_source_location)
        .with_target(config.show_target)
        .with_thread_ids(config.show_thread_id);
    
    
    // 同时配置文件输出（如果需要）
    // if config.to_file {
    //     if let Some(file_path) = &config.file_path {
    //         let dir = file_path.parent().unwrap_or_else(|| Path::new("."));
    //         let file_name = file_path.file_name()
    //             .map(|n| n.to_string_lossy().to_string())
    //             .unwrap_or_else(|| "app.log".to_string());
    // 
    //         // 确保目录存在
    //         if !dir.exists() {
    //             std::fs::create_dir_all(dir)
    //                 .map_err(|e| format!("Failed to create log directory: {}", e))?;
    //         }
    // 
    //         // 解析轮转策略
    //         let rotation = match config.rotation.to_lowercase().as_str() {
    //             "hourly" => Rotation::HOURLY,
    //             "minutely" => Rotation::MINUTELY,
    //             "daily" => Rotation::DAILY,
    //             _ => Rotation::DAILY, // 默认每日轮转
    //         };
    // 
    //         // 创建文件附加器
    //         let file_appender = match RollingFileAppender::builder()
    //             .rotation(rotation)
    //             .filename_prefix(file_name)
    //             .max_log_files(config.max_files as usize)
    //             .build(dir) {
    //             Ok(appender) => appender,
    //             Err(e) => return Err(format!("Failed to create log file appender: {}", e)),
    //         };
    // 
    //         // 非阻塞写入
    //         let (non_blocking, guard) = NonBlocking::new(file_appender);
    //         guards.push(guard);
    // 
    //         // 创建文件层
    //         let file_layer = fmt::layer()
    //             .json()
    //             .with_current_span(true)
    //             .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    //             .with_writer(non_blocking)
    //             .with_ansi(config.use_ansi_colors)
    //             .with_file(config.show_source_location)
    //             .with_line_number(config.show_source_location)
    //             .with_target(config.show_target)
    //             .with_thread_ids(config.show_thread_id);
    //     } else {
    //         return Err("File path not specified for file logging".to_string());
    //     }
    // }


    // 设置全局订阅器
    registry.with(console_layer).init();

    // 保存配置和 guards
    let log_state = LogState {
        config,
        _guards: guards,
    };

    LOGGER.set(Arc::new(Mutex::new(log_state)))
        .map_err(|_| "Failed to set global logger state".to_string())?;

    Ok(())
}


fn console_layer<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    let timer = UtcTime::rfc_3339();
    fmt::layer()
        .with_timer(timer)
        .with_thread_ids(true)
        .with_target(true)
        .compact()
        .boxed()
}

// pub fn fs_layer<S>(log_dir: &PathBuf) -> Box<dyn Layer<S> + Send + Sync + 'static>
// where
//     S: Subscriber,
//     for<'a> S: LookupSpan<'a>,
// {
//     // create dir, build appender and timer, etc
//     fmt::layer()
//         .with_writer(file_appender)
//         .with_timer(timer.clone())
//         .with_thread_ids(true)
//         .with_thread_names(true)
//         .with_target(true)
//         .with_file(true)
//         .with_line_number(true)
//         .with_ansi(false)
//         .boxed()
// }

/// 创建格式化层
fn create_fmt_layer<W>(
    config: &LogConfig,
    writer: W,
    use_ansi: bool,
    timer: CustomTime,
) -> Box<dyn Layer<Registry> + Send + Sync>
where
    W: for<'a> MakeWriterExt<'a> + Send + Sync + 'static,
{
    match config.format.to_lowercase().as_str() {
        "json" => {
            let mut layer = fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(writer)
                .with_ansi(use_ansi);

            if config.show_source_location {
                layer = layer.with_file(true).with_line_number(true);
            } else {
                layer = layer.with_file(false).with_line_number(false);
            }

            if !config.show_target {
                layer = layer.with_target(false);
            }

            if config.show_thread_id {
                layer = layer.with_thread_ids(true);
            }

            // if config.show_timestamp {
            //     layer = layer.with_timer(timer);
            // } else {
            //     layer = layer.without_time();
            // }
            
            Box::new(layer)
        },
        "pretty_json" => {
            let mut layer = fmt::layer()
                .json()
                .pretty()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(writer)
                .with_ansi(use_ansi);

            // if config.show_timestamp {
            //     layer = layer.with_timer(timer);
            // } else {
            //     layer = layer.without_time();
            // }

            if config.show_source_location {
                layer = layer.with_file(true).with_line_number(true);
            } else {
                layer = layer.with_file(false).with_line_number(false);
            }

            if !config.show_target {
                layer = layer.with_target(false);
            }

            if config.show_thread_id {
                layer = layer.with_thread_ids(true);
            }

            Box::new(layer)
        },
        _ => { // 默认文本格式
            let mut layer = fmt::layer()
                .with_writer(writer)
                .with_ansi(use_ansi);

            // if config.show_timestamp {
            //     layer = layer.with_timer(timer);
            // } else {
            //     layer = layer.without_time();
            // }

            if config.show_source_location {
                layer = layer.with_file(true)
                    .with_line_number(true)
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
            }

            if !config.show_target {
                layer = layer.with_target(false);
            }

            if config.show_thread_id {
                layer = layer.with_thread_ids(true);
            }

            Box::new(layer)
        }
    }
    
    
}

/// 获取当前日志配置
pub fn get_config() -> Option<LogConfig> {
    LOGGER.get().map(|state| {
        let lock = state.lock().expect("Logger state lock poisoned");
        lock.config.clone()
    })
}

/// 重新配置日志系统
///
/// 注意：此方法不会改变已设置的格式和输出目标，只能调整过滤级别
pub fn reconfigure(level: &str, module_filters: Option<HashMap<String, String>>) -> Result<(), String> {
    let logger = LOGGER.get().ok_or("Logger not initialized")?;
    let mut logger_state = logger.lock().unwrap();

    // 更新全局级别
    logger_state.config.level = level.to_string();

    // 更新模块过滤器
    if let Some(filters) = module_filters {
        logger_state.config.module_filters = filters;
    }

    // 构建新的过滤器
    let mut filter = match Level::from_str(level) {
        Ok(level) => EnvFilter::new(format!("{}", level)),
        Err(_) => return Err(format!("Invalid log level: {}", level)),
    };

    // 添加模块级别过滤器
    for (module, level) in &logger_state.config.module_filters {
        let directive = format!("{}={}", module, level);
        match directive.parse() {
            Ok(directive) => filter = filter.add_directive(directive),
            Err(e) => return Err(format!("Invalid filter directive '{}': {}", directive, e)),
        }
    }

    // 应用新过滤器
    // 注意：这里我们只能修改过滤器，无法重新配置已初始化的输出格式和目标
    // 实际中这需要使用可重载的订阅器层，如 tracing_subscriber::reload
    // 由于这超出了当前函数的范围，此处仅作注释说明

    Ok(())
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
    debug, error, event, info, instrument,
    span,    // 用于跟踪函数调用
    trace, warn,   // 用于更细粒度的跟踪控制
    Level,         // 日志级别类型
};
use tracing::instrument::WithSubscriber;
use tracing::Subscriber;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::registry::LookupSpan;

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