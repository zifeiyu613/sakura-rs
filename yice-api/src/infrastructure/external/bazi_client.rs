use crate::error::YiceError;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha1::Sha1;
use std::collections::HashMap;
use url::Url;

const BZ_URL: &str = "https://ap-guangzhou.cloudmarket-apigw.com/services-4mq5lolqo/bazi";

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

    fn get_authorization(&self) -> Result<String, YiceError> {
        let datetime = Self::get_datetime();
        let sign_str = format!("x-date: {}", datetime);

        let mut mac = Hmac::<Sha1>::new_from_slice(self.secret_key.as_bytes())
            .map_err(|_| YiceError::HmacError)?;
        mac.update(sign_str.as_bytes());

        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        Ok(serde_json::json!({
            "id": self.secret_id,
            "x-date": datetime,
            "signature": signature
        })
        .to_string())
    }

    pub async fn get_bazi_content(
        &self,
        xing: &str,
        ming: &str,
        sex: &str,
        birthday: &str,
        year_type: i32,
    ) -> Result<String, YiceError> {
        // 解析日期
        let date = chrono::NaiveDateTime::parse_from_str(birthday, "%Y-%m-%d %H:%M")
            .map_err(|e| YiceError::Custom(e.to_string()))?;

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
        let mut url = Url::parse(BZ_URL)?;
        url.set_query(Some(&serde_urlencoded::to_string(&query_params)?));

        // 获取授权信息
        let authorization = self.get_authorization()?;

        // 发送请求
        let response = self
            .client
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_bazi_client() {
        let client = BaziClient::new(
            "KnK5mZKIHnT5lZPg".to_string(),
            "w3vWoRYczvLh9r9Ae5W1pmkjTcvN2o5G".to_string(),
        );

        let result = client
            .get_bazi_content("王", "小明", "1", "1990-01-01 12:00", 1)
            .await
            .unwrap();

        println!("Bazi Content: {}", result);
    }
}
