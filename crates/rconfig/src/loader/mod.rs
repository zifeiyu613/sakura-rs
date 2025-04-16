//! 配置加载器模块，提供从不同源加载配置的功能。

mod file;
mod env;
mod args;
mod remote;

pub use file::{FileLoader, FileFormat};
pub use env::EnvLoader;
pub use args::ArgsLoader;
pub use remote::{RemoteLoader, RemoteContentType};

use crate::error::ConfigError;
use config::{Config, Value, Map};

/// 配置加载器特质，定义从各种源加载配置的通用接口
pub trait ConfigLoader {
    /// 从配置源加载配置
    fn load() -> Result<Value, ConfigError>;
}

/// 将 rconfig::Config 构建器构建为最终的配置Map
pub(crate) fn build_config(config: Config) -> Result<Map<String, Value>, ConfigError> {
    config.try_deserialize::<Map<String, Value>>()
        .map_err(|e| ConfigError::Deserialization(e.to_string()))
}

/// 合并两个配置Map
pub(crate) fn merge_maps(base: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (key, value) in overlay {
        match (base.get_mut(key), value) {
            // 如果两边都是Map，递归合并
            (Some(Value::Table(ref mut base_map)), Value::Table(overlay_map)) => {
                merge_maps(base_map, overlay_map);
            },
            // 否则直接覆盖
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

/// 将路径字符串（如 "database.main.host"）转换为嵌套的Map
pub(crate) fn path_to_map(path: &str, value: Value) -> Map<String, Value> {
    let parts: Vec<&str> = path.split('.').collect();
    path_to_map_recursive(&parts, value)
}

fn path_to_map_recursive(parts: &[&str], value: Value) -> Map<String, Value> {
    let mut result = Map::new();

    if parts.is_empty() {
        return result;
    }

    if parts.len() == 1 {
        result.insert(parts[0].to_string(), value);
    } else {
        let nested = path_to_map_recursive(&parts[1..], value);
        result.insert(parts[0].to_string(), Value::Table(nested));
    }

    result
}

/// 展平嵌套的Map到单层key-value对
pub(crate) fn flatten_map(map: &Map<String, Value>, prefix: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();

    for (key, value) in map {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            Value::Table(nested) => {
                let nested_flattened = flatten_map(nested, &full_key);
                result.extend(nested_flattened);
            },
            _ => {
                if let Some(value_str) = value_to_string(value) {
                    result.push((full_key, value_str));
                }
            }
        }
    }

    result
}

/// 将 rconfig::Value 转换为字符串表示
fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_string()),
        Value::String(s) => Some(s.clone()),
        Value::Integer(i) => Some(i.to_string()),
        Value::Float(f) => Some(f.to_string()),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter()
                .filter_map(value_to_string)
                .collect();
            Some(format!("[{}]", items.join(", ")))
        },
        Value::Table(_) => None, // 表格类型在flatten_map中单独处理
    }
}

/// 解析环境变量名中的分隔符
pub(crate) fn parse_env_key(key: &str, separator: &str) -> Vec<String> {
    key.split(separator)
        .map(|s| s.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_map() {
        let value = Value::String("localhost".to_string());
        let map = path_to_map("database.main.host", value.clone());

        assert!(map.contains_key("database"));
        if let Some(Value::Table(db)) = map.get("database") {
            assert!(db.contains_key("main"));
            if let Some(Value::Table(main)) = db.get("main") {
                assert!(main.contains_key("host"));
                assert_eq!(main.get("host"), Some(&value));
            } else {
                panic!("Expected 'main' to be a Table");
            }
        } else {
            panic!("Expected 'database' to be a Table");
        }
    }

    #[test]
    fn test_merge_maps() {
        let mut base = Map::new();
        base.insert("service".to_string(), Value::Table({
            let mut m = Map::new();
            m.insert("name".to_string(), Value::String("base-service".to_string()));
            m.insert("port".to_string(), Value::Integer(8080));
            m
        }));

        let overlay = {
            let mut m = Map::new();
            m.insert("service".to_string(), Value::Table({
                let mut m = Map::new();
                m.insert("name".to_string(), Value::String("overlay-service".to_string()));
                m
            }));
            m.insert("database".to_string(), Value::Table({
                let mut m = Map::new();
                m.insert("host".to_string(), Value::String("localhost".to_string()));
                m
            }));
            m
        };

        merge_maps(&mut base, &overlay);

        // Check that service.name was overridden
        if let Some(Value::Table(service)) = base.get("service") {
            assert_eq!(
                service.get("name"),
                Some(&Value::String("overlay-service".to_string()))
            );
            // Check that service.port was preserved
            assert_eq!(
                service.get("port"),
                Some(&Value::Integer(8080))
            );
        } else {
            panic!("Expected 'service' to be a Table");
        }

        // Check that database was added
        assert!(base.contains_key("database"));
    }

    #[test]
    fn test_flatten_map() {
        let mut map = Map::new();
        map.insert("service".to_string(), Value::Table({
            let mut m = Map::new();
            m.insert("name".to_string(), Value::String("test-service".to_string()));
            m.insert("port".to_string(), Value::Integer(8080));
            m
        }));

        let flattened = flatten_map(&map, "");

        assert_eq!(flattened.len(), 2);
        assert!(flattened.contains(&("service.name".to_string(), "test-service".to_string())));
        assert!(flattened.contains(&("service.port".to_string(), "8080".to_string())));
    }

    #[test]
    fn test_parse_env_key() {
        let parts = parse_env_key("APP_SERVICE_NAME", "_");
        assert_eq!(parts, vec!["app", "service", "name"]);

        let parts = parse_env_key("app.service.name", ".");
        assert_eq!(parts, vec!["app", "service", "name"]);
    }
}
