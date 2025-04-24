use std::collections::HashMap;
use std::marker::PhantomData;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{Form, Json, RequestExt};
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use tracing::log::{debug, warn};
use crate::errors::{ApiError, BusinessCode};
use crate::errors::response::ApiResponse;
use crate::middleware::decryptor::{BaseRequestFields, RequestData};


// 嵌套字段特征 - 为每种嵌套DTO类型定义字段名
pub trait NestedField {
    fn field_name() -> &'static str;
}


/// ApiRequest提取器，可从解密后的数据中提取
#[derive(Debug)]
pub struct ApiRequest<N>
where
    N: DeserializeOwned + Default + NestedField,
{
    pub base: Option<BaseRequestFields>,   // 基础字段
    pub nested: Option<N>,                 // 嵌套对象
    pub raw_json: Option<Value>,           // 原始JSON
    pub request_data: Option<RequestData>, // 请求元数据，包括加密/解密信息
    _nested_type: PhantomData<N>,
}

impl<N> Default for ApiRequest<N>
where
    N: DeserializeOwned + Default + NestedField,
{
    fn default() -> Self {
        Self {
            base: None,
            nested: None,
            raw_json: None,
            request_data: None,
            _nested_type: PhantomData,
        }
    }
}

impl<S, N> FromRequest<S> for ApiRequest<N>
where
    S: Send + Sync,
    N: DeserializeOwned + Default + Send + NestedField,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let mut api_request = Self::default();

        // 获取嵌套对象的字段名
        let nested_field_name = N::field_name();

        // 获取请求元数据
        api_request.request_data = req.extensions().get::<RequestData>().cloned();

        // 尝试从请求扩展中获取已解析的JSON数据 - 统一入口
        if let Some(json) = req.extensions().get::<Value>() {
            // 使用解密中间件已解析的JSON
            let json = json.clone();
            api_request.raw_json = Some(json.clone());

            // 解析基础字段
            api_request.base = match serde_json::from_value::<BaseRequestFields>(json.clone()) {
                Ok(base) => Some(base),
                Err(e) => {
                    debug!("解析基础字段失败: {}", e);
                    return Err(ApiError::business_with_message(BusinessCode::ValidationError,format!("请求失败! 公参 -> {}", e)))
                }
            };

            // 解析嵌套对象
            if let Some(nested_obj) = json.get(nested_field_name) {
                api_request.nested = match serde_json::from_value::<N>(nested_obj.clone()) {
                    Ok(nested) => Some(nested),
                    Err(e) => {
                        debug!("解析嵌套对象{}失败: {}", nested_field_name, e);
                        // None
                        return Err(ApiError::business_with_message(BusinessCode::ValidationError,format!("请求失败! {} -> {}", nested_field_name, e)))
                    }
                };
            } else {
                // 嵌套字段不存在，使用默认值
                debug!("JSON中不存在嵌套对象字段'{}'", nested_field_name);
                // api_request.nested = Some(N::default());
                return Err(ApiError::business_with_message(BusinessCode::ValidationError,format!("{} 对象字段不存在", nested_field_name)))
            }
        } else {
            // 解密中间件应该已经处理过所有请求，如果没有找到解析后的JSON，记录错误
            warn!("没有在请求扩展中找到已解析的JSON数据，这可能意味着解密中间件未正确处理请求");
            return Err(ApiError::business(BusinessCode::BadRequest))
        }

        Ok(api_request)
    }
}

// 支持多个嵌套对象的版本
#[derive(Debug)]
pub struct MultiNestedRequest {
    pub base: Option<BaseRequestFields>,
    pub nested_objects: HashMap<String, Value>,
    pub raw_json: Option<Value>,
    pub request_data: Option<RequestData>,
}

impl Default for MultiNestedRequest {
    fn default() -> Self {
        Self {
            base: None,
            nested_objects: HashMap::new(),
            raw_json: None,
            request_data: None,
        }
    }
}

impl MultiNestedRequest {
    // 获取指定类型的嵌套对象
    pub fn get_nested<N>(&self) -> Option<N>
    where
        N: DeserializeOwned + NestedField,
    {
        let field_name = N::field_name();
        self.nested_objects
            .get(field_name)
            .and_then(|value| serde_json::from_value::<N>(value.clone()).ok())
    }

    // 检查是否包含特定嵌套对象
    pub fn has_nested<N>(&self) -> bool
    where
        N: NestedField,
    {
        self.nested_objects.contains_key(N::field_name())
    }
}

impl<S> FromRequest<S> for MultiNestedRequest
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let mut api_request = Self::default();

        // 获取请求元数据
        api_request.request_data = req.extensions().get::<RequestData>().cloned();

        // 尝试从请求扩展中获取已解析的JSON数据 - 统一入口
        if let Some(json) = req.extensions().get::<Value>() {
            // 使用解密中间件已解析的JSON
            let json = json.clone();
            api_request.raw_json = Some(json.clone());

            // 解析基础字段
            api_request.base = serde_json::from_value::<BaseRequestFields>(json.clone()).ok();

            // 收集所有对象字段
            if let Value::Object(map) = &json {
                for (key, value) in map {
                    if value.is_object() {
                        api_request.nested_objects.insert(key.clone(), value.clone());
                    }
                }
            }
        } else {
            // 解密中间件应该已经处理过所有请求，如果没有找到解析后的JSON，记录错误
            warn!("没有在请求扩展中找到已解析的JSON数据，这可能意味着解密中间件未正确处理请求");
            return Err(StatusCode::BAD_REQUEST);
        }

        Ok(api_request)
    }
}