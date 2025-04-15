-- 支付订单表
CREATE TABLE payment_orders (
                                id VARCHAR(36) PRIMARY KEY,
                                merchant_id VARCHAR(64) NOT NULL,
                                order_id VARCHAR(64) NOT NULL,
                                amount DECIMAL(20, 6) NOT NULL,
                                currency VARCHAR(3) NOT NULL,
                                status VARCHAR(20) NOT NULL,
                                channel VARCHAR(20) NOT NULL,
                                method VARCHAR(20) NOT NULL,
                                region VARCHAR(20) NOT NULL,
                                subject VARCHAR(256) NOT NULL,
                                description TEXT,
                                metadata JSONB,
                                created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                                updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
                                expires_at TIMESTAMP WITH TIME ZONE,
                                callback_url TEXT NOT NULL,
                                return_url TEXT,
                                client_ip VARCHAR(39)
);

CREATE INDEX idx_payment_orders_order_id ON payment_orders(order_id);
CREATE INDEX idx_payment_orders_merchant_id ON payment_orders(merchant_id);
CREATE INDEX idx_payment_orders_status ON payment_orders(status);

-- 支付交易表
CREATE TABLE payment_transactions (
                                      id VARCHAR(36) PRIMARY KEY,
                                      payment_order_id VARCHAR(36) NOT NULL,
                                      transaction_id VARCHAR(64) NOT NULL,
                                      channel_transaction_id VARCHAR(64),
                                      amount DECIMAL(20, 6) NOT NULL,
                                      status VARCHAR(20) NOT NULL,
                                      created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                                      updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
                                      metadata JSONB,
                                      error_code VARCHAR(64),
                                      error_message TEXT,
                                      FOREIGN KEY (payment_order_id) REFERENCES payment_orders(id)
);

CREATE INDEX idx_payment_transactions_payment_order_id ON payment_transactions(payment_order_id);
CREATE INDEX idx_payment_transactions_transaction_id ON payment_transactions(transaction_id);
CREATE INDEX idx_payment_transactions_channel_transaction_id ON payment_transactions(channel_transaction_id);

-- 退款订单表
CREATE TABLE refund_orders (
                               id VARCHAR(36) PRIMARY KEY,
                               payment_order_id VARCHAR(36) NOT NULL,
                               transaction_id VARCHAR(36) NOT NULL,
                               amount DECIMAL(20, 6) NOT NULL,
                               reason TEXT NOT NULL,
                               status VARCHAR(20) NOT NULL,
                               refund_id VARCHAR(64),
                               channel_refund_id VARCHAR(64),
                               created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                               updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
                               metadata JSONB,
                               FOREIGN KEY (payment_order_id) REFERENCES payment_orders(id),
                               FOREIGN KEY (transaction_id) REFERENCES payment_transactions(id)
);

CREATE INDEX idx_refund_orders_payment_order_id ON refund_orders(payment_order_id);
CREATE INDEX idx_refund_orders_transaction_id ON refund_orders(transaction_id);
CREATE INDEX idx_refund_orders_refund_id ON refund_orders(refund_id);