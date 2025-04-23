//! 日志配置

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use super::Validate;
use std::path::PathBuf;
use tracing::log;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogConfig {
    /// 日志级别: trace, debug, info, warn, error
    #[serde(default = "default_level")]
    pub level: String,

    /// 是否输出到控制台
    #[serde(default = "default_to_console")]
    pub to_console: bool,
    
    /// 控制台是否使用颜色
    #[serde(default = "default_to_console")]
    pub use_ansi_colors: bool,
    
    /// 是否输出到文件
    #[serde(default)]
    pub to_file: bool,

    /// 日志文件路径 (如果to_file=true)
    #[serde(default)]
    pub file_path: Option<PathBuf>,

    /// 日志格式: json, text
    #[serde(default = "default_format")]
    pub format: String,

    /// 是否显示源代码位置
    #[serde(default = "default_show_source_location")]
    pub show_source_location: bool,

    /// 日志文件最大大小(MB)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// 保留的日志文件数量
    #[serde(default = "default_max_files")]
    pub max_files: u32,
    
    /// 轮转策略 (daily, hourly, minutely, size)
    #[serde(default = "default_rotation")]
    pub rotation: String,
    /// 是否显示时间戳
    #[serde(default)]
    pub show_timestamp: bool,
    /// 是否显示目标模块
    #[serde(default)]
    pub show_target: bool,
    /// 是否显示线程ID
    #[serde(default)]
    pub show_thread_id: bool,
    /// 模块级别过滤器
    pub module_filters: HashMap<String, String>,
    
}

fn default_level() -> String {
    "info".to_string()
}

fn default_to_console() -> bool {
    true
}

fn default_format() -> String {
    "text".to_string()
}

fn default_show_source_location() -> bool {
    false
}

fn default_max_file_size() -> u64 {
    10
}

fn default_max_files() -> u32 {
    5
}

fn default_rotation() -> String {
    "daily".to_string()
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            to_console: default_to_console(),
            use_ansi_colors: false,
            to_file: false,
            file_path: None,
            format: default_format(),
            show_source_location: default_show_source_location(),
            max_file_size: default_max_file_size(),
            max_files: default_max_files(),
            rotation: "daily".to_string(),
            show_timestamp: false,
            show_target: false,
            show_thread_id: false,
            module_filters: HashMap::new(),
        }
    }
}

impl LogConfig {
    /// 将字符串日志级别转换为log crate的Level
    pub fn parse_level(&self) -> log::LevelFilter {
        match self.level.to_lowercase().as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            "off" => log::LevelFilter::Off,
            _ => log::LevelFilter::Info,
        }
    }
}

impl Validate for LogConfig {
    fn validate(&self) -> Result<()> {
        // 如果配置了输出到文件，需要文件路径
        if self.to_file && self.file_path.is_none() {
            return Err(crate::error::ConfigError::ValidationError(
                "输出日志到文件时，需要指定文件路径".to_string()
            ));
        }

        // 检查日志级别是否有效
        if !["trace", "debug", "info", "warn", "error", "off"]
            .contains(&self.level.to_lowercase().as_str()) {
            return Err(crate::error::ConfigError::ValidationError(
                format!("无效的日志级别: {}", self.level)
            ));
        }

        // 检查日志格式是否有效
        if !["json", "text"].contains(&self.format.to_lowercase().as_str()) {
            return Err(crate::error::ConfigError::ValidationError(
                format!("无效的日志格式: {}", self.format)
            ));
        }

        Ok(())
    }
}
