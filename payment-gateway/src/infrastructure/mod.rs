pub mod config;
mod payment;
mod repository;
mod cache;
pub mod messaging;
pub mod logging;

use anyhow::Result;
use sqlx::migrate::Migrator;
use std::path::Path;
use sqlx::MySqlPool;
use tracing::info;

/// 静态引用数据库迁移文件
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// 运行数据库迁移
pub async fn run_migrations(pool: &MySqlPool) -> Result<()> {
    info!("Running database migrations");
    MIGRATOR.run(pool).await?;
    info!("Database migrations completed successfully");
    Ok(())
}

/// 检查数据库连接
pub async fn check_database_connection(pool: &MySqlPool) -> Result<()> {
    info!("Checking database connection");
    let result = sqlx::query("SELECT 1").execute(pool).await?;
    info!("Database connection successful: {:?}", result);
    Ok(())
}