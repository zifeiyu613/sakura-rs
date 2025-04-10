use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::middleware::extract::NestedField;
use crate::utils::string_or_number_option;
use crate::impl_nested_field;


/// 订单
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OrderDTO {

    /// 支付类型
    #[serde(rename = "payType", deserialize_with = "string_or_number_option")]
    pub pay_type: Option<u16>,

    /// 支付子类型
    #[serde(rename = "paySubType", deserialize_with = "string_or_number_option")]
    pub pay_subtype: Option<u16>,

    // 其他可能的字段，全部设为可选
    #[serde(default)]
    pub price: Option<f64>,

    #[serde(default)]
    pub product_id: Option<String>,

    // 使用flatten处理未知字段
    #[serde(flatten)]
    pub extra_fields: HashMap<String, Value>,
}


/// 用户
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserInfoDTO {

    pub username: Option<String>,

    #[serde(rename = "tarUid")]
    pub tar_uid: Option<u64>,
}


/// 实现NestedField特征
impl_nested_field!(OrderDTO, "orderDTO");
impl_nested_field!(UserInfoDTO, "userInfoDTO");



