//! 系统限制相关常量

/// 最大分页大小
pub const MAX_PAGE_SIZE: u32 = 100;

/// 最大文件上传大小(字节)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// 最大订单查询天数
pub const MAX_ORDER_QUERY_DAYS: i64 = 90;

/// 最大退款金额限制(分)
pub const MAX_REFUND_AMOUNT: i64 = 1000000; // 1万元

/// 最大批量操作数量
pub const MAX_BATCH_SIZE: usize = 100;

/// 最大请求频率(每分钟)
pub const MAX_REQUEST_RATE: u32 = 60;