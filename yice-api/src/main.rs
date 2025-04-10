use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::log::info;
use tracing_subscriber::EnvFilter;
use yice_api::server::create_app;

#[tokio::main]
async fn main() {

    let sqlx_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug,sqlx=debug"));

    tracing_subscriber::fmt()
        .with_target(true)  // 显示日志来源
        .with_thread_ids(false)  // 显示线程ID
        .with_env_filter(sqlx_filter)
        .init();

    let app = create_app().await.unwrap();
    // 处理未定义Paths
    let app= app.fallback(handler_404);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("server started on port 3000");
    info!("listening on port 3000");
    axum::serve(listener, app).await.unwrap();

}


async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
