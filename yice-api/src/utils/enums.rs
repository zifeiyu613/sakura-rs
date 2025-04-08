use sqlx::{Decode, Encode, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type )]
#[sqlx(type_name = "TINYINT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Open = 1,
    Closed = 2,
    Deleted = 3,
}

impl State {
    pub fn is_open(&self) -> bool {
        matches!(self, State::Open)
    }
    pub fn is_closed(&self) -> bool {
        matches!(self, State::Closed)
    }

}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbStatus {
    Open = 1,
    Closed = 2,
    Deleted = 3,
}

impl DbStatus {
    // 从整数创建枚举
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Open),
            2 => Some(Self::Closed),
            3 => Some(Self::Deleted),
            _ => None,
        }
    }

    // 获取枚举对应的整数值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}