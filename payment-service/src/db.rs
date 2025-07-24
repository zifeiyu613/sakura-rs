use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

pub async fn create_pool(database_url: &str) -> anyhow::Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    Ok(pool)
}

// 初始化数据库表
pub async fn init_db(pool: &MySqlPool) -> anyhow::Result<()> {
    // 创建支付订单表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS payment_orders (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            order_id VARCHAR(64) NOT NULL UNIQUE,
            tenant_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            payment_type INT NOT NULL,
            payment_sub_type INT NOT NULL,
            amount BIGINT NOT NULL,
            currency VARCHAR(10) NOT NULL DEFAULT 'CNY',
            status VARCHAR(20) NOT NULL,
            third_party_order_id VARCHAR(255),
            callback_url VARCHAR(500),
            notify_url VARCHAR(500),
            extra_data JSON,
            created_at TIMESTAMP NOT NULL,
            updated_at TIMESTAMP NOT NULL,
            INDEX idx_tenant_user (tenant_id, user_id),
            INDEX idx_status (status),
            INDEX idx_created_at (created_at)
        )
        "#
    )
        .execute(pool)
        .await?;

    // 创建退款订单表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refund_orders (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            refund_id VARCHAR(64) NOT NULL UNIQUE,
            order_id VARCHAR(64) NOT NULL,
            refund_amount BIGINT NOT NULL,
            refund_reason TEXT,
            status VARCHAR(20) NOT NULL,
            third_party_refund_id VARCHAR(255),
            created_at TIMESTAMP NOT NULL,
            updated_at TIMESTAMP NOT NULL,
            INDEX idx_order_id (order_id),
            INDEX idx_created_at (created_at)
        )
        "#
    )
        .execute(pool)
        .await?;

    // 创建支付配置表
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
            created_at TIMESTAMP NOT NULL,
            updated_at TIMESTAMP NOT NULL,
            UNIQUE KEY uk_tenant_payment (tenant_id, payment_sub_type)
        )
        "#
    )
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool() {
        let result = create_pool("mysql://root:password@localhost/payment_service_test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_db() -> anyhow::Result<()> {
        let pool = create_pool("mysql://root:password@localhost/payment_service_test").await?;
        let result = init_db(&pool).await;
        assert!(result.is_ok());
        Ok(())
    }
}