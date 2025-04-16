use crate::config::AppConfig;
use anyhow::{Result, Context};
use sqlx::postgres::{PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

pub async fn init_database(config: &AppConfig) -> Result<PgPool> {
    info!("Initializing database connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(Duration::from_secs(config.database.connection_timeout))
        .connect(&config.database.url)
        .await
        .context("Failed to connect to database")?;

    // 测试连接
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .context("Failed to execute test query")?;

    info!("Database connection pool initialized successfully");

    // 运行数据库迁移
    if !config.is_testing() {
        info!("Running database migrations");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run database migrations")?;
        info!("Database migrations completed successfully");
    }

    Ok(pool)
}

// 获取测试用数据库连接（用于集成测试）
#[cfg(test)]
pub async fn get_test_db_pool() -> Result<PgPool> {
    use dotenv::dotenv;
    dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/payment_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to test database")?;

    // 清理测试数据库并运行迁移
    sqlx::query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&pool)
        .await
        .context("Failed to reset test database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations on test database")?;

    Ok(pool)
}
