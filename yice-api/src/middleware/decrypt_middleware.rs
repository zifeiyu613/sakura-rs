use axum::body::{Body, Bytes};
use crate::app::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use serde::Deserialize;
use serde_json::Value;
use tower::Layer;
use tracing::log::debug;
use crate::error::YiceError;

/// 解密中间件


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

// 解析表单数据的结构
#[derive(Deserialize)]
struct RequestForm {

    data: String,

    #[serde(default)]
    plain_text: Option<String>,
}


async fn decrypt_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode>
{
    // 处理"application/x-www-form-urlencoded"请求
    if request.headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .contains("application/x-www-form-urlencoded") {

        let (parts, body) = request.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap_or_else(|err| {
            tracing::warn!("Failed to buffer request body: {}", err);
            Bytes::new()
        });

        // 解析表单数据
        let form: RequestForm = serde_urlencoded::from_bytes(&bytes)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // 处理数据
        let processed_data = if form.plain_text.is_some() && form.plain_text.unwrap() == "true" {
            // 明文模式 - 不需要解密
            form.data
        } else {
            // 密文模式 - 进行 AES 解密
            // {(byte) 0x12, (byte) 0x34, (byte) 0x56, (byte) 0x78, (byte) 0x90, (byte) 0xAB, (byte) 0xCD, (byte) 0xEF}
            let iv = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF_u8];
            let crypto_config = CryptoConfig::new("spef11kg".to_string(), iv);

            decrypt_data(&form.data, &crypto_config)
                .map_err(|_| StatusCode::BAD_REQUEST)?
        };

        // 解析 JSON
        let json_data: Value = serde_json::from_str(&processed_data)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // 创建新的请求体，将解密后的 JSON 数据放入请求的扩展中
        let mut modified_req = Request::from_parts(parts, Body::empty());

        modified_req.extensions_mut().insert(json_data);

        // 继续处理请求
        Ok(next.run(modified_req).await)

    } else {
        // 对于非表单请求，直接传递给下一个处理器
        Ok(next.run(request).await)
    }
}

use crypto_utils::symmetric::des::des_encrypt_string;

fn decrypt_data(encrypted_data: &str, config: &CryptoConfig) -> Result<String, StatusCode> {
    let key1 = config.key.as_bytes();
    println!("key1: {:?}", key1);
    println!("key1.len: {:?}", key1.len());
    let mut key = [0u8; 8];
    key.copy_from_slice(key1);

    let iv = config.iv;

    des_encrypt_string(&key, encrypted_data, Some(iv))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

// 提取为单独的中间件层函数
// fn decrypt_layer(state: AppState) -> impl Layer<axum::routing::Route> + Clone {
//     middleware::from_fn_with_state(state, decrypt_middleware)
// }


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decrypt_data() {
        let cpiter = "0OSQhJvlfRmcbqDk2S900CCCg32hO2U+m5Gs3tYEC9ZdgTRTBNbCO8DQLujuQtnJG+3hhfuIkA84CLNPxcvw4g0UEWczPnJBxZkFUtlS+HW/bTXg1zD2xp2UR/5oXkc+3aek0ejN07Oq5J0WESiyl1SBEaPveNKRAIehfkQmb7WZMolwF2bHTUuhAyAC5d085DcXhcnjXEpbJ9hPrvPJcdvs1eLxWGZqc8A59yAxfwVLV/Kp76wALFuipzxy9tfexcNjbYvqaqLBbvH4cvYQtA==";

        let iv_bytes = [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF];
        let decrypt = decrypt_data(cpiter, &CryptoConfig::new("spef11kg".to_string(), iv_bytes)).unwrap();

        println!("{}", decrypt);
    }
}