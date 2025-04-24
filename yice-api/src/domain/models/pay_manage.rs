use chrono::NaiveDateTime;
use sea_query::Iden;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::utils::datetime_format;

// 定义表结构
#[derive(Iden)]
pub enum TAppPayManage {
    Table,  // 对应表结构 TAppPayManage ---> t_app_pay_manage
    Id,
    Name,
    PayLogo,
    TenantId,
    PayStatus,
    PayType,
    PaySubType,
    ExtJson,
    PackageName,
    Remark,
    Sort,
    CreateTime,
    UpdateTime,
}

// 响应结构
#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct AppPayManageRecord {
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