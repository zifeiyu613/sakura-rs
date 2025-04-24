//! 提供 NaiveDateTime 的自定义序列化和反序列化功能
//! 使用格式化日期时间字符串: YYYY-MM-DD HH:MM:SS

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{self, Deserialize, Deserializer, Serializer};

// 引用核心时间工具模块的格式常量
use super::datetime::formats;

/// 将 NaiveDateTime 序列化为标准格式字符串
pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.format(formats::DATETIME).to_string();
    serializer.serialize_str(&s)
}

/// 从多种可能的格式解析日期时间字符串
pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // 尝试从接受的格式列表中解析
    for format in formats::ACCEPTED_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, format) {
            return Ok(dt);
        }
    }

    // 对于仅有日期的情况，添加默认时间 (00:00:00)
    if let Ok(date) = NaiveDate::parse_from_str(&s, formats::DATE) {
        return Ok(NaiveDateTime::new(
            date,
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        ));
    }

    // 所有格式都解析失败
    Err(serde::de::Error::custom(format!(
        "日期时间字符串 '{}' 不符合任何支持的格式",
        s
    )))
}

/// 可选日期时间序列化（处理Option<NaiveDateTime>）
pub mod opt {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(opt_date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt_date {
            Some(date) => super::serialize(date, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 首先尝试反序列化为可选字符串
        let opt_str = Option::<String>::deserialize(deserializer)?;

        match opt_str {
            Some(s) if s.is_empty() => Ok(None),
            Some(s) => {
                // 尝试所有格式
                for format in formats::ACCEPTED_FORMATS {
                    if let Ok(dt) = NaiveDateTime::parse_from_str(&s, format) {
                        return Ok(Some(dt));
                    }
                }

                // 尝试仅日期
                if let Ok(date) = NaiveDate::parse_from_str(&s, formats::DATE) {
                    return Ok(Some(NaiveDateTime::new(
                        date,
                        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    )));
                }

                Err(serde::de::Error::custom(format!(
                    "可选日期时间字符串 '{}' 不符合任何支持的格式",
                    s
                )))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::datetime_format;
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct UserRecord {
        #[serde(rename = "createTime", with = "datetime_format")]
        create_time: NaiveDateTime,

        #[serde(rename = "updateTime", with = "datetime_format::opt")]
        update_time: Option<NaiveDateTime>,

        name: String,
        active: bool,
    }

    #[test]
    fn serialization_example() {
        let user = UserRecord {
            create_time: chrono::Local::now().naive_local(),
            update_time: Some(chrono::Local::now().naive_local()),
            name: "张三".to_string(),
            active: true,
        };

        // 序列化为JSON
        let json = serde_json::to_string_pretty(&user).unwrap();
        println!(
            "JSON输出:{}",
            json
        );
        // 输出时间格式为 "2023-04-10 15:30:45"

        // 反序列化
        let parsed: UserRecord = serde_json::from_str(&json).unwrap();
        println!("反序列化后: {:#?}", parsed);
    }
}
