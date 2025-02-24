use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc, TimeZone};
use hmac::{Hmac, Mac, NewMac};
use reqwest::{header, Client};
use sha1::Sha1;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum BaziError {
    #[error("HTTP request error")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parse error")]
    UrlError(#[from] url::ParseError),

    #[error("HMAC error")]
    HmacError,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

struct BaziClient {
    secret_id: String,
    secret_key: String,
    client: Client,
}

impl BaziClient {
    pub fn new(secret_id: String, secret_key: String) -> Self {
        Self {
            secret_id,
            secret_key,
            client: Client::new(),
        }
    }

    fn get_datetime() -> String {
        Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }

    fn get_authorization(&self) -> Result<String, BaziError> {
        let datetime = Self::get_datetime();
        let sign_str = format!("x-date: {}", datetime);

        let mut mac = Hmac::<Sha1>::new_varkey(self.secret_key.as_bytes())
            .map_err(|_| BaziError::HmacError)?;
        mac.update(sign_str.as_bytes());

        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        Ok(serde_json::json!({
            "id": self.secret_id,
            "x-date": datetime,
            "signature": signature
        }).to_string())
    }

    pub async fn get_bazi_content(
        &self,
        xing: &str,
        ming: &str,
        sex: &str,
        birthday: &str,
        year_type: i32
    ) -> Result<String, BaziError> {
        // 解析日期
        let date = chrono::NaiveDateTime::parse_from_str(
            birthday,
            "%Y-%m-%d %H:%M"
        ).map_err(|e| BaziError::Unknown(e.to_string()))?;

        // 构建查询参数
        let mut query_params = HashMap::new();
        query_params.insert("year", date.format("%Y").to_string());
        query_params.insert("month", date.format("%m").to_string());
        query_params.insert("day", date.format("%d").to_string());
        query_params.insert("hour", date.format("%H").to_string());
        query_params.insert("minute", date.format("%M").to_string());
        query_params.insert("xing", xing.to_string());
        query_params.insert("ming", ming.to_string());
        query_params.insert("sex", sex.to_string());
        query_params.insert("yearType", year_type.to_string());

        // 构建 URL
        let mut url = Url::parse("https://ap-guangzhou.cloudmarket-apigw.com/services-4mq5lolqo/bazi")?;
        url.set_query(Some(&serde_urlencoded::to_string(&query_params)?));

        // 获取授权信息
        let authorization = self.get_authorization()?;

        // 发送请求
        let response = self.client
            .get(url.as_str())
            .header("request-id", uuid::Uuid::new_v4().to_string())
            .header("Authorization", authorization)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}

// 使用示例
#[tokio::main]
async fn main() -> Result<(), BaziError> {
    let client = BaziClient::new(
        "your_secret_id".to_string(),
        "your_secret_key".to_string()
    );

    let result = client.get_bazi_content(
        "王",
        "小明",
        "1",
        "1990-01-01 12:00",
        1
    ).await?;

    println!("Bazi Content: {}", result);

    Ok(())
}