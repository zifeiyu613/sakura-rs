use chrono::prelude::*;
// use sea_orm::entity::prelude::*;
use serde::{ Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, FromRow)]
pub struct UserMain {

    pub uid: i64,
    pub password: Option<String>,
    pub mobile: Option<String>,
    pub fid: i64,
    pub status: i8,
    pub tm_login: chrono::DateTime<Utc>,
    pub tm_reg: NaiveDateTime,
    pub token: Option<String>,
    pub token_validity_time: Option<DateTime<Utc>>,
    pub im_token: Option<String>,

    // 数据库字段
    #[serde(rename="type")]
    #[sqlx(rename="type")]
    pub rs_type: u8,
    pub subtype: i8,
    pub source: i8,

    #[serde(rename="inviteCode")]
    pub invite_code: Option<i32>,
    pub login_type: i8,
    pub wx_unionid: Option<String>,
    pub wx_openid: Option<String>,
    pub app_version: Option<String>,
    pub last_open_time: i64,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub city_code: Option<String>,
    pub location: Option<String>,
    pub application: Option<String>,
    pub channel: Option<String>,
    pub sub_channel: Option<String>,
    pub reg_version: Option<String>,
    pub device_code: Option<String>,
    pub new_device_code: Option<String>,
    pub mobile_login: Option<String>,
    pub register_progress: Option<i32>,
    pub current_version: Option<String>,
    pub current_source: i8,
    pub device_code_login: Option<String>, // 设备号（账户号）
    pub source_desc: Option<String>, // 注册来源描述
    pub source_type: Option<i8>,     // 注册来源类型
    pub lcountry: Option<String>,    // 国家
    pub lprovince: Option<String>,   // 省份

    // 城市
    pub lcity: Option<String>,
    pub larea: Option<String>,

    // 最新IP
    pub ip: Option<String>,
    pub mm131_id: Option<String>,
    pub app_id: Option<String>,

    // 设备型号
    pub device_type: Option<String>,
}


// impl From<i64> for UserMain {
//     fn from(uid: i64) -> Self {
//         UserMain {
//             uid: uid as u64, // 手动转换 i64 为 u64
//         }
//     }
// }