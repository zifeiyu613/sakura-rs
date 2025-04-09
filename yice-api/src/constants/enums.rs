use std::fmt;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Type};

/// 资源状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TINYINT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    /// 开启状态
    Open,
    /// 关闭状态
    Closed,
    /// 待处理状态
    Pending,
    /// 已删除状态
    Deleted,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Open => write!(f, "open"),
            State::Closed => write!(f, "closed"),
            State::Pending => write!(f, "pending"),
            State::Deleted => write!(f, "deleted"),
        }
    }
}

impl State {

    pub fn code(&self) -> i32 {
        match self {
            State::Open => 1,
            State::Closed => 2,
            State::Pending => 3,
            State::Deleted => 4,
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self, State::Open)
    }
    pub fn is_closed(&self) -> bool {
        matches!(self, State::Closed)
    }

}






/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    /// 待支付
    Pending,
    /// 已支付
    Paid,
    /// 已取消
    Cancelled,
    /// 已退款
    Refunded,
    /// 部分退款
    PartialRefunded,
}

impl OrderStatus {
    pub fn code(&self) -> i32 {
        match self {
            OrderStatus::Pending => 0,
            OrderStatus::Paid => 1,
            OrderStatus::Cancelled => 2,
            OrderStatus::Refunded => 3,
            OrderStatus::PartialRefunded => 4,
        }
    }
}