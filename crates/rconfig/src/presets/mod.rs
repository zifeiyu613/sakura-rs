//! 预设配置模块

pub mod server;
pub mod database;
pub mod redis;
pub mod rabbitmq;
pub mod logging;

// 用于验证的共用特性
pub trait Validate {
    fn validate(&self) -> crate::error::Result<()>;
}
