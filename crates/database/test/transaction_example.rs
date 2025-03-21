// src/models/user.rs
pub struct User {
    pub id: i64,
    pub username: String,
    pub balance: f64,
}

// src/models/order.rs
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub total_amount: f64,
    pub status: String,
}

// src/models/order_item.rs
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub price: f64,
}

// src/repositories/order_repository.rs
use crate::database::transaction::DatabaseTransaction;
use crate::models::{User, Order, OrderItem};
use sqlx::{MySql, MySqlPool, Transaction, Error as SqlxError};
use std::pin::Pin;
use std::future::Future;
use anyhow::{Result, Context};

pub struct OrderRepository {
    pool: MySqlPool,
}

impl OrderRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // 创建订单的主方法
    pub async fn create_order(&self, user_id: i64, items: Vec<(i64, i32)>) -> Result<Order> {
        // 使用事务确保订单创建过程的原子性
        self.pool.transaction(|tx| Box::pin(async move {
            // 1. 检查用户是否存在及余额是否充足
            let user = self.get_user_by_id(tx, user_id).await?;

            // 2. 计算订单总金额
            let mut total_amount = 0.0;
            for (product_id, quantity) in &items {
                let product = self.get_product_by_id(tx, *product_id).await?;
                total_amount += product.price * (*quantity as f64);
            }

            // 3. 检查余额是否足够
            if user.balance < total_amount {
                return Err(anyhow::anyhow!("Insufficient balance"));
            }

            // 4. 创建订单记录
            let order = self.insert_order(tx, user_id, total_amount).await?;

            // 5. 使用嵌套事务添加订单项和减少库存
            // 这确保如果任何一个产品出现问题（如库存不足），整个订单都会回滚
            self.add_order_items_and_update_inventory(tx, order.id, items.clone()).await?;

            // 6. 更新用户余额
            self.update_user_balance(tx, user_id, user.balance - total_amount).await?;

            Ok(order)
        })).await.context("Failed to create order")
    }

    // 使用嵌套事务添加订单项和更新库存
    async fn add_order_items_and_update_inventory(
        &self,
        tx: &mut Transaction<'_, MySql>,
        order_id: i64,
        items: Vec<(i64, i32)>
    ) -> Result<()> {
        // 嵌套事务处理
        self.pool.nested_transaction(|nested_tx| Box::pin(async move {
            for (product_id, quantity) in items {
                // 检查库存
                let inventory = self.get_inventory(nested_tx, product_id).await?;
                if inventory < quantity {
                    return Err(anyhow::anyhow!("Insufficient inventory for product {}", product_id));
                }

                // 添加订单项
                let product = self.get_product_by_id(nested_tx, product_id).await?;
                self.insert_order_item(nested_tx, order_id, product_id, quantity, product.price).await?;

                // 更新库存
                self.update_inventory(nested_tx, product_id, inventory - quantity).await?;
            }

            Ok(())
        })).await.context("Failed to process order items")
    }

    // 模拟查询用户信息
    async fn get_user_by_id(&self, tx: &mut Transaction<'_, MySql>, user_id: i64) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, balance FROM users WHERE id = ?",
            user_id
        )
            .fetch_one(&mut *tx)
            .await
            .context("Failed to fetch user")?;

        Ok(user)
    }

    // 模拟查询产品信息
    async fn get_product_by_id(&self, tx: &mut Transaction<'_, MySql>, product_id: i64) -> Result<ProductInfo> {
        let product = sqlx::query_as!(
            ProductInfo,
            "SELECT id, name, price FROM products WHERE id = ?",
            product_id
        )
            .fetch_one(&mut *tx)
            .await
            .context("Failed to fetch product")?;

        Ok(product)
    }

    // 创建订单记录
    async fn insert_order(&self, tx: &mut Transaction<'_, MySql>, user_id: i64, total_amount: f64) -> Result<Order> {
        let order_id = sqlx::query!(
            "INSERT INTO orders (user_id, total_amount, status) VALUES (?, ?, 'pending')",
            user_id,
            total_amount
        )
            .execute(&mut *tx)
            .await
            .context("Failed to insert order")?
            .last_insert_id();

        Ok(Order {
            id: order_id as i64,
            user_id,
            total_amount,
            status: "pending".to_string(),
        })
    }

    // 添加订单项
    async fn insert_order_item(
        &self,
        tx: &mut Transaction<'_, MySql>,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        price: f64
    ) -> Result<i64> {
        let id = sqlx::query!(
            "INSERT INTO order_items (order_id, product_id, quantity, price) VALUES (?, ?, ?, ?)",
            order_id,
            product_id,
            quantity,
            price
        )
            .execute(&mut *tx)
            .await
            .context("Failed to insert order item")?
            .last_insert_id();

        Ok(id as i64)
    }

    // 查询库存
    async fn get_inventory(&self, tx: &mut Transaction<'_, MySql>, product_id: i64) -> Result<i32> {
        let inventory = sqlx::query!(
            "SELECT quantity FROM inventory WHERE product_id = ?",
            product_id
        )
            .fetch_one(&mut *tx)
            .await
            .context("Failed to fetch inventory")?
            .quantity;

        Ok(inventory)
    }

    // 更新库存
    async fn update_inventory(&self, tx: &mut Transaction<'_, MySql>, product_id: i64, new_quantity: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE inventory SET quantity = ? WHERE product_id = ?",
            new_quantity,
            product_id
        )
            .execute(&mut *tx)
            .await
            .context("Failed to update inventory")?;

        Ok(())
    }

    // 更新用户余额
    async fn update_user_balance(&self, tx: &mut Transaction<'_, MySql>, user_id: i64, new_balance: f64) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET balance = ? WHERE id = ?",
            new_balance,
            user_id
        )
            .execute(&mut *tx)
            .await
            .context("Failed to update user balance")?;

        Ok(())
    }
}

// 产品信息结构体
struct ProductInfo {
    id: i64,
    name: String,
    price: f64,
}

// src/services/order_service.rs
use crate::repositories::OrderRepository;
use anyhow::Result;

pub struct OrderService {
    order_repository: OrderRepository,
}

impl OrderService {
    pub fn new(order_repository: OrderRepository) -> Self {
        Self { order_repository }
    }

    // 创建订单的服务方法
    pub async fn create_order(&self, user_id: i64, items: Vec<(i64, i32)>) -> Result<i64> {
        let order = self.order_repository.create_order(user_id, items).await?;
        Ok(order.id)
    }
}

// 使用示例 (在应用程序的入口点或API处理程序中)
// src/main.rs 或 src/handlers/order_handler.rs
use crate::database::get_database_pool;
use crate::repositories::OrderRepository;
use crate::services::OrderService;

async fn handle_create_order(user_id: i64, items: Vec<(i64, i32)>) -> Result<i64, String> {
    // 获取数据库连接池
    let pool = get_database_pool().await.map_err(|e| e.to_string())?;

    // 创建仓库和服务
    let order_repository = OrderRepository::new(pool);
    let order_service = OrderService::new(order_repository);

    // 创建订单
    let order_id = order_service.create_order(user_id, items)
        .await
        .map_err(|e| format!("Order creation failed: {}", e))?;

    Ok(order_id)
}

// 辅助函数示例，用于获取数据库连接池
// src/database/mod.rs
use sqlx::MySqlPool;
use std::env;

pub async fn get_database_pool() -> Result<MySqlPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    MySqlPool::connect(&database_url).await
}
