use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::payment::{
    PaymentProcessor, PaymentProcessorFactory, PaymentRequest, RefundRequest
};
use crate::domain::service::{
    CreatePaymentRequest, CreatePaymentResponse, CreateRefundRequest,
    CreateRefundResponse, PaymentService, VerifyPaymentRequest
};
use crate::infrastructure::repository::{
    PaymentOrderRepository, PaymentTransactionRepository, RefundOrderRepository
};

use std::collections::HashMap;

use crate::domain::models::{
    PaymentChannelType, PaymentMethodType, PaymentOrder, PaymentRegion,
    PaymentStatus, PaymentTransaction, RefundOrder
};

/// 支付服务请求
#[derive(Debug, Clone)]
pub struct CreatePaymentRequest {
    pub merchant_id: String,
    pub order_id: String,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub subject: String,
    pub description: Option<String>,
    pub channel: PaymentChannelType,
    pub method: PaymentMethodType,
    pub region: PaymentRegion,
    pub callback_url: String,
    pub return_url: Option<String>,
    pub client_ip: Option<String>,
    pub metadata: HashMap<String, String>,
    pub device_info: Option<HashMap<String, String>>,
    pub user_info: Option<HashMap<String, String>>,
}

/// 支付服务响应
#[derive(Debug, Clone)]
pub struct CreatePaymentResponse {
    pub order_id: String,
    pub payment_id: String,
    pub redirect_url: Option<String>,
    pub html_form: Option<String>,
    pub qr_code: Option<String>,
    pub sdk_params: Option<HashMap<String, String>>,
}

/// 验证支付请求
#[derive(Debug, Clone)]
pub struct VerifyPaymentRequest {
    pub channel: PaymentChannelType,
    pub method: PaymentMethodType,
    pub payload: String,
    pub headers: HashMap<String, String>,
}

/// 退款请求
#[derive(Debug, Clone)]
pub struct CreateRefundRequest {
    pub merchant_id: String,
    pub payment_order_id: String,
    pub amount: rust_decimal::Decimal,
    pub reason: String,
    pub metadata: HashMap<String, String>,
}

/// 退款响应
#[derive(Debug, Clone)]
pub struct CreateRefundResponse {
    pub refund_id: String,
    pub status: PaymentStatus,
}

/// 支付服务接口
#[async_trait::async_trait]
pub trait PaymentService: Send + Sync {
    /// 创建支付订单
    async fn create_payment(&self, request: CreatePaymentRequest) -> Result<CreatePaymentResponse, String>;

    /// 验证支付结果
    async fn verify_payment(&self, request: VerifyPaymentRequest) -> Result<PaymentTransaction, String>;

    /// 查询支付订单
    async fn query_payment(&self, order_id: &str) -> Result<PaymentOrder, String>;

    /// 查询支付交易
    async fn query_transaction(&self, transaction_id: &str) -> Result<PaymentTransaction, String>;

    /// 创建退款
    async fn create_refund(&self, request: CreateRefundRequest) -> Result<CreateRefundResponse, String>;

    /// 查询退款
    async fn query_refund(&self, refund_id: &str) -> Result<RefundOrder, String>;
}



pub struct PaymentServiceImpl {
    processor_factory: Arc<PaymentProcessorFactory>,
    order_repository: Arc<dyn PaymentOrderRepository>,
    transaction_repository: Arc<dyn PaymentTransactionRepository>,
    refund_repository: Arc<dyn RefundOrderRepository>,
}

impl PaymentServiceImpl {
    pub fn new(
        processor_factory: Arc<PaymentProcessorFactory>,
        order_repository: Arc<dyn PaymentOrderRepository>,
        transaction_repository: Arc<dyn PaymentTransactionRepository>,
        refund_repository: Arc<dyn RefundOrderRepository>,
    ) -> Self {
        Self {
            processor_factory,
            order_repository,
            transaction_repository,
            refund_repository,
        }
    }

    // 获取支付处理器
    async fn get_processor(
        &self,
        method: PaymentMethodType,
        region: PaymentRegion,
        merchant_id: &str,
    ) -> Result<Arc<dyn PaymentProcessor>, String> {
        self.processor_factory
            .get_processor(method, region, merchant_id)
            .ok_or_else(|| format!("Unsupported payment method: {:?} in region: {:?}", method, region))
    }
}

#[async_trait::async_trait]
impl PaymentService for PaymentServiceImpl {
    async fn create_payment(&self, request: CreatePaymentRequest) -> Result<CreatePaymentResponse, String> {
        // 创建支付订单
        let payment_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let order = PaymentOrder {
            id: payment_id.clone(),
            merchant_id: request.merchant_id.clone(),
            order_id: request.order_id.clone(),
            amount: request.amount,
            currency: request.currency.clone(),
            status: PaymentStatus::Created,
            channel: request.channel,
            method: request.method,
            region: request.region,
            subject: request.subject.clone(),
            description: request.description.clone(),
            metadata: request.metadata.clone(),
            created_at: now,
            updated_at: now,
            expires_at: Some(now + chrono::Duration::minutes(30)), // 30分钟过期
            callback_url: request.callback_url.clone(),
            return_url: request.return_url.clone(),
            client_ip: request.client_ip.clone(),
        };

        // 保存支付订单
        self.order_repository.save(&order).await.map_err(|e| e.to_string())?;

        // 获取支付处理器
        let processor = self.get_processor(request.method, request.region, &request.merchant_id).await?;

        // 构建支付请求
        let payment_request = PaymentRequest {
            order: order.clone(),
            device_info: request.device_info.clone(),
            user_info: request.user_info.clone(),
        };

        // 创建支付
        let payment_response = processor.create_payment(payment_request).await
            .map_err(|e| format!("Failed to create payment: {}", e))?;

        // 保存交易记录
        self.transaction_repository.save(&payment_response.transaction).await
            .map_err(|e| e.to_string())?;

        // 更新订单状态
        let mut updated_order = order.clone();
        updated_order.status = PaymentStatus::Processing;
        self.order_repository.update(&updated_order).await
            .map_err(|e| e.to_string())?;

        // 构建响应
        let response = CreatePaymentResponse {
            order_id: request.order_id,
            payment_id,
            redirect_url: payment_response.redirect_url,
            html_form: payment_response.html_form,
            qr_code: payment_response.qr_code,
            sdk_params: payment_response.sdk_params,
        };

        Ok(response)
    }

    async fn verify_payment(&self, request: VerifyPaymentRequest) -> Result<PaymentTransaction, String> {
        // 从请求中提取支付方式信息
        let method = request.method;
        let region = PaymentRegion::China; // 这里需要根据实际情况获取，可以从请求中提取或从数据库查询

        // 查找合适的处理器
        // 注意：这里简化了实现，实际应该从通知中提取订单号，然后查询订单获取merchant_id
        let merchant_id = "default_merchant"; // 应该从请求或数据库中获取
        let processor = self.get_processor(method, region, merchant_id).await?;

        // 验证支付结果
        let transaction = processor.verify_payment(request.payload, request.headers).await
            .map_err(|e| format!("Failed to verify payment: {}", e))?;

        // 查询原始订单
        let order = self.order_repository.find_by_id(&transaction.payment_order_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Order not found: {}", transaction.payment_order_id))?;

        // 查询原始交易
        let original_transaction = self.transaction_repository
            .find_by_payment_order_id(&transaction.payment_order_id).await
            .map_err(|e| e.to_string())?
            .first()
            .cloned()
            .ok_or_else(|| format!("Transaction not found for order: {}", transaction.payment_order_id))?;

        // 更新交易记录
        let mut updated_transaction = original_transaction.clone();
        updated_transaction.status = transaction.status;
        updated_transaction.channel_transaction_id = transaction.channel_transaction_id;
        updated_transaction.updated_at = Utc::now();
        updated_transaction.metadata = transaction.metadata;

        self.transaction_repository.update(&updated_transaction).await
            .map_err(|e| e.to_string())?;

        // 更新订单状态
        let mut updated_order = order;
        updated_order.status = transaction.status;
        updated_order.updated_at = Utc::now();

        self.order_repository.update(&updated_order).await
            .map_err(|e| e.to_string())?;

        Ok(updated_transaction)
    }

    async fn query_payment(&self, order_id: &str) -> Result<PaymentOrder, String> {
        // 查询订单
        let order = self.order_repository.find_by_order_id(order_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Order not found: {}", order_id))?;

        // 如果订单已完成，直接返回
        if order.status == PaymentStatus::Successful ||
            order.status == PaymentStatus::Failed ||
            order.status == PaymentStatus::Cancelled {
            return Ok(order);
        }

        // 获取处理器
        let processor = self.get_processor(order.method, order.region, &order.merchant_id).await?;

        // 查询支付状态
        let transaction = processor.query_payment(order_id).await
            .map_err(|e| format!("Failed to query payment: {}", e))?;

        // 更新交易记录
        let original_transactions = self.transaction_repository
            .find_by_payment_order_id(&order.id).await
            .map_err(|e| e.to_string())?;

        if let Some(original_transaction) = original_transactions.first() {
            let mut updated_transaction = original_transaction.clone();
            updated_transaction.status = transaction.status;
            updated_transaction.channel_transaction_id = transaction.channel_transaction_id;
            updated_transaction.updated_at = Utc::now();

            self.transaction_repository.update(&updated_transaction).await
                .map_err(|e| e.to_string())?;
        }

        // 更新订单状态
        let mut updated_order = order.clone();
        updated_order.status = transaction.status;
        updated_order.updated_at = Utc::now();

        self.order_repository.update(&updated_order).await
            .map_err(|e| e.to_string())?;

        Ok(updated_order)
    }

    async fn query_transaction(&self, transaction_id: &str) -> Result<PaymentTransaction, String> {
        // 查询交易
        let transaction = self.transaction_repository.find_by_id(transaction_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Transaction not found: {}", transaction_id))?;

        Ok(transaction)
    }

    async fn create_refund(&self, request: CreateRefundRequest) -> Result<CreateRefundResponse, String> {
        // 查询原始订单
        let order = self.order_repository.find_by_order_id(&request.payment_order_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Order not found: {}", request.payment_order_id))?;

        // 查询原始交易
        let transaction = self.transaction_repository
            .find_by_payment_order_id(&order.id).await
            .map_err(|e| e.to_string())?
            .first()
            .cloned()
            .ok_or_else(|| format!("Transaction not found for order: {}", order.id))?;

        // 检查订单状态
        if order.status != PaymentStatus::Successful {
            return Err(format!("Order is not in successful state: {}", order.status));
        }

        // 获取处理器
        let processor = self.get_processor(order.method, order.region, &request.merchant_id).await?;

        // 构建退款请求
        let refund_request = RefundRequest {
            payment_order_id: order.id.clone(),
            transaction_id: transaction.id.clone(),
            amount: request.amount,
            reason: request.reason.clone(),
            metadata: request.metadata.clone(),
        };

        // 创建退款
        let refund_response = processor.refund(refund_request).await
            .map_err(|e| format!("Failed to create refund: {}", e))?;

        // 保存退款记录
        self.refund_repository.save(&refund_response.refund_order).await
            .map_err(|e| e.to_string())?;

        // 如果退款成功，更新订单状态
        if refund_response.refund_order.status == PaymentStatus::Refunded {
            // 如果退款金额等于订单金额，标记为完全退款
            let refund_status = if request.amount == order.amount {
                PaymentStatus::Refunded
            } else {
                PaymentStatus::PartiallyRefunded
            };

            let mut updated_order = order;
            updated_order.status = refund_status;
            updated_order.updated_at = Utc::now();

            self.order_repository.update(&updated_order).await
                .map_err(|e| e.to_string())?;
        }

        // 构建响应
        let response = CreateRefundResponse {
            refund_id: refund_response.refund_order.id.clone(),
            status: refund_response.refund_order.status,
        };

        Ok(response)
    }

    async fn query_refund(&self, refund_id: &str) -> Result<RefundOrder, String> {
        // 查询退款记录
        let refund = self.refund_repository.find_by_id(refund_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Refund not found: {}", refund_id))?;

        // 如果退款已完成，直接返回
        if refund.status == PaymentStatus::Refunded || refund.status == PaymentStatus::Failed {
            return Ok(refund);
        }

        // 查询原始订单
        let order = self.order_repository.find_by_id(&refund.payment_order_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Order not found: {}", refund.payment_order_id))?;

        // 获取处理器
        let merchant_id = order.merchant_id.clone();
        let processor = self.get_processor(order.method, order.region, &merchant_id).await?;

        // 查询退款状态
        let refund_order = processor.query_refund(refund.refund_id.as_ref().unwrap_or(&refund.id)).await
            .map_err(|e| format!("Failed to query refund: {}", e))?;

        // 更新退款记录
        let mut updated_refund = refund.clone();
        updated_refund.status = refund_order.status;
        updated_refund.updated_at = Utc::now();

        self.refund_repository.update(&updated_refund).await
            .map_err(|e| e.to_string())?;

        // 如果退款完成，更新订单状态
        if refund_order.status == PaymentStatus::Refunded {
            // 检查是否有其他退款
            let all_refunds = self.refund_repository.find_by_payment_order_id(&order.id).await
                .map_err(|e| e.to_string())?;

            // 计算总退款金额
            let total_refund_amount = all_refunds.iter()
                .filter(|r| r.status == PaymentStatus::Refunded)
                .fold(rust_decimal::Decimal::ZERO, |acc, r| acc + r.amount);

            // 如果退款金额等于订单金额，标记为完全退款
            let refund_status = if total_refund_amount == order.amount {
                PaymentStatus::Refunded
            } else {
                PaymentStatus::PartiallyRefunded
            };

            let mut updated_order = order;
            updated_order.status = refund_status;
            updated_order.updated_at = Utc::now();

            self.order_repository.update(&updated_order).await
                .map_err(|e| e.to_string())?;
        }

        Ok(updated_refund)
    }
}