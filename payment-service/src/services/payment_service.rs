use std::sync::Arc;
use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;

use crate::error::PaymentError;
use crate::models::payment::*;
use crate::models::enums::{PaymentType, OrderStatus};
use crate::payment::factory::PaymentFactory;
use crate::config::cache::ConfigCache;
use crate::domain::payment::PaymentOrder;
use crate::domain::money::{Money, Currency};
use crate::repository::payment_repository::{PaymentRepository, MySqlPaymentRepository};

pub struct PaymentService {
    pool: MySqlPool,
    factory: Arc<PaymentFactory>,
    config_cache: Arc<ConfigCache>,
    repository: Arc<dyn PaymentRepository>,
}

impl PaymentService {
    pub fn new(
        pool: MySqlPool,
        factory: Arc<PaymentFactory>,
        config_cache: Arc<ConfigCache>,
    ) -> Self {
        let repository = Arc::new(MySqlPaymentRepository::new(pool.clone()));

        Self {
            pool,
            factory,
            config_cache,
            repository,
        }
    }

    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        // 1. 获取支付配置
        let config = self.config_cache
            .get_config(request.tenant_id, request.payment_type)
            .await?;

        // 2. 创建领域订单对象
        let currency = match request.currency.as_str() {
            "CNY" => Currency::CNY,
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            "JPY" => Currency::JPY,
            _ => return Err(PaymentError::Configuration(format!("不支持的货币: {}", request.currency))),
        };

        let mut order = PaymentOrder::new(
            request.tenant_id,
            request.user_id,
            request.payment_type,
            Money::new(request.amount, currency),
            request.callback_url.clone(),
            request.notify_url.clone(),
            request.extra_data.clone(),
        );

        // 3. 保存订单
        self.repository.save(&mut order).await?;

        // 4. 获取支付策略并创建第三方订单
        let strategy = self.factory.get_strategy(&request.payment_type)?;
        let response = strategy.create_order(&order, &config, &request).await?;

        // 5. 更新订单状态
        order.initiate_payment(response.payment_url.clone())?;
        self.repository.save(&mut order).await?;

        Ok(response)
    }

    pub async fn query_payment(
        &self,
        order_id: &str,
    ) -> Result<OrderStatus, PaymentError> {
        // 1. 获取订单信息
        let order = self.repository.find_by_id(order_id).await?
            .ok_or_else(|| PaymentError::OrderNotFound(order_id.to_string()))?;

        // 2. 获取支付配置
        let config = self.config_cache
            .get_config(order.tenant_id, order.payment_type)
            .await?;

        // 3. 查询第三方订单状态
        let strategy = self.factory.get_strategy(&order.payment_type)?;
        let status = strategy.query_order(&order, &config).await?;

        // 4. 更新本地订单状态
        if status != order.status {
            self.repository.update_status(order_id, status).await?;
        }

        Ok(status)
    }

    pub async fn handle_callback(
        &self,
        payment_type: PaymentType,
        tenant_id: i64,
        callback_data: serde_json::Value,
    ) -> Result<(), PaymentError> {
        // 1. 获取支付配置
        let config = self.config_cache
            .get_config(tenant_id, payment_type)
            .await?;

        // 2. 处理回调
        let strategy = self.factory.get_strategy(&payment_type)?;
        let (order_id, status) = strategy.handle_callback(&config, &callback_data).await?;

        // 3. 获取并更新订单
        let mut order = self.repository.find_by_id(&order_id).await?
            .ok_or_else(|| PaymentError::OrderNotFound(order_id.clone()))?;

        match status {
            OrderStatus::Success => {
                // 从回调中提取第三方订单ID
                let third_party_id = callback_data.get("transaction_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                order.complete_payment(third_party_id)?;
            },
            OrderStatus::Failed => {
                let reason = callback_data.get("error_msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知原因")
                    .to_string();

                order.fail_payment(reason)?;
            },
            _ => return Err(PaymentError::InvalidOrderStatus {
                current: format!("{:?}", status),
                expected: vec!["Success".to_string(), "Failed".to_string()],
            }),
        }

        // 保存更新后的订单
        self.repository.save(&mut order).await?;

        // 4. 触发业务回调
        self.trigger_business_callback(&order_id).await?;

        Ok(())
    }

    pub async fn refund_payment(
        &self,
        refund_request: RefundRequest,
    ) -> Result<String, PaymentError> {
        // 1. 获取订单信息
        let mut order = self.repository.find_by_id(&refund_request.order_id).await?
            .ok_or_else(|| PaymentError::OrderNotFound(refund_request.order_id.clone()))?;

        // 2. 验证订单状态
        if order.status != OrderStatus::Success {
            return Err(PaymentError::InvalidOrderStatus {
                current: format!("{:?}", order.status),
                expected: vec!["Success".to_string()],
            });
        }

        // 3. 获取支付配置
        let config = self.config_cache
            .get_config(order.tenant_id, order.payment_type)
            .await?;

        // 4. 生成退款ID
        let refund_id = Uuid::new_v4().to_string();

        // 5. 发起退款
        let strategy = self.factory.get_strategy(&order.payment_type)?;
        let third_party_refund_id = strategy.refund(&order, &config, &refund_request).await?;

        // 6. 更新订单状态
        if refund_request.refund_amount >= order.amount.amount {
            order.request_refund(refund_id.clone(), refund_request.refund_amount)?;
        } else {
            // 部分退款逻辑可以扩展...
            order.request_refund(refund_id.clone(), refund_request.refund_amount)?;
        }

        self.repository.save(&mut order).await?;

        // 7. 保存退款记录
        self.save_refund_record(
            &refund_id,
            &refund_request.order_id,
            refund_request.refund_amount,
            refund_request.refund_reason.as_deref().unwrap_or(""),
            &third_party_refund_id,
        ).await?;

        Ok(refund_id)
    }

    // 辅助方法
    async fn trigger_business_callback(&self, order_id: &str) -> Result<(), PaymentError> {
        // 查询订单获取回调URL
        let order = self.repository.find_by_id(order_id).await?
            .ok_or_else(|| PaymentError::OrderNotFound(order_id.to_string()))?;

        if let Some(callback_url) = order.callback_url {
            // 实际项目中可以使用消息队列异步处理，避免阻塞
            // 这里简化为直接HTTP调用
            if !callback_url.is_empty() {
                let client = reqwest::Client::new();
                let _ = client.post(&callback_url)
                    .json(&serde_json::json!({
                        "order_id": order_id,
                        "status": format!("{:?}", order.status),
                        "time": Utc::now().to_rfc3339()
                    }))
                    .send()
                    .await
                    .map_err(|e| PaymentError::Internal(format!("回调失败: {}", e)))?;
            }
        }

        Ok(())
    }

    async fn save_refund_record(
        &self,
        refund_id: &str,
        order_id: &str,
        refund_amount: i64,
        refund_reason: &str,
        third_party_refund_id: &str,
    ) -> Result<(), PaymentError> {
        sqlx::query!(
            r#"
            INSERT INTO refund_orders
            (refund_id, order_id, refund_amount, refund_reason, status, third_party_refund_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            refund_id,
            order_id,
            refund_amount,
            refund_reason,
            "SUCCESS",
            third_party_refund_id,
            Utc::now(),
            Utc::now()
        )
            .execute(&self.pool)
            .await
            .map_err(PaymentError::Database)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use sqlx::MySqlPool;
    use crate::config::cache::ConfigCache;
    use crate::models::enums::PaymentType;
    use crate::models::payment::CreatePaymentRequest;
    use crate::payment::factory::PaymentFactory;
    use crate::services::payment_service::PaymentService;

    // tests/payment_service_tests.rs
    #[tokio::test]
    async fn test_payment_flow() -> anyhow::Result<()> {
        // 使用真实的数据库连接（测试数据库）
        let pool = MySqlPool::connect("mysql://root:password@localhost/test_db").await?;

        // 设置测试数据
        setup_test_data(&pool).await?;

        // 创建真实的服务实例
        let config_cache = Arc::new(ConfigCache::new(pool.clone(), Duration::from_secs(60)));
        let factory = Arc::new(PaymentFactory::new(config_cache.clone()));
        let service = PaymentService::new(pool.clone(), factory, config_cache);

        // 执行测试
        let request = CreatePaymentRequest {
            tenant_id: 1,
            user_id: 100,
            payment_type: PaymentType::WxH5,
            amount: 10000,
            currency: "CNY".to_string(),
            product_name: "测试商品".to_string(),
            product_desc: None,
            callback_url: None,
            notify_url: None,
            extra_data: None,
        };

        let result = service.create_payment(request).await;
        assert!(result.is_ok());

        // 清理测试数据
        cleanup_test_data(&pool).await?;

        Ok(())
    }

    async fn setup_test_data(pool: &MySqlPool) -> anyhow::Result<()> {
        // 插入测试配置数据
        sqlx::query!(
        "INSERT INTO payment_configs (tenant_id, payment_type, payment_sub_type, merchant_id, gateway_url, notify_url, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        1, 5, 5, "test_merchant", "https://example.com", "https://example.com/notify", true, chrono::Utc::now(), chrono::Utc::now()
    ).execute(pool).await?;

        Ok(())
    }

    async fn cleanup_test_data(pool: &MySqlPool) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM payment_configs WHERE tenant_id = ?", 1)
            .execute(pool)
            .await?;

        sqlx::query!("DELETE FROM payment_orders WHERE tenant_id = ?", 1)
            .execute(pool)
            .await?;

        Ok(())
    }
}