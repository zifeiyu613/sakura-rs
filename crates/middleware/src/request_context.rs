use std::collections::HashMap;
use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestData {

    pub data: String, // data 可能是 JSON 字符串，也可能是加密的密文

    #[serde(rename = "plainText")]
    pub plain_text: Option<bool>,

    #[serde(skip)]
    pub files: HashMap<String, web::Bytes>,

    // pub fields: Option<HashMap<String, String>>,

}


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RequestContext {
    pub trace_id: String,
    pub user_id: Option<String>,
    pub token: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_data: Option<String>,  // 新增字段
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        }
    }
}