use actix_web::{
    middleware::{Logger, NormalizePath},
    web, App, HttpServer, HttpResponse, Responder, Error,
};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures_util::future::{ok, Ready, LocalBoxFuture};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Future},
    sync::Arc,
    pin::Pin,
};
use tokio::sync::{Mutex, oneshot};
use tracing::{info, error};
use dotenvy::dotenv;
use std::env;
use std::task::{Context, Poll};

/// **WebService Trait**
pub trait WebService: Send + Sync {
    fn configure(&self, cfg: &mut web::ServiceConfig);
}

/// **通用 Web 服务器**
pub struct WebServer {
    services: Vec<Arc<dyn WebService>>,
    port: u16,
    stop_signal: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WebServer {
    /// **创建 WebServer**
    pub fn new(services: Vec<Arc<dyn WebService>>, port: u16) -> Self {
        Self {
            services,
            port,
            stop_signal: Arc::new(Mutex::new(None)),
        }
    }

    /// **启动服务器**
    pub async fn start(&self) -> std::io::Result<()> {
        let services = self.services.clone();
        let port = self.port;
        let (tx, rx) = oneshot::channel();
        *self.stop_signal.lock().await = Some(tx);

        info!("🚀 WebServer is running on port: {}", port);

        HttpServer::new(move || {
            let mut app = App::new()
                .wrap(Logger::default())  // 请求日志
                .wrap(NormalizePath::trim()); // 处理 URL 末尾斜杠

            for service in &services {
                app = app.configure(|cfg| service.configure(cfg));
            }

            // app.wrap(AuthMiddleware) // JWT 认证
            app
        })
            .bind(("0.0.0.0", port))?
            .run()
            .await?;

        // 监听 `stop` 信号
        let _ = rx.await;
        info!("🛑 Server is shutting down...");
        Ok(())
    }

    /// **停止服务器**
    pub async fn stop(&self) {
        if let Some(tx) = self.stop_signal.lock().await.take() {
            let _ = tx.send(());
        }
    }
}

/// **健康检查 API**
pub struct HealthService;
impl WebService for HealthService {
    fn configure(&self, cfg: &mut web::ServiceConfig) {
        cfg.service(web::resource("/health").route(web::get().to(Self::health_check)));
    }
}
impl HealthService {
    async fn health_check() -> impl Responder {
        HttpResponse::Ok().body("OK")
    }
}

// /// **JWT 认证**
// #[derive(Debug, Serialize, Deserialize)]
// struct Claims {
//     sub: String,
//     exp: usize,
// }

// /// **JWT 认证中间件**
// struct AuthMiddleware;
// impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
//     S::Future: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type InitError = ();
//     type Transform = AuthMiddlewareService<S>;
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;
//
//     fn new_transform(&self, service: S) -> Self::Future {
//         ok(AuthMiddlewareService { service })
//     }
// }

// struct AuthMiddlewareService<S> {
//     service: S,
// }

// impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
//     S::Future: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
//
//     forward_ready!(service);
//
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         let headers = req.headers().clone();
//         let token = headers.get("Authorization").and_then(|h| h.to_str().ok());
//
//         if let Some(token) = token {
//             if let Some(auth) = token.strip_prefix("Bearer ") {
//                 let decoding_key = DecodingKey::from_secret(b"secret");
//                 if decode::<Claims>(auth, &decoding_key, &Validation::default()).is_ok() {
//                     return Box::pin(self.service.call(req));
//                 }
//             }
//         }
//
//         let res = req.into_response(HttpResponse::Unauthorized().finish());
//         Box::pin(async { Ok(res.map_into_right_body()) })
//     }
// }

/// **请求解密**
async fn decrypt_request(req: web::Json<String>) -> impl Responder {
    let encrypted_data = req.into_inner();
    let decrypted_data = aes_decrypt(encrypted_data.as_bytes()); // AES 解密
    HttpResponse::Ok().body(decrypted_data)
}

/// **AES 解密 (示例)**
fn aes_decrypt(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string() // 这里只是模拟解密
}

/// **读取配置**
fn get_port() -> u16 {
    dotenv().ok();
    env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

/// **Web 服务管理**
pub struct WebServerManager {
    server: Arc<Mutex<Option<WebServer>>>,
}

impl WebServerManager {
    pub fn new() -> Self {
        Self {
            server: Arc::new(Mutex::new(None)),
        }
    }

    /// **启动 Web 服务**
    pub async fn start_server(&self) {
        let mut server_lock = self.server.lock().await;
        if server_lock.is_none() {
            let services: Vec<Arc<dyn WebService>> = vec![Arc::new(HealthService)];
            let server = WebServer::new(services, get_port());
            *server_lock = Some(server);
        }
        if let Some(server) = &*server_lock {
            let _ = server.start().await;
        }
    }

    /// **停止 Web 服务**
    pub async fn stop_server(&self) {
        if let Some(server) = &*self.server.lock().await {
            server.stop().await;
        }
    }
}
