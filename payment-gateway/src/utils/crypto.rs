use anyhow::{Result, Context};
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use base64::{Engine as _};
use rand::{rng, Rng};
use std::collections::BTreeMap;
use chrono::Utc;

// SHA-256 哈希
pub fn sha256(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let hash = hasher.finalize();
    hex::encode(hash)
}

// HMAC-SHA256 签名
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> Result<String> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)
        .context("Invalid HMAC key length")?;
    mac.update(message);
    let result = mac.finalize().into_bytes();

    Ok(hex::encode(result))
}



// 生成随机字符串
pub fn random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// 生成支付订单签名
pub fn generate_payment_signature(params: &BTreeMap<String, String>, merchant_secret: &str) -> Result<String> {
    // 按照键的字典序将参数排序
    let mut query_string = String::new();

    for (key, value) in params.iter() {
        // 跳过sign参数
        if key == "sign" {
            continue;
        }

        if !query_string.is_empty() {
            query_string.push('&');
        }

        query_string.push_str(&format!("{}={}", key, value));
    }

    // 添加商户密钥
    query_string.push_str(&format!("&key={}", merchant_secret));

    // 使用SHA256计算签名
    Ok(sha256(query_string.as_bytes()))
}

// 验证支付订单签名
pub fn verify_payment_signature(params: &BTreeMap<String, String>, merchant_secret: &str) -> Result<bool> {
    // 获取参数中的签名
    let provided_sign = match params.get("sign") {
        Some(sign) => sign,
        None => return Ok(false),
    };

    // 生成签名
    let calculated_sign = generate_payment_signature(params, merchant_secret)?;

    // 比较签名
    Ok(calculated_sign == *provided_sign)
}

// 验证时间戳是否在有效期内
pub fn verify_timestamp(timestamp: i64, valid_seconds: i64) -> bool {
    let current_time = Utc::now().timestamp();
    let time_diff = current_time - timestamp;

    // 时间戳不能是未来时间，且不能超过有效期
    time_diff >= 0 && time_diff <= valid_seconds
}

// 生成随机的32字节密钥（适用于AES-256）
pub fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rng().fill(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let input = b"hello world";
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(sha256(input), expected);
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret";
        let message = b"hello world";
        let result = hmac_sha256(key, message).unwrap();
        let expected = "734cc62f32841568f45715aeb9f4d7891324e6d948e4c6c60c0621cdac48623a";
        assert_eq!(result, expected);
    }


    #[test]
    fn test_payment_signature() {
        let mut params = BTreeMap::new();
        params.insert("merchant_id".to_string(), "M2023001".to_string());
        params.insert("amount".to_string(), "100.00".to_string());
        params.insert("order_id".to_string(), "ORDER123".to_string());

        let merchant_secret = "test_secret";

        let signature = generate_payment_signature(&params, merchant_secret).unwrap();
        params.insert("sign".to_string(), signature.clone());

        assert!(verify_payment_signature(&params, merchant_secret).unwrap());

        // 修改参数后，签名应该失效
        let mut tampered_params = params.clone();
        tampered_params.insert("amount".to_string(), "200.00".to_string());
        assert!(!verify_payment_signature(&tampered_params, merchant_secret).unwrap());
    }
}
