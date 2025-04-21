//! 日志配置

use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::LogError;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// 跟踪级别 - 最详细
    Trace,
    /// 调试级别
    Debug,
    /// 信息级别
    Info,
    /// 警告级别
    Warn,
    /// 错误级别
    Error,
}

impl FromStr for LogLevel {
    type Err = LogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" | "err" => Ok(LogLevel::Error),
            _ => Err(LogError::ConfigError(format!("未知的日志级别: {}", s))),
        }
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// 日志格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// 文本格式
    Text,
    /// JSON 格式
    Json,
    /// 美化的 JSON 格式
    PrettyJson,
    /// 紧凑格式
    Compact,
}

impl FromStr for LogFormat {
    type Err = LogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            "prettyjson" | "pretty_json" => Ok(LogFormat::PrettyJson),
            "compact" => Ok(LogFormat::Compact),
            _ => Err(LogError::ConfigError(format!("未知的日志格式: {}", s))),
        }
    }
}

/// 控制台日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLogConfig {
    /// 是否启用控制台日志
    #[serde(default = "default_console_enabled")]
    pub enabled: bool,
    /// 是否启用颜色
    #[serde(default = "default_use_colors")]
    pub use_colors: bool,
}

fn default_console_enabled() -> bool {
    true
}

fn default_use_colors() -> bool {
    true
}

impl Default for ConsoleLogConfig {
    fn default() -> Self {
        Self {
            enabled: default_console_enabled(),
            use_colors: default_use_colors(),
        }
    }
}

/// 文件日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLogConfig {
    /// 是否启用文件日志
    #[serde(default = "default_file_enabled")]
    pub enabled: bool,
    /// 日志文件路径
    #[serde(default = "default_file_path")]
    pub path: String,
    /// 日志轮转策略 (daily, hourly, minutely, size:123)
    #[serde(default = "default_rotation")]
    pub rotation: String,
    /// 日志文件最大尺寸 (字节)
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    /// 保存的最大文件数量
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

fn default_file_enabled() -> bool {
    false
}

fn default_file_path() -> String {
    "logs/app.log".to_string()
}

fn default_rotation() -> String {
    "daily".to_string()
}

fn default_max_size() -> u64 {
    10 * 1024 * 1024 // 10 MB
}

fn default_max_files() -> usize {
    7
}

impl Default for FileLogConfig {
    fn default() -> Self {
        Self {
            enabled: default_file_enabled(),
            path: default_file_path(),
            rotation: default_rotation(),
            max_size: default_max_size(),
            max_files: default_max_files(),
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 全局日志级别
    #[serde(default = "default_level")]
    pub level: LogLevel,
    /// 日志格式
    #[serde(default = "default_format")]
    pub format: LogFormat,
    /// 是否显示模块路径
    #[serde(default = "default_show_module_path")]
    pub show_module_path: bool,
    /// 是否显示行号
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,
    /// 是否显示目标
    #[serde(default = "default_show_target")]
    pub show_target: bool,
    /// 是否显示线程ID
    #[serde(default = "default_show_thread_id")]
    pub show_thread_id: bool,
    /// 是否使用异步记录器
    #[serde(default = "default_async_logging")]
    pub async_logging: bool,
    /// 控制台日志配置
    #[serde(default)]
    pub console: ConsoleLogConfig,
    /// 文件日志配置
    #[serde(default)]
    pub file: FileLogConfig,
    /// 模块级别配置
    #[serde(default)]
    pub module_levels: HashMap<String, LogLevel>,
}

fn default_level() -> LogLevel {
    LogLevel::Info
}

fn default_format() -> LogFormat {
    LogFormat::Text
}

fn default_show_module_path() -> bool {
    true
}

fn default_show_line_numbers() -> bool {
    false
}

fn default_show_target() -> bool {
    true
}

fn default_show_thread_id() -> bool {
    false
}

fn default_async_logging() -> bool {
    true
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            format: default_format(),
            show_module_path: default_show_module_path(),
            show_line_numbers: default_show_line_numbers(),
            show_target: default_show_target(),
            show_thread_id: default_show_thread_id(),
            async_logging: default_async_logging(),
            console: ConsoleLogConfig::default(),
            file: FileLogConfig::default(),
            module_levels: HashMap::new(),
        }
    }
}

impl LogConfig {
    /// 从配置对象创建日志配置
    pub fn from_config<T>(config: &T) -> crate::Result<Self>
    where
        T: rconfig::ConfigAccess + ?Sized,
    {
        // 尝试直接获取日志配置部分
        match config.get::<LogConfig>("log") {
            Ok(log_config) => Ok(log_config),
            Err(_) => {
                // 如果不存在完整的日志配置，则尝试构建一个默认配置
                let mut log_config = LogConfig::default();

                // 读取单独的配置项
                if let Ok(level) = config.get::<String>("log.level") {
                    log_config.level = level.parse::<LogLevel>()?;
                }

                if let Ok(format) = config.get::<String>("log.format") {
                    log_config.format = format.parse::<LogFormat>()?;
                }

                if let Ok(path) = config.get::<String>("log.file.path") {
                    log_config.file.path = path;
                    log_config.file.enabled = true;
                }

                // 返回默认值或部分覆盖的配置
                Ok(log_config)
            }
        }
    }
}
