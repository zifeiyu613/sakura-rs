use std::sync::Arc;
use std::time::Duration;

use crate::domain::models::{PaymentMethodType, PaymentRegion};
use crate::domain::payment::{
    PaymentConfig, PaymentProcessorFactory, DEFAULT_TIMEOUT
};
use crate::domain::service::PaymentServiceImpl;
use crate::infrastructure::payment::{
    AlipayProcessor, BoostWalletProcessor, WechatPayProcessor
};
use crate::infrastructure::repository::{
    PaymentOrderRepositoryImpl, PaymentTransactionRepositoryImpl, RefundOrderRepositoryImpl
};

pub struct AppState {
    pub payment_service: Arc<dyn crate::domain::service::PaymentService>,
}

impl AppState {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        // 创建支付配置
        let wechat_config = PaymentConfig {
            app_id: std::env::var("WECHAT_APP_ID").unwrap_or_default(),
            api_key: std::env::var("WECHAT_API_KEY").unwrap_or_default(),
            private_key: std::env::var("WECHAT_PRIVATE_KEY").ok(),
            public_key: std::env::var("WECHAT_PUBLIC_KEY").ok(),
            merchant_id: std::env::var("WECHAT_MERCHANT_ID").unwrap_or_default(),
            api_url: std::env::var("WECHAT_API_URL")
                .unwrap_or_else(|_| "https://api.mch.weixin.qq.com".to_string()),
            timeout: Duration::from_secs(
                std::env::var("PAYMENT_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT),
            ),
        };

        let alipay_config = PaymentConfig {
            app_id: std::env::var("ALIPAY_APP_ID").unwrap_or_default(),
            api_key: std::env::var("ALIPAY_API_KEY").unwrap_or_default(),
            private_key: std::env::var("ALIPAY_PRIVATE_KEY").ok(),
            public_key: std::env::var("ALIPAY_PUBLIC_KEY").ok(),
            merchant_id: std::env::var("ALIPAY_MERCHANT_ID").unwrap_or_default(),
            api_url: std::env::var("ALIPAY_API_URL")
                .unwrap_or_else(|_| "https://openapi.alipay.com/gateway.do".to_string()),
            timeout: Duration::from_secs(
                std::env::var("PAYMENT_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT),
            ),
        };

        let boost_config = PaymentConfig {
            app_id: std::env::var("BOOST_APP_ID").unwrap_or_default(),
            api_key: std::env::var("BOOST_API_KEY").unwrap_or_default(),
            private_key: None,
            public_key: None,
            merchant_id: std::env::var("BOOST_MERCHANT_ID").unwrap_or_default(),
            api_url: std::env::var("BOOST_API_URL")
                .unwrap_or_else(|_| "https://api.boost.com.my/v1".to_string()),
            timeout: Duration::from_secs(
                std::env::var("PAYMENT_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_TIMEOUT),
            ),
        };

        // 创建处理器实例
        let wechat_processor = Arc::new(WechatPayProcessor::new(wechat_config));
        let alipay_processor = Arc::new(AlipayProcessor::new(alipay_config));
        let boost_processor = Arc::new(BoostWalletProcessor::new(boost_config));

        // 创建处理器工厂
        let mut processor_factory = PaymentProcessorFactory::new();

        // 注册处理器
        // 微信支付
        processor_factory.register(
            PaymentMethodType::App,
            PaymentRegion::China,
            "default_merchant",
            wechat_processor.clone(),
        );
        processor_factory.register(
            PaymentMethodType::JsApi,
            PaymentRegion::China,
            "default_merchant",
            wechat_processor.clone(),
        );
        processor_factory.register(
            PaymentMethodType::Native,
            PaymentRegion::China,
            "default_merchant",
            wechat_processor.clone(),
        );

        // 支付宝
        processor_factory.register(
            PaymentMethodType::App,
            PaymentRegion::China,
            "default_merchant",
            alipay_processor.clone(),
        );
        processor_factory.register(
            PaymentMethodType::Web,
            PaymentRegion::China,
            "default_merchant",
            alipay_processor.clone(),
        );
        processor_factory.register(
            PaymentMethodType::Native,
            PaymentRegion::China,
            "default_merchant",
            alipay_processor.clone(),
        );

        // Boost钱包
        processor_factory.register(
            PaymentMethodType::BoostWallet,
            PaymentRegion::Malaysia,
            "default_merchant",
            boost_processor.clone(),
        );

        // 创建仓库实例
        let order_repository = Arc::new(PaymentOrderRepositoryImpl::new(db_pool.clone()));
        let transaction_repository = Arc::new(PaymentTransactionRepositoryImpl::new(db_pool.clone()));
        let refund_repository = Arc::new(RefundOrderRepositoryImpl::new(db_pool));

        // 创建支付服务实例
        let processor_factory = Arc::new(processor_factory);
        let payment_service = Arc::new(PaymentServiceImpl::new(
            processor_factory,
            order_repository,
            transaction_repository,
            refund_repository,
        ));

        Self {
            payment_service,
        }
    }
}