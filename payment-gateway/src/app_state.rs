use crate::config::AppConfig;
use crate::adapters::PaymentAdapterRegistry;
use crate::services::{
    payment::PaymentService,
    notification::NotificationService,
    risk::RiskControlService,
};
use crate::repositories::{
    OrderRepository,
    TransactionRepository,
    RefundRepository,
    MerchantRepository,
};
use sqlx::MySqlPool;
use redis::aio::ConnectionManager as RedisConnectionManager;
use lapin::Connection as RabbitMqConnection;
use std::sync::Arc;

/// 应用状态，包含共享资源和服务
pub struct AppState {
    pub config: AppConfig,

    // 数据库和缓存
    pub db_pool: MySqlPool,
    pub redis_manager: RedisConnectionManager,
    pub mq_connection: Arc<RabbitMqConnection>,

    // 仓储层
    pub order_repository: Arc<OrderRepository>,
    pub transaction_repository: Arc<TransactionRepository>,
    pub refund_repository: Arc<RefundRepository>,
    pub merchant_repository: Arc<MerchantRepository>,

    // 服务层
    pub payment_service: Arc<PaymentService>,
    pub notification_service: Arc<NotificationService>,
    pub risk_service: Arc<RiskControlService>,

    // 支付渠道适配器
    pub payment_adapters: Arc<PaymentAdapterRegistry>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        db_pool: MySqlPool,
        redis_manager: RedisConnectionManager,
        mq_connection: Arc<RabbitMqConnection>,
    ) -> Self {
        // 初始化仓储
        let order_repository = Arc::new(OrderRepository::new(db_pool.clone()));
        let transaction_repository = Arc::new(TransactionRepository::new(db_pool.clone()));
        let refund_repository = Arc::new(RefundRepository::new(db_pool.clone()));
        let merchant_repository = Arc::new(MerchantRepository::new(db_pool.clone()));

        // 初始化支付渠道适配器
        let payment_adapters = Arc::new(PaymentAdapterRegistry::new(&config));

        // 初始化服务
        let risk_service = Arc::new(RiskControlService::new(
            config.clone(),
            redis_manager.clone(),
        ));

        let notification_service = Arc::new(NotificationService::new(
            config.clone(),
            mq_connection.clone(),
            order_repository.clone(),
            transaction_repository.clone(),
        ));

        let payment_service = Arc::new(PaymentService::new(
            config.clone(),
            order_repository.clone(),
            transaction_repository.clone(),
            refund_repository.clone(),
            merchant_repository.clone(),
            payment_adapters.clone(),
            risk_service.clone(),
            notification_service.clone(),
        ));

        Self {
            config,
            db_pool,
            redis_manager,
            mq_connection,
            order_repository,
            transaction_repository,
            refund_repository,
            merchant_repository,
            payment_service,
            notification_service,
            risk_service,
            payment_adapters,
        }
    }
}
