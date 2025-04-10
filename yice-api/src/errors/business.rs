//! 业务状态码定义

/// 业务状态码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BusinessCode {
    // 成功响应: 0
    Success = 0,

    // 通用错误: 1000-1999
    UnknownError = 1000,
    ValidationError = 1001,
    Unauthorized = 1002,
    Forbidden = 1003,
    ResourceNotFound = 1004,
    DuplicateResource = 1005,
    ServiceUnavailable = 1006,
    RequestTimeout = 1007,
    InvalidLength = 1008,

    // 用户相关错误: 2000-2999
    UserNotFound = 2000,
    InvalidCredentials = 2001,
    AccountLocked = 2002,
    TokenExpired = 2003,
    InvalidToken = 2004,

    // 订单相关错误: 3000-3999
    OrderNotFound = 3000,
    OrderStatusInvalid = 3001,
    PaymentFailed = 3002,
    RefundFailed = 3003,
    PriceChanged = 3004,
    ProductUnavailable = 3005,

    // 数据库错误: 4000-4999
    DatabaseError = 4000,
    TransactionFailed = 4001,
    QueryFailed = 4002,

    // 外部服务错误: 5000-5999
    ExternalApiError = 5000,
    ThirdPartyServiceError = 5001,
    NetworkError = 5002,
    RedisError = 5003,
    MessageQueueError = 5004,
    ConfigError = 5005,
    IOError = 5006,
    InternalError = 5007,
    BadRequest = 5008,
    ParseError = 5009,
}

impl BusinessCode {
    /// 获取状态码数值
    pub fn value(&self) -> i32 {
        *self as i32
    }

    /// 获取状态码的默认错误消息
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::Success => "操作成功",
            Self::UnknownError => "未知错误",
            Self::ValidationError => "参数验证失败",
            Self::Unauthorized => "未授权的操作",
            Self::Forbidden => "禁止的操作",
            Self::ResourceNotFound => "资源不存在",
            Self::DuplicateResource => "资源已存在",
            Self::ServiceUnavailable => "服务不可用",
            Self::RequestTimeout => "请求超时",
            Self::InvalidLength => "无效长度",

            Self::UserNotFound => "用户不存在",
            Self::InvalidCredentials => "用户名或密码错误",
            Self::AccountLocked => "账户已锁定",
            Self::TokenExpired => "令牌已过期",
            Self::InvalidToken => "无效的令牌",

            Self::OrderNotFound => "订单不存在",
            Self::OrderStatusInvalid => "订单状态不允许此操作",
            Self::PaymentFailed => "支付失败",
            Self::RefundFailed => "退款失败",
            Self::PriceChanged => "价格已变更",
            Self::ProductUnavailable => "商品不可用",

            Self::DatabaseError => "数据库操作错误",
            Self::TransactionFailed => "事务处理失败",
            Self::QueryFailed => "查询执行失败",

            Self::ExternalApiError => "外部接口调用失败",
            Self::ThirdPartyServiceError => "第三方服务异常",
            Self::NetworkError => "网络连接错误",
            Self::RedisError => "Redis连接错误",
            Self::MessageQueueError => "Rabbit连接错误",
            Self::ConfigError => "配置错误",
            Self::IOError => "IO错误",
            Self::InternalError => "网络错误",
            Self::BadRequest => "请求错误",
            Self::ParseError => "解析错误",
        }
    }
}