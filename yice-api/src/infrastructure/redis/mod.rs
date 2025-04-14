pub mod lock;
pub mod sorted_set;
pub mod serializer;
pub mod list;
pub mod client;
pub mod hash;
pub mod counter;
pub mod health;
pub mod cache;
pub mod manager;

pub use cache::*;
pub use client::*;
pub use counter::*;
pub use hash::*;
pub use health::*;
pub use list::*;
pub use lock::*;
pub use sorted_set::*;
pub use manager::*;
pub use serializer::*;


// 常用时间常量
pub mod duration {

    use std::time::Duration;

    pub const SECOND: Duration = Duration::from_secs(1);
    pub const MINUTE: Duration = Duration::from_secs(60);
    pub const HOUR: Duration = Duration::from_secs(60 * 60);     // 3600
    pub const DAY: Duration = Duration::from_secs(24 * 60 * 60);  // 86400
    pub const WEEK: Duration = Duration::from_secs(7 * 60 * 60);  // 604800
    pub const MONTH: Duration = Duration::from_secs(60 * 60 * 24);
    pub const YEAR: Duration = Duration::from_secs(365 * 24 * 60 * 60);

}


pub mod error {

    use std::fmt;
    use std::error::Error as StdError;
    use std::result::Result as StdResult;

    #[derive(Debug)]
    pub enum RedisError {
        /// Redis库错误
        Redis(redis::RedisError),
        /// 连接错误
        Connection(String),
        /// 序列化错误
        Serialization(String),
        /// 反序列化错误
        Deserialization(String),
        /// 健康检查错误
        HealthCheck(String),
        /// 配置错误
        Configuration(String),
        /// 未找到值
        NotFound,
        /// 分布式锁获取失败
        LockAcquisitionFailed(String),
        /// 分布式锁释放失败
        LockReleaseFailed(String),
        /// 其他错误
        Other(String),
    }

    impl fmt::Display for RedisError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Redis(e) => write!(f, "Redis error: {}", e),
                Self::Connection(e) => write!(f, "Redis connection error: {}", e),
                Self::Serialization(e) => write!(f, "Serialization error: {}", e),
                Self::Deserialization(e) => write!(f, "Deserialization error: {}", e),
                Self::HealthCheck(e) => write!(f, "Health check failed: {}", e),
                Self::Configuration(e) => write!(f, "Configuration error: {}", e),
                Self::NotFound => write!(f, "Value not found in cache"),
                Self::LockAcquisitionFailed(e) => write!(f, "Failed to acquire lock: {}", e),
                Self::LockReleaseFailed(e) => write!(f, "Failed to release lock: {}", e),
                Self::Other(e) => write!(f, "Other error: {}", e),
            }
        }
    }

    impl StdError for RedisError {}


    impl From<redis::RedisError> for RedisError {
        fn from(err: redis::RedisError) -> Self {
            RedisError::Redis(err)
        }
    }

    impl From<serde_json::Error> for RedisError {
        fn from(err: serde_json::Error) -> Self {
            if err.is_data() {
                RedisError::Deserialization(err.to_string())
            } else {
                RedisError::Serialization(err.to_string())
            }
        }
    }

    /// Redis操作的结果类型
    pub type Result<T> = StdResult<T, RedisError>;

}