use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;
use axum::{
    extract::{Extension, Form},
    response::{IntoResponse, Json},
    routing::post,
    Router,
    http::StatusCode,
};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::marker::PhantomData;

// ===== 基础数据结构 =====  

// 基础请求字段结构  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRequestFields {
    application: String,
    channel: String,
    #[serde(rename = "deviceCode")]
    device_code: String,
    #[serde(rename = "packageName")]
    package_name: String,
    #[serde(rename = "plainText")]
    plain_text: bool,
    source: u32,
    #[serde(rename = "subChannel")]
    sub_channel: String,
    uid: u64,
}

// 加密请求结构  
#[derive(Debug, Deserialize)]
pub struct EncryptedRequest {
    data: String,
    #[serde(default)]
    plain_text: String,
}

// 动态请求模型 - 用于初始解析  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRequest {
    #[serde(flatten)]
    base: BaseRequestFields,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// API响应包装器  
#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}

// ===== 服务接口入参结构(示例) =====  

// 用户信息服务的入参  
#[derive(Debug, Clone)]
pub struct UserInfoParams {
    pub uid: u64,
    pub target_uid: u64,
    pub with_details: bool,
}

// 订单信息服务的入参  
#[derive(Debug, Clone)]
pub struct OrderInfoParams {
    pub order_id: String,
    pub user_id: u64,
    pub include_history: bool,
}

// 默认服务的入参  
#[derive(Debug, Clone)]
pub struct DefaultServiceParams {
    pub operation: String,
    pub data: Value,
}

// ===== 服务处理结果(示例) =====  

// 用户信息服务的结果  
#[derive(Debug, Serialize)]
pub struct UserInfoResult {
    uid: u64,
    target_uid: u64,
    nickname: Option<String>,
    avatar: Option<String>,
    level: u32,
}

// 订单信息服务的结果  
#[derive(Debug, Serialize)]
pub struct OrderInfoResult {
    order_id: String,
    user_id: u64,
    amount: f64,
    status: String,
    create_time: String,
}

// ===== 类型擦除相关类型 =====  

// 服务参数特征 - 作为类型擦除的基础  
pub trait ServiceParams: Debug + Send + Sync {
    fn param_type(&self) -> &'static str;
}

// 为具体参数类型实现ServiceParams特征  
impl ServiceParams for UserInfoParams {
    fn param_type(&self) -> &'static str {
        "UserInfo"
    }
}

impl ServiceParams for OrderInfoParams {
    fn param_type(&self) -> &'static str {
        "OrderInfo"
    }
}

impl ServiceParams for DefaultServiceParams {
    fn param_type(&self) -> &'static str {
        "Default"
    }
}

// ===== DTO处理器特征 =====  

// DTO处理结果 - 类型擦除的结果包装  
pub struct DtoProcessResult {
    // 类型擦除的服务参数  
    params: Box<dyn ServiceParams>,
    // 处理器名称，用于日志和调试  
    handler_name: String,
}

impl DtoProcessResult {
    pub fn new<P: ServiceParams + 'static>(params: P, handler_name: String) -> Self {
        Self {
            params: Box::new(params),
            handler_name,
        }
    }

    pub fn params(&self) -> &dyn ServiceParams {
        self.params.as_ref()
    }

    pub fn handler_name(&self) -> &str {
        &self.handler_name
    }

    // 尝试将参数转换为特定类型  
    pub fn downcast_params<T: ServiceParams + 'static>(&self) -> Option<&T> {
        self.params.as_ref().as_any().downcast_ref::<T>()
    }
}

// 为ServiceParams添加as_any方法以支持类型转换  
pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 组合特征以简化使用  
// pub trait ServiceParams: Debug + Send + Sync + AsAny {
//     fn param_type(&self) -> &'static str;
// }

// DTO处理器特征 - 定义处理DTO的接口  
pub trait DtoHandler: Send + Sync {
    // 处理DTO并返回服务参数  
    fn process(&self, request: &DynamicRequest) -> Result<DtoProcessResult, String>;

    // 处理器名称  
    fn name(&self) -> &str;
}

// 类型化的DTO处理器特征 - 使处理器可以返回具体类型  
pub trait TypedDtoHandler<P: ServiceParams + 'static>: Send + Sync {
    // 处理DTO并返回具体类型的服务参数  
    fn process_typed(&self, request: &DynamicRequest) -> Result<P, String>;

    // 处理器名称  
    fn name(&self) -> &str;
}

// 为所有实现TypedDtoHandler的类型自动实现DtoHandler  
impl<H, P> DtoHandler for H
where
    H: TypedDtoHandler<P>,
    P: ServiceParams + 'static,
{
    fn process(&self, request: &DynamicRequest) -> Result<DtoProcessResult, String> {
        let result = self.process_typed(request)?;
        Ok(DtoProcessResult::new(result, self.name().to_string()))
    }

    fn name(&self) -> &str {
        TypedDtoHandler::<P>::name(self)
    }
}

// ===== 处理器注册表 =====  

// 处理器注册表 - 管理所有DTO处理器  
pub struct HandlerRegistry {
    // 处理器映射，键为DTO类型标识符  
    handlers: HashMap<String, Box<dyn DtoHandler>>,

    // LRU缓存，键是请求DTO类型，值是对应的处理器键  
    cache: Mutex<LruCache<String, String>>,
}

impl HandlerRegistry {
    pub fn new(cache_size: usize) -> Self {
        Self {
            handlers: HashMap::new(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
        }
    }

    // 注册新的处理器  
    pub fn register<H>(&mut self, key: &str, handler: H)
    where
        H: DtoHandler + 'static,
    {
        self.handlers.insert(key.to_string(), Box::new(handler));
    }

    // 设置默认处理器  
    pub fn set_default_handler<H>(&mut self, handler: H)
    where
        H: DtoHandler + 'static,
    {
        self.handlers.insert("__default__".to_string(), Box::new(handler));
    }

    // 查找DTO键  
    fn find_dto_key(&self, data: &DynamicRequest) -> Option<String> {
        for key in data.extra.keys() {
            if key.contains("DTO") {
                return Some(key.clone());
            }
        }
        None
    }

    // 查找匹配的处理器  
    pub fn find_handler<'a>(&'a self, data: &DynamicRequest) -> &'a Box<dyn DtoHandler> {
        // 1. 找到DTO键  
        let dto_key = match self.find_dto_key(data) {
            Some(key) => key,
            None => return self.handlers.get("__default__").unwrap(),
        };

        // 2. 检查缓存  
        let mut cache = self.cache.lock().unwrap();
        if let Some(handler_key) = cache.get(&dto_key) {
            if let Some(handler) = self.handlers.get(handler_key) {
                return handler;
            }
        }

        // 3. 缓存未命中，直接查找  
        if let Some(handler) = self.handlers.get(&dto_key) {
            // 更新缓存  
            cache.put(dto_key.clone(), dto_key.clone());
            return handler;
        }

        // 4. 没有找到匹配的处理器，使用默认处理器  
        self.handlers.get("__default__").unwrap()
    }
}

// ===== 服务处理接口 =====  

// 服务处理接口特征 - 定义处理业务逻辑的接口  
pub trait ServiceHandler<P, R>
where
    P: ServiceParams,
    R: Serialize,
{
    // 处理业务逻辑并返回结果  
    fn handle(&self, params: P) -> Result<R, String>;
}

// 服务处理器基础结构  
pub struct ServiceHandlerBase<P, R, F>
where
    P: ServiceParams,
    R: Serialize,
    F: Fn(P) -> Result<R, String> + Send + Sync,
{
    handler_fn: F,
    _params_type: PhantomData<P>,
    _result_type: PhantomData<R>,
}

impl<P, R, F> ServiceHandlerBase<P, R, F>
where
    P: ServiceParams,
    R: Serialize,
    F: Fn(P) -> Result<R, String> + Send + Sync,
{
    pub fn new(handler_fn: F) -> Self {
        Self {
            handler_fn,
            _params_type: PhantomData,
            _result_type: PhantomData,
        }
    }
}

impl<P, R, F> ServiceHandler<P, R> for ServiceHandlerBase<P, R, F>
where
    P: ServiceParams + Clone,  // 需要Clone以便在downcast后克隆  
    R: Serialize,
    F: Fn(P) -> Result<R, String> + Send + Sync,
{
    fn handle(&self, params: P) -> Result<R, String> {
        (self.handler_fn)(params)
    }
}

// 服务处理注册表  
pub struct ServiceRegistry {
    // 用户信息服务  
    user_info_service: Box<dyn ServiceHandler<UserInfoParams, UserInfoResult> + Send + Sync>,

    // 订单信息服务  
    order_info_service: Box<dyn ServiceHandler<OrderInfoParams, OrderInfoResult> + Send + Sync>,

    // 默认服务  
    default_service: Box<dyn Fn(DefaultServiceParams) -> Result<Value, String> + Send + Sync>,
}

impl ServiceRegistry {
    pub fn new(
        user_info_service: impl ServiceHandler<UserInfoParams, UserInfoResult> + Send + Sync + 'static,
        order_info_service: impl ServiceHandler<OrderInfoParams, OrderInfoResult> + Send + Sync + 'static,
        default_service: impl Fn(DefaultServiceParams) -> Result<Value, String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            user_info_service: Box::new(user_info_service),
            order_info_service: Box::new(order_info_service),
            default_service: Box::new(default_service),
        }
    }

    // 处理服务请求并返回JSON响应  
    pub fn handle_request(&self, process_result: DtoProcessResult) -> Result<Value, String> {
        match process_result.params().param_type() {
            "UserInfo" => {
                if let Some(params) = process_result.downcast_params::<UserInfoParams>() {
                    // 克隆参数以便传递给服务处理器  
                    let result = self.user_info_service.handle(params.clone())?;
                    Ok(serde_json::to_value(result).unwrap())
                } else {
                    Err("无法转换为UserInfoParams".to_string())
                }
            },
            "OrderInfo" => {
                if let Some(params) = process_result.downcast_params::<OrderInfoParams>() {
                    let result = self.order_info_service.handle(params.clone())?;
                    Ok(serde_json::to_value(result).unwrap())
                } else {
                    Err("无法转换为OrderInfoParams".to_string())
                }
            },
            "Default" => {
                if let Some(params) = process_result.downcast_params::<DefaultServiceParams>() {
                    (self.default_service)(params.clone())
                } else {
                    Err("无法转换为DefaultServiceParams".to_string())
                }
            },
            _ => Err(format!("未知的参数类型: {}", process_result.params().param_type())),
        }
    }
}

// ===== DTO处理器实现 =====  

// UserInfoDTO处理器  
pub struct UserInfoDtoHandler;

impl TypedDtoHandler<UserInfoParams> for UserInfoDtoHandler {
    fn process_typed(&self, request: &DynamicRequest) -> Result<UserInfoParams, String> {
        // 从请求中提取必要信息  
        let target_uid = request.extra.get("userInfoDTO")
            .and_then(|v| v.get("tarUid"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "缺少tarUid字段".to_string())?;

        Ok(UserInfoParams {
            uid: request.base.uid,
            target_uid,
            with_details: true,  // 默认值或从DTO中提取  
        })
    }

    fn name(&self) -> &str {
        "UserInfoDtoHandler"
    }
}

// UserInfoDTO1处理器  
pub struct UserInfoDto1Handler;

impl TypedDtoHandler<UserInfoParams> for UserInfoDto1Handler {
    fn process_typed(&self, request: &DynamicRequest) -> Result<UserInfoParams, String> {
        let target_uid = request.extra.get("userInfoDTO1")
            .and_then(|v| v.get("tarUid"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "缺少tarUid字段".to_string())?;

        Ok(UserInfoParams {
            uid: request.base.uid,
            target_uid,
            with_details: false,  // DTO1可能有不同的默认值  
        })
    }

    fn name(&self) -> &str {
        "UserInfoDto1Handler"
    }
}

// OrderDTO处理器  
pub struct OrderDtoHandler;

impl TypedDtoHandler<OrderInfoParams> for OrderDtoHandler {
    fn process_typed(&self, request: &DynamicRequest) -> Result<OrderInfoParams, String> {
        let order_id = request.extra.get("orderDTO")
            .and_then(|v| v.get("orderId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少orderId字段".to_string())?;

        Ok(OrderInfoParams {
            order_id: order_id.to_string(),
            user_id: request.base.uid,
            include_history: true,
        })
    }

    fn name(&self) -> &str {
        "OrderDtoHandler"
    }
}

// 默认处理器  
pub struct DefaultDtoHandler;

impl TypedDtoHandler<DefaultServiceParams> for DefaultDtoHandler {
    fn process_typed(&self, request: &DynamicRequest) -> Result<DefaultServiceParams, String> {
        Ok(DefaultServiceParams {
            operation: "unknown".to_string(),
            data: json!({  
                "uid": request.base.uid,  
                "available_fields": request.extra.keys().collect::<Vec<_>>(),  
            }),
        })
    }

    fn name(&self) -> &str {
        "DefaultDtoHandler"
    }
}

// ===== 服务处理器实现 =====  

// 用户信息服务实现  
fn handle_user_info(params: UserInfoParams) -> Result<UserInfoResult, String> {
    // 这里应该是实际的业务逻辑  
    println!("处理用户信息请求: uid={}, target_uid={}", params.uid, params.target_uid);

    Ok(UserInfoResult {
        uid: params.uid,
        target_uid: params.target_uid,
        nickname: Some("测试用户".to_string()),
        avatar: Some("https://example.com/avatar.jpg".to_string()),
        level: 10,
    })
}

// 订单信息服务实现  
fn handle_order_info(params: OrderInfoParams) -> Result<OrderInfoResult, String> {
    // 这里应该是实际的业务逻辑  
    println!("处理订单信息请求: order_id={}, user_id={}", params.order_id, params.user_id);

    Ok(OrderInfoResult {
        order_id: params.order_id,
        user_id: params.user_id,
        amount: 99.99,
        status: "已付款".to_string(),
        create_time: "2023-06-01 12:00:00".to_string(),
    })
}

// 默认服务实现  
fn handle_default(params: DefaultServiceParams) -> Result<Value, String> {
    println!("处理默认请求: operation={}", params.operation);

    Ok(json!({  
        "message": "未知的请求类型",  
        "operation": params.operation,  
        "data": params.data,  
    }))
}

// ===== 解密和请求处理 =====  

// 解密函数（模拟）  
fn decrypt_data(encrypted_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 在实际应用中，这里应该实现解密操作  
    Ok(r#"{"application":"yice","channel":"TEST_CHANNEL","deviceCode":"HONOR-DUK-AL20","packageName":"com.kaiqi.yice","plainText":false,"source":1,"subChannel":"TEST_SUB_CHANNEL","uid":1, "userInfoDTO":{"tarUid":100}}"#.to_string())
}

// 解析加密请求  
fn parse_request(encrypted_req: &EncryptedRequest) -> Result<DynamicRequest, String> {
    // 检查是否是明文模式  
    let is_plain_text = encrypted_req.plain_text == "true";

    // 解密数据  
    let json_data = if is_plain_text {
        encrypted_req.data.clone()
    } else {
        match decrypt_data(&encrypted_req.data) {
            Ok(data) => data,
            Err(e) => return Err(format!("解密失败: {}", e)),
        }
    };

    // 解析为动态请求模型  
    let dynamic_req: DynamicRequest = match serde_json::from_str(&json_data) {
        Ok(req) => req,
        Err(e) => return Err(format!("JSON解析错误: {}", e)),
    };

    Ok(dynamic_req)
}

// ===== Web接口处理 =====  

// 请求处理管道 - 连接DTO处理和服务处理  
async fn process_request(
    dynamic_req: DynamicRequest,
    handler_registry: &HandlerRegistry,
    service_registry: &ServiceRegistry,
) -> Result<Value, (StatusCode, String)> {
    // 1. 查找DTO处理器  
    let handler = handler_registry.find_handler(&dynamic_req);
    println!("使用处理器: {}", handler.name());

    // 2. 处理DTO，生成服务参数  
    let process_result = handler.process(&dynamic_req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    println!("DTO处理完成，参数类型: {}", process_result.params().param_type());

    // 3. 调用服务，处理业务逻辑  
    let result = service_registry.handle_request(process_result)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(result)
}

// 处理表单请求的Axum处理函数  
async fn form_handler(
    Form(encrypted_req): Form<EncryptedRequest>,
    Extension(handler_registry): Extension<Arc<HandlerRegistry>>,
    Extension(service_registry): Extension<Arc<ServiceRegistry>>,
) -> impl IntoResponse {
    // 解析请求  
    let dynamic_req = match parse_request(&encrypted_req) {
        Ok(req) => req,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<Value>::error(400, &e))
            );
        }
    };

    // 处理请求  
    match process_request(dynamic_req, &handler_registry, &service_registry).await {
        Ok(result) => {
            (
                StatusCode::OK,
                Json(ApiResponse::success(result))
            )
        },
        Err((status, error)) => {
            (
                status,
                Json(ApiResponse::<Value>::error(status.as_u16() as i32, &error))
            )
        }
    }
}

// ===== 服务初始化和注册处理器 =====  

// 应用初始化函数  
fn initialize_app() -> Router {
    // 创建并配置处理器注册表  
    let mut handler_registry = HandlerRegistry::new(100);

    // 设置默认处理器  
    handler_registry.set_default_handler(DefaultDtoHandler);

    // 注册所有DTO处理器  
    handler_registry.register("userInfoDTO", UserInfoDtoHandler);
    handler_registry.register("userInfoDTO1", UserInfoDto1Handler);
    handler_registry.register("orderDTO", OrderDtoHandler);

    // 创建共享的处理器注册表  
    let shared_handler_registry = Arc::new(handler_registry);

    // 创建服务处理器  
    let user_info_service = ServiceHandlerBase::new(handle_user_info);
    let order_info_service = ServiceHandlerBase::new(handle_order_info);

    // 创建服务注册表  
    let service_registry = ServiceRegistry::new(
        user_info_service,
        order_info_service,
        handle_default,
    );

    // 创建共享的服务注册表  
    let shared_service_registry = Arc::new(service_registry);

    // 构建Axum路由  
    Router::new()
        .route("/api", post(form_handler))
        .layer(Extension(shared_handler_registry))
        .layer(Extension(shared_service_registry))
}

// 主函数  
#[tokio::main]
async fn main() {
    // 初始化日志  
    tracing_subscriber::fmt::init();

    // 初始化应用  
    let app = initialize_app();

    // 启动服务器  
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::info!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}