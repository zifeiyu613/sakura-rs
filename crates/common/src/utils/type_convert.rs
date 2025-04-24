//! 提供JSON字段类型转换功能
//!
//! 这个模块允许在反序列化时接受不同类型的字段值，
//! 比如可以同时接受数字和字符串表示的数值。

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Deserializer};

/// 从字符串或数字类型反序列化为目标类型
///
/// 这个函数可以处理JSON中同一字段既可能是字符串也可能是数字的情况。
///
/// # 例子
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Request {
///     #[serde(deserialize_with = "crate::utils::type_convert::string_or_number")]
///     id: u64,
///     #[serde(deserialize_with = "crate::utils::type_convert::string_or_number")]
///     status: i32,
/// }
/// ```
///
/// # 类型参数
///
/// * `T`: 目标类型，必须实现 `FromStr` 和 `Deserialize` 特性
///
/// # 错误
///
/// 当字符串不能被解析为目标类型时，会返回错误。
pub fn string_or_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    T::Err: fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber<T> {
        String(String),
        Number(T),
    }

    match StringOrNumber::<T>::deserialize(deserializer)? {
        StringOrNumber::String(s) => {
            T::from_str(&s).map_err(|e| serde::de::Error::custom(format!("无法将字符串 '{}' 解析为数字: {}", s, e)))
        },
        StringOrNumber::Number(n) => Ok(n),
    }
}

/// 从字符串或布尔值反序列化为布尔类型
///
/// 这个函数可以处理JSON中布尔字段既可能是布尔值也可能是字符串的情况。
/// 接受的字符串值:
/// - "true", "True", "TRUE", "1", "yes", "Y", "on" 被解析为 true
/// - "false", "False", "FALSE", "0", "no", "N", "off" 被解析为 false
///
/// # 例子
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct UserSettings {
///     #[serde(deserialize_with = "crate::utils::type_convert::string_or_bool")]
///     active: bool,
/// }
/// ```
pub fn string_or_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        String(String),
        Bool(bool),
    }

    match StringOrBool::deserialize(deserializer)? {
        StringOrBool::String(s) => {
            let s = s.to_lowercase();
            match s.as_str() {
                "true" | "1" | "yes" | "y" | "on" => Ok(true),
                "false" | "0" | "no" | "n" | "off" => Ok(false),
                _ => Err(serde::de::Error::custom(format!(
                    "无法将字符串 '{}' 解析为布尔值", s
                ))),
            }
        },
        StringOrBool::Bool(b) => Ok(b),
    }
}

/// 从字符串或数字反序列化为Option<T>
///
/// 这个函数可以处理以下情况:
/// - 数字值: `"source": 5` → 解析为 `Some(5)`
/// - 字符串数字: `"source": "5"` → 解析为 `Some(5)`
/// - `null` 值: `"source": null` → 解析为 `None`
/// - 字段缺失: 没有 `source` 字段 → 解析为 `None`（需要配合 `#[serde(default)]` 属性）
/// - 空字符串: `"source": ""` → 解析为 `None`
///
/// # 例子
///
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Request {
///     #[serde(deserialize_with = "crate::utils::type_convert::string_or_number_option", default)]
///     source: Option<u8>,
/// }
/// ```
pub fn string_or_number_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    T::Err: fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumberOrNull<T> {
        String(String),
        Number(T),
        Null,
    }

    // 使用 Option 包装我们的枚举，这样可以处理 null 值
    let opt = Option::<StringOrNumberOrNull<T>>::deserialize(deserializer)?;

    match opt {
        Some(StringOrNumberOrNull::String(s)) => {
            if s.is_empty() {
                return Ok(None);
            }
            match T::from_str(&s) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(serde::de::Error::custom(format!(
                    "无法将字符串 '{}' 解析为指定类型: {}", s, e
                ))),
            }
        },
        Some(StringOrNumberOrNull::Number(n)) => Ok(Some(n)),
        Some(StringOrNumberOrNull::Null) | None => Ok(None),
    }
}

/// 从字符串或布尔值反序列化为Option<bool>
///
/// 处理可能为null或缺失的布尔值
pub fn string_or_bool_option<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBoolOrNull {
        String(String),
        Bool(bool),
        Null,
    }

    match StringOrBoolOrNull::deserialize(deserializer)? {
        StringOrBoolOrNull::String(s) => {
            if s.is_empty() {
                return Ok(None);
            }
            let s = s.to_lowercase();
            match s.as_str() {
                "true" | "1" | "yes" | "y" | "on" => Ok(Some(true)),
                "false" | "0" | "no" | "n" | "off" => Ok(Some(false)),
                _ => Err(serde::de::Error::custom(format!(
                    "无法将字符串 '{}' 解析为布尔值", s
                ))),
            }
        },
        StringOrBoolOrNull::Bool(b) => Ok(Some(b)),
        StringOrBoolOrNull::Null => Ok(None),
    }
}

/// 处理字符串与数组之间的转换
///
/// 可以将单个字符串值解析为单元素数组
///
/// # 例子
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Filter {
///     #[serde(deserialize_with = "crate::utils::type_convert::string_or_vec")]
///     tags: Vec<String>,
/// }
/// ```
pub fn string_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    T::Err: fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec<T> {
        String(String),
        Vec(Vec<T>),
    }

    match StringOrVec::<T>::deserialize(deserializer)? {
        StringOrVec::String(s) => {
            if s.is_empty() {
                return Ok(vec![]);
            }
            match T::from_str(&s) {
                Ok(value) => Ok(vec![value]),
                Err(e) => Err(serde::de::Error::custom(format!(
                    "无法将字符串 '{}' 解析为指定类型: {}", s, e
                ))),
            }
        },
        StringOrVec::Vec(v) => Ok(v),
    }
}

/// 处理可能为空字符串的Option字段
///
/// 当字符串为空时返回None
///
/// # 例子
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct User {
///     #[serde(deserialize_with = "crate::utils::type_convert::empty_string_as_none")]
///     nickname: Option<String>,
/// }
/// ```
pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.filter(|s| !s.is_empty()))
}

// ------------------------------ 以下可直接调用 见test_convert ---------------------------------------

/// 将字符串转换为数字类型的辅助函数
///
/// 这个函数可以直接调用，不依赖于反序列化过程。
///
/// # 例子
///
/// ```rust
///
/// // 转换字符串为u32
/// use common::{convert_to_number, convert_to_number_option, convert_option_to_number_option};
///
/// let value1 = convert_to_number::<u32>("123").unwrap();
/// assert_eq!(value1, 123);
///
/// // 转换空字符串 (返回None)
/// let value2 = convert_to_number_option::<i32>("").unwrap();
/// assert_eq!(value2, None);
///
/// // 处理已有的Option
/// let value3 = convert_option_to_number_option::<u8>(Some("42")).unwrap();
/// assert_eq!(value3, Some(42));
/// ```
pub fn convert_to_number<T>(value: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    value.parse::<T>().map_err(|e| format!("无法将字符串 '{}' 解析为数字: {}", value, e))
}

/// 将字符串转换为可选数字类型的辅助函数
///
/// 返回:
/// - 有效数字字符串 -> Some(数字)
/// - 空字符串 -> None
/// - 无效数字字符串 -> 错误
pub fn convert_to_number_option<T>(value: &str) -> Result<Option<T>, String>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    if value.is_empty() {
        return Ok(None);
    }

    value.parse::<T>()
        .map(Some)
        .map_err(|e| format!("无法将字符串 '{}' 解析为数字: {}", value, e))
}

/// 处理可能为None的字符串，转换为数字类型
///
/// 返回:
/// - None -> None
/// - Some(空字符串) -> None
/// - Some(有效数字字符串) -> Some(数字)
/// - Some(无效数字字符串) -> 错误
pub fn convert_option_to_number_option<T>(value: Option<&str>) -> Result<Option<T>, String>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    match value {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => s.parse::<T>()
            .map(Some)
            .map_err(|e| format!("无法将字符串 '{}' 解析为数字: {}", s, e))
    }
}

/// 处理字符串值转换为布尔值
///
/// 接受的字符串值:
/// - "true", "True", "1", "yes", "y", "on" 被解析为 true
/// - "false", "False", "0", "no", "n", "off" 被解析为 false
pub fn convert_to_bool(value: &str) -> Result<bool, String> {
    let value = value.to_lowercase();
    match value.as_str() {
        "true" | "1" | "yes" | "y" | "on" => Ok(true),
        "false" | "0" | "no" | "n" | "off" => Ok(false),
        _ => Err(format!("无法将字符串 '{}' 解析为布尔值", value))
    }
}

/// 处理字符串值转换为可选布尔值
pub fn convert_to_bool_option(value: &str) -> Result<Option<bool>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    convert_to_bool(value).map(Some)
}

/// 将任意类型尝试转换为目标数字类型
///
/// 支持从字符串或其他数字类型转换
pub fn try_into_number<T, F>(value: F) -> Result<T, String>
where
    T: FromStr,
    T::Err: fmt::Display,
    F: ToString,
{
    convert_to_number(&value.to_string())
}



#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Request {
        version: String,

        // 可选的source字段，可以是数字或字符串
        #[serde(deserialize_with = "string_or_number_option", default)]
        source: Option<u8>,

        name: String,
    }

    #[test]
    fn test_deserializer() {
        // 1. 数字值
        let json1 = r#"{"version":"12800","source":2,"name":"测试1"}"#;
        let req1: Request = serde_json::from_str(json1).unwrap();
        println!("数字值: {:?}", req1); // source: Some(2)

        // 2. 字符串数字
        let json2 = r#"{"version":"12800","source":"3","name":"测试2"}"#;
        let req2: Request = serde_json::from_str(json2).unwrap();
        println!("字符串数字: {:?}", req2); // source: Some(3)

        // 3. null值
        let json3 = r#"{"version":"12800","source":null,"name":"测试3"}"#;
        let req3: Request = serde_json::from_str(json3).unwrap();
        println!("null值: {:?}", req3); // source: None

        // 4. 字段缺失
        let json4 = r#"{"version":"12800","name":"测试4"}"#;
        let req4: Request = serde_json::from_str(json4).unwrap();
        println!("字段缺失: {:?}", req4); // source: None

        // 5. 空字符串
        let json5 = r#"{"version":"12800","source":"","name":"测试5"}"#;
        let req5: Request = serde_json::from_str(json5).unwrap();
        println!("空字符串: {:?}", req5); // source: None
    }


    #[test]
    fn test_convert() {
        // 1. 基本字符串转数字
        match convert_to_number::<u32>("123") {
            Ok(num) => println!("转换成功: {}", num),
            Err(e) => println!("转换失败: {}", e),
        }

        // 2. 处理可能为空的字符串
        let empty_string = "";
        match convert_to_number_option::<i32>(empty_string) {
            Ok(Some(num)) => println!("转换为数字: {}", num),
            Ok(None) => println!("空字符串转换为None"),
            Err(e) => println!("转换失败: {}", e),
        }

        // 3. 处理可能为None的输入
        let optional_input: Option<&str> = Some("42");
        match convert_option_to_number_option::<u8>(optional_input) {
            Ok(Some(num)) => println!("可选输入转换为数字: {}", num),
            Ok(None) => println!("没有输入或空输入"),
            Err(e) => println!("转换失败: {}", e),
        }

        // 4. 布尔值转换
        match convert_to_bool("yes") {
            Ok(true) => println!("解析为true"),
            Ok(false) => println!("解析为false"),
            Err(e) => println!("布尔值解析失败: {}", e),
        }

        // 5. 实际应用场景：API参数转换
        fn process_api_request(user_id: Option<&str>, quantity: Option<&str>) {
            // 转换user_id为u64
            let user_id_parsed = match convert_option_to_number_option::<u64>(user_id) {
                Ok(id) => id,
                Err(e) => {
                    println!("用户ID解析失败: {}", e);
                    return;
                }
            };

            // 转换quantity为可选的i32
            let quantity_parsed = match convert_option_to_number_option::<i32>(quantity) {
                Ok(qty) => qty,
                Err(e) => {
                    println!("数量解析失败: {}", e);
                    return;
                }
            };

            println!("处理请求 - 用户ID: {:?}, 数量: {:?}", user_id_parsed, quantity_parsed);
        }

        // 调用API处理函数
        process_api_request(Some("1001"), Some("5"));
        process_api_request(Some(""), None);
        process_api_request(Some("invalid"), Some("10")); // 会失败
    }
}