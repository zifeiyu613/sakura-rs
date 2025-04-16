use crate::adapters::PaymentAdapterRegistry;
use crate::config::AppConfig;
use crate::domain::entities::{PaymentOrder, Transaction, Refund};
use crate::domain::enums::{PaymentStatus, TransactionStatus, TransactionType, RefundStatus};
use crate::repositories::{
    OrderRepositoryTrait, TransactionRepositoryTrait,
    RefundRepositoryTrait, MerchantRepositoryTrait
};
use crate::services::notification::NotificationService;
use crate::services::risk::RiskControlService;
use crate::utils::errors::{ServiceError, AdapterError};
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error, warn};

pub mod dto;
use dto::{
    CreateOrderRequest, OrderResponse, PaymentStatusResponse,
    RefundRequest, RefundResponse, PaymentChannelResponse
};

// 支付服务
pub struct PaymentService {
    config: AppConfig,
    order_repository: Arc<dyn OrderRepositoryTrait>,
    transaction_repository: Arc<dyn TransactionRepositoryTrait>,
    refund_repository: Arc<dyn RefundRepositoryTrait>,
    merchant_repository: Arc<dyn MerchantRepositoryTrait>,
    payment_adapters: Arc<PaymentAdapterRegistry>,
    risk_service: Arc<RiskControlService>,
    notification_service: Arc<NotificationService>,
}

impl PaymentService {
    pub fn new(
        config: AppConfig,
        order_repository: Arc<dyn OrderRepositoryTrait>,
        transaction_repository: Arc<dyn TransactionRepositoryTrait>,
        refund_repository: Arc<dyn RefundRepositoryTrait>,
        merchant_repository: Arc<dyn MerchantRepositoryTrait>,
        payment_adapters: Arc<PaymentAdapterRegistry>,
        risk_service: Arc<RiskControlService>,
        notification_service: Arc<NotificationService>,
    ) -> Self {
        Self {
            config,
            order_repository,
            transaction_repository,
            refund_repository,
            merchant_repository,
            payment_adapters,
            risk_service,
            notification_service,
        }
    }

    // 创建支付订单
    pub async fn create_order(&self, request: CreateOrderRequest) -> Result<OrderResponse, ServiceError> {
        info!("Creating payment order for merchant: {}", request.merchant_id);

        // 验证商户信息
        let merchant = self.merchant_repository.find_by_id(&request.merchant_id).await?
            .ok_or_else(|| ServiceError::MerchantNotFound(request.merchant_id.clone()))?;

        // 检查订单是否已存在
        if let Some(existing_order) = self.order_repository.find_by_merchant_order_id(
            &request.merchant_id,
            &request.merchant_order_id
        ).await? {
            warn!("Order already exists: {}", existing_order.id);
            return Err(ServiceError::OrderAlreadyExists(request.merchant_order_id));
        }

        // 风控检查
        self.risk_service.check_order_risk(&request).await?;

        // 创建订单实体
        let order = PaymentOrder::new(
            request.merchant_id,
            request.merchant_order_id,
            request.amount,
            request.currency,
            request.channel,
            request.method,
            request.subject,
            request.callback_url,
            request.return_url,
            request.client_ip,
            request.metadata,
            request.expire_time,
        );

        // 存储订单
        self.order_repository.create(&order).await?;

        // 获取支付渠道适配器
        let adapter = self.payment_adapters.get(order.channel)
            .ok_or_else(|| ServiceError::UnsupportedChannel(format!("{:?}", order.channel)))?;

        // 创建支付交易
        let transaction = Transaction::new(
            order.id,
            order.amount,
            TransactionType::Payment,
        );

        self.transaction_repository.create(&transaction).await?;

        // 请求支付渠道创建交易
        let payment_result = adapter.create_payment(&order).await
            .map_err(|e| ServiceError::AdapterError(e))?;

        // 更新交易信息
        let mut updated_transaction = transaction.clone();
        if let Some(channel_transaction_id) = &payment_result.channel_transaction_id {
            updated_transaction.set_channel_transaction_id(channel_transaction_id.clone());
        }

        updated_transaction.set_gateway_response(
            None,
            None,
            Some(payment_result.raw_response.clone()),
        );

        self.transaction_repository.update(&updated_transaction).await?;

        // 构建响应
        let response = OrderResponse {
            order_id: order.id,
            merchant_order_id: order.merchant_order_id,
            amount: order.amount,
            currency: order.currency,
            status: order.status,
            payment_url: payment_result.payment_url,
            qr_code: payment_result.qr_code,
            html_form: payment_result.html_form,
            app_parameters: payment_result.app_parameters,
            expire_time: order.expire_time,
            created_at: order.created_at,
        };

        info!("Payment order created: {}", order.id);
        Ok(response)
    }

    // 查询支付订单状态
    pub async fn query_order_status(&self, order_id: Uuid) -> Result<PaymentStatusResponse, ServiceError> {
        info!("Querying payment status for order: {}", order_id);

        // 获取订单信息
        let order = self.order_repository.find_by_id(order_id).await?
            .ok_or_else(|| ServiceError::OrderNotFound(order_id.to_string()))?;

        // 如果订单已经成功，直接返回结果
        if order.is_paid() {
            return Ok(PaymentStatusResponse {
                order_id: order.id,
                merchant_order_id: order.merchant_order_id,
                status: order.status,
                paid_amount: Some(order.amount),
                paid_time: None, // 从交易记录中获取
                channel_transaction_id: None, // 从交易记录中获取
            });
        }

        // 如果订单已关闭、失败或过期，直接返回结果
        if order.is_closed() {
            return Ok(PaymentStatusResponse {
                order_id: order.id,
                merchant_order_id: order.merchant_order_id,
                status: order.status,
                paid_amount: None,
                paid_time: None,
                channel_transaction_id: None,
            });
        }

        // 获取支付渠道适配器
        let adapter = self.payment_adapters.get(order.channel)
            .ok_or_else(|| ServiceError::UnsupportedChannel(format!("{:?}", order.channel)))?;

        // 查询支付渠道的支付状态
        let payment_status = adapter.query_payment(&order).await
            .map_err(|e| ServiceError::AdapterError(e))?;

        // 如果支付成功，更新订单状态
        if payment_status.is_paid && !order.is_paid() {
            let mut updated_order = order.clone();
            updated_order.update_status(PaymentStatus::Success);

            self.order_repository.update(&updated_order).await?;

            // 更新交易记录
            let transactions = self.transaction_repository.find_by_order_id(order_id).await?;
            if let Some(mut transaction) = transactions.into_iter().next() {
                transaction.update_status(TransactionStatus::Success);
                if let Some(transaction_id) = &payment_status.transaction_id {
                    transaction.set_channel_transaction_id(transaction_id.clone());
                }

                self.transaction_repository.update(&transaction).await?;

                // 发送通知
                self.notification_service.send_payment_success_notification(&updated_order, &transaction).await?;
            }

            // 返回更新后的状态
            return Ok(PaymentStatusResponse {
                order_id: updated_order.id,
                merchant_order_id: updated_order.merchant_order_id,
                status: updated_order.status,
                paid_amount: payment_status.paid_amount,
                paid_time: payment_status.paid_time,
                channel_transaction_id: payment_status.transaction_id,
            });
        }

        // 返回当前状态
        Ok(PaymentStatusResponse {
            order_id: order.id,
            merchant_order_id: order.merchant_order_id,
            status: order.status,
            paid_amount: payment_status.paid_amount,
            paid_time: payment_status.paid_time,
            channel_transaction_id: payment_status.transaction_id,
        })
    }

    // 处理支付回调通知
    pub async fn handle_payment_notification(
        &self,
        channel: crate::domain::enums::PaymentChannel,
        notification_data: &str,
    ) -> Result<String, ServiceError> {
        info!("Handling payment notification from channel: {:?}", channel);

        // 获取支付渠道适配器
        let adapter = self.payment_adapters.get(channel)
            .ok_or_else(|| ServiceError::UnsupportedChannel(format!("{:?}", channel)))?;

        // 处理通知数据
        let notification = adapter.handle_notification(notification_data).await
            .map_err(|e| ServiceError::AdapterError(e))?;

        // 查找对应的订单
        let order = self.order_repository.find_by_merchant_order_id(
            &notification.order_id.split('_').next().unwrap_or("").to_string(),
            &notification.order_id,
        ).await?;

        let order = match order {
            Some(o) => o,
            None => {
                // 订单不存在，记录错误但仍返回成功响应给支付网关
                error!("Order not found for notification: {}", notification.order_id);
                return Ok(notification.response_data);
            }
        };

        // 如果订单已经处理过，直接返回成功
        if order.is_paid() || order.is_closed() {
            info!("Order {} already processed, status: {:?}", order.id, order.status);
            return Ok(notification.response_data);
        }

        // 更新订单状态
        let mut updated_order = order.clone();
        if notification.is_successful {
            updated_order.update_status(PaymentStatus::Success);
        } else {
            updated_order.update_status(PaymentStatus::Failed);
        }

        self.order_repository.update(&updated_order).await?;

        // 更新交易记录
        let transactions = self.transaction_repository.find_by_order_id(order.id).await?;
        if let Some(mut transaction) = transactions.into_iter().next() {
            if notification.is_successful {
                transaction.update_status(TransactionStatus::Success);
            } else {
                transaction.update_status(TransactionStatus::Failed);
            }

            transaction.set_channel_transaction_id(notification.transaction_id.clone());

            self.transaction_repository.update(&transaction).await?;

            // 发送通知
            if notification.is_successful {
                self.notification_service.send_payment_success_notification(&updated_order, &transaction).await?;
            } else {
                self.notification_service.send_payment_failed_notification(&updated_order, &transaction).await?;
            }
        }

        info!("Payment notification processed for order: {}", order.id);
        Ok(notification.response_data)
    }

    // 发起退款
    pub async fn create_refund(&self, request: RefundRequest) -> Result<RefundResponse, ServiceError> {
        info!("Creating refund for order: {}", request.order_id);

        // 获取订单信息
        let order = self.order_repository.find_by_id(request.order_id).await?
            .ok_or_else(|| ServiceError::OrderNotFound(request.order_id.to_string()))?;

        // 检查订单是否可以退款
        if !order.can_refund() {
            return Err(ServiceError::OrderNotRefundable(request.order_id.to_string()));
        }

        // 获取订单的交易记录
        let transactions = self.transaction_repository.find_by_order_id(order.id).await?;

        let transaction = transactions.into_iter()
            .find(|t| t.is_successful() && t.transaction_type == TransactionType::Payment)
            .ok_or_else(|| ServiceError::NoSuccessfulTransaction(order.id.to_string()))?;

        // 检查退款金额是否有效
        if request.amount > transaction.amount {
            return Err(ServiceError::InvalidRefundAmount(format!(
                "Refund amount {} exceeds original payment amount {}",
                request.amount, transaction.amount
            )));
        }

        // 创建退款记录
        let refund = Refund::new(
            order.id,
            transaction.id,
            request.amount,
            request.reason.clone(),
        );

        self.refund_repository.create(&refund).await?;

        // 获取支付渠道适配器
        let adapter = self.payment_adapters.get(order.channel)
            .ok_or_else(|| ServiceError::UnsupportedChannel(format!("{:?}", order.channel)))?;

        // 请求支付渠道退款
        let refund_result = adapter.create_refund(&refund, &order).await
            .map_err(|e| ServiceError::AdapterError(e))?;

        // 更新退款记录
        let mut updated_refund = refund.clone();

        if refund_result.is_accepted {
            updated_refund.update_status(RefundStatus::Processing);
        } else {
            updated_refund.update_status(RefundStatus::Failed);
        }

        if let Some(refund_id) = &refund_result.channel_refund_id {
            updated_refund.set_channel_refund_id(refund_id.clone());
        }

        updated_refund.set_gateway_response(
            None,
            None,
            Some(refund_result.raw_response.clone()),
        );

        self.refund_repository.update(&updated_refund).await?;

        // 创建退款交易记录
        let refund_transaction = Transaction::new(
            order.id,
            request.amount,
            TransactionType::Refund,
        );

        self.transaction_repository.create(&refund_transaction).await?;

        // 构建响应
        let response = RefundResponse {
            refund_id: updated_refund.id,
            order_id: order.id,
            amount: updated_refund.amount,
            status: updated_refund.status,
            created_at: updated_refund.created_at,
        };

        info!("Refund created: {}", updated_refund.id);
        Ok(response)
    }

    // 查询退款状态
    pub async fn query_refund_status(&self, refund_id: Uuid) -> Result<RefundResponse, ServiceError> {
        info!("Querying refund status: {}", refund_id);

        // 获取退款记录
        let refund = self.refund_repository.find_by_id(refund_id).await?
            .ok_or_else(|| ServiceError::RefundNotFound(refund_id.to_string()))?;

        // 如果退款已完成或失败，直接返回结果
        if refund.is_successful() || refund.is_failed() {
            return Ok(RefundResponse {
                refund_id: refund.id,
                order_id: refund.order_id,
                amount: refund.amount,
                status: refund.status,
                created_at: refund.created_at,
            });
        }

        // 获取订单信息
        let order = self.order_repository.find_by_id(refund.order_id).await?
            .ok_or_else(|| ServiceError::OrderNotFound(refund.order_id.to_string()))?;

        // 获取支付渠道适配器
        let adapter = self.payment_adapters.get(order.channel)
            .ok_or_else(|| ServiceError::UnsupportedChannel(format!("{:?}", order.channel)))?;

        // 查询支付渠道的退款状态
        let refund_status = adapter.query_refund(&refund, &order).await
            .map_err(|e| ServiceError::AdapterError(e))?;

        // 如果退款状态有变化，更新退款记录
        if (refund_status.is_success && !refund.is_successful()) || (!refund_status.is_success && refund.is_pending()) {
            let mut updated_refund = refund.clone();

            if refund_status.is_success {
                updated_refund.update_status(RefundStatus::Success);
            } else {
                updated_refund.update_status(RefundStatus::Failed);
            }

            self.refund_repository.update(&updated_refund).await?;

            // 发送通知
            if refund_status.is_success {
                self.notification_service.send_refund_success_notification(&updated_refund, &order).await?;
            } else {
                self.notification_service.send_refund_failed_notification(&updated_refund, &order).await?;
            }

            return Ok(RefundResponse {
                refund_id: updated_refund.id,
                order_id: updated_refund.order_id,
                amount: updated_refund.amount,
                status: updated_refund.status,
                created_at: updated_refund.created_at,
            });
        }

        // 返回当前状态
        Ok(RefundResponse {
            refund_id: refund.id,
            order_id: refund.order_id,
            amount: refund.amount,
            status: refund.status,
            created_at: refund.created_at,
        })
    }

    // 获取可用的支付渠道
    pub async fn get_available_payment_channels(
        &self,
        merchant_id: &str,
        currency: crate::domain::enums::Currency,
        amount: Decimal,
    ) -> Result<Vec<PaymentChannelResponse>, ServiceError> {
        // 获取商户信息
        let merchant = self.merchant_repository.find_by_id(merchant_id).await?
            .ok_or_else(|| ServiceError::MerchantNotFound(merchant_id.to_string()))?;

        // 根据商户配置、货币和金额筛选可用支付渠道
        let mut available_channels = Vec::new();

        // 这里简化处理，实际应根据商户配置、区域、货币等因素动态确定可用渠道
        if currency == crate::domain::enums::Currency::CNY {
            // 国内支付渠道
            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::Wechat,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::App,
                    crate::domain::enums::PaymentMethod::MiniProgram,
                    crate::domain::enums::PaymentMethod::QrCode,
                ],
                display_name: "微信支付".to_string(),
                logo_url: "https://example.com/logos/wechat.png".to_string(),
            });

            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::Alipay,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::App,
                    crate::domain::enums::PaymentMethod::MiniProgram,
                    crate::domain::enums::PaymentMethod::QrCode,
                ],
                display_name: "支付宝".to_string(),
                logo_url: "https://example.com/logos/alipay.png".to_string(),
            });

            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::UnionPay,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::App,
                    crate::domain::enums::PaymentMethod::QrCode,
                ],
                display_name: "银联支付".to_string(),
                logo_url: "https://example.com/logos/unionpay.png".to_string(),
            });
        } else if currency == crate::domain::enums::Currency::MYR {
            // 马来西亚支付渠道
            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::Boost,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::App,
                ],
                display_name: "Boost".to_string(),
                logo_url: "https://example.com/logos/boost.png".to_string(),
            });

            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::GrabPay,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::App,
                ],
                display_name: "GrabPay".to_string(),
                logo_url: "https://example.com/logos/grabpay.png".to_string(),
            });

            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::TouchNGo,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::App,
                ],
                display_name: "Touch 'n Go".to_string(),
                logo_url: "https://example.com/logos/tng.png".to_string(),
            });
        } else {
            // 国际支付渠道
            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::PayPal,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::Web,
                ],
                display_name: "PayPal".to_string(),
                logo_url: "https://example.com/logos/paypal.png".to_string(),
            });

            available_channels.push(PaymentChannelResponse {
                channel: crate::domain::enums::PaymentChannel::Stripe,
                methods: vec![
                    crate::domain::enums::PaymentMethod::H5,
                    crate::domain::enums::PaymentMethod::Web,
                ],
                display_name: "Stripe".to_string(),
                logo_url: "https://example.com/logos/stripe.png".to_string(),
            });
        }

        Ok(available_channels)
    }
}
