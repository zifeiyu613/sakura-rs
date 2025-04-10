use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::utils::datetime_format;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct PayManageList {
    pub id: i64,
    pub name: String,
    #[serde(rename = "payLogo")]
    pub pay_logo: Option<String>,

    #[serde(rename = "payType")]
    pub pay_type: i8,

    #[serde(rename = "paySubType")]
    pub pay_sub_type: i16,

    #[serde(rename = "packageName")]
    pub package_name: Option<String>,

    #[serde(rename = "extJson")]
    ext_json: Option<String>,

    #[serde(rename = "createTime", default, with = "datetime_format")]
    create_time: NaiveDateTime,

}