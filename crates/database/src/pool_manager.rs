use config::app_config::{get_config, MysqlConfig};
use errors::error::DatabaseError;
use once_cell::sync::Lazy;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub static POOL_MANAGER: Lazy<PoolManager> = Lazy::new(PoolManager::new);

/// **全局数据库连接池管理器**
pub struct PoolManager {
    mysql_pools: RwLock<HashMap<String, Arc<Pool<MySql>>>>, // 以数据库名称为 key 存储池
    db_config: RwLock<HashMap<String, MysqlConfig>>,        // 仅供内部管理
}

impl PoolManager {
    /// **创建 `PoolManager` 实例**
    pub fn new() -> Self {
        Self {
            // pools: RwLock::new(HashMap::new()),
            mysql_pools: RwLock::new(HashMap::new()),
            db_config: RwLock::new(HashMap::new()),
        }
    }

    // /// **自动加载配置（仅在首次调用时加载）**
    fn load_db_config(&self) {
        let mut db_config = self.db_config.write().unwrap();

        // 若已经加载过，则不重复加载
        if db_config.is_empty() {
            let config = get_config().unwrap().mysql;
            *db_config = config;
        }
    }

    /// **获取 MySQL 连接池（若不存在则懒加载初始化）**
    ///
    /// ***推荐使用 &Pool<MySql>（连接池）***
    /// - 更简洁：无需手动获取 conn，直接传 &Pool<MySql>。
    /// - 更高效：sqlx 内部自动获取连接，查询执行完毕后会自动归还连接。
    ///	- 并发更友好：连接池支持多个任务并发执行 SQL，而手动 conn 需要管理生命周期。
    ///
    /// ***使用方法***
    /// - 方案 1：解引用 Arc 以获取 &Pool<MySql> (这样 Arc 仍然保持引用计数，并且 &Pool<MySql> 可以正确传递给 sqlx。)
    /// - 方案 2：克隆 Arc，然后解引用(这种方式适用于多个线程共享 Pool 的情况。)
    ///
    /// ***使用案例***
    /// ```rust
    /// use database::pool_manager::POOL_MANAGER;
    ///
    /// let pool = POOL_MANAGER.get_mysql_pool("phoenix").await.unwrap();
    /// let row: (i32, String) = sqlx::query_as("SELECT uid, username FROM t_user_main WHERE uid = ?")
    ///     .bind(2)
    ///    .fetch_one(&*pool) // ✅ 需要解引用 `Arc<Pool<MySql>>`
    ///    .await
    ///    .unwrap();
    /// ```
    pub async fn get_mysql_pool(&self, db_name: &str) -> Result<Arc<Pool<MySql>>, DatabaseError> {
        // **1. 先检查是否已有连接池**
        {
            let pools = self
                .mysql_pools
                .read()
                .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
            if let Some(pool) = pools.get(db_name) {
                return Ok(pool.clone());
            }
        }

        // 2. 加载数据库配置
        self.load_db_config();

        // 3. 获取数据库配置
        let mysql_config = {
            let db_config = self.db_config.read().unwrap();
            db_config.get(db_name).cloned()
        };

        if let Some(db_config_entry) = mysql_config {
            // 4. 创建连接池
            let pool = MySqlPoolOptions::new()
                .max_connections(db_config_entry.max_connections)
                .idle_timeout(Duration::from_secs(db_config_entry.idle_timeout))
                .connect(&db_config_entry.url)
                .await.map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

            let arc_pool = Arc::new(pool);

            // 5. 将池插入到共享的池管理器中
            let mut pools = self.mysql_pools.write().unwrap();
            pools.insert(db_name.to_string(), arc_pool.clone());
            println!("Loaded mysql pool [{}] success!", db_name);
            return Ok(arc_pool);
        }
        Err(DatabaseError::ConnectionError(format!("No mysql config found for {}", db_name)))
    }

    /// **获取 MySQL 连接（自动初始化）**
    ///
    /// - PoolConnection<MySql> 不是 Executor，但 MySqlConnection 是 Executor。
    /// - PoolConnection<MySql> 实现了 DerefMut，所以 &mut *conn 可以转成 &mut MySqlConnection，符合 Executor 要求。
    ///
    /// ***适用场景：***
    /// - 事务 (begin_transaction)
    ///	- 需要在一个连接中执行多个查询
    ///	- 需要 LOCK 等操作，保证后续 SQL 仍然在同一个连接中执行
    ///
    /// ***⚠️但不推荐在普通查询中使用 conn***
    ///	- 手动管理 conn 不便：使用 conn 时，需要手动确保它的生命周期正确。
    ///	- 影响并发性能：若多个任务需要并发查询，手动获取 conn 可能会限制性能，而 pool 则由 sqlx 自动管理。
    pub async fn get_mysql_connection(
        &self,
        db_type: &str,
    ) -> Result<PoolConnection<MySql>, DatabaseError> {
        let pool = self.get_mysql_pool(db_type).await?;
        pool.acquire()
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::pool_manager::POOL_MANAGER;
    use chrono::{DateTime, Utc};
    use sqlx::Row;
    use strum::IntoEnumIterator;
    use strum_macros::{Display, EnumIter, EnumString, VariantNames};
    use config::app_config::{load_config, AppConfig};
    use errors::error::DatabaseError;

    #[derive(
        Debug, Eq, PartialEq, Hash, Clone, Copy, EnumIter, EnumString, VariantNames, Display,
    )]
    #[strum(serialize_all = "snake_case")]
    pub enum DatabaseType {
        Phoenix,
        HuajianActivity,
        HuajianLive,
    }

    #[tokio::test]
    async fn test_pool_manager() -> Result<(), DatabaseError> {
        let path = "/Users/will/RustroverProjects/sakura/config.toml";
        load_config(Some(path)).expect("L");

        let pool = POOL_MANAGER
            .get_mysql_pool(&DatabaseType::Phoenix.to_string())
            .await?;
        let pool1 = POOL_MANAGER
            .get_mysql_pool(&DatabaseType::HuajianActivity.to_string())
            .await?;
        let _pool2 = POOL_MANAGER
            .get_mysql_pool(&DatabaseType::HuajianLive.to_string())
            .await?;

        // let conn = get_mysql_connection(DatabaseType::Phoenix).await.unwrap();

        println!("{:?}", pool1);

        let row = sqlx::query("select * from t_user_main where uid = ?")
            .bind(2)
            .fetch_one(&*pool)
            .await;

        match row {
            Ok(row) => {
                let uid = row.get::<i32, usize>(0);
                println!("uid:{}", uid);

                let token = row.try_get::<&str, usize>(7);
                match token {
                    Ok(token) => println!("token: {}", token),
                    Err(e) => println!("Error getting token: {:?}", e),
                }

                let token: Option<&str> = row.get("token");
                println!("token:{:?}", token);

                let tm_login: Option<DateTime<Utc>> = row.get("tm_login");
                println!("tm_login:{:?}", tm_login);

                let password: Option<String> = row.get("password");
                println!("password:{:?}", password)
            }
            _ => println!("err"),
        }

        Ok(())
    }
}
