use crate::config::AppConfig;
use crate::domain::enums::RiskLevel;
use crate::services::payment::dto::CreateOrderRequest;
use crate::utils::errors::ServiceError;
use redis::aio::ConnectionManager as RedisConnectionManager;
use redis::AsyncCommands;
use tracing::{info, warn};
use uuid::Uuid;
use std::time::Duration;

// 风控服务
pub struct RiskControlService {
    config: AppConfig,
    redis_manager: RedisConnectionManager,
}

impl RiskControlService {
    pub fn new(
        config: AppConfig,
        redis_manager: RedisConnectionManager,
    ) -> Self {
        Self {
            config,
            redis_manager,
        }
    }

    // 检查订单风险
    pub async fn check_order_risk(&self, request: &CreateOrderRequest) -> Result<(), ServiceError> {
        info!("Checking risk for order: {}", request.merchant_order_id);

        // 1. 检查IP风险
        if let Some(client_ip) = &request.client_ip {
            let ip_risk = self.check_ip_risk(client_ip).await?;

            if ip_risk >= RiskLevel::High {
                warn!("High risk IP detected: {}", client_ip);
                return Err(ServiceError::RiskControlRejection(format!("High risk IP: {}", client_ip)));
            }
        }

        // 2. 检查交易频率限制
        let frequency_risk = self.check_transaction_frequency(&request.merchant_id).await?;

        if frequency_risk >= RiskLevel::High {
            warn!("High transaction frequency for merchant: {}", request.merchant_id);
            return Err(ServiceError::RiskControlRejection("Transaction frequency limit exceeded".to_string()));
        }

        // 3. 检查金额风险
        let amount_risk = self.check_amount_risk(request.amount, &request.merchant_id).await?;

        if amount_risk >= RiskLevel::High {
            warn!("Unusual transaction amount: {}", request.amount);
            return Err(ServiceError::RiskControlRejection("Unusual transaction amount".to_string()));
        }

        // 4. 记录交易请求，用于频率控制
        self.record_transaction_request(&request.merchant_id).await?;

        info!("Risk check passed for order: {}", request.merchant_order_id);
        Ok(())
    }

    // 检查IP风险
    async fn check_ip_risk(&self, ip: &str) -> Result<RiskLevel, ServiceError> {
        let mut conn = self.redis_manager.clone();

        // 检查IP是否在黑名单中
        let is_blacklisted: bool = conn.exists(format!("risk:ip:blacklist:{}", ip)).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        if is_blacklisted {
            return Ok(RiskLevel::Critical);
        }

        // 检查IP近期失败次数
        let failure_count: Option<i64> = conn.get(format!("risk:ip:failures:{}", ip)).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        if let Some(count) = failure_count {
            if count > 10 {
                return Ok(RiskLevel::High);
            } else if count > 5 {
                return Ok(RiskLevel::Medium);
            }
        }

        Ok(RiskLevel::Low)
    }

    // 检查交易频率
    async fn check_transaction_frequency(&self, merchant_id: &str) -> Result<RiskLevel, ServiceError> {
        let mut conn = self.redis_manager.clone();

        // 获取最近5分钟的交易次数
        let count: Option<i64> = conn.get(format!("risk:merchant:freq:{}:5min", merchant_id)).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        if let Some(count) = count {
            // 根据商户等级或配置来确定限制
            let merchant_limit = 100; // 这里简化处理，实际应从商户配置获取

            if count > merchant_limit {
                return Ok(RiskLevel::High);
            } else if count > merchant_limit * 0.8 {
                return Ok(RiskLevel::Medium);
            }
        }

        Ok(RiskLevel::Low)
    }

    // 检查金额风险
    async fn check_amount_risk(&self, amount: rust_decimal::Decimal, merchant_id: &str) -> Result<RiskLevel, ServiceError> {
        let mut conn = self.redis_manager.clone();

        // 获取商户历史平均交易金额
        let avg_amount: Option<String> = conn.get(format!("risk:merchant:avg_amount:{}", merchant_id)).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        if let Some(avg_str) = avg_amount {
            if let Ok(avg) = avg_str.parse::<rust_decimal::Decimal>() {
                // 如果当前交易金额是平均值的5倍以上，判定为高风险
                if amount > avg * rust_decimal::Decimal::from(5) {
                    return Ok(RiskLevel::High);
                } else if amount > avg * rust_decimal::Decimal::from(3) {
                    return Ok(RiskLevel::Medium);
                }
            }
        }

        Ok(RiskLevel::Low)
    }

    // 记录交易请求，用于频率控制
    async fn record_transaction_request(&self, merchant_id: &str) -> Result<(), ServiceError> {
        let mut conn = self.redis_manager.clone();

        // 增加5分钟计数器
        let key_5min = format!("risk:merchant:freq:{}:5min", merchant_id);
        let _: () = conn.incr(&key_5min, 1).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        // 设置过期时间（如果尚未设置）
        let _: () = conn.expire_if_needed(&key_5min, 300, redis::SetExpiry::IfNeeded).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        // 增加1小时计数器
        let key_1hour = format!("risk:merchant:freq:{}:1hour", merchant_id);
        let _: () = conn.incr(&key_1hour, 1).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        // 设置过期时间（如果尚未设置）
        let _: () = conn.expire_if_needed(&key_1hour, 3600, redis::SetExpiry::IfNeeded).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        Ok(())
    }

    // 记录交易失败，用于风控分析
    pub async fn record_transaction_failure(&self, merchant_id: &str, ip: Option<&str>) -> Result<(), ServiceError> {
        let mut conn = self.redis_manager.clone();

        // 增加商户失败计数
        let merchant_key = format!("risk:merchant:failures:{}", merchant_id);
        let _: () = conn.incr(&merchant_key, 1).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        // 设置过期时间（24小时）
        let _: () = conn.expire_if_needed(&merchant_key, 86400, redis::SetExpiry::IfNeeded).await
            .map_err(|e| ServiceError::CacheError(e.to_string()))?;

        // 如果有IP，增加IP失败计数
        if let Some(ip) = ip {
            let ip_key = format!("risk:ip:failures:{}", ip);
            let _: () = conn.incr(&ip_key, 1).await
                .map_err(|e| ServiceError::CacheError(e.to_string()))?;

            // 设置过期时间（24小时）
            let _: () = conn.expire_if_needed(&ip_key, 86400, redis::SetExpiry::IfNeeded).await
                .map_err(|e| ServiceError::CacheError(e.to_string()))?;

            // 检查是否需要自动拉黑
            let count: i64 = conn.get(&ip_key).await
                .map_err(|e| ServiceError::CacheError(e.to_string()))?;

            if count > 20 {
                // 自动将IP加入黑名单
                let blacklist_key = format!("risk:ip:blacklist:{}", ip);
                let _: () = conn.set(&blacklist_key, 1).await
                    .map_err(|e| ServiceError::CacheError(e.to_string()))?;

                // 黑名单有效期7天
                let _: () = conn.expire(&blacklist_key, 7 * 86400).await
                    .map_err(|e| ServiceError::CacheError(e.to_string()))?;

                warn!("IP automatically blacklisted due to multiple failures: {}", ip);
            }
        }

        Ok(())
    }
}
