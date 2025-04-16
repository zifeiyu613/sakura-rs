use std::collections::HashMap;
use clap::{Arg, ArgAction, Command};
use config::{ConfigError as SourceError, Source, Value, Map};
use crate::error::ConfigError;
use crate::loader::ConfigLoader;

pub struct ArgsLoader {
    prefix: String,
    separator: String,
}

impl ArgsLoader {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            separator: ".".to_string(),
        }
    }

    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    // 解析命令行参数为Map
    fn parse_args(&self) -> Result<Map<String, Value>, ConfigError> {
        let app = Command::new("Application")
            .arg(
                Arg::new("rconfig")
                    .short('c')
                    .long("rconfig")
                    .value_name("FILE")
                    .help("Sets a custom rconfig file")
            )
            .arg(
                Arg::new("set")
                    .short('s')
                    .long("set")
                    .value_name("KEY=VALUE")
                    .action(ArgAction::Append)
                    .help("Override a configuration value (can be used multiple times)")
            );

        let matches = app.try_get_matches().map_err(|e| {
            ConfigError::CliArgsError(e.to_string())
        })?;

        let mut map = Map::new();

        // 处理 --set KEY=VALUE 参数
        if let Some(values) = matches.get_many::<String>("set") {
            for arg in values {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();

                if parts.len() != 2 {
                    return Err(ConfigError::CliArgsError(
                        format!("Invalid --set argument '{}': expected KEY=VALUE", arg)
                    ));
                }

                let key = parts[0];
                let value = parts[1];

                // 将值解析为适当的类型
                let parsed_value = self.parse_value(value)?;

                // 如果有前缀要求，则检查键是否符合要求
                if !self.prefix.is_empty() && !key.starts_with(&self.prefix) {
                    continue;
                }

                // 移除前缀（如果有）
                let config_key = if !self.prefix.is_empty() {
                    key.replacen(&self.prefix, "", 1)
                } else {
                    key.to_string()
                };

                // 使用分隔符拆分路径并构建嵌套结构
                self.insert_value_at_path(&mut map, &config_key, parsed_value)?;
            }
        }

        Ok(map)
    }

    // 将值插入到嵌套的Map结构中
    fn insert_value_at_path(
        &self,
        map: &mut Map<String, Value>,
        path: &str,
        value: Value
    ) -> Result<(), ConfigError> {
        let parts: Vec<&str> = path.split(&self.separator).collect();

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
                        parts[..=i].join(&self.separator),
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

impl ConfigLoader for ArgsLoader {
    fn load() -> Result<config::Value, ConfigError> {
        let loader = ArgsLoader::new("");
        let map = loader.parse_args()?;
        Ok(Value::from(map))
    }
}

// 实现Source特质以便与config库集成
impl Source for ArgsLoader {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, SourceError> {
        self.parse_args().map_err(|e| SourceError::Foreign(Box::new(e)))
    }
}

impl Clone for ArgsLoader {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            separator: self.separator.clone(),
        }
    }
}
