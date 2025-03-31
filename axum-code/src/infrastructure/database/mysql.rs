use sqlx::MySqlPool;

use crate::config::Config;
use crate::error::AppError;

pub async fn init_mysql(config: &Config) -> Result<MySqlPool, AppError> {
    tracing::info!("Initializing MySQL connection pool");

    let pool = MySqlPool::connect_with(
        sqlx::mysql::MySqlConnectOptions::new()
            .host(&config.database.url.split(':').next().unwrap_or("localhost"))
            .username(&config.database.url.split('@').next().unwrap_or("root"))
            .password("password") // In production, get from config securely
            .database("api_service")
            .ssl_mode(sqlx::mysql::MySqlSslMode::Preferred)
            .to_owned()
    )
        .await?;

    // 运行迁移
    // sqlx::migrate!("./src/infrastructure/database/migrations")
    //     .run(&pool)
    //     .await?;

    Ok(pool)
}
