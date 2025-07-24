use std::collections::HashMap;
use std::sync::Arc;
use crate::models::enums::PaymentType;
use crate::payment::strategy::{PaymentStrategy, RateLimitedStrategy};
use crate::payment::providers::*;
use crate::error::PaymentError;
use crate::config::cache::ConfigCache;

pub struct PaymentFactory {
    strategies: HashMap<PaymentType, Arc<dyn PaymentStrategy>>,
    config_cache: Arc<ConfigCache>,
}

impl PaymentFactory {
    pub fn new(config_cache: Arc<ConfigCache>) -> Self {
        let mut strategies: HashMap<PaymentType, Arc<dyn PaymentStrategy>> = HashMap::new();

        // 注册所有支付策略，添加限流封装
        let wx_h5 = Arc::new(wechat::WechatH5Strategy::new());
        strategies.insert(
            PaymentType::WxH5,
            Arc::new(RateLimitedStrategy::new(wx_h5, 50))
        );

        let wx_sdk = Arc::new(wechat::WechatSdkStrategy::new());
        strategies.insert(
            PaymentType::WxSdk,
            Arc::new(RateLimitedStrategy::new(wx_sdk, 100))
        );

        let zfb_h5 = Arc::new(alipay::AlipayH5Strategy::new());
        strategies.insert(
            PaymentType::ZfbH5,
            Arc::new(RateLimitedStrategy::new(zfb_h5, 50))
        );

        let zfb_sdk = Arc::new(alipay::AlipaySdkStrategy::new());
        strategies.insert(
            PaymentType::ZfbSdk,
            Arc::new(RateLimitedStrategy::new(zfb_sdk, 100))
        );

        let apple_iap = Arc::new(apple::AppleIapStrategy::new());
        strategies.insert(
            PaymentType::AppleIap,
            Arc::new(RateLimitedStrategy::new(apple_iap, 200))
        );

        // ... 其他支付方式

        Self { strategies, config_cache }
    }

    pub fn get_strategy(&self, payment_type: &PaymentType) -> Result<Arc<dyn PaymentStrategy>, PaymentError> {
        self.strategies
            .get(payment_type)
            .cloned()
            .ok_or_else(|| PaymentError::UnsupportedPaymentType(payment_type.to_string()))
    }

    pub fn config_cache(&self) -> Arc<ConfigCache> {
        self.config_cache.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::mysql::MySqlPoolOptions;

    #[tokio::test]
    async fn test_payment_factory() -> Result<(), Box<dyn std::error::Error>> {
        // 创建模拟的数据库连接池
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect("mysql://root:password@localhost/payment_service_test")
            .await?;

        // 创建配置缓存
        let config_cache = Arc::new(ConfigCache::new(pool, std::time::Duration::from_secs(60)));

        // 创建工厂
        let factory = PaymentFactory::new(config_cache);

        // 测试获取已注册的策略
        let wx_h5_strategy = factory.get_strategy(&PaymentType::WxH5);
        assert!(wx_h5_strategy.is_ok());

        let zfb_h5_strategy = factory.get_strategy(&PaymentType::ZfbH5);
        assert!(zfb_h5_strategy.is_ok());

        // 测试获取未注册的策略
        let unknown_strategy = factory.get_strategy(&PaymentType::PaypalH5);
        assert!(unknown_strategy.is_err());
        if let Err(err) = unknown_strategy {
            match err {
                PaymentError::UnsupportedPaymentType(_) => {},
                _ => panic!("预期错误类型不匹配"),
            }
        }

        Ok(())
    }
}