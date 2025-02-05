use crate::db_config::DbConfig;
use once_cell::sync::Lazy;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

pub enum DatabasePool {
    // Postgres(Pool<Postgres>),
    MySql(Pool<MySql>),
    // Sqlite(Pool<Sqlite>),
}

/// 数据库类型枚举
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum DatabaseType {
    Phoenix,
    HuajianActivity,
    HuajianLive,
}

/// **全局数据库连接池管理器**
pub struct PoolManager {
    // pools: RwLock<HashMap<DatabaseType, DatabasePool>>,
    mysql_pools: RwLock<HashMap<DatabaseType, Pool<MySql>>>,
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

    /// **获取 MySQL 连接池（若不存在则懒加载初始化）**
    pub async fn get_mysql_pool(&self, db_type: DatabaseType) -> Result<Pool<MySql>, sqlx::Error> {
        // **1. 先检查是否已有连接池**
        {
            let pools = self.mysql_pools.read().unwrap();
            if let Some(pool) = pools.get(&db_type) {
                return Ok(pool.clone());
            }
        }

        // **2. 懒加载数据库配置**
        self.load_config();

        // **3. 获取数据库配置**
        let db_config = {
            let db_config = self.db_config.read().unwrap();
            db_config.clone()
        };

        if let Some(config) = db_config {
            let db_entry = match db_type {
                DatabaseType::Phoenix => &config.phoenix,
                DatabaseType::HuajianActivity => &config.huajian_activity,
                DatabaseType::HuajianLive => &config.huajian_live,
            };

            if let Some(db_config) = db_entry {
                let pool = MySqlPoolOptions::new()
                    .max_connections(db_config.max_connections)
                    .idle_timeout(Duration::from_secs(db_config.idle_timeout))
                    .connect(db_config.url.as_str())
                    .await?;

                let mut pools = self.mysql_pools.write().unwrap();
                pools.insert(db_type, pool.clone());

                return Ok(pool);
            }
        }

        Err(sqlx::Error::PoolTimedOut)
    }

    /// **获取 MySQL 连接（自动初始化）**
    pub async fn get_mysql_connection(&self, db_type: DatabaseType) -> Result<PoolConnection<MySql>, sqlx::Error> {
        let pool = self.get_mysql_pool(db_type).await?;
        pool.acquire().await
    }



    // pub fn get_mysql_pool(&self, db_type: DatabaseType) -> Option<Pool<MySql>> {
    //
    //     self.mysql_pools.read().unwrap().get(&db_type).cloned()
    // }
    // 获取数据库连接池
    // pub fn get_pool(&self, db_type: DatabaseType) -> Option<DatabasePool> {
    //     if let Some(DatabasePool::MySql(pool)) = self.pools.read().unwrap().get(&db_type) {
    //         return Some(pool.clone());
    //     }
    //     None
    // }

    // 添加一个数据库连接池
    // pub fn add_pool(&self, db_type: DatabaseType, pool: Box<dyn Send + Sync>) {
    //     self.pools.write().unwrap().insert(db_type, pool);
    // }

    // pub fn add_mysql_pool(&self, db_type: DatabaseType, pool: Pool<MySql>) {
    //     self.mysql_pools.write().unwrap().insert(db_type, pool);
    // }
}

pub static POOL_MANAGER: Lazy<PoolManager> = Lazy::new(PoolManager::new);

// 从配置文件初始化所有连接池
// pub async fn init_pools(config: &DbConfig) -> Result<(), sqlx::Error> {
//     // 初始化 Phoenix 数据库连接池
//     if let Some(phoenix) = &config.phoenix {
//         let phoenix_pool = MySqlPoolOptions::new()
//             .max_connections(phoenix.max_connections)
//             .idle_timeout(Duration::from_secs(phoenix.idle_timeout))
//             .connect(phoenix.url.as_str())
//             .await?;
//         POOL_MANAGER.add_mysql_pool(DatabaseType::Phoenix, phoenix_pool);
//     }
//
//     if let Some(activity) = &config.huajian_activity {
//         // let activity_pool = MySqlPool::connect(&activity.database_url).await?;
//         let activity_pool = MySqlPoolOptions::new()
//             .max_connections(activity.max_connections)
//             .idle_timeout(Duration::from_secs(activity.idle_timeout))
//             .connect(activity.url.as_str())
//             .await?;
//
//         POOL_MANAGER.add_mysql_pool(DatabaseType::HuajianActivity, activity_pool);
//     }
//
//     if let Some(live) = &config.huajian_live {
//         let live_pool = MySqlPoolOptions::new()
//             .max_connections(live.max_connections)
//             .idle_timeout(Duration::from_secs(live.idle_timeout))
//             .connect(live.url.as_str())
//             .await?;
//         POOL_MANAGER.add_mysql_pool(DatabaseType::HuajianLive, live_pool);
//     }
//     Ok(())
// }




#[cfg(test)]
mod tests {
    use crate::pool_manager::{DatabaseType, PoolManager};
    use chrono::{DateTime, Utc};
    use sqlx::Row;

    #[tokio::test]
    async fn test_pool_manager() -> Result<(), sqlx::Error> {

        let pool = PoolManager::new().get_mysql_pool(DatabaseType::Phoenix).await?;

        // let conn = get_mysql_connection(DatabaseType::Phoenix).await.unwrap();

        let row = sqlx::query("select * from t_user_main where uid = ?")
            .bind(2)
            .fetch_one(&pool)
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
