pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;
mod utils;
mod server;

// 重新导出关键组件，便于外部调用
pub use application::service::PaymentServiceImpl;
pub use domain::models;
pub use domain::service::PaymentService;