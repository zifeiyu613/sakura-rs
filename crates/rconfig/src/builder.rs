//! 配置构建器模块，提供流式API来构建应用配置。  

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use config::{Config, Value, Map, ValueKind};

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
    pub(crate) config_builder: config::ConfigBuilder<config::builder::DefaultState>,
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
            config_builder: Config::builder(),
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
        service_map.insert("name".to_string(), Value::from(service_name));
        service_map.insert("version".to_string(), Value::from(service_version));
        service_map.insert("environment".to_string(), Value::from("development".to_string()));
        service_map.insert("host".to_string(), Value::from("0.0.0.0".to_string()));
        service_map.insert("port".to_string(), Value::from(8080));

        self.defaults.insert("service".to_string(), Value::from(service_map));

        // 推断默认配置文件路径  
        if self.default_config_path.is_none() {
            // 使用常量或静态数组避免每次调用都重新创建向量  
            const DEFAULT_PATHS: &[&str] = &[
                "config/config.toml",
                "config/config.yaml",
                "config/config.json",
                "config.toml",
                "config.yaml",
                "config.json",
            ];

            for &path in DEFAULT_PATHS {
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
            return self;
        }

        // 创建临时的 Config，用于组合嵌套值  
        let temp_config = match Config::builder()
            .set_default(&key, value)
            .and_then(|b| b.build())
            .and_then(|c| c.try_deserialize::<HashMap<String, Value>>()) {
            Ok(map) => map,
            Err(_) => {
                // 失败则简单插入顶层键  
                self.defaults.insert(parts[0].to_string(), Value::new(None, ValueKind::Nil));
                return self;
            }
        };

        // 将新构建的嵌套结构合并到现有的 defaults 中  
        if let Some(root_value) = temp_config.get(parts[0]) {
            let root_key = parts[0].to_string();

            if self.defaults.contains_key(&root_key) {
                // 如果根键已存在，需要合并表格  
                if let Some(existing) = self.defaults.get_mut(&root_key) {
                    self.merge_values(existing, root_value);
                }
            } else {
                // 根键不存在，直接插入  
                self.defaults.insert(root_key, root_value.clone());
            }
        }

        self
    }

    // 辅助方法：合并两个值  
    fn merge_values(&self, existing: &mut Value, new_value: &Value) {
        // 两者都是表格，进行深度合并  
        if existing.kind == ValueKind::Table && new_value.kind == ValueKind::Table {
            if let (Ok(mut existing_table), Ok(new_table)) = (
                existing.clone().try_deserialize::<HashMap<String, Value>>(),
                new_value.clone().try_deserialize::<HashMap<String, Value>>(),
            ) {
                // 合并表格  
                for (k, v) in new_table {
                    if let Some(existing_v) = existing_table.get_mut(&k) {
                        self.merge_values(existing_v, &v);
                    } else {
                        existing_table.insert(k, v);
                    }
                }

                // 更新现有值  
                if let Ok(merged) = Value::try_from(existing_table) {
                    *existing = merged;
                }
            }
        } else {
            // 类型不兼容，直接替换  
            *existing = new_value.clone();
        }
    }

    /// 从文件加载配置，可选指定格式  
    pub fn with_file<P: AsRef<Path>>(mut self, path: P, format: Option<FileFormat>) -> Self {
        let path_ref = path.as_ref();
        let mut file_loader = FileLoader::new(path_ref);

        if let Some(fmt) = format {
            file_loader = file_loader.with_format(fmt);
        }

        self.config_builder = self.config_builder.add_source(file_loader);
        self.default_config_path = Some(path_ref.to_string_lossy().into_owned());
        self
    }

    /// 从文件加载配置，指定格式 (向后兼容)  
    pub fn with_file_format<P: AsRef<Path>>(self, path: P, format: FileFormat) -> Self {
        self.with_file(path, Some(format))
    }

    /// 从环境变量加载配置，可选指定前缀  
    pub fn with_env(mut self) -> Self {
        let env_loader = EnvLoader::new();
        self.config_builder = self.config_builder.add_source(env_loader);
        self
    }

    /// 从环境变量加载配置，指定前缀  
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        let prefix = prefix.into();
        let env_loader = EnvLoader::new(&prefix);
        self.config_builder = self.config_builder.add_source(env_loader);
        self.env_prefix = Some(prefix);
        self
    }

    /// 从命令行参数加载配置  
    pub fn with_cli_args(mut self) -> Self {
        let args_loader = ArgsLoader::new();
        self.config_builder = self.config_builder.add_source(args_loader);
        self.load_cli_args = true;
        self
    }

    /// 添加远程配置源  
    pub fn with_remote(self, url: impl Into<String>) -> Self {
        self.with_remote_internal(url.into(), None)
    }

    /// 添加远程配置源，带身份验证  
    pub fn with_remote_auth(
        self,
        url: impl Into<String>,
        token: impl Into<String>
    ) -> Self {
        self.with_remote_internal(url.into(), Some(token.into()))
    }

    // 内部方法，处理远程配置加载逻辑  
    fn with_remote_internal(mut self, url: String, token: Option<String>) -> Self {
        let mut remote_loader = RemoteLoader::new(url);

        if let Some(auth_token) = token {
            remote_loader = remote_loader.with_auth_token(auth_token);
        }

        self.config_builder = self.config_builder.add_source(remote_loader);
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

        if let Ok(value) = serde_json::to_value(&extension) {
            if let Ok(config_value) = convert_serde_value_to_config(value) {
                self.extensions.insert(key, config_value);
            }
        }

        self
    }

    /// 应用模板引擎  
    pub fn with_template_engine(mut self, engine: &TemplateEngine) -> Self {
        if let Some(path) = &self.default_config_path {
            self.apply_template_to_file(path, engine);
        }
        self
    }

    // 提取模板处理逻辑为单独方法  
    fn apply_template_to_file(&mut self, path: &str, engine: &TemplateEngine) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(processed) = engine.process(&content) {
                    // 使用临时文件方案，但在生产环境应考虑更好的实现  
                    let temp_path = format!("{}.processed", path);
                    if std::fs::write(&temp_path, processed).is_ok() {
                        let file_loader = FileLoader::new(&temp_path);
                        self.config_builder = self.config_builder.add_source(file_loader);
                    }
                }
            },
            Err(_) => {
                // 文件读取失败时不执行额外操作  
            }
        }
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
        // 修复: 使用config_builder而不是不存在的config字段  
        let built_config = self.config_builder.clone().build()
            .map_err(|e| ConfigError::Building(format!("Failed to build config: {}", e)))?;

        // 提取配置值到Map结构  
        let loaded_config: Map<String, Value> = built_config.try_deserialize()
            .map_err(|e| ConfigError::Deserialization(format!("Failed to deserialize config: {}", e)))?;

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
            .map_err(|e| ConfigError::Deserialization(format!("Failed to deserialize config: {}", e)))?;

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
            // 简化数字处理逻辑  
            if n.is_i64() {
                Ok(Value::from(n.as_i64().unwrap()))
            } else if n.is_f64() {
                Ok(Value::from(n.as_f64().unwrap()))
            } else {
                Err(ConfigError::Parsing("Unable to convert number type".to_string()))
            }
        },
        serde_json::Value::String(s) => Ok(Value::from(s)),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<Value>, ConfigError> = arr
                .into_iter()
                .map(convert_serde_value_to_config)
                .collect();
            Ok(Value::from(values?))
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