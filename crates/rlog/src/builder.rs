//! 日志构建器

use std::path::Path;
use std::collections::HashMap;

use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};
use tracing_appender::{
    rolling::{RollingFileAppender, Rotation},
    non_blocking::NonBlocking,
};
use time::UtcOffset;

use crate::config::{LogConfig, LogLevel, LogFormat};
use crate::{LogError, Result};

// 这里不再需要自定义格式化器，直接使用 tracing-subscriber 的
// 但我们可以添加自定义的时间格式化

/// 自定义时间格式化
struct CustomTime;

impl tracing_subscriber::fmt::time::FormatTime for CustomTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        // 使用 RFC3339 格式
        let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        write!(w, "{}", now.format(&time::format_description::well_known::Rfc3339).unwrap())
    }
}

/// 日志构建器
///
/// 用于配置和初始化日志系统
#[derive(Debug)]
pub struct LogBuilder {
    config: LogConfig,
}

impl LogBuilder {
    /// 创建新的日志构建器
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
        }
    }

    /// 从配置创建日志构建器
    pub fn from_config(config: LogConfig) -> Self {
        Self { config }
    }

    /// 设置日志级别
    pub fn level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    /// 设置日志格式
    pub fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }

    /// 设置是否显示模块路径
    pub fn show_module_path(mut self, show: bool) -> Self {
        self.config.show_module_path = show;
        self
    }

    /// 设置是否显示行号
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.config.show_line_numbers = show;
        self
    }

    /// 设置是否显示目标
    pub fn show_target(mut self, show: bool) -> Self {
        self.config.show_target = show;
        self
    }

    /// 设置是否显示线程ID
    pub fn show_thread_id(mut self, show: bool) -> Self {
        self.config.show_thread_id = show;
        self
    }

    /// 启用控制台日志
    pub fn with_console(mut self, enabled: bool) -> Self {
        self.config.console.enabled = enabled;
        self
    }

    /// 设置控制台颜色
    pub fn with_console_colors(mut self, use_colors: bool) -> Self {
        self.config.console.use_colors = use_colors;
        self
    }

    /// 启用文件日志
    pub fn with_file(mut self, enabled: bool) -> Self {
        self.config.file.enabled = enabled;
        self
    }

    /// 设置文件路径
    pub fn with_file_path(mut self, path: &str) -> Self {
        self.config.file.path = path.to_string();
        self.config.file.enabled = true;
        self
    }

    /// 设置文件轮转策略
    pub fn with_file_rotation(mut self, rotation: &str) -> Self {
        self.config.file.rotation = rotation.to_string();
        self
    }

    /// 设置模块级别
    pub fn with_module_level(mut self, module: &str, level: LogLevel) -> Self {
        self.config.module_levels.insert(module.to_string(), level);
        self
    }

    /// 设置异步日志
    pub fn with_async_logging(mut self, enabled: bool) -> Self {
        self.config.async_logging = enabled;
        self
    }

    /// 构建日志系统
    ///
    /// 此方法创建一个可初始化的日志订阅器
    pub fn build(self) -> Result<LogSubscriber> {
        // 创建环境过滤器
        let filter = self.create_filter()?;

        // 创建各层
        let layers = self.create_layers()?;

        // 返回可初始化的日志订阅器
        Ok(LogSubscriber {
            registry: Registry::default()
                .with(filter)
                .with(layers),
        })
    }

    /// 创建环境过滤器
    fn create_filter(&self) -> Result<EnvFilter> {
        let mut filter = EnvFilter::new("");

        // 设置全局级别
        let global_directive = format!("info={}", tracing::Level::from(self.config.level));
        filter = filter.add_directive(global_directive.parse()
            .map_err(|e| LogError::ConfigError(format!("解析指令错误: {}", e)))?);

        // 设置模块级别
        for (module, level) in &self.config.module_levels {
            let directive = format!("{}={}", module, tracing::Level::from(*level));
            filter = filter.add_directive(directive.parse()
                .map_err(|e| LogError::ConfigError(format!("解析模块级别错误: {}", e)))?);
        }

        Ok(filter)
    }

    /// 创建各层组合
    fn create_layers(&self) -> Result<Box<dyn Layer<Registry> + Send + Sync>> {
        let mut layers = Vec::new();

        // 创建控制台日志层
        if self.config.console.enabled {
            layers.push(self.create_console_layer()?);
        }

        // 创建文件日志层
        if self.config.file.enabled {
            layers.push(self.create_file_layer()?);
        }

        // 合并所有层
        Ok(Box::new(layers))
    }

    /// 创建控制台日志层
    fn create_console_layer(&self) -> Result<impl Layer<Registry> + Send + Sync> {
        let timer = CustomTime;

        // 根据格式创建层
        let console_layer = match self.config.format {
            LogFormat::Json => {
                let mut layer = fmt::Layer::default()
                    .json()
                    .with_timer(timer)
                    .with_ansi(self.config.console.use_colors);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
            LogFormat::PrettyJson => {
                let mut layer = fmt::Layer::default()
                    .json()
                    .pretty()
                    .with_timer(timer)
                    .with_ansi(self.config.console.use_colors);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
            LogFormat::Compact => {
                let mut layer = fmt::Layer::default()
                    .compact()
                    .with_timer(timer)
                    .with_ansi(self.config.console.use_colors);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
            LogFormat::Text => {
                let mut layer = fmt::Layer::default()
                    .with_timer(timer)
                    .with_ansi(self.config.console.use_colors);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
        };

        Ok(console_layer)
    }

    /// 创建文件日志层
    fn create_file_layer(&self) -> Result<impl Layer<Registry> + Send + Sync> {
        // 确保日志目录存在
        if let Some(parent) = Path::new(&self.config.file.path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LogError::IoError(e))?;
            }
        }

        // 解析轮转策略
        let rotation = if self.config.file.rotation.starts_with("size:") {
            let size_str = self.config.file.rotation.strip_prefix("size:").unwrap_or("1048576");
            let bytes = size_str.parse::<u64>().unwrap_or(1048576);
            Rotation::new_with_max_size(bytes)
        } else {
            match self.config.file.rotation.as_str() {
                "hourly" => Rotation::HOURLY,
                "minutely" => Rotation::MINUTELY,
                "daily" | _ => Rotation::DAILY,
            }
        };

        // 获取目录和文件名
        let directory = Path::new(&self.config.file.path).parent().unwrap_or_else(|| Path::new("."));
        let filename = Path::new(&self.config.file.path)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("app.log");

        // 创建轮转文件写入器
        let file_appender = RollingFileAppender::new(rotation, directory, filename);

        // 使用非阻塞写入器
        let (non_blocking, _guard) = if self.config.async_logging {
            tracing_appender::non_blocking(file_appender)
        } else {
            // 非异步模式下，直接使用文件写入器
            // 注意：我们仍然用 non_blocking 包装，但会将 _guard 存储起来
            tracing_appender::non_blocking(file_appender)
        };

        // 保存 _guard 以防止过早 drop 导致日志丢失
        // 正常情况下应该在整个程序生命周期持有
        // 这里使用 Box::leak 泄漏内存，确保 _guard 的生命周期
        Box::leak(Box::new(_guard));

        let timer = CustomTime;

        // 根据格式创建层
        let file_layer = match self.config.format {
            LogFormat::Json => {
                let mut layer = fmt::Layer::default()
                    .json()
                    .with_timer(timer)
                    .with_writer(non_blocking)
                    .with_ansi(false); // 文件日志不使用ANSI颜色

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
            LogFormat::PrettyJson => {
                let mut layer = fmt::Layer::default()
                    .json()
                    .pretty()
                    .with_timer(timer)
                    .with_writer(non_blocking)
                    .with_ansi(false);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
            LogFormat::Compact => {
                let mut layer = fmt::Layer::default()
                    .compact()
                    .with_timer(timer)
                    .with_writer(non_blocking)
                    .with_ansi(false);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
            LogFormat::Text => {
                let mut layer = fmt::Layer::default()
                    .with_timer(timer)
                    .with_writer(non_blocking)
                    .with_ansi(false);

                if !self.config.show_module_path {
                    layer = layer.with_file(false);
                }

                if self.config.show_line_numbers {
                    layer = layer.with_line_number(true);
                }

                if !self.config.show_target {
                    layer = layer.with_target(false);
                }

                if self.config.show_thread_id {
                    layer = layer.with_thread_ids(true);
                }

                layer
            },
        };

        Ok(file_layer)
    }
}

/// 日志订阅器
///
/// 封装了最终构建的日志订阅器，可以被初始化
pub struct LogSubscriber {
    registry: tracing_subscriber::registry::Registry<
        tracing_subscriber::layer::Layered<
            EnvFilter,
            tracing_subscriber::layer::Layered<
                Box<dyn Layer<Registry> + Send + Sync>,
                Registry
            >
        >
    >,
}

impl LogSubscriber {
    /// 初始化日志系统
    ///
    /// 这会设置全局默认订阅器
    pub fn init(self) -> Result<()> {
        self.registry.try_init()
            .map_err(|e| LogError::InitError(format!("初始化日志系统失败: {}", e)))?;
        Ok(())
    }
}

impl Default for LogBuilder {
    fn default() -> Self {
        Self::new()
    }
}
