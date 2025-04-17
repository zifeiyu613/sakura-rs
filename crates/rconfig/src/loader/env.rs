use crate::error::ConfigError;
use crate::loader::ConfigLoader;
use config::{ConfigError as SourceError, Map, Source, Value};
use std::env;

#[derive(Debug)]
pub struct EnvLoader {
    prefix: String,
    separator: String,
}

impl EnvLoader {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            separator: "__".to_string(),
        }
    }

    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    // 将环境变量转换为嵌套的配置结构
    fn parse_env_vars(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut config_map: Map<String, Value> = Map::new();
        let prefix = format!("{}{}", self.prefix, self.separator);

        for (key, value) in env::vars() {
            if !key.starts_with(&prefix) {
                continue;
            }

            // 移除前缀并转换分隔符为点号以形成嵌套结构
            let config_key = key.replacen(&prefix, "", 1)
                .replace(&self.separator, ".");

            // 尝试将值转换为适当的类型
            let config_value = self.parse_value(&value)?;

            // 使用点号路径将值插入嵌套的配置结构
            self.insert_value_at_path(&mut config_map, &config_key, config_value)?;
        }

        Ok(config_map)
    }

    // 将值插入到嵌套的Map结构中
    fn insert_value_at_path(
        &self,
        map: &mut Map<String, Value>,
        path: &str,
        value: Value
    ) -> Result<(), ConfigError> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: path.to_string(),
                message: "Empty path".to_string(),
            });
        }

        let mut current_map = map;

        // 处理除了最后一部分以外的路径
        for (i, &part) in parts.iter().enumerate().take(parts.len() - 1) {
            if !current_map.contains_key(part) {
                current_map.insert(part.to_string(), Value::from(Map::new()));
            }

            // 获取或创建嵌套的Map
            let next_value = current_map.get_mut(part).unwrap();

            match next_value {
                Value::Table(ref mut next_map) => {
                    current_map = next_map;
                }
                _ => {
                    // 如果路径中的某个部分已经是非Map值，则报错
                    return Err(ConfigError::ConflictingValues(format!(
                        "Path conflict at '{}' in '{}'",
                        parts[..=i].join("."),
                        path
                    )));
                }
            }
        }

        // 插入值到最终位置
        let last_part = parts.last().unwrap();
        current_map.insert(last_part.to_string(), value);

        Ok(())
    }

    // 尝试将字符串值解析为适当的类型
    fn parse_value(&self, value_str: &str) -> Result<Value, ConfigError> {
        // 尝试解析为布尔值
        if value_str.eq_ignore_ascii_case("true") {
            return Ok(Value::from(true));
        }
        if value_str.eq_ignore_ascii_case("false") {
            return Ok(Value::from(false));
        }

        // 尝试解析为数字
        if let Ok(int_val) = value_str.parse::<i64>() {
            return Ok(Value::from(int_val));
        }
        if let Ok(float_val) = value_str.parse::<f64>() {
            return Ok(Value::from(float_val));
        }

        // 如果都不是，则作为字符串返回
        Ok(Value::from(value_str.to_string()))
    }
}

impl ConfigLoader for EnvLoader {
    fn load() -> Result<config::Value, ConfigError> {
        // 默认加载所有环境变量，无前缀
        let loader = EnvLoader::new("");
        let map = loader.parse_env_vars()?;
        Ok(Value::from(map))
    }
}

// 实现Source特质以便与config库集成
impl Source for EnvLoader {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, SourceError> {
        self.parse_env_vars().map_err(|e| SourceError::Foreign(Box::new(e)))
    }
}

impl Clone for EnvLoader {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            separator: self.separator.clone(),
        }
    }
}
