//! rlog - 灵活的日志组件
//!
//! 基于 tracing 生态系统构建的简单易用的日志组件，
//! 提供灵活的配置选项和与 myconfig 的无缝集成。

mod config;
mod builder;
mod macros;

pub use config::{LogConfig, LogLevel, LogFormat};
pub use builder::LogBuilder;

// 重新导出 tracing 中常用的宏和类型，方便使用
pub use tracing::{trace, debug, info, warn, error, span, Level, Span};

use std::error::Error;
use std::fmt;

/// 日志错误类型
#[derive(Debug)]
pub enum LogError {
    /// IO 错误
    IoError(std::io::Error),
    /// 配置错误
    ConfigError(String),
    /// 初始化错误
    InitError(String),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogError::IoError(e) => write!(f, "IO 错误: {}", e),
            LogError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            LogError::InitError(msg) => write!(f, "初始化错误: {}", msg),
        }
    }
}

impl Error for LogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LogError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, LogError>;

/// 使用默认配置初始化日志系统
pub fn init() -> Result<()> {
    LogBuilder::new().build()?.init()
}

/// 从配置对象初始化日志系统
///
/// # 示例
/// ```
/// use rconfig::AppConfig;
/// use rlog::init_from_config;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = AppConfig::new().build()?;
///     
///     // 从配置初始化日志
///     init_from_config(&config.logging)?;
///     
///     // 使用日志
///     tracing::info!("应用启动");
///     
///     Ok(())
/// }
/// ```
pub fn init_from_config<T>(config: &rconfig::LogConfig) -> Result<()>
{
    let log_config = LogConfig::from_config(config)?;
    LogBuilder::from_config(log_config).build()?.init()
}

/// 创建一个新的日志构建器
pub fn builder() -> LogBuilder {
    LogBuilder::new()
}
