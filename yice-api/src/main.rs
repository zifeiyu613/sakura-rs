use tracing::log::info;
use crate::app::create_app;

mod infrastructure;
mod service;
mod api;
mod app;
mod config;
mod error;
mod middleware;

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)  // 显示日志来源
        .with_thread_ids(true)  // 显示线程ID
        .init();

    let app = create_app().await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("server started on port 3000");
    info!("listening on port 3000");
    axum::serve(listener, app).await.unwrap();

}

