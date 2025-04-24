use base64::encode;
use chrono::{DateTime, Datelike, Timelike, Utc};
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use sha1::Sha1;
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;

/// 生成授权信息
fn get_authorization(secret_id: &str, secret_key: &str) -> Result<String, Box<dyn Error>> {
    let datetime = get_datetime()?;
    let sign_str = format!("x-date: {}", datetime);

    // 计算 HMAC-SHA1 签名
    let mut mac = Hmac::<Sha1>::new_from_slice(secret_key.as_bytes())?;
    mac.update(sign_str.as_bytes());
    let sign = encode(&mac.finalize().into_bytes());

    Ok(format!(
        r#"{{"id":"{}", "x-date":"{}", "signature":"{}"}}"#,
        secret_id, datetime, sign
    ))
}

/// 获取当前 UTC 时间，格式为: "EEE, dd MMM yyyy HH:mm:ss 'GMT'"
fn get_datetime() -> Result<String, Box<dyn Error>> {
    let datetime: DateTime<Utc> = Utc::now();
    Ok(datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
}

/// 获取八字文案
#[derive(Serialize)]
struct BaziQueryParams {
    year: String,
    month: String,
    day: String,
    hour: String,
    minute: String,
    xing: String,
    ming: String,
    sex: String,
    #[serde(rename = "yearType")]
    year_type: String,
}

async fn get_bazi_content(
    secret_id: &str,
    secret_key: &str,
    xing: &str,
    ming: &str,
    sex: &str,
    birthday: &str,
    year_type: u32,
) -> Result<String, Box<dyn Error>> {
    // 获取授权信息
    let authorization = get_authorization(secret_id, secret_key)?;

    // 解析生日时间并格式化
    let datetime = chrono::NaiveDateTime::parse_from_str(birthday, "%Y-%m-%d %H:%M")?;
    let year = datetime.year().to_string();
    let month = format!("{:02}", datetime.month());
    let day = format!("{:02}", datetime.day());
    let hour = format!("{:02}", datetime.hour());
    let minute = format!("{:02}", datetime.minute());

    // 构建请求参数
    let query_params = BaziQueryParams {
        year,
        month,
        day,
        hour,
        minute,
        xing: xing.to_string(),
        ming: ming.to_string(),
        sex: sex.to_string(),
        year_type: year_type.to_string(),
    };

    // 编码查询参数
    let query_string = serde_urlencoded::to_string(&query_params)?;

    // 请求的 URL
    let url = format!(
        "https://ap-guangzhou.cloudmarket-apigw.com/services-4mq5lolqo/bazi?{}",
        query_string
    );

    // 请求头
    let mut headers = HeaderMap::new();
    headers.insert("request-id", HeaderValue::from_str(&Uuid::new_v4().to_string())?);
    headers.insert("Authorization", HeaderValue::from_str(&authorization)?);

    // 发起请求
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .headers(headers)
        .timeout(Duration::new(5, 0))
        .send()
        .await?;

    let result = res.text().await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_bazi_content() {
        // 示例参数
        let secret_id = "KnK5mZKIHnT5lZPg";
        let secret_key = "w3vWoRYczvLh9r9Ae5W1pmkjTcvN2o5G";
        let xing = "林";
        let ming = "三";
        let sex = "1"; // 男性
        let birthday = "1990-01-01 08:30";
        let year_type = 1; // 阳历

        match get_bazi_content(secret_id, secret_key, xing, ming, sex, birthday, year_type).await {
            Ok(content) => println!("Bazi content: {}", content),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

