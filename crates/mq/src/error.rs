
use thiserror::Error;
use lapin::Error as LapinError;
use serde_json::Error as SerdeError;

#[derive(Error, Debug)]
pub enum MessageQueueError {

    #[error("RabbitMQ Error: {0}")]
    LapinError(#[from] LapinError),

    #[error("Serde JSON Error: {0}")]
    SerdeError(#[from] SerdeError),

    #[error("Generic Error: {0}")]
    GenericError(String),
}


// impl From<LapinError> for MessageQueueError {
//     fn from(error: LapinError) -> Self {
//         MessageQueueError::LapinError(error)
//     }
// }
//
// impl From<SerdeError> for MessageQueueError {
//     fn from(error: SerdeError) -> Self {
//         MessageQueueError::SerdeError(error)
//     }
// }

// // 泛型实现 From<Error>，支持任何类型的错误
// impl<T: std::error::Error + 'static> From<T> for MessageQueueError {
//     fn from(err: T) -> Self {
//         // 将传入的错误包装成 MessageQueueError
//         MessageQueueError::CustomError(err.to_string()) // 默认返回一个 `CustomError` 错误，可以根据需要处理
//     }
// }
//
// impl From<lapin::Error> for MessageQueueError {
//     fn from(err: Error) -> Self {
//         match err {
//             Error::ChannelsLimitReached => MessageQueueError::RateLimitExceeded,
//             // Error::InvalidProtocolVersion(_) => {}
//             // Error::InvalidChannel(_) => {}
//             // Error::InvalidChannelState(_) => {}
//             // Error::InvalidConnectionState(_) => {}
//             // Error::IOError(_) => {}
//             // Error::ParsingError(_) => {}
//             // Error::ProtocolError(_) => {}
//             // Error::SerialisationError(_) => {}
//             // Error::MissingHeartbeatError => {}
//             _ => MessageQueueError::ConsumeError(err.to_string()),
//         }
//     }
// }
//
// impl From<serde_json::Error> for MessageQueueError {
//     fn from(value: serde_json::Error) -> Self {
//         MessageQueueError::ConsumeError(value.to_string())
//     }
// }
