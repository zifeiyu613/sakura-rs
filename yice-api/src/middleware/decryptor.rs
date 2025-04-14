use crate::server::AppState;
use crate::utils::{ string_or_number_option};
use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::{
    Json,
    body::{Body, Bytes},
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crypto_utils::prelude::des_decrypt_string;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;
use tracing::log::{debug, info, warn};
use crate::errors::ApiError;

// 定义 DES 解密所需的密钥和 IV
#[derive(Clone)]
struct CryptoConfig {
    key: String,
    iv: [u8; 8],
}

impl CryptoConfig {
    fn new(key: String, iv: [u8; 8]) -> CryptoConfig {
        CryptoConfig { key, iv }
    }
}

#[derive(Deserialize)]
struct RequestForm {
    data: String,
    #[serde(default, rename = "plainText")]
    plain_text: Option<String>,
}

// 将请求体和元数据存储在扩展中，用于日志中间件访问
#[derive(Clone, Debug)]
pub struct RequestData {
    pub original_body: Bytes,
    pub processed_body: Option<String>, // 解密后或处理后的数据
    pub is_decrypted: bool,             // 标识数据是否已解密
    pub json_data: Option<Value>,       // 解析后的JSON
    pub content_type: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct BaseRequestFields {
    pub version: String,
    #[serde(deserialize_with = "string_or_number_option", default)]
    pub source: Option<u8>,
    #[serde(rename = "packageName")]
    pub package_name: Option<String>,
    #[serde(rename = "deviceCode")]
    pub device_code: Option<String>,
    pub platform: Option<String>,
    pub uid: Option<u64>,
    pub token: Option<String>,
    #[serde(rename = "subChannel")]
    pub sub_channel: Option<String>,
    pub network: Option<String>,

    // 使用flatten处理未知字段
    #[serde(flatten)]
    pub base_extra_fields: HashMap<String, Value>,

}

impl BaseRequestFields {
    fn parse_version(&self) -> Result<u32, String> {
        // 移除所有非数字字符
        let cleaned: String = self
            .version
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        // 尝试转换
        cleaned
            .parse::<u32>()
            .map_err(|_| format!("无法将 {} 转换为 u32", &self.version))
    }
}

pub async fn decrypt(mut request: Request, next: Next) -> Result<Response, ApiError> {
    // 获取内容类型
    let content_type = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .to_string();

    let is_form = content_type.contains("application/x-www-form-urlencoded");
    let is_json = content_type.contains("application/json");
    debug!("请求内容类型: {}", content_type);

    // 读取请求体
    let (parts, body) = request.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_else(|err| {
            tracing::warn!("Failed to buffer request body: {}", err);
            Bytes::new()
        });

    // 创建请求数据容器
    let mut request_data = RequestData {
        original_body: bytes.clone(),
        processed_body: None,
        is_decrypted: false,
        json_data: None,
        content_type: content_type.clone(),
    };

    // 处理表单数据并解密
    if is_form {
        debug!("处理表单类型请求");
        // 解析表单数据
        return if let Ok(form) = serde_urlencoded::from_bytes::<RequestForm>(&bytes) {
            let mut processed_data = form.data.clone();
            let mut is_decrypted = false;

            // 尝试解密
            let iv = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF_u8];
            let crypto_config = CryptoConfig::new("spef11kg".to_string(), iv);

            if let Ok(decrypted) = decrypt_data(&form.data, &crypto_config) {
                processed_data = decrypted;
                is_decrypted = true;
                debug!("成功解密请求数据");
            } else {
                debug!("解密失败，将数据视为明文");
            }

            // 保存处理后的数据
            request_data.processed_body = Some(processed_data.clone());
            request_data.is_decrypted = is_decrypted;

            // 尝试解析 JSON
            match serde_json::from_str::<Value>(&processed_data) {
                Ok(json) => {
                    debug!("成功将解密/明文数据解析为JSON");
                    request_data.json_data = Some(json.clone());

                    // 重建请求，插入解析后的 JSON 数据
                    let mut new_request = Request::from_parts(parts, Body::empty());
                    new_request.extensions_mut().insert(json);
                    new_request.extensions_mut().insert(request_data);

                    Ok(next.run(new_request).await)
                }
                Err(e) => {
                    warn!("无法将解密/明文数据解析为JSON: {}", e);
                    Err(e.into())
                }
            }
        } else {
            warn!("无法解析表单数据");
            Err(ApiError::UrlencodedParseError(serde_urlencoded::ser::Error::Custom("无法解析表单数据".into())))
        }
    } else if is_json || bytes.len() > 0 {
        // 处理直接提交的JSON请求或其他包含正文的请求
        debug!("处理JSON或其他非表单请求");

        // 将请求体作为字符串
        return match std::str::from_utf8(&bytes) {
            Ok(body_str) => {
                // 保存处理后的数据
                request_data.processed_body = Some(body_str.to_string());

                // 尝试解析为JSON
                match serde_json::from_str::<Value>(body_str) {
                    Ok(json) => {
                        debug!("成功解析请求体为JSON");
                        request_data.json_data = Some(json.clone());

                        // 重建请求，插入解析后的JSON数据
                        let mut new_request = Request::from_parts(parts, Body::empty());
                        new_request.extensions_mut().insert(json);
                        new_request.extensions_mut().insert(request_data);

                        Ok(next.run(new_request).await)
                    }
                    Err(e) => {
                        debug!("请求体不是有效的JSON: {}", e);
                        // 对于非JSON请求，我们仍然传递请求数据
                        let mut new_request = Request::from_parts(parts, Body::from(bytes));
                        new_request.extensions_mut().insert(request_data);
                        Ok(next.run(new_request).await)
                    }
                }
            }
            Err(e) => {
                warn!("请求体不是有效的UTF-8: {}", e);
                // 对于二进制数据，我们仍然传递请求
                let mut new_request = Request::from_parts(parts, Body::from(bytes));
                new_request.extensions_mut().insert(request_data);
                Ok(next.run(new_request).await)
            }
        }
    }

    // 处理空请求
    debug!("处理空请求体");
    let mut new_request = Request::from_parts(parts, Body::empty());
    new_request.extensions_mut().insert(request_data);

    // 为空请求添加空JSON，以保持一致性
    let empty_json = serde_json::json!({});
    new_request.extensions_mut().insert(empty_json);

    Ok(next.run(new_request).await)
}

fn decrypt_data(encrypted_data: &str, config: &CryptoConfig) -> Result<String, StatusCode> {
    // 保持原实现不变...
    let key1 = config.key.as_bytes();
    let mut key = [0u8; 8];
    key.copy_from_slice(key1);

    let iv = config.iv;

    des_decrypt_string(&key, encrypted_data, Some(iv)).map_err(|_| StatusCode::BAD_REQUEST)
}

