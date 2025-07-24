use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use sqlx::MySqlPool;

use crate::models::enums::PaymentType;
use crate::models::payment::PaymentConfig;
use crate::error::PaymentError;

#[derive(Debug, Clone)]
struct CacheEntry {
    config: Arc<PaymentConfig>,
    expires_at: Instant,
}

pub struct ConfigCache {
    configs: RwLock<HashMap<(i64, i32), CacheEntry>>,
    ttl: Duration,
    pool: MySqlPool,
}

impl ConfigCache {
    pub fn new(pool: MySqlPool, ttl: Duration) -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            ttl,
            pool,
        }
    }

    pub async fn get_config(&self, tenant_id: i64, payment_type: PaymentType) -> Result<Arc<PaymentConfig>, PaymentError> {
        let sub_type = payment_type.sub_type_code();
        let key = (tenant_id, sub_type);

        // 尝试从缓存读取
        {
            let configs = self.configs.read().await;
            if let Some(entry) = configs.get(&key) {
                if entry.expires_at > Instant::now() {
                    return Ok(entry.config.clone());
                }
            }
        }

        // 缓存未命中或已过期，从数据库加载
        let config = self.load_from_db(tenant_id, payment_type).await?;
        let config_arc = Arc::new(config);

        // 更新缓存
        {
            let mut configs = self.configs.write().await;
            configs.insert(key, CacheEntry {
                config: config_arc.clone(),
                expires_at: Instant::now() + self.ttl,
            });
        }

        Ok(config_arc)
    }

    pub async fn invalidate(&self, tenant_id: i64, payment_type: PaymentType) {
        let sub_type = payment_type.sub_type_code();
        let key = (tenant_id, sub_type);

        let mut configs = self.configs.write().await;
        configs.remove(&key);
    }

    async fn load_from_db(&self, tenant_id: i64, payment_type: PaymentType) -> Result<PaymentConfig, PaymentError> {
        let sub_type = payment_type.sub_type_code();

        let config = sqlx::query_as::<_, PaymentConfig>(
            r#"
            SELECT * FROM payment_configs 
            WHERE tenant_id = ? AND payment_sub_type = ? AND enabled = true
            "#
        )
            .bind(tenant_id)
            .bind(sub_type)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::Configuration(
                    format!("找不到支付配置: tenant_id={}, payment_type={:?}", tenant_id, payment_type)
                ),
                err => PaymentError::Database(err),
            })?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::mysql::{MySqlPoolOptions, MySqlConnectOptions};
    use sqlx::ConnectOptions;
    use std::str::FromStr;
    use chrono::Utc;

    #[tokio::test]
    async fn test_config_cache() -> Result<(), Box<dyn std::error::Error>> {
        let options = MySqlConnectOptions::from_str("mysql://root:password@localhost/payment_service_test")?
            .disable_statement_logging();
        let pool = MySqlPoolOptions::new().connect_with(options).await?;

        // 创建测试表并插入数据
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS payment_configs (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                tenant_id BIGINT NOT NULL,
                payment_type INT NOT NULL,
                payment_sub_type INT NOT NULL,
                merchant_id VARCHAR(255) NOT NULL,
                app_id VARCHAR(255),
                private_key TEXT,
                public_key TEXT,
                api_key VARCHAR(255),
                api_secret VARCHAR(255),
                gateway_url VARCHAR(500) NOT NULL,
                notify_url VARCHAR(500) NOT NULL,
                return_url VARCHAR(500),
                extra_config JSON,
                enabled BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                UNIQUE KEY uk_tenant_payment (tenant_id, payment_sub_type)
            )
            "#
        )
            .execute(&pool)
            .await?;

        let now = Utc::now();

        // 清理之前的测试数据
        sqlx::query("DELETE FROM payment_configs WHERE tenant_id = 999")
            .execute(&pool)
            .await?;

        // 插入测试数据
        sqlx::query(
            r#"
            INSERT INTO payment_configs 
            (tenant_id, payment_type, payment_sub_type, merchant_id, app_id, gateway_url, notify_url, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
            .bind(999i64)
            .bind(5i32)
            .bind(5i32)
            .bind("test_merchant")
            .bind(Some("wx12345"))
            .bind("https://api.example.com")
            .bind("https://notify.example.com")
            .bind(true)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await?;

        // 创建缓存
        let cache = ConfigCache::new(pool.clone(), Duration::from_secs(1));

        // 测试缓存读取
        let config1 = cache.get_config(999, PaymentType::WxH5).await?;
        assert_eq!(config1.tenant_id, 999);
        assert_eq!(config1.payment_sub_type, 5);
        assert_eq!(config1.merchant_id, "test_merchant");

        // 测试缓存命中
        let config2 = cache.get_config(999, PaymentType::WxH5).await?;
        assert!(Arc::ptr_eq(&config1, &config2), "应该返回相同的缓存对象");

        // 等待缓存过期
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 测试缓存重新加载
        let config3 = cache.get_config(999, PaymentType::WxH5).await?;
        assert_eq!(config3.tenant_id, 999);
        assert!(!Arc::ptr_eq(&config1, &config3), "应该返回新的缓存对象");

        // 测试缓存失效
        cache.invalidate(999, PaymentType::WxH5).await;
        let config4 = cache.get_config(999, PaymentType::WxH5).await?;
        assert!(!Arc::ptr_eq(&config3, &config4), "缓存失效后应该返回新的对象");

        // 清理测试数据
        sqlx::query("DELETE FROM payment_configs WHERE tenant_id = 999")
            .execute(&pool)
            .await?;

        Ok(())
    }
}