use serde::{Deserialize, Serialize};
use sqlx::Type;


/// 资源状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TINYINT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    /// 开启状态
    Open = 1,
    /// 关闭状态
    Closed = 2,
    /// 待处理状态
    Pending = 3,
    /// 已删除状态
    Deleted = 4,
}

impl State {

    pub fn is_open(&self) -> bool {
        matches!(self, State::Open)
    }
    pub fn is_closed(&self) -> bool {
        matches!(self, State::Closed)
    }

}


impl From<State> for i8 {
    fn from(state: State) -> i8 {
        state as i8
    }
}