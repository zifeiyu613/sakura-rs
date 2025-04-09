use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct PayManageList {
    id: i64,
    name: String,
    #[serde(rename = "payLogo")]
    pay_logo: Option<String>,
    #[serde(rename = "paySubType")]
    pay_sub_type: u8,
    #[serde(rename = "payType")]
    pay_type: u8,
}