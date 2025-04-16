use anyhow::{Result, Context, anyhow};
use reqwest::{Client, RequestBuilder, Response, header};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{info, warn, error, debug, Span};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::time::sleep;
use tokio::sync::Mutex;
use std::sync::Arc;
use serde_json::Value;

// HTTP客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
    pub user_agent: String,
    pub connect_timeout: Duration,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_count: 3,
            retry_delay: Duration::from_secs(1),
            user_agent: format!("PaymentGateway/1.0 Rust/{}", env!("CARGO_PKG_VERSION")),
            connect_timeout: Duration::from_secs(10),
        }
    }
}

// HTTP客户端日志记录器
#[derive(Clone)]
pub struct RequestLogger {
    enabled: bool,
    mask_fields: Vec<String>,
}

impl RequestLogger {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            mask_fields: vec![
                "password".to_string(),
                "token".to_string(),
                "secret".to_string(),
                "key".to_string(),
                "card_number".to_string(),
                "cvv".to_string(),
            ],
        }
    }

    pub fn add_mask_field(&mut self, field: &str) -> &mut Self {
        self.mask_fields.push(field.to_string());
        self
    }

    // 遮蔽敏感字段
    fn mask_sensitive_data(&self, data: &str) -> String {
        let mut result = data.to_string();

        // 尝试解析为JSON
        if let Ok(mut json_value) = serde_json::from_str::<Value>(&result) {
            self.mask_json_value(&mut json_value);
            return json_value.to_string();
        }

        // 如果不是JSON，尝试处理URL编码或表单数据
        for field in &self.mask_fields {
            // 匹配模式: field=value 或 "field":"value" 或 "field": "value"
            let patterns = vec![
                format!(r#"{}=([^&]+)"#, field),
                format!(r#""{}":"([^"]+)""#, field),
                format!(r#""{}":\s*"([^"]+)""#, field),
            ];

            for pattern in patterns {
                if let Ok(regex) = regex::Regex::new(&pattern) {
                    result = regex.replace_all(&result, format!("{}=*****", field)).to_string();
                }
            }
        }

        result
    }

    // 递归遮蔽JSON中的敏感字段
    fn mask_json_value(&self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    if self.mask_fields.iter().any(|field| key.to_lowercase().contains(&field.to_lowercase())) {
                        if val.is_string() {
                            *val = Value::String("*****".to_string());
                        } else if val.is_number() {
                            *val = Value::String("*****".to_string());
                        }
                    } else {
                        self.mask_json_value(val);
                    }
                }
            }
            Value::Array(array) => {
                for val in array.iter_mut() {
                    self.mask_json_value(val);
                }
            }
            _ => {}
        }
    }
}


// HTTP客户端
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    config: HttpClientConfig,
    logger: RequestLogger,
    default_headers: Arc<Mutex<header::HeaderMap>>,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            .pool_max_idle_per_host(10)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            config,
            logger: RequestLogger::new(true),
            default_headers: Arc::new(Mutex::new(header::HeaderMap::new())),
        })
    }

    // 设置默认请求头
    pub async fn set_default_header(&self, key: &str, value: &str) -> Result<()> {
        let header_name = header::HeaderName::from_bytes(key.as_bytes())
            .context("Invalid header name")?;
        let header_value = header::HeaderValue::from_str(value)
            .context("Invalid header value")?;

        let mut headers = self.default_headers.lock().await;
        headers.insert(header_name, header_value);

        Ok(())
    }

    // 清除默认请求头
    pub async fn clear_default_headers(&self) {
        let mut headers = self.default_headers.lock().await;
        headers.clear();
    }

    // 构建GET请求
    pub async fn get(&self, url: &str) -> RequestBuilder {
        let mut req = self.client.get(url);
        let headers = self.default_headers.lock().await.clone();
        req = req.headers(headers);
        req
    }

    // 构建POST请求
    pub async fn post(&self, url: &str) -> RequestBuilder {
        let mut req = self.client.post(url);
        let headers = self.default_headers.lock().await.clone();
        req = req.headers(headers);
        req
    }

    // 构建PUT请求
    pub async fn put(&self, url: &str) -> RequestBuilder {
        let mut req = self.client.put(url);
        let headers = self.default_headers.lock().await.clone();
        req = req.headers(headers);
        req
    }

    // 构建DELETE请求
    pub async fn delete(&self, url: &str) -> RequestBuilder {
        let mut req = self.client.delete(url);
        let headers = self.default_headers.lock().await.clone();
        req = req.headers(headers);
        req
    }

    // 执行请求并返回JSON响应
    pub async fn request_json<T, U>(&self, method: &str, url: &str, body: Option<&T>) -> Result<U>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned,
    {
        let request_id = Uuid::new_v4().to_string();
        let span = tracing::info_span!("http_request", %method, %url, %request_id);
        let _guard = span.enter();

        let request_builder = match method.to_uppercase().as_str() {
            "GET" => self.get(url).await,
            "POST" => {
                let mut builder = self.post(url).await;
                if let Some(data) = body {
                    builder = builder.json(data);
                }
                builder
            }
            "PUT" => {
                let mut builder = self.put(url).await;
                if let Some(data) = body {
                    builder = builder.json(data);
                }
                builder
            }
            "DELETE" => self.delete(url).await,
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        // 记录请求
        if self.logger.enabled {
            let body_str = match body {
                Some(b) => serde_json::to_string(b).unwrap_or_else(|_| "<<unparseable body>>".to_string()),
                None => "<<no body>>".to_string(),
            };

            let masked_body = self.logger.mask_sensitive_data(&body_str);
            debug!(request_id = %request_id, method = %method, url = %url, body = %masked_body, "HTTP request");
        }

        // 发送请求（带重试）
        let response = self.send_with_retry(request_builder).await?;

        // 记录响应
        if self.logger.enabled {
            let status = response.status();
            let response_body = response.text().await.context("Failed to get response body")?;
            let masked_response = self.logger.mask_sensitive_data(&response_body);

            debug!(
                request_id = %request_id,
                status = %status.as_u16(),
                response = %masked_response,
                "HTTP response"
            );

            // 需要反序列化响应文本
            match serde_json::from_str::<U>(&response_body) {
                Ok(parsed) => Ok(parsed),
                Err(e) => {
                    error!(
                        request_id = %request_id,
                        error = %e,
                        response = %masked_response,
                        "Failed to parse response"
                    );
                    Err(anyhow!("Failed to parse response: {}", e))
                }
            }
        } else {
            // 直接从响应反序列化
            response.json::<U>().await.context("Failed to parse response")
        }
    }

    // 带重试的请求发送
    async fn send_with_retry(&self, request: RequestBuilder) -> Result<Response> {
        let mut retry_count = 0;
        let max_retries = self.config.retry_count;

        loop {
            let request_clone = request.try_clone().ok_or_else(|| anyhow!("Cannot clone request"))?;

            match request_clone.send().await {
                Ok(response) => {
                    let status = response.status();

                    // 检查是否需要重试
                    let should_retry = status.is_server_error() ||  // 5xx错误
                        status.as_u16() == 429;     // 太多请求

                    if !should_retry || retry_count >= max_retries {
                        return Ok(response);
                    }

                    // 记录重试
                    warn!(
                        retry = retry_count + 1,
                        max_retries = max_retries,
                        status = %status.as_u16(),
                        "Retrying request due to server error"
                    );

                    retry_count += 1;
                    sleep(self.config.retry_delay).await;
                }
                Err(e) => {
                    // 处理网络类错误
                    if retry_count >= max_retries {
                        return Err(anyhow!("HTTP request failed after {} retries: {}", max_retries, e));
                    }

                    // 记录错误并重试
                    warn!(
                        retry = retry_count + 1,
                        max_retries = max_retries,
                        error = %e,
                        "Retrying request due to network error"
                    );

                    retry_count += 1;
                    sleep(self.config.retry_delay).await;
                }
            }
        }
    }

    // 简化的GET请求（适用于JSON响应）
    pub async fn get_json<T>(&self, url: &str, query: Option<&HashMap<String, String>>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut builder = self.get(url).await;
        if let Some(params) = query {
            builder = builder.query(params);
        }

        let response = self.send_with_retry(builder).await?;
        response.json::<T>().await.context("Failed to parse response")
    }

    // 简化的POST请求（适用于JSON请求和响应）
    pub async fn post_json<T, U>(&self, url: &str, body: &T) -> Result<U>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned,
    {
        self.request_json("POST", url, Some(body)).await
    }
}




