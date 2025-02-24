use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// XingpanDTO 结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct XingpanDTO {
    pub longitude: f64,
    pub latitude: f64,
    pub time_zone: String,
    pub birthday: String,   // "yyyy-MM-dd HH:mm"
    pub transit_day: String,
    pub sex: String,
}


/// API 响应结构
#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub data: serde_json::Value,  // 兼容不同数据结构
}

/// API 客户端
pub struct XinPanClient {
    client: Client,
    base_url: String,
    access_token: String,
}

impl XinPanClient {
    pub fn new(access_token: String, base_url: String) -> Self {
        XinPanClient {
            client: Client::new(),
            base_url,
            access_token,
        }
    }

    /// 发送 POST 请求
    async fn post(&self, endpoint: &str, params: serde_json::Value) -> Result<ApiResponse, Box<dyn Error>> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self.client.post(&url)
            .json(&params)
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;

        Ok(response)
    }

    /// 获取年运语料
    pub async fn luck_year(&self, xingpan: &XingpanDTO) -> Result<ApiResponse, Box<dyn Error>> {
        let params = json!({
            "access_token": self.access_token,
            "longitude": xingpan.longitude,
            "latitude": xingpan.latitude,
            "tz": xingpan.time_zone,
            "birthday": xingpan.birthday,
            "transitday": xingpan.transit_day
        });

        self.post("luck/year", params).await
    }

    /// 获取八字信息
    pub async fn get_eight_char(&self, xingpan: &XingpanDTO) -> Result<ApiResponse, Box<dyn Error>> {
        let date = DateTime::parse_from_str(&xingpan.birthday, "%Y-%m-%d %H:%M")?
            .with_timezone(&Utc);

        let params = json!({
            "access_token": self.access_token,
            "year": date.format("%Y").to_string(),
            "moth": date.format("%m").to_string(),
            "day": date.format("%d").to_string(),
            "hour": date.format("%H").to_string(),
            "minute": date.format("%M").to_string(),
            "second": "00",
            "gender": xingpan.sex
        });

        self.post("eightchar/get", params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_eight_char() {

        let xinpan_client = XinPanClient::new(
            "http://yong.xingpan.vip/corpus/".to_string(),
            "53b9dd693d75d16525480ebe1036ea35".to_string(),
        );

        let xingpan = XingpanDTO {
            longitude: 121.4737,
            latitude: 31.2304,
            time_zone: "Asia/Shanghai".to_string(),
            birthday: "1990-01-01 08:30".to_string(),
            transit_day: "2025-01-01".to_string(),
            sex: "male".to_string(),
        };

        // 获取年运语料
        match xinpan_client.luck_year(&xingpan).await {
            Ok(response) => println!("Luck Year Response: {:?}", response),
            Err(err) => eprintln!("Error fetching luck year: {}", err),
        }

        // 获取八字信息
        match xinpan_client.get_eight_char(&xingpan).await {
            Ok(response) => println!("Eight Char Response: {:?}", response),
            Err(err) => eprintln!("Error fetching eight char: {}", err),
        }
    }
}