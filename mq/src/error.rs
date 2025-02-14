use lapin::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageQueueError {
    #[error("RabbitMQ connection error: {0}")]
    ConnectionError(String),

    #[error("Message publish error: {0}")]
    PublishError(String),

    #[error("Message consume error: {0}")]
    ConsumeError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Custom error: {0}")]
    CustomError(String),

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

}

// 泛型实现 From<Error>，支持任何类型的错误
impl<T: std::error::Error + 'static> From<T> for MessageQueueError {
    fn from(err: T) -> Self {
        // 将传入的错误包装成 MessageQueueError
        MessageQueueError::CustomError(err.to_string()) // 默认返回一个 `CustomError` 错误，可以根据需要处理
    }
}


impl From<lapin::Error> for MessageQueueError {

    fn from(err: Error) -> Self {
        match err {
            Error::ChannelsLimitReached => { MessageQueueError::RateLimitExceeded }
            // Error::InvalidProtocolVersion(_) => {}
            // Error::InvalidChannel(_) => {}
            // Error::InvalidChannelState(_) => {}
            // Error::InvalidConnectionState(_) => {}
            // Error::IOError(_) => {}
            // Error::ParsingError(_) => {}
            // Error::ProtocolError(_) => {}
            // Error::SerialisationError(_) => {}
            // Error::MissingHeartbeatError => {}
                _=> MessageQueueError::ConsumeError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for MessageQueueError {
    fn from(value: serde_json::Error) -> Self {
        MessageQueueError::ConsumeError(value.to_string())
    }
}