//! 配置构建器模块，提供流式API来构建应用配置。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use config::{Config, Value, Map};

use crate::{
    AppConfig, ConfigError,
    loader::{
        FileLoader, FileFormat, EnvLoader, ArgsLoader, RemoteLoader,
        ConfigLoader, build_config, merge_maps
    },
    validation::ValidatorChain,
    template::TemplateEngine,
};

/// 配置构建器，提供流式API来构建和定制配置
pub struct ConfigBuilder {
    /// 底层config库构建器
    pub(crate) config: Config,
    /// 默认配置文件路径
    pub(crate) default_config_path: Option<String>,
    /// 环境变量前缀
    pub(crate) env_prefix: Option<String>,
    /// 是否加载命令行参数
    pub(crate) load_cli_args: bool,
    /// 扩展配置数据
    pub(crate) extensions: HashMap<String, Value>,
    /// 默认值
    pub(crate) defaults: Map<String, Value>,
}

impl ConfigBuilder {
    /// 创建新的配置构建器实例
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            default_config_path: None,
            env_prefix: None,
            load_cli_args: false,
            extensions: HashMap::new(),
            defaults: Map::new(),
        }
    }

    /// 添加默认配置
    pub fn with_default_config(mut self) -> Self {
        let service_name = env!("CARGO_PKG_NAME").to_string();
        let service_version = env!("CARGO_PKG_VERSION").to_string();

        // 添加基本的服务信息作为默认值
        let mut service_map = Map::new();
        service_map.insert("name".to_string(), Value::String(service_name));
        service_map.insert("version".to_string(), Value::String(service_version));
        service_map.insert("environment".to_string(), Value::String("development".to_string()));
        service_map.insert("host".to_string(), Value::String("0.0.0.0".to_string()));
        service_map.insert("port".to_string(), Value::Integer(8080));

        self.defaults.insert("service".to_string(), Value::Table(service_map));

        // 推断默认配置文件路径
        if self.default_config_path.is_none() {
            let mut default_paths = vec![
                "config/config.yaml",
                "config/config.toml",
                "config/config.json",
                "config.yaml",
                "config.toml",
                "config.json",
            ];

            for path in default_paths.drain(..) {
                if Path::new(path).exists() {
                    self.default_config_path = Some(path.to_string());
                    break;
                }
            }
        }

        self
    }

    /// 设置默认值
    pub fn with_default<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        let key = key.into();
        let value = value.into();

        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 1 {
            self.defaults.insert(key, value);
        } else {
            let mut current = &mut self.defaults;
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    current.insert(part.to_string(), value);
                    break;
                }

                if !current.contains_key(*part) {
                    current.insert(part.to_string(), Value::Table(Map::new()));
                }

                if let Some(Value::Table(ref mut next)) = current.get_mut(*part) {
                    current = next;
                } else {
                    // 已存在但不是Map，覆盖它
                    let new_map = Map::new();
                    current.insert(part.to_string(), Value::Table(new_map));
                    if let Some(Value::Table(ref mut next)) = current.get_mut(*part) {
                        current = next;
                    }
                }
            }
        }

        self
    }

    /// 从文件加载配置
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let file_loader = FileLoader::new(path.as_ref());
        self.config = self.config.add_source(file_loader);
        self.default_config_path = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    /// 从文件加载配置，指定格式
    pub fn with_file_format<P: AsRef<Path>>(mut self, path: P, format: FileFormat) -> Self {
        let file_loader = FileLoader::new(path.as_ref())
            .with_format(format);
        self.config = self.config.add_source(file_loader);
        self.default_config_path = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    /// 从环境变量加载配置
    pub fn with_env(mut self) -> Self {
        let env_loader = EnvLoader::new();
        self.config = self.config.add_source(env_loader);
        self
    }

    /// 从环境变量加载配置，指定前缀
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        let env_loader = EnvLoader::new()
            .with_prefix(&prefix);
        self.config = self.config.add_source(env_loader);
        self.env_prefix = Some(prefix);
        self
    }

    /// 从命令行参数加载配置
    pub fn with_cli_args(mut self) -> Self {
        let args_loader = ArgsLoader::new();
        self.config = self.config.add_source(args_loader);
        self.load_cli_args = true;
        self
    }

    /// 添加远程配置源
    pub fn with_remote(mut self, url: impl Into<String>) -> Self {
        let remote_loader = RemoteLoader::new(url.into());
        self.config = self.config.add_source(remote_loader);
        self
    }

    /// 添加远程配置源，带身份验证
    pub fn with_remote_auth(
        mut self,
        url: impl Into<String>,
        token: impl Into<String>
    ) -> Self {
        let remote_loader = RemoteLoader::new(url.into())
            .with_auth_token(token.into());
        self.config = self.config.add_source(remote_loader);
        self
    }

    /// 添加扩展配置
    pub fn with_extension<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.extensions.insert(key.into(), value.into());
        self
    }

    /// 添加实现了ConfigExtension特质的扩展配置
    pub fn with_extension_trait<T: serde::Serialize>(mut self, extension: T) -> Self {
        let type_name = std::any::type_name::<T>();
        let key = type_name.split("::").last().unwrap_or(type_name).to_lowercase();

        match serde_json::to_value(&extension) {
            Ok(value) => {
                if let Ok(config_value) = convert_serde_value_to_config(value) {
                    self.extensions.insert(key, config_value);
                }
            },
            Err(_) => {
                // 忽略序列化错误
            }
        }

        self
    }

    /// 应用模板引擎
    pub fn with_template_engine(mut self, engine: &TemplateEngine) -> Self {
        // 要实现模板功能，需要对FileLoader进行扩展
        // 由于需要在读取文件后应用模板，这是一个简化实现
        // 实际可能需要完全重新设计FileLoader
        if let Some(path) = &self.default_config_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(processed) = engine.process(&content) {
                    // 创建临时文件
                    let temp_path = format!("{}.processed", path);
                    if std::fs::write(&temp_path, processed).is_ok() {
                        let file_loader = FileLoader::new(&temp_path);
                        self.config = self.config.add_source(file_loader);
                    }
                }
            }
        }
        self
    }

    /// 获取默认配置路径
    pub fn default_config_path(&self) -> Option<&String> {
        self.default_config_path.as_ref()
    }

    /// 构建最终配置
    pub fn build(&self) -> Result<AppConfig, ConfigError> {
        // 首先构建包含默认值的基础配置
        let mut config_map = self.defaults.clone();

        // 然后从所有配置源加载并合并
        let loaded_config = build_config(self.config.clone())?;
        merge_maps(&mut config_map, &loaded_config);

        // 添加扩展配置
        for (key, value) in &self.extensions {
            let mut extension_map = Map::new();
            extension_map.insert(key.clone(), value.clone());
            merge_maps(&mut config_map, &extension_map);
        }

        // 将Map反序列化为AppConfig
        let config_value = Value::Table(config_map);
        let app_config: AppConfig = config_value
            .try_deserialize()
            .map_err(|e| ConfigError::Deserialization(format!("无法反序列化配置: {}", e)))?;

        Ok(app_config)
    }

    /// 使用验证器
    pub fn validate(self, validator: &ValidatorChain) -> Result<AppConfig, ConfigError> {
        let config = self.build()?;
        validator.validate(&config)?;
        Ok(config)
    }

    /// 启用热重载
    pub fn with_hot_reload(self) -> Result<crate::watcher::ConfigWatcherHandle, ConfigError> {
        let config = self.build()?;
        Ok(crate::with_hot_reload(config, self))
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new().with_default_config()
    }
}

/// 将 serde_json::Value 转换为 config::Value
fn convert_serde_value_to_config(value: serde_json::Value) -> Result<Value, ConfigError> {
    match value {
        serde_json::Value::Null => Ok(Value::from(())),
        serde_json::Value::Bool(b) => Ok(Value::from(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::from(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::from(f))
            } else {
                Err(ConfigError::Parsing("无法转换数字类型".to_string()))
            }
        },
        serde_json::Value::String(s) => Ok(Value::from(s)),
        serde_json::Value::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                values.push(convert_serde_value_to_config(item)?);
            }
            Ok(Value::from(values))
        },
        serde_json::Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k, convert_serde_value_to_config(v)?);
            }
            Ok(Value::Table(map))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::ServiceConfig;

    #[test]
    fn test_default_builder() {
        let builder = ConfigBuilder::default();
        assert!(builder.defaults.contains_key("service"));
    }

    #[test]
    fn test_with_default() {
        let builder = ConfigBuilder::new()
            .with_default("service.name", "test-app")
            .with_default("service.port", 9090);

        assert!(builder.defaults.contains_key("service"));
        if let Some(Value::Table(service)) = builder.defaults.get("service") {
            assert_eq!(
                service.get("name"),
                Some(&Value::String("test-app".to_string()))
            );
            assert_eq!(
                service.get("port"),
                Some(&Value::Integer(9090))
            );
        } else {
            panic!("Expected 'service' to be a Table");
        }
    }

    #[test]
    fn test_build_simple_config() {
        let builder = ConfigBuilder::new()
            .with_default("service.name", "test-app")
            .with_default("service.port", 9090);

        let config = builder.build().unwrap();
        assert_eq!(config.service.name, "test-app");
        assert_eq!(config.service.port, 9090);
    }

    #[test]
    fn test_extension() {
        #[derive(serde::Serialize)]
        struct TestExt {
            value: String,
        }

        let ext = TestExt { value: "test".to_string() };
        let builder = ConfigBuilder::new()
            .with_extension_trait(ext);

        assert!(builder.extensions.contains_key("testext"));
    }
}
