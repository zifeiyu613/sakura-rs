use std::collections::HashMap;
use std::pin::Pin;
use std::future::{ready, Ready, Future};
use std::sync::Arc;
use actix_web::{dev, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, error, Error, HttpMessage, HttpResponse};
use serde_json::Value;
use actix_web::web;
use futures::StreamExt;
use actix_http::h1;
use actix_multipart::Multipart;
use super::request_context::{RequestContext, FormData};

pub struct RequestExtractor;

impl Default for RequestExtractor {
    fn default() -> Self {
        Self
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for RequestExtractor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestExtractorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestExtractorMiddleware { service: Arc::new(service) }))
    }
}

pub struct RequestExtractorMiddleware<S> {
     // This is special: We need this to avoid lifetime issues.
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestExtractorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let mut context = RequestContext::new();

        // 提取 header 参数
        if let Some(token) = req.headers().get("Authorization") {
            context.token = token.to_str().ok().map(|s| s.to_string());
        }

        if let Some(user_id) = req.headers().get("X-User-Id") {
            context.user_id = user_id.to_str().ok().map(|s| s.to_string());
        }

        // 提取客户端 IP
        context.client_ip = req.connection_info().realip_remote_addr()
            .map(|s| s.to_string());

        // 提取 User-Agent
        context.user_agent = req.headers().get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let svc = Arc::clone(&self.service);

        // Clone necessary data for async block
        let content_type = req.headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        // Split request into parts
        Box::pin(async move {
             let (http_req, payload) = req.into_parts();

            // Handle different content types
            if let Some(content_type) = content_type {
                if content_type.starts_with("application/json") {
                    // Handle JSON content
                    // let mut body = web::BytesMut::new();
                    let body = extract_body_with_limit(payload, 1024 * 1024).await?;
                    if let Ok(json) = serde_json::from_slice::<Value>(&body) {
                        println!("application/json: {:?}", json);
                        context.form_data = serde_json::from_value::<FormData>(json.clone()).ok();
                    }
                    req = ServiceRequest::from_parts(http_req, bytes_to_payload(body));
                }
                // Add support for form-urlencoded if needed
                else if content_type.starts_with("application/x-www-form-urlencoded") {
                    let body = extract_body_with_limit(payload, 1024 * 1024).await?;
                    if let Ok(form_data) = serde_urlencoded::from_bytes::<FormData>(&body) {
                        println!("application/x-www-form-urlencoded: {:?}", form_data);
                        context.form_data = Some(form_data);
                    }

                    req = ServiceRequest::from_parts(http_req, bytes_to_payload(body));
                } else if content_type.starts_with("multipart/form-data") {
                    let multipart = Multipart::new(http_req.headers(), payload);
                    context.form_data = Some(handle_multipart(multipart).await?);
                    println!("application/form-data: {:?}", context.form_data);
                    // 重新组装请求体为空 Payload
                    req = ServiceRequest::from_parts(http_req, bytes_to_payload(web::Bytes::new()));
                } else {
                    println!("content_type: {:?}", &content_type);
                    req = ServiceRequest::from_parts(http_req, payload);
                }
            } else {
                println!("no content_type");
                // If no content-type, just reconstruct the request
                req = ServiceRequest::from_parts(http_req, payload);
            }

            // Insert context into request extensions
            req.extensions_mut().insert(context);

            // Call the next service in the chain
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}

fn bytes_to_payload(buf: web::Bytes) -> dev::Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    dev::Payload::from(pl)
}

/**
* payload 请求体
* max_size 请求体大小限制。例如限制为 1MB：
**/
async fn extract_body_with_limit(mut payload: dev::Payload, max_size: usize) -> Result<web::Bytes, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > max_size {
            // return Err(actix_web::error::ErrorBadRequest("payload too large"));
            return Err(error::ErrorPayloadTooLarge("payload too large"));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body.freeze())
}

// 处理 multipart/form-data 的字段和文件
async fn handle_multipart(mut multipart: Multipart) -> Result<FormData, Error> {
    let mut form_data = FormData::default();

    while let Some(field) = multipart.next().await {
        let mut field = field?;

        let name = field.name().unwrap().to_string();

        let mut data = web::BytesMut::new();
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(&chunk?);
        }

        if field.content_disposition().is_some() && field.content_type().is_some() {
            // 文件字段
            form_data.files.insert(name, data.freeze());
        } else {
            // 普通表单字段
            if let Ok(text) = String::from_utf8(data.to_vec()) {
                let mut fields = HashMap::new();
                fields.insert(name, text);
                form_data.fields = Some(fields);
            }
        }
    }

    Ok(form_data)
}



/// 测试中间件是否正确处理并保留请求体。
#[cfg(test)]
mod tests {
    use actix_http::HttpMessage;
    use actix_web::{test, web, App, HttpRequest, HttpResponse};
    use crate::{RequestContext, RequestExtractor};

    #[actix_web::test]
    async fn test_multipart_form_data() {
        let app = test::init_service(
            App::new()
                .wrap(RequestExtractor::default())
                .route(
                    "/upload",
                    web::post().to(|req: HttpRequest| async move {
                        let context: RequestContext = req.extensions().get::<RequestContext>().unwrap().clone();
                        assert!(context.form_data.is_some()); // 验证解析是否成功
                        println!("{:?}", context);
                        HttpResponse::Ok().finish()
                    }),
                ),
        )
            .await;

        let payload = "--boundary\r\n\
                   Content-Disposition: form-data; name=\"username\"\r\n\r\n\
                   john_doe\r\n\
                   --boundary\r\n\
                   Content-Disposition: form-data; name=\"file\"; filename=\"file.txt\"\r\n\
                   Content-Type: text/plain\r\n\r\n\
                   hello world\r\n\
                   --boundary--\r\n";

        let req = test::TestRequest::post()
            .uri("/upload")
            .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
            .set_payload(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}