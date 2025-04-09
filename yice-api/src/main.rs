use tracing::log::info;
use tracing_subscriber::EnvFilter;
use yice_api::server::create_app;

#[tokio::main]
async fn main() {

    let sqlx_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=debug"));

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)  // 显示日志来源
        .with_thread_ids(true)  // 显示线程ID
        .with_env_filter(sqlx_filter)
        .init();

    let app = create_app().await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("server started on port 3000");
    info!("listening on port 3000");
    axum::serve(listener, app).await.unwrap();

}

