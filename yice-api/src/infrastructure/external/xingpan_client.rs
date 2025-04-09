use crate::errors::error::YiceError;
use chrono::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::ops::Add;
use crate::status::BusinessCode;

const URL: &str = "http://yong.xingpan.vip/corpus/";
const ACCESS_TOKEN: &str = "53b9dd693d75d16525480ebe1036ea35";

#[derive(Debug, Serialize, Deserialize)]
struct XingpanDTO {
    pub birthday: String,
    pub sex: i32,
    pub longitude: f64,
    pub latitude: f64,
    pub time_zone: String,
    pub transit_day: String,
}

#[derive(Debug, Deserialize)]
pub struct XingpanResponse {
    pub code: u32,
    pub msg: String,
    pub data: Value,
}

async fn post(
    endpoint: &str,
    params: HashMap<&str, &str>,
) -> Result<Option<String>, YiceError> {
    let url = format!("{}{}", URL, endpoint);
    let client = Client::new();
    let response = client.post(&url).form(&params).send().await?;

    // 检查响应状态码是否成功，并获取响应体
    if response.status().is_success() {
        let body = response.text().await?;
        Ok(Some(body))
    } else {
        Err(YiceError::business_with_message(BusinessCode::ThirdPartyServiceError, format!("请求状态码异常: {:?}", response)))
    }
}

/// 获取年运语料
pub async fn luck_year(xingpan: &XingpanDTO) -> Result<XingpanResponse, YiceError> {
    // 创建一个 HashMap 来存储表单数据
    let mut params = HashMap::new();

    let lo = xingpan.longitude.to_string();
    let la = xingpan.latitude.to_string();

    let birthday = xingpan.birthday.clone().add(":00");

    params.insert("access_token", ACCESS_TOKEN);
    params.insert("longitude", &lo);
    params.insert("latitude", &la);
    params.insert("tz", xingpan.time_zone.as_str());
    params.insert("birthday", birthday.as_str());
    params.insert("transitday", xingpan.transit_day.as_str());

    let res = post("luck/year", params).await;

    match res {
        Ok(res) => parse_data(res),
        Err(err) => Err(err),
    }
}

/// 八字基础信息
pub async fn get_eight_char(xingpan: &XingpanDTO) -> Result<XingpanResponse, YiceError> {
    let path = "eightchar/get";

    // 解析生日字符串
    let date = NaiveDateTime::parse_from_str(&xingpan.birthday, "%Y-%m-%d %H:%M")
        .map_err(|e| YiceError::DateParseError(e))?;

    // 准备请求参数
    let mut params = HashMap::new();
    params.insert("access_token", ACCESS_TOKEN);
    // 使用 let 绑定将格式化后的日期存储在变量中，避免临时值被销毁
    // 使用元组来存储所有格式化的日期部分
    let (year, month, day, hour, minute) = (
        date.format("%Y").to_string(),
        date.format("%m").to_string(),
        date.format("%d").to_string(),
        date.format("%H").to_string(),
        date.format("%M").to_string(),
    );
    let gender = xingpan.sex.to_string();

    params.insert("year", &year);
    params.insert("moth", &month);
    params.insert("day", &day);
    params.insert("hour", &hour);
    params.insert("minute", &minute);
    params.insert("second", "00");
    params.insert("gender", &gender);

    let res = post(path, params).await;

    match res {
        Ok(res) => parse_data(res),
        Err(err) => Err(err),
    }
}

pub fn parse_data(res: Option<String>) -> Result<XingpanResponse, YiceError> {
    match res {
        None => Ok(XingpanResponse {
            code: 0,
            msg: "".to_string(),
            data: Default::default(),
        }),
        Some(body) => Ok(serde_json::from_str::<XingpanResponse>(&body)
            .map_err(YiceError::DataParseError)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_xingpan_client() {
        let xingpan = XingpanDTO {
            longitude: 121.4737,
            latitude: 31.2304,
            time_zone: "8".to_string(),
            birthday: "2000-01-01 12:30:00".to_string(),
            transit_day: "1990-01-01 08:30:12".to_string(),
            sex: 1,
        };

        match get_eight_char(&xingpan).await {
            Ok(result) => println!("八字结果: {:?}", result),
            Err(e) => eprintln!("发生错误: {}", e),
        }

        match luck_year(&xingpan).await {
            Ok(result) => println!("年运结果: {:?}", result),
            Err(e) => eprintln!("发生错误: {}", e),
        }
    }
}
