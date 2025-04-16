use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use config::{ConfigError as SourceError, Source, Value, Map};
use crate::error::ConfigError;
use crate::loader::ConfigLoader;

pub enum FileFormat {
    Yaml,
    Toml,
    Json,
    Auto, // 自动检测格式
}

pub struct FileLoader {
    path: PathBuf,
    format: FileFormat,
    required: bool,
}

impl FileLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        // 根据文件扩展名自动检测格式
        let format = Self::detect_format(&path_buf);

        Self {
            path: path_buf,
            format,
            required: true,
        }
    }

    pub fn with_format(mut self, format: FileFormat) -> Self {
        self.format = format;
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    // 根据文件扩展名检测格式
    fn detect_format(path: &Path) -> FileFormat {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            match ext_str.as_str() {
                "yml" | "yaml" => FileFormat::Yaml,
                "toml" => FileFormat::Toml,
                "json" => FileFormat::Json,
                _ => FileFormat::Auto,
            }
        } else {
            FileFormat::Auto
        }
    }

    // 读取文件内容
    fn read_file(&self) -> Result<String, ConfigError> {
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                if !self.required {
                    // 如果文件不是必需的，则返回空Map
                    return Ok("".to_string());
                }
                return Err(ConfigError::FileRead(e));
            }
        };

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(ConfigError::FileRead)?;

        Ok(content)
    }

    // 解析文件内容
    fn parse_content(&self, content: &str) -> Result<Map<String, Value>, ConfigError> {
        if content.is_empty() {
            return Ok(Map::new());
        }

        match &self.format {
            FileFormat::Yaml => self.parse_yaml(content),
            FileFormat::Toml => self.parse_toml(content),
            FileFormat::Json => self.parse_json(content),
            FileFormat::Auto => {
                // 尝试各种格式，按可能性顺序
                self.parse_yaml(content)
                    .or_else(|_| self.parse_toml(content))
                    .or_else(|_| self.parse_json(content))
            }
        }
    }

    fn parse_yaml(&self, content: &str) -> Result<Map<String, Value>, ConfigError> {
        use yaml_rust::{YamlLoader, Yaml};

        let docs = YamlLoader::load_from_str(content)
            .map_err(|e| ConfigError::YamlParse(e.to_string()))?;

        if docs.is_empty() {
            return Ok(Map::new());
        }

        // 转换YAML到config::Value
        self.convert_yaml_to_map(&docs[0])
    }

    fn convert_yaml_to_map(&self, yaml: &yaml_rust::Yaml) -> Result<Map<String, Value>, ConfigError> {
        let mut map = Map::new();

        match yaml {
            yaml_rust::Yaml::Hash(hash) => {
                for (key, value) in hash {
                    if let yaml_rust::Yaml::String(k) = key {
                        let v = self.convert_yaml_to_value(value)?;
                        map.insert(k.clone(), v);
                    }
                }
            }
            _ => return Err(ConfigError::YamlParse("Root element is not a map".to_string())),
        }

        Ok(map)
    }

    fn convert_yaml_to_value(&self, yaml: &yaml_rust::Yaml) -> Result<Value, ConfigError> {
        match yaml {
            yaml_rust::Yaml::String(s) => Ok(Value::from(s.clone())),
            yaml_rust::Yaml::Integer(i) => Ok(Value::from(*i)),
            yaml_rust::Yaml::Real(r) => {
                let f = r.parse::<f64>().map_err(|_| {
                    ConfigError::YamlParse(format!("Invalid float value: {}", r))
                })?;
                Ok(Value::from(f))
            }
            yaml_rust::Yaml::Boolean(b) => Ok(Value::from(*b)),
            yaml_rust::Yaml::Array(arr) => {
                let mut values = Vec::new();
                for item in arr {
                    values.push(self.convert_yaml_to_value(item)?);
                }
                Ok(Value::from(values))
            }
            yaml_rust::Yaml::Hash(hash) => {
                let mut map = Map::new();
                for (key, value) in hash {
                    if let yaml_rust::Yaml::String(k) = key {
                        let v = self.convert_yaml_to_value(value)?;
                        map.insert(k.clone(), v);
                    }
                }
                Ok(Value::from(map))
            }
            yaml_rust::Yaml::Null => Ok(Value::from(())),
            yaml_rust::Yaml::BadValue => Err(ConfigError::YamlParse("Bad YAML value".to_string())),
            yaml_rust::Yaml::Alias(_) => Err(ConfigError::YamlParse("YAML aliases are not supported".to_string())),
        }
    }

    fn parse_toml(&self, content: &str) -> Result<Map<String, Value>, ConfigError> {
        let value: toml::Value = toml::from_str(content)?;
        self.convert_toml_to_map(&value)
    }

    fn convert_toml_to_map(&self, toml: &toml::Value) -> Result<Map<String, Value>, ConfigError> {
        match toml {
            toml::Value::Table(table) => {
                let mut map = Map::new();
                for (key, value) in table {
                    map.insert(key.clone(), self.convert_toml_to_value(value)?);
                }
                Ok(map)
            }
            _ => Err(ConfigError::TomlParse(toml::de::Error::custom("Root element is not a table"))),
        }
    }

    fn convert_toml_to_value(&self, toml: &toml::Value) -> Result<Value, ConfigError> {
        match toml {
            toml::Value::String(s) => Ok(Value::from(s.clone())),
            toml::Value::Integer(i) => Ok(Value::from(*i)),
            toml::Value::Float(f) => Ok(Value::from(*f)),
            toml::Value::Boolean(b) => Ok(Value::from(*b)),
            toml::Value::Datetime(d) => Ok(Value::from(d.to_string())),
            toml::Value::Array(arr) => {
                let mut values = Vec::new();
                for item in arr {
                    values.push(self.convert_toml_to_value(item)?);
                }
                Ok(Value::from(values))
            }
            toml::Value::Table(table) => {
                let mut map = Map::new();
                for (key, value) in table {
                    map.insert(key.clone(), self.convert_toml_to_value(value)?);
                }
                Ok(Value::from(map))
            }
        }
    }

    fn parse_json(&self, content: &str) -> Result<Map<String, Value>, ConfigError> {
        let value: serde_json::Value = serde_json::from_str(content)?;
        self.convert_json_to_map(&value)
    }

    fn convert_json_to_map(&self, json: &serde_json::Value) -> Result<Map<String, Value>, ConfigError> {
        match json {
            serde_json::Value::Object(obj) => {
                let mut map = Map::new();
                for (key, value) in obj {
                    map.insert(key.clone(), self.convert_json_to_value(value)?);
                }
                Ok(map)
            }
            _ => Err(ConfigError::JsonParse(serde_json::Error::custom("Root element is not an object"))),
        }
    }

    fn convert_json_to_value(&self, json: &serde_json::Value) -> Result<Value, ConfigError> {
        match json {
            serde_json::Value::Null => Ok(Value::from(())),
            serde_json::Value::Bool(b) => Ok(Value::from(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::from(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::from(f))
                } else {
                    Err(ConfigError::JsonParse(serde_json::Error::custom("Invalid number")))
                }
            }
            serde_json::Value::String(s) => Ok(Value::from(s.clone())),
            serde_json::Value::Array(arr) => {
                let mut values = Vec::new();
                for item in arr {
                    values.push(self.convert_json_to_value(item)?);
                }
                Ok(Value::from(values))
            }
            serde_json::Value::Object(obj) => {
                let mut map = Map::new();
                for (key, value) in obj {
                    map.insert(key.clone(), self.convert_json_to_value(value)?);
                }
                Ok(Value::from(map))
            }
        }
    }
}

impl ConfigLoader for FileLoader {
    fn load() -> Result<config::Value, ConfigError> {
        // 默认尝试加载当前目录下的config文件
        for path in &["./config.yaml", "./config.toml", "./config.json"] {
            if Path::new(path).exists() {
                let loader = FileLoader::new(path);
                let content = loader.read_file()?;
                let map = loader.parse_content(&content)?;
                return Ok(Value::from(map));
            }
        }

        // 如果没有找到任何配置文件，返回空映射
        Ok(Value::from(Map::new()))
    }
}

// 实现Source特质以便与config库集成
impl Source for FileLoader {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, SourceError> {
        let content = self.read_file().map_err(|e| SourceError::Foreign(Box::new(e)))?;
        self.parse_content(&content).map_err(|e| SourceError::Foreign(Box::new(e)))
    }
}

impl Clone for FileLoader {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            format: match self.format {
                FileFormat::Yaml => FileFormat::Yaml,
                FileFormat::Toml => FileFormat::Toml,
                FileFormat::Json => FileFormat::Json,
                FileFormat::Auto => FileFormat::Auto,
            },
            required: self.required,
        }
    }
}
