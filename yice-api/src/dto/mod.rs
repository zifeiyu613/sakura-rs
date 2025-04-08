use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dto::base_request::{BaseRequest, DynamicRequest};
use crate::dto::userinfo_dto::UserInfoDTO;

pub(crate) mod base_request;
pub(crate) mod userinfo_dto;
pub(crate) mod order_dto;
mod response;


// // DTO处理接口特征
// trait DtoHandler: Send + Sync {
//     // 关联类型，表示处理器返回的具体类型
//     type Output: Serialize + Debug;
//
//     // 处理请求并返回具体类型
//     fn handle(&self, data: &DynamicRequest) -> Result<Self::Output, String>;
//
//     // 获取处理器名称（用于调试）
//     fn name(&self) -> &str;
// }
//
// // 类型擦除的处理结果 - 用于存储不同类型的处理结果
// enum HandlerResult {
//     UserInfo(UserInfoDTO),
//     // OrderInfo(OrderInfoResponse),
//     Error(String),
// }
//
//
// // 类型擦除的处理器封装
// struct BoxedDtoHandler {
//     name: String,
//     handler_fn: Box<dyn Fn(&DynamicRequest) -> HandlerResult + Send + Sync>,
// }
//
// impl BoxedDtoHandler {
//     fn new<H>(name: &str, handler: H) -> Self
//     where
//         H: DtoHandler + 'static,
//     {
//         Self {
//             name: name.to_string(),
//             handler_fn: Box::new(move |req| {
//                 match handler.handle(req) {
//                     Ok(output) => {
//                         // 根据具体类型转换为HandlerResult
//                         match name  {
//                             "UserInfoDtoHandler" | "UserInfoDto1Handler" | "UserInfoDto3Handler" => {
//                                 // 此处假设Output是UserInfoResponse
//                                 let json = serde_json::to_value(&output).unwrap();
//                                 let user_info: UserInfoDTO = serde_json::from_value(json).unwrap();
//                                 HandlerResult::UserInfo(user_info)
//                             },
//                             // "OrderDtoHandler" => {
//                             //     // 此处假设Output是OrderInfoResponse
//                             //     let json = serde_json::to_value(&output).unwrap();
//                             //     let order_info: OrderInfoResponse = serde_json::from_value(json).unwrap();
//                             //     HandlerResult::OrderInfo(order_info)
//                             // },
//                             _ => HandlerResult::Error("未知的处理器类型".to_string()),
//                         }
//                     },
//                     Err(e) => HandlerResult::Error(e),
//                 }
//             }),
//         }
//     }
//
//     fn handle(&self, req: &DynamicRequest) -> HandlerResult {
//         (self.handler_fn)(req)
//     }
//
//     fn name(&self) -> &str {
//         &self.name
//     }
// }
//
// // 缓存优化的处理器注册表
// struct HandlerRegistry {
//     // 处理器映射，键为DTO类型标识符
//     handlers: HashMap<String, BoxedDtoHandler>,
//
//     // 默认处理器
//     default_handler: BoxedDtoHandler,
//
//     // LRU缓存，键是请求特征哈希，值是对应的处理器键
//     cache: Mutex<LruCache<String, String>>,
// }
//
//
//
// impl HandlerRegistry {
//     fn new(default_handler: BoxedDtoHandler, cache_size: usize) -> Self {
//         Self {
//             handlers: HashMap::new(),
//             default_handler,
//             cache: Mutex::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
//         }
//     }
//
//     // 注册新的处理器
//     fn register<H>(&mut self, key: &str, handler: H)
//     where
//         H: DtoHandler + 'static,
//     {
//         let handler_name = handler.name().to_string();
//         let boxed_handler = BoxedDtoHandler::new(&handler_name, handler);
//         self.handlers.insert(key.to_string(), boxed_handler);
//     }
//
//     // 查找DTO键
//     fn find_dto_key(&self, data: &DynamicRequest) -> Option<String> {
//         for key in data.extra.keys() {
//             if key.contains("DTO") {
//                 return Some(key.clone());
//             }
//         }
//         None
//     }
//
//     // 查找匹配的处理器
//     fn find_handler<'a>(&'a self, data: &DynamicRequest) -> &'a BoxedDtoHandler {
//         // 1. 找到DTO键
//         let dto_key = match self.find_dto_key(data) {
//             Some(key) => key,
//             None => return &self.default_handler,
//         };
//
//         // 2. 检查缓存
//         let mut cache = self.cache.lock().unwrap();
//         if let Some(handler_key) = cache.get(&dto_key) {
//             if let Some(handler) = self.handlers.get(handler_key) {
//                 return handler;
//             }
//         }
//
//         // 3. 缓存未命中，直接查找
//         if let Some(handler) = self.handlers.get(&dto_key) {
//             // 更新缓存
//             cache.put(dto_key.clone(), dto_key.clone());
//             return handler;
//         }
//
//         // 4. 没有找到匹配的处理器，使用默认处理器
//         &self.default_handler
//     }
// }