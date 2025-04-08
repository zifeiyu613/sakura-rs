use crate::server::AppState;
use axum::http::header::CONTENT_TYPE;
use axum::{
    body::{Body, Bytes},
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crypto_utils::prelude::des_decrypt_string;
use serde::Deserialize;
use serde_json::Value;
use tracing::log::{info, warn, debug};

// 定义 AES 解密所需的密钥和 IV
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
#[derive(Clone)]
pub struct RequestData {
    pub original_body: Bytes,
    pub processed_body: Option<String>,  // 解密后或处理后的数据
    pub is_decrypted: bool,              // 标识数据是否已解密
    pub json_data: Option<Value>,        // 解析后的JSON
}

pub async fn decrypt(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode>

{
    // 检查内容类型
    let is_form = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .contains("application/x-www-form-urlencoded");

    info!("Received data: {:?}", request);

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
    };

    // 处理表单数据并解密
    if is_form {
        // 解析表单数据
        if let Ok(form) = serde_urlencoded::from_bytes::<RequestForm>(&bytes) {
            let mut processed_data = form.data.clone();
            let mut is_decrypted = false;

            // 尝试解密
            let iv = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF_u8];
            let crypto_config = CryptoConfig::new("spef11kg".to_string(), iv);

            if let Ok(decrypted) = decrypt_data(&form.data, &crypto_config) {
                processed_data = decrypted;
                is_decrypted = true;
                debug!("Successfully decrypted request data");
            } else {
                debug!("Decryption failed, treating data as plaintext");
            }

            // 保存处理后的数据
            request_data.processed_body = Some(processed_data.clone());
            request_data.is_decrypted = is_decrypted;

            // 尝试解析 JSON
            if let Ok(json) = serde_json::from_str::<Value>(&processed_data) {
                request_data.json_data = Some(json.clone());

                // 重建请求，插入解析后的 JSON 数据
                let mut new_request = Request::from_parts(parts, Body::empty());
                new_request.extensions_mut().insert(json);
                new_request.extensions_mut().insert(request_data);

                return Ok(next.run(new_request).await);
            } else {
                warn!("Failed to parse decrypted/plaintext data as JSON");
                return Err(StatusCode::BAD_REQUEST);
            }
        } else {
            warn!("Failed to parse form data");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 对于非表单请求，直接传递
    let mut new_request = Request::from_parts(parts, Body::from(bytes));
    new_request.extensions_mut().insert(request_data);

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
