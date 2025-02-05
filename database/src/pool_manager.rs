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

/// 全局的连接池管理器
pub struct PoolManager {
    // pools: RwLock<HashMap<DatabaseType, DatabasePool>>,
    mysql_pools: RwLock<HashMap<DatabaseType, Pool<MySql>>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            // pools: RwLock::new(HashMap::new()),
            mysql_pools: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_mysql_pool(&self, db_type: DatabaseType) -> Option<Pool<MySql>> {
        // if let Some(DatabasePool::MySql(pool)) = self.pools.read().unwrap().get(&db_type) {
        //     return Some(pool.clone());
        // }
        // None
        self.mysql_pools.read().unwrap().get(&db_type).cloned()
    }
    /// 获取数据库连接池
    // pub fn get_pool(&self, db_type: DatabaseType) -> Option<DatabasePool> {
    //     if let Some(DatabasePool::MySql(pool)) = self.pools.read().unwrap().get(&db_type) {
    //         return Some(pool.clone());
    //     }
    //     None
    // }

    /// 添加一个数据库连接池
    // pub fn add_pool(&self, db_type: DatabaseType, pool: Box<dyn Send + Sync>) {
    //     self.pools.write().unwrap().insert(db_type, pool);
    // }

    pub fn add_mysql_pool(&self, db_type: DatabaseType, pool: Pool<MySql>) {
        self.mysql_pools.write().unwrap().insert(db_type, pool);
    }
}

pub static POOL_MANAGER: Lazy<PoolManager> = Lazy::new(PoolManager::new);

/// 从配置文件初始化所有连接池
pub async fn init_pools(config: &DbConfig) -> Result<(), sqlx::Error> {
    // 初始化 Phoenix 数据库连接池
    if let Some(phoenix) = &config.phoenix {
        let phoenix_pool = MySqlPoolOptions::new()
            .max_connections(phoenix.max_connections)
            .idle_timeout(Duration::from_secs(phoenix.idle_timeout))
            .connect(phoenix.url.as_str())
            .await?;
        POOL_MANAGER.add_mysql_pool(DatabaseType::Phoenix, phoenix_pool);
    }

    if let Some(activity) = &config.huajian_activity {
        // let activity_pool = MySqlPool::connect(&activity.database_url).await?;
        let activity_pool = MySqlPoolOptions::new()
            .max_connections(activity.max_connections)
            .idle_timeout(Duration::from_secs(activity.idle_timeout))
            .connect(activity.url.as_str())
            .await?;

        POOL_MANAGER.add_mysql_pool(DatabaseType::HuajianActivity, activity_pool);
    }

    if let Some(live) = &config.huajian_live {
        let live_pool = MySqlPoolOptions::new()
            .max_connections(live.max_connections)
            .idle_timeout(Duration::from_secs(live.idle_timeout))
            .connect(live.url.as_str())
            .await?;
        POOL_MANAGER.add_mysql_pool(DatabaseType::HuajianLive, live_pool);
    }
    Ok(())
}

pub async fn get_mysql_connection(
    db_type: DatabaseType,
) -> Result<PoolConnection<MySql>, sqlx::Error> {
    if let Some(pool) = POOL_MANAGER.get_mysql_pool(db_type) {
        return pool.acquire().await;
    }
    Err(sqlx::Error::PoolTimedOut)
}

#[cfg(test)]
mod tests {
    use crate::db_config::{DatabaseConfig, DbConfig};
    use crate::pool_manager::{init_pools, DatabaseType, POOL_MANAGER};
    use chrono::{DateTime, Utc};
    use sqlx::Row;

    #[tokio::test]
    async fn test_pool_manager() {
        let config = DbConfig {
            phoenix: Some(
                DatabaseConfig {
                    url: "mysql://srvjava_uat:Test2018RDBSrvjava1210@rm-m5eu9o0h47352viavio.mysql.rds.aliyuncs.com:3306/phoenix?useUnicode=true&characterEncoding=utf8&allowMultiQueries=true&serverTimezone=GMT%2B8".to_string(),
                    max_connections: 5,
                    idle_timeout: 10,
                },
            ),
            huajian_activity: None,
            huajian_live: None,
        };

        init_pools(&config).await.unwrap();

        let pool = POOL_MANAGER.get_mysql_pool(DatabaseType::Phoenix).unwrap();

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
    }
}
