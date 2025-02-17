use crate::db_config::DbConfig;
use once_cell::sync::Lazy;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use strum::{IntoEnumIterator};
use strum_macros::{Display, EnumIter, EnumString, VariantNames};

// pub enum DatabasePool {
    // Postgres(Pool<Postgres>),
    // MySql(Pool<MySql>),
    // Sqlite(Pool<Sqlite>),
// }

pub static POOL_MANAGER: Lazy<PoolManager> = Lazy::new(PoolManager::new);


/// 数据库类型枚举
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, EnumIter, EnumString, VariantNames, Display)]
#[strum(serialize_all = "snake_case")]
pub enum DatabaseType {
    Phoenix,
    HuajianActivity,
    HuajianLive,
}

/// **全局数据库连接池管理器**
pub struct PoolManager {
    mysql_pools: RwLock<HashMap<DatabaseType, Arc<Pool<MySql>>>>,
    db_config: RwLock<Option<DbConfig>>, // 仅供内部管理
}

impl PoolManager {
    /// **创建 `PoolManager` 实例**
    pub fn new() -> Self {
        Self {
            // pools: RwLock::new(HashMap::new()),
            mysql_pools: RwLock::new(HashMap::new()),
            db_config: RwLock::new(None),
        }
    }

    /// **自动加载配置（仅在首次调用时加载）**
    fn load_config(&self) {
        let mut db_config = self.db_config.write().unwrap();

        // 若已经加载过，则不重复加载
        if db_config.is_none() {
            *db_config = Some(DbConfig::load_config());
        }
    }

    /// **自动加载数据库连接池**
    async fn init_pools(&self) ->Result<(), sqlx::Error> {
        self.load_config();
        let db_config = self.db_config.read().unwrap().clone().unwrap();

        for db_type in DatabaseType::iter() {
            println!("DatabaseType::iter() {}", &db_type);
            if let Some(config) = db_config.databases.get(&db_type.to_string()) {
                println!("Loading database {}, {:?}", db_type, config);
                let pool = MySqlPoolOptions::new()
                    .max_connections(config.max_connections)
                    .idle_timeout(Duration::from_secs(config.idle_timeout))
                    .connect(&config.url)
                    .await?;

                self.mysql_pools.write().unwrap().insert(db_type, Arc::new(pool));
                println!("✅ 已初始化 `{}` 连接池", db_type);
            }
        }
        Ok(())
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
    /// let pool = POOL_MANAGER.get_mysql_pool(DatabaseType::Phoenix).await.unwrap();
    /// let row: (i32, String) = sqlx::query_as("SELECT uid, username FROM t_user_main WHERE uid = ?")
    ///     .bind(2)
    ///    .fetch_one(&*pool) // ✅ 需要解引用 `Arc<Pool<MySql>>`
    ///    .await
    ///    .unwrap();
    /// ```
    pub async fn get_mysql_pool(&self, db_type: DatabaseType) -> Result<Arc<Pool<MySql>>, sqlx::Error> {
        // **1. 先检查是否已有连接池**
        {
            let pools = self.mysql_pools.read().unwrap();
            let pool = pools.get(&db_type);
            println!("type:{}, pool: {:?}", &db_type, pool);
            if let Some(pool) = pools.get(&db_type) {
                return Ok(pool.clone());
            }
        }

        // **2. 若连接池不存在，则自动初始化**
        self.init_pools().await?;

        // **3. 获取数据库配置**
        let pools = self.mysql_pools.read().unwrap();
        pools.get(&db_type).cloned().ok_or(sqlx::Error::PoolTimedOut)

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
    pub async fn get_mysql_connection(&self, db_type: DatabaseType) -> Result<PoolConnection<MySql>, sqlx::Error> {
        let pool = self.get_mysql_pool(db_type).await?;
        pool.acquire().await
    }

}



#[cfg(test)]
mod tests {
    use crate::pool_manager::{DatabaseType, POOL_MANAGER};
    use chrono::{DateTime, Utc};
    use sqlx::Row;

    #[tokio::test]
    async fn test_pool_manager() -> Result<(), sqlx::Error> {

        let pool = POOL_MANAGER.get_mysql_pool(DatabaseType::Phoenix).await?;
        let pool1 = POOL_MANAGER.get_mysql_pool(DatabaseType::HuajianActivity).await?;
        let pool2 = POOL_MANAGER.get_mysql_pool(DatabaseType::HuajianLive).await?;

        // let conn = get_mysql_connection(DatabaseType::Phoenix).await.unwrap();

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
