mod app;
mod service;
mod repository;

use web_core::web_service::WebServerManager;


#[tokio::main]
async fn main() {
    let manager = WebServerManager::new();
    manager.start_server().await;
}



pub async fn stop() {
    let manager = WebServerManager::new();
    manager.stop_server().await;
}