use async_trait::async_trait;
use std::sync::Arc;
use crate::error::PaymentError;
use crate::models::payment::*;
use crate::domain::payment::PaymentOrder;
use crate::models::enums::OrderStatus;

#[async_trait]
pub trait PaymentStrategy: Send + Sync {
    /// 创建支付订单
    async fn create_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError>;

    /// 查询订单状态
    async fn query_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
    ) -> Result<OrderStatus, PaymentError>;

    /// 处理支付回调
    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError>;

    /// 发起退款
    async fn refund(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        refund_request: &RefundRequest,
    ) -> Result<String, PaymentError>;
}

// 添加限流装饰器
pub struct RateLimitedStrategy<T: PaymentStrategy> {
    inner: Arc<T>,
    limiter: Arc<tokio::sync::Semaphore>,
}

impl<T: PaymentStrategy> RateLimitedStrategy<T> {
    pub fn new(inner: Arc<T>, max_concurrent: usize) -> Self {
        Self {
            inner,
            limiter: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
        }
    }
}

#[async_trait]
impl<T: PaymentStrategy> PaymentStrategy for RateLimitedStrategy<T> {
    async fn create_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        request: &CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        let _permit = self.limiter.try_acquire()
            .map_err(|_| PaymentError::RateLimited)?;

        self.inner.create_order(order, config, request).await
    }

    async fn query_order(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
    ) -> Result<OrderStatus, PaymentError> {
        let _permit = self.limiter.try_acquire()
            .map_err(|_| PaymentError::RateLimited)?;

        self.inner.query_order(order, config).await
    }

    async fn handle_callback(
        &self,
        config: &PaymentConfig,
        callback_data: &serde_json::Value,
    ) -> Result<(String, OrderStatus), PaymentError> {
        // 回调处理不限流
        self.inner.handle_callback(config, callback_data).await
    }

    async fn refund(
        &self,
        order: &PaymentOrder,
        config: &PaymentConfig,
        refund_request: &RefundRequest,
    ) -> Result<String, PaymentError> {
        let _permit = self.limiter.try_acquire()
            .map_err(|_| PaymentError::RateLimited)?;

        self.inner.refund(order, config, refund_request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    // 创建模拟的支付策略
    mock! {
        PaymentStrategyMock {}
        
        #[async_trait]
        impl PaymentStrategy for PaymentStrategyMock {
            async fn create_order(
                &self,
                order: &PaymentOrder,
                config: &PaymentConfig,
                request: &CreatePaymentRequest,
            ) -> Result<CreatePaymentResponse, PaymentError>;
            
            async fn query_order(
                &self,
                order: &PaymentOrder,
                config: &PaymentConfig,
            ) -> Result<OrderStatus, PaymentError>;
            
            async fn handle_callback(
                &self,
                config: &PaymentConfig,
                callback_data: &serde_json::Value,
            ) -> Result<(String, OrderStatus), PaymentError>;
            
            async fn refund(
                &self,
                order: &PaymentOrder,
                config: &PaymentConfig,
                refund_request: &RefundRequest,
            ) -> Result<String, PaymentError>;
        }
    }

    #[tokio::test]
    async fn test_rate_limited_strategy() {
        // 准备测试数据
        let mut mock = MockPaymentStrategyMock::new();

        // 设置期望
        mock.expect_create_order()
            .times(1)
            .returning(|_, _, _| {
                Ok(CreatePaymentResponse {
                    order_id: "test123".to_string(),
                    payment_url: Some("http://example.com".to_string()),
                    payment_params: None,
                })
            });

        // 创建限流装饰器
        let strategy = RateLimitedStrategy::new(Arc::new(mock), 1);

        // 创建测试参数
        let order = PaymentOrder::new(
            1, 1, crate::models::enums::PaymentType::WxH5,
            crate::domain::money::Money::cny(100),
            None, None, None
        );

        let config = PaymentConfig {
            id: 1,
            tenant_id: 1,
            payment_type: 5,
            payment_sub_type: 5,
            merchant_id: "test".to_string(),
            app_id: Some("test".to_string()),
            private_key: None,
            public_key: None,
            api_key: None,
            api_secret: None,
            gateway_url: "http://example.com".to_string(),
            notify_url: "http://example.com".to_string(),
            return_url: None,
            extra_config: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let request = CreatePaymentRequest {
            tenant_id: 1,
            user_id: 1,
            payment_type: crate::models::enums::PaymentType::WxH5,
            amount: 100,
            currency: "CNY".to_string(),
            product_name: "Test".to_string(),
            product_desc: None,
            callback_url: None,
            notify_url: None,
            extra_data: None,
        };

        // 第一次调用应该成功
        let result = strategy.create_order(&order, &config, &request).await;
        assert!(result.is_ok());

        // 模拟并发，耗尽信号量
        let _permit = strategy.limiter.try_acquire().unwrap();

        // 第二次调用应该被限流
        let result = strategy.create_order(&order, &config, &request).await;
        assert!(matches!(result, Err(PaymentError::RateLimited)));
    }
}