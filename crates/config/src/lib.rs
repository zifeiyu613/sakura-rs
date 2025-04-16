//! # ConfigHub
//!
//! 一个灵活、可扩展的配置管理库，提供了预设的基础配置和自定义配置能力。
//!
//! ## 特点
//!
//! - 支持多种配置源（文件、环境变量、命令行参数、远程服务）
//! - 多种文件格式（YAML, TOML, JSON）
//! - 预设常用服务组件配置
//! - 灵活的扩展机制
//! - 配置热重载
//! - 配置验证和模板功能
//! - 统一的错误处理
//!
//! ## 示例
//!
//! ```rust
//! use confighub::{AppConfig, ConfigBuilder};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 基本用法 - 默认加载
//!     let config = ConfigBuilder::default().build()?;
//!     
//!     // 访问配置
//!     println!("Service name: {}", config.service_name());
//!     
//!     Ok(())
//! }
//! ```

mod app_config;
mod builder;
mod error;
mod loader;
pub mod presets;
pub mod extension;
pub mod validation;
pub mod template;
pub mod watcher;

pub use app_config::AppConfig;
pub use builder::ConfigBuilder;
pub use error::ConfigError;
pub use loader::{ConfigLoader, EnvLoader, FileLoader, FileFormat, ArgsLoader, RemoteLoader};
pub use validation::{ConfigValidator, ValidatorChain, RequiredFieldsValidator, RangeValidator, EnvironmentValidator};
pub use template::TemplateEngine;
pub use watcher::{ConfigWatcher, ConfigChangeObserver, LoggingObserver};

/// 从所有可用的配置源加载配置
pub fn load_config() -> Result<AppConfig, ConfigError> {
    ConfigBuilder::default().build()
}

/// 验证配置是否完整
pub fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    // 创建默认验证器链
    let validator = ValidatorChain::default()
        .add(RequiredFieldsValidator::new()
            .require("service.name")
            .require("service.environment"))
        .add(EnvironmentValidator::default())
        .add(RangeValidator::new()
            .validate_range("service.port", Some(1), Some(65535)));

    validator.validate(config)
}

/// 创建带有热重载功能的配置
pub fn with_hot_reload(config: AppConfig, builder: ConfigBuilder) -> watcher::ConfigWatcherHandle {
    // 检查配置文件路径
    let file_paths = if let Some(path) = builder.default_config_path() {
        vec![std::path::PathBuf::from(path)]
    } else {
        vec![]
    };

    // 创建并启动监视器
    let watcher = ConfigWatcher::new(config, builder)
        .with_interval(std::time::Duration::from_secs(30));

    // 添加配置文件
    let watcher = file_paths.into_iter().fold(watcher, |w, path| {
        w.watch_file(path)
    });

    // 添加日志观察者
    watcher.add_observer(LoggingObserver).start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine() {
        let mut engine = TemplateEngine::default();
        engine.set_variable("NAME", "TestApp")
            .set_variable("PORT", "8080");

        let template = r#"
        service:
          name: ${NAME}
          port: ${PORT}
          environment: development
        "#;

        let processed = engine.process(template).unwrap();
        assert!(processed.contains("name: TestApp"));
        assert!(processed.contains("port: 8080"));
    }

    #[test]
    fn test_validator_chain() {
        // 创建验证器链
        let validator = ValidatorChain::default()
            .add(RequiredFieldsValidator::new()
                .require("service.name")
                .require("service.port"))
            .add(EnvironmentValidator::default())
            .add(RangeValidator::new()
                .validate_range("service.port", Some(1), Some(65535)));

        // 创建有效配置
        let valid_config = AppConfig {
            service: presets::ServiceConfig {
                name: "test-app".to_string(),
                port: 8080,
                environment: "development".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        // 验证应该通过
        assert!(validator.validate(&valid_config).is_ok());

        // 创建无效配置（空名称）
        let invalid_config = AppConfig {
            service: presets::ServiceConfig {
                name: "".to_string(),
                port: 8080,
                environment: "development".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        // 验证应该失败
        assert!(validator.validate(&invalid_config).is_err());
    }
}
