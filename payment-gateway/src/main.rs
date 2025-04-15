mod domain;
mod infrastructure;

use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

use payment_gateway::interfaces::api;
use payment_gateway::infrastructure::config::AppState;
use payment_gateway::infrastructure::database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载环境变量  
    dotenv().ok();
    env_logger::init();

    log::info!("Starting payment gateway server...");

    // 创建数据库连接池  
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let max_connections = env::var("MAX_DB_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u32>()
        .expect("Failed to parse MAX_DB_CONNECTIONS");

    log::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // 数据库迁移  
    database::run_migrations(&pool).await
        .expect("Failed to run database migrations");

    // 创建应用状态  
    let app_state = web::Data::new(AppState::new(pool));

    // 获取服务器配置  
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    log::info!("Starting server on {}:{}", host, port);

    // 启动HTTP服务器  
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .service(
                web::scope("/api")
                    .configure(api::configure_routes)
            )
    })
        .bind((host, port))?
        .run()
        .await
}