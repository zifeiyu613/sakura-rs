mod config;
mod payment;
mod repository;

use anyhow::Result;
use sqlx::migrate::Migrator;
use sqlx::PgPool;
use std::path::Path;

/// 静态引用数据库迁移文件
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// 运行数据库迁移
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    log::info!("Running database migrations");
    MIGRATOR.run(pool).await?;
    log::info!("Database migrations completed successfully");
    Ok(())
}

/// 检查数据库连接
pub async fn check_database_connection(pool: &PgPool) -> Result<()> {
    log::info!("Checking database connection");
    let result = sqlx::query("SELECT 1").execute(pool).await?;
    log::info!("Database connection successful: {:?}", result);
    Ok(())
}