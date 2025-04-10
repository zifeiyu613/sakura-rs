use serde::{Deserialize, Serialize};


// 通用DTO提取器 - 根据字段名自动提取所需DTO
pub struct RequestDto<T: for<'de> Deserialize<'de>> {
    // 提取出的DTO
    pub inner: Option<T>,
    // 基础请求字段
    pub base: Option<BaseRequestFields>,
}


// 基础请求字段结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRequestFields {
    application: Option<String>,
    channel: Option<String>,
    #[serde(rename = "deviceCode")]
    device_code: Option<String>,
    #[serde(rename = "packageName")]
    package_name: Option<String>,

    #[serde(rename = "packageVersion")]
    source: Option<u8>,

    #[serde(rename = "subChannel")]
    sub_channel: Option<String>,

    uid: u64,

    #[serde(rename = "plainText")]
    plain_text: Option<bool>,
}


// 基础字段的helper方法
impl BaseRequestFields {
    pub fn uid(&self) -> u64 {
        self.uid
    }

    pub fn application(&self) -> &str {
        self.application.as_deref().unwrap_or("")
    }

    pub fn channel(&self) -> &str {
        self.channel.as_deref().unwrap_or("")
    }

    pub fn sub_channel(&self) -> &str {
        &self.sub_channel.as_deref().unwrap_or("")
    }

    pub fn source(&self) -> Option<u8> {
        self.source
    }

    pub fn device_code(&self) -> &str {
        &self.device_code.as_deref().unwrap_or("")
    }

    pub fn package_name(&self) -> &str {
        &self.package_name.as_deref().unwrap_or("")
    }

    pub fn plain_text(&self) -> bool {
        self.plain_text.unwrap_or(false)
    }

}


/// 订单
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDTO {

    pub pay_type: Option<u16>,
    pub pay_subtype: Option<u16>,

}


/// 用户
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoDTO {

    pub username: Option<String>,

    #[serde(rename = "tarUid")]
    pub tar_uid: Option<u64>,
}



// 带类型的完整请求
// #[derive(Debug, Serialize, Deserialize)]
// pub struct TypedRequest<T> {
//     #[serde(flatten)]
//     pub base: BaseRequestFields,
//     #[serde(flatten)]
//     pub params: T,
// }
//
// // 完全动态的请求模型
// #[derive(Debug, Serialize, Deserialize)]
// pub struct DynamicRequest {
//     #[serde(flatten)]
//     pub base: BaseRequestFields,
//     #[serde(flatten)]
//     pub extra: HashMap<String, Value>,
// }


