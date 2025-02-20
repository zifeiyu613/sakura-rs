use std::collections::HashMap;
use actix_http::h1;
use actix_web::body::MessageBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{dev, http, web, Error};
use futures::{StreamExt};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

// 中间件工厂
pub struct RequestLogger;


/// S 后续服务的类型 即当前中间件讲请求传递给那个服务
/// B 响应体的类型 指定了从服务返回的响应内容的格式
impl<S: 'static, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}

// 中间件实现
pub struct RequestLoggerMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    // 实现了 poll_ready 方法，用于检查服务是否准备好处理请求
    forward_ready!(service);

    // 用于实现具体请求
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        // 获取开始时间
        // let start_time = Instant::now();

        Box::pin(async move {
            // 提取请求体的 Payload
            let (http_req, mut payload) = req.into_parts();

            let mut req_body = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                req_body.extend_from_slice(&chunk);
            }

            let bytes = req_body.freeze();
            //
            // // Handle content types
            if let Some(content_type) = http_req.headers().get(http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()) {
                if content_type.starts_with("application/json") {
                    if let Ok(json_data) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        println!("JSON Payload: {:?}", json_data);
                    } else {
                        println!("Error parsing JSON payload");
                    }
                } else if content_type.starts_with("application/x-www-form-urlencoded") {
                    if let Ok(form_data) = serde_urlencoded::from_bytes::<HashMap<String, String>>(&bytes) {
                        println!("Form Payload: {:?}", form_data);
                    } else {
                        println!("Error parsing form payload");
                    }
                } else {
                    println!("Raw Payload: {} bytes", bytes.len());
                }
            }
            // extract bytes from request body
            // let body = req.extract::<web::Bytes>().await.unwrap();
            // println!("request body (middleware): {body:?}");

            // re-insert body back into request to be used by handlers
            // req.set_payload(bytes_to_payload(body));


            let res = svc.call(ServiceRequest::from_parts(http_req, bytes_to_payload(bytes))).await?;

            // println!("response: {:?}", res.headers());
            Ok(res)
        })
    }
}

fn bytes_to_payload(buf: web::Bytes) -> dev::Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    dev::Payload::from(pl)
}
