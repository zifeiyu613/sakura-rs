use actix_web::{
    middleware::{Logger, NormalizePath},
    web, App, HttpServer, HttpResponse, Responder, Error,
};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Future},
    pin::Pin,
};
use tokio::sync::{Mutex, oneshot};
use tracing::{info, error};
use std::env;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use lazy_static::lazy_static;
use sakura_macros::service;


/** **WebService Trait** */
pub trait WebService: Send + Sync {
    fn configure(&self, cfg: &mut web::ServiceConfig);

}


// // 全局服务注册表，使用 RwLock 确保线程安全
// lazy_static! {
//     pub static ref SERVICES: RwLock<Vec<Arc<dyn WebService>>> = RwLock::new(Vec::new());
// }
//
// // 服务注册函数
// pub fn register_service(service: Arc<dyn WebService>) {
//     SERVICES.write().unwrap().push(service);
// }

/// **通用 Web 服务器**
pub struct WebServer {
    // services: Vec<Arc<dyn WebService>>,
    port: u16,
    stop_signal: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WebServer {
    /// **创建 WebServer**
    pub fn new(port: u16) -> Self {

        Self {
            // services,
            port,
            stop_signal: Arc::new(Mutex::new(None)),
        }
    }

    /// **启动服务器**
    pub async fn start(&self) -> std::io::Result<()> {
        // let services = self.services.clone();
        let port = self.port;
        let (tx, rx) = oneshot::channel();
        *self.stop_signal.lock().await = Some(tx);

        HttpServer::new(move || {
            let mut app = App::new()
                .wrap(Logger::default())  // 请求日志
                .wrap(NormalizePath::trim()); // 处理 URL 末尾斜杠

            let service_count = inventory::iter::<&dyn WebService>().count();
            println!("service_count:{}", service_count);

            for service in inventory::iter::<&dyn WebService>.into_iter() {
                app = app.configure(|cfg| service.configure(cfg));
            }

            // app.wrap(AuthMiddleware) // JWT 认证
            app
        })
            .bind(("0.0.0.0", port))?
            .run().await?;

        info!("🚀 WebServer is running on port: {}", port);
        println!("🚀 WebServer is running on port: {}", port);

        // 等待 stop 信号
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
#[service]
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



/// **Web 服务管理**
pub struct WebServerManager {
    server: Arc<Mutex<Option<WebServer>>>,
}

impl WebServerManager {

    pub fn new(port: u16) -> Self {
        Self {
            server: Arc::new(Mutex::new(Some(WebServer::new(port)))),
        }
    }

    /// **启动 Web 服务**
    pub async fn start_server(&self) -> std::io::Result<()> {
        let server_lock = self.server.lock().await;
        if let Some(server) = &*server_lock {
            server.start().await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Server not initialized"
            ))
        }
    }

    /// **停止 Web 服务**
    pub async fn stop_server(&self) {
        println!("Stopping server...");
        if let Some(server) = &*self.server.lock().await {
            server.stop().await;
            println!("Stopping server successfully ...");
        }
    }

}

inventory::collect!(&'static dyn WebService);
