use std::time::Duration;
use config::{ConfigError as SourceError, Source, Value, Map};
use async_trait::async_trait;
use crate::error::ConfigError;
use crate::loader::ConfigLoader;
use crate::loader::file::{FileFormat, FileLoader};

pub struct RemoteLoader {
    url: String,
    timeout: Duration,
    retry_count: u32,
    auth_token: Option<String>,
    content_type: RemoteContentType,
}

pub enum RemoteContentType {
    Json,
    Yaml,
    Toml,
    Auto,
}

impl RemoteLoader {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            timeout: Duration::from_secs(5),
            retry_count: 3,
            auth_token: None,
            content_type: RemoteContentType::Auto,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retries(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    pub fn with_content_type(mut self, content_type: RemoteContentType) -> Self {
        self.content_type = content_type;
        self
    }

    // 从远程URL获取配置内容
    fn fetch_content(&self) -> Result<String, ConfigError> {
        // 这里使用阻塞的HTTP请求，实际应用中可以使用异步HTTP客户端
        let client = ureq::AgentBuilder::new()
            .timeout_connect(self.timeout)
            .timeout_read(self.timeout)
            .build();

        let mut request = client.get(&self.url);

        // 添加Authorization头（如果提供了token）
        if let Some(token) = &self.auth_token {
            request = request.set("Authorization", &format!("Bearer {}", token));
        }

        // 添加Accept头
        let accept_header = match self.content_type {
            RemoteContentType::Json => "application/json",
            RemoteContentType::Yaml => "application/yaml,application/x-yaml",
            RemoteContentType::Toml => "application/toml",
            RemoteContentType::Auto => "application/json,application/yaml,application/x-yaml,application/toml",
        };
        request = request.set("Accept", accept_header);

        // 发送请求并处理重试
        let mut last_error = None;
        for _ in 0..=self.retry_count {
            match request.clone().call() {
                Ok(response) => {
                    // 检查HTTP状态码
                    if response.status() != 200 {
                        let error = ConfigError::Other(format!(
                            "Failed to fetch configuration: HTTP {} - {}",
                            response.status(),
                            response.status_text()
                        ));
                        last_error = Some(error);
                        continue; // 重试
                    }

                    // 读取响应内容
                    match response.into_string() {
                        Ok(content) => return Ok(content),
                        Err(e) => {
                            last_error = Some(ConfigError::Other(format!(
                                "Failed to read response content: {}", e
                            )));
                            continue; // 重试
                        }
                    }
                },
                Err(e) => {
                    last_error = Some(ConfigError::Other(format!(
                        "Failed to connect to configuration server: {}", e
                    )));
                    continue; // 重试
                }
            }
        }

        // 所有重试都失败了
        Err(last_error.unwrap_or_else(||
            ConfigError::Other("Failed to fetch remote configuration".to_string())
        ))
    }

    // 解析内容为配置Map
    fn parse_content(&self, content: &str) -> Result<Map<String, Value>, ConfigError> {
        if content.is_empty() {
            return Ok(Map::new());
        }

        // 检测内容类型（如果是Auto）或使用指定的类型
        let content_type = if let RemoteContentType::Auto = self.content_type {
            // 简单的内容类型检测
            if content.trim().starts_with('{') {
                RemoteContentType::Json
            } else if content.contains(':') && !content.contains('=') {
                RemoteContentType::Yaml
            } else {
                RemoteContentType::Toml
            }
        } else {
            self.content_type.clone()
        };

        // 根据内容类型解析
        match content_type {
            RemoteContentType::Json => {
                let file_loader = FileLoader::new("temp.json")
                    .with_format(FileFormat::Json);
                file_loader.parse_content(content)
            },
            RemoteContentType::Yaml => {
                let file_loader = FileLoader::new("temp.yaml")
                    .with_format(FileFormat::Yaml);
                file_loader.parse_content(content)
            },
            RemoteContentType::Toml => {
                let file_loader = FileLoader::new("temp.toml")
                    .with_format(FileFormat::Toml);
                file_loader.parse_content(content)
            },
            RemoteContentType::Auto => {
                // 尝试各种解析方式，先从最可能的开始
                let file_loader = FileLoader::new("temp")
                    .with_format(FileFormat::Auto);
                file_loader.parse_content(content)
            }
        }
    }
}

impl ConfigLoader for RemoteLoader {
    fn load() -> Result<config::Value, ConfigError> {
        // 默认实现只有在显式设置了URL时才会工作
        Err(ConfigError::Other("Remote loader requires explicit URL configuration".to_string()))
    }
}

// 实现Source特质以便与config库集成
impl Source for RemoteLoader {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, SourceError> {
        let content = self.fetch_content()
            .map_err(|e| SourceError::Foreign(Box::new(e)))?;

        self.parse_content(&content)
            .map_err(|e| SourceError::Foreign(Box::new(e)))
    }
}

impl Clone for RemoteLoader {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            timeout: self.timeout,
            retry_count: self.retry_count,
            auth_token: self.auth_token.clone(),
            content_type: match self.content_type {
                RemoteContentType::Json => RemoteContentType::Json,
                RemoteContentType::Yaml => RemoteContentType::Yaml,
                RemoteContentType::Toml => RemoteContentType::Toml,
                RemoteContentType::Auto => RemoteContentType::Auto,
            },
        }
    }
}

impl Clone for RemoteContentType {
    fn clone(&self) -> Self {
        match self {
            RemoteContentType::Json => RemoteContentType::Json,
            RemoteContentType::Yaml => RemoteContentType::Yaml,
            RemoteContentType::Toml => RemoteContentType::Toml,
            RemoteContentType::Auto => RemoteContentType::Auto,
        }
    }
}
