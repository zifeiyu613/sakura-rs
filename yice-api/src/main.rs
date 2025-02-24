mod app;
mod service;
mod repository;
mod controller;

use inventory::collect;
use web_core::web_service::{WebServerManager, WebService};


#[tokio::main]
async fn main() {
    let manager = WebServerManager::new(8080);
    manager.start_server().await.expect("TODO: panic message");
}



pub async fn stop() {
    let manager = WebServerManager::new(8080);
    manager.stop_server().await;
}