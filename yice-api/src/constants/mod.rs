/// 常量管理模块
pub mod defaults;
pub mod limits;
pub mod settings;
pub mod enums;

// 重新导出常用常量，方便直接使用
pub use defaults::{DEFAULT_PAGE_SIZE, DEFAULT_SORT_ORDER, DEFAULT_PACKAGE_NAME};
pub use limits::{MAX_PAGE_SIZE, MAX_FILE_SIZE};
pub use enums::State;