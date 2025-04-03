use std::collections::HashMap;
use std::time::Duration;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use crate::config::Config;
use crate::error::YiceError;

#[derive(Clone, Debug)]
pub struct DbManager {
    pools: HashMap<String, MySqlPool>,
}

impl DbManager {

    pub async fn new(config: &Config) -> Result<Self, YiceError> {
        let mut pools = HashMap::new();

        for (db_name, db_config) in &config.mysql {
            tracing::info!("Initializing MySQL connection pool for {}", db_name);

            let pool = MySqlPoolOptions::new()
                .max_connections(db_config.max_connections)
                .idle_timeout(Duration::from_secs(db_config.idle_timeout))
                .connect(&db_config.url)
                .await?;
            pools.insert(db_name.clone(), pool);
        }

        Ok(Self { pools })
    }

    pub fn get(&self, db_name: &str) -> Option<&MySqlPool> {
        self.pools.get(db_name)
    }

    // 提供便捷方法访问常用数据库
    pub fn sm_phoenix(&self) -> Option<&MySqlPool> {
        self.get("sm_phoenix")
    }

    pub fn sakura_pay(&self) -> Option<&MySqlPool> {
        self.get("sakura_pay")
    }

}