//! # R-Database
//!
//! 数据库连接池管理库，与myconfig配置库配合使用，
//! 提供便捷的数据库连接功能，支持多数据源和常见Web框架集成。
//!
//! ## 特性
//!
//! - 支持多种数据库（MySQL, PostgreSQL, SQLite）
//! - 直接从rconfig配置创建连接池
//! - 支持多数据源管理
//! - 便捷的查询和事务API
//!
//! ## 示例
//!
//! ```rust
//! use rdatabase::{DbPool, PoolOptions};
//! use rconfig::AppConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 加载配置
//!     let config = AppConfig::new()
//!         .add_default("config/default")
//!         .build()?;
//!
//!     // 创建数据库连接池
//!     let pool = DbPool::from_config(&config, None).await?;
//!
//!     // 查询示例
//!     let users = sqlx::query!("SELECT id, name FROM users LIMIT 10")
//!         .fetch_all(pool.conn())
//!         .await?;
//!
//!     println!("查询到 {} 个用户", users.len());
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod pool;
pub mod query;


mod macros;

// 主要类型重导出
pub use pool::{DbPool, PoolOptions, DbType};
pub use error::{DbError, Result};


// 方便使用的类型别名
/// MySQL连接池类型别名
#[cfg(feature = "mysql")]
pub type MySqlPool = sqlx::MySqlPool;

/// PostgreSQL连接池类型别名
#[cfg(feature = "postgres")]
pub type PgPool = sqlx::PgPool;

/// SQLite连接池类型别名
#[cfg(feature = "sqlite")]
pub type SqlitePool = sqlx::SqlitePool;

/// 通用数据库连接池类型
pub type AnyPool = sqlx::Pool<sqlx::Any>;
