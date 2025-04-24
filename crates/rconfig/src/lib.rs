//! 配置管理包，提供预设服务配置结构和简便的配置加载方法
//!
//! # 示例
//! ```
//! use rconfig::AppConfig;
//!
//! let config = AppConfig::new()
//!     .add_default("config/default")
//!     .add_environment()
//!     .build()
//!     .unwrap();
//!     
//! let server = config.server();
//! println!("服务器运行在: {}:{}", server.host, server.port);
//! ```

pub mod error;
pub mod config;
pub mod presets;
pub mod extension;

pub use config::AppConfig;
pub use error::ConfigError;

// 重导出常用预设，方便使用
pub use presets::server::ServerConfig;
pub use presets::database::DatabaseConfig;
pub use presets::redis::RedisConfig;
pub use presets::rabbitmq::RabbitMqConfig;
pub use presets::logging::LogConfig;
