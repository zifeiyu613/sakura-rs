//! 数据库连接池管理模块

use sqlx::AnyPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use rconfig::{AppConfig, DatabaseConfig};

use crate::error::{DbError, Result};

/// 支持的数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbType {
    /// MySQL数据库
    MySql,
    /// PostgreSQL数据库
    Postgres,
    /// SQLite数据库
    Sqlite,
    /// 未知数据库类型
    Unknown,
}

impl From<&str> for DbType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mysql" => DbType::MySql,
            "postgres" | "postgresql" => DbType::Postgres,
            "sqlite" | "sqlite3" => DbType::Sqlite,
            _ => DbType::Unknown,
        }
    }
}

/// 连接池配置选项
#[derive(Debug, Clone)]
pub struct PoolOptions {
    /// 最小连接数
    pub min_connections: u32,
    /// 最大连接数
    pub max_connections: u32,
    /// 连接超时（秒）
    pub timeout: u64,
    /// 连接生命周期（秒）
    pub max_lifetime: Option<u64>,
    /// 闲置超时（秒）
    pub idle_timeout: Option<u64>,
    /// 测试前检查
    pub test_before_acquire: bool,
}

impl Default for PoolOptions {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            timeout: 30,
            max_lifetime: Some(1800),
            idle_timeout: Some(600),
            test_before_acquire: true,
        }
    }
}

impl From<&DatabaseConfig> for PoolOptions {
    fn from(config: &DatabaseConfig) -> Self {
        Self {
            min_connections: config.min_connections,
            max_connections: config.max_connections,
            timeout: config.timeout,
            ..Default::default()
        }
    }
}

/// 数据库池管理器，支持多数据源
#[derive(Debug, Clone)]
pub struct DbPool {
    /// 默认连接池
    default_pool: AnyPool,

    /// 命名连接池集合
    pools: Arc<RwLock<HashMap<String, AnyPool>>>,

    /// 默认数据库类型
    db_type: DbType,
}

impl DbPool {
    /// 从配置创建数据库连接池
    ///
    /// # Arguments
    /// * `config` - 应用配置
    /// * `source` - 数据源名称，如果为None则使用默认数据源
    ///
    /// # Returns
    /// * `Result<DbPool>` - 数据库连接池
    ///
    /// # Example
    /// ```
    /// use rdatabase::DbPool;
    /// use rconfig::AppConfig;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = AppConfig::new().build()?;
    ///     let pool = DbPool::from_config(&config, None).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn from_config(config: &AppConfig, source: Option<&str>) -> Result<Self> {
        // 获取指定数据源配置
        let db_config = match source {
            None => &config.database,
            Some("default") => &config.database,
            Some(name) => config.get_database(Some(name))
                .ok_or_else(|| DbError::SourceNotFound(name.to_string()))?,
        };

        // 创建默认连接池
        let db_url = db_config.connection_url()?;
        let db_type = DbType::from(db_config.db_type.as_str());
        let pool_options = PoolOptions::from(db_config);

        let default_pool = create_pool(&db_url, &pool_options).await?;

        Ok(DbPool {
            default_pool,
            pools: Arc::new(RwLock::new(HashMap::new())),
            db_type,
        })
    }

    /// 加载所有配置的数据源
    ///
    /// # Arguments
    /// * `config` - 应用配置
    ///
    /// # Returns
    /// * `Result<DbPool>` - 数据库连接池
    pub async fn load_all_sources(config: &AppConfig) -> Result<Self> {
        // 先创建默认连接池
        let pool = Self::from_config(config, None).await?;

        // 加载所有其他命名数据源
        for source_name in config.database_names() {
            if source_name != "default" {
                // 避免重复加载默认数据源
                let _ = pool.add_source(config, source_name).await?;
            }
        }

        Ok(pool)
    }

    /// 添加一个命名数据源连接池
    ///
    /// # Arguments
    /// * `config` - 应用配置
    /// * `source_name` - 数据源名称
    ///
    /// # Returns
    /// * `Result<()>` - 操作结果
    pub async fn add_source(&self, config: &AppConfig, source_name: &str) -> Result<()> {
        // 检查数据源是否已存在
        {
            let pools = self.pools.read().await;
            if pools.contains_key(source_name) {
                return Ok(());  // 已存在，无需重复添加
            }
        }

        // 获取数据源配置
        let db_config = config.get_database(Some(source_name))
            .ok_or_else(|| DbError::SourceNotFound(source_name.to_string()))?;

        // 创建连接池
        let db_url = db_config.connection_url()?;
        let pool_options = PoolOptions::from(db_config);
        let pool = create_pool(&db_url, &pool_options).await?;

        // 添加到集合
        let mut pools = self.pools.write().await;
        pools.insert(source_name.to_string(), pool);

        Ok(())
    }

    /// 获取默认连接池
    pub fn conn(&self) -> &AnyPool {
        &self.default_pool
    }

    /// 获取指定名称的连接池
    ///
    /// # Arguments
    /// * `name` - 数据源名称
    ///
    /// # Returns
    /// * `Option<&AnyPool>` - 连接池引用，如果不存在则返回None
    pub async fn get_pool(&self, name: &str) -> Option<AnyPool> {
        if name == "default" {
            return Some(self.default_pool.clone());
        }

        let pools = self.pools.read().await;
        pools.get(name).cloned()
    }

    /// 获取数据库类型
    pub fn db_type(&self) -> DbType {
        self.db_type
    }
    

    /// 检查数据库连接
    ///
    /// # Returns
    /// * `Result<()>` - 检查结果
    pub async fn check_connection(&self) -> Result<()> {
        self.default_pool.acquire().await?;
        Ok(())
    }

    /// 获取所有可用的数据源名称
    ///
    /// # Returns
    /// * `Vec<String>` - 数据源名称列表
    pub async fn sources(&self) -> Vec<String> {
        let mut sources = vec!["default".to_string()];
        let pools = self.pools.read().await;
        sources.extend(pools.keys().cloned());
        sources
    }
}

/// 创建数据库连接池
async fn create_pool(url: &str, options: &PoolOptions) -> Result<AnyPool> {
    let pool = sqlx::any::AnyPoolOptions::new()
        .min_connections(options.min_connections)
        .max_connections(options.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(options.timeout))
        .test_before_acquire(options.test_before_acquire);

    // 设置可选参数
    let pool = if let Some(lifetime) = options.max_lifetime {
        pool.max_lifetime(std::time::Duration::from_secs(lifetime))
    } else {
        pool
    };

    let pool = if let Some(idle) = options.idle_timeout {
        pool.idle_timeout(std::time::Duration::from_secs(idle))
    } else {
        pool
    };

    // 连接数据库
    let pool = pool.connect(url).await
        .map_err(|e| DbError::ConnectionError(format!("无法连接数据库: {}", e)))?;

    Ok(pool)
}
