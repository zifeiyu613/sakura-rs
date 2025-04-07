use crate::dto::request_dto::TypedRequest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::dto::{DtoHandler, DynamicRequest};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserInfoDTO {

    username: Option<String>,

    #[serde(rename = "tarUid")]
    tar_uid: u64,
}

impl DtoHandler for UserInfoDTO {
    type Output = UserInfoDTO;

    fn handle(&self, data: &DynamicRequest) -> Result<Self::Output, String> {
        let tar_uid = data.extra.get(self.name())
            .and_then(|v| v.get("tarUid"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "缺少tarUid字段".to_string())?;

        Ok(UserInfoDTO {
            username: Some("DTO3用户".to_string()),
            tar_uid,
        })
    }

    fn name(&self) -> &str {
        self.name()
    }
}


// ===== DTO处理器实现 =====

// UserInfoDTO处理器
// struct UserInfoDtoHandler;

// impl DtoHandler for UserInfoDtoHandler {
//     fn handle(&self, data: &DynamicRequest) -> String {
//         // 从data中提取必要信息
//         let tar_uid = data.extra.get("userInfoDTO")
//             .and_then(|v| v.get("tarUid"))
//             .and_then(|v| v.as_u64())
//             .unwrap_or(0);
//
//         format!("UserInfoDTO处理 - UID: {}, Target UID: {}", data.base.uid, tar_uid)
//     }
//
//     fn name(&self) -> &str {
//         "UserInfoDtoHandler"
//     }
// }

// 别名 简化使用
// pub(crate) type UserInfoReqDTO = TypedRequest<UserInfoDTO>;
//
//
// impl UserInfoReqDTO {
//
//     pub fn username(&self) -> String {
//         self.dto.username.clone()
//     }
//
//     pub fn tar_uid(&self) -> u64 {
//         self.dto.user_info.tar_uid
//     }
//
// }


// 默认处理器
struct DefaultDtoHandler;

impl DtoHandler for DefaultDtoHandler {
    type Output = Value;  // 动态JSON值

    fn handle(&self, data: &DynamicRequest) -> Result<Self::Output, String> {
        Ok(json!({
            "uid": data.base.uid,
            "message": "未知的请求类型",
            "available_fields": data.extra.keys().collect::<Vec<_>>(),
        }))
    }

    fn name(&self) -> &str {
        "DefaultDtoHandler"
    }
}