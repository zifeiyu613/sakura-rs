use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::product::Product;
use crate::error::AppError;
use crate::server::AppState;
use crate::utils::pagination::{Paginated, PaginationParams};

pub struct ProductService {
    state: Arc<AppState>,
}

impl ProductService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn list_products(&self, pagination: PaginationParams) -> Result<Paginated<Product>, AppError> {
        let page = pagination.page.unwrap_or(1);
        let page_size = pagination.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 获取总记录数
        let total = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM products"#
        )
            .fetch_one(&self.state.db)
            .await?
            .count as u64;

        // 获取分页数据
        let products = sqlx::query_as!(
            Product,
            r#"
            SELECT * FROM products
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            page_size,
            offset
        )
            .fetch_all(&self.state.db)
            .await?;

        Ok(Paginated {
            items: products,
            total,
            page,
            page_size,
        })
    }

    pub async fn get_product(&self, id: Uuid) -> Result<Product, AppError> {
        let product = sqlx::query_as!(
            Product,
            r#"SELECT * FROM products WHERE id = ?"#,
            id
        )
            .fetch_optional(&self.state.db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product with ID {} not found", id)))?;

        Ok(product)
    }

    pub async fn create_product(
        &self,
        name: &str,
        description: &str,
        price: f64,
        stock: i32
    ) -> Result<Product, AppError> {
        // 创建产品
        let product = Product::new(name, description, price, stock);

        // 存储到数据库
        sqlx::query!(
            r#"
            INSERT INTO products (id, name, description, price, stock, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            product.id,
            product.name,
            product.description,
            product.price,
            product.stock,
            product.created_at,
            product.updated_at
        )
            .execute(&self.state.db)
            .await?;

        // 发送产品创建事件
        self.publish_product_event("product.created", &product).await?;

        Ok(product)
    }

    pub async fn update_product(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        price: Option<f64>,
        stock: Option<i32>
    ) -> Result<Product, AppError> {
        // 检查产品是否存在
        let mut product = self.get_product(id).await?;

        // 更新字段
        if let Some(new_name) = &name {
            product.name = new_name.clone();
        }

        if let Some(new_description) = &description {
            product.description = new_description.clone();
        }

        if let Some(new_price) = price {
            product.price = new_price;
        }

        if let Some(new_stock) = stock {
            product.stock = new_stock;
        }

        product.updated_at = chrono::Utc::now();

        // 更新数据库
        sqlx::query!(
            r#"
            UPDATE products
            SET name = ?, description = ?, price = ?, stock = ?, updated_at = ?
            WHERE id = ?
            "#,
            product.name,
            product.description,
            product.price,
            product.stock,
            product.updated_at,
            product.id
        )
            .execute(&self.state.db)
            .await?;

        // 发送产品更新事件
        self.publish_product_event("product.updated", &product).await?;

        Ok(product)
    }

    pub async fn delete_product(&self, id: Uuid) -> Result<(), AppError> {
        // 检查产品是否存在
        let product = self.get_product(id).await?;

        // 删除产品
        sqlx::query!(
            r#"DELETE FROM products WHERE id = ?"#,
            id
        )
            .execute(&self.state.db)
            .await?;

        // 发送产品删除事件
        self.publish_product_event("product.deleted", &product).await?;

        Ok(())
    }

    async fn publish_product_event(&self, event_type: &str, product: &Product) -> Result<(), AppError> {
        use lapin::{
            options::BasicPublishOptions,
            BasicProperties,
        };
        use serde_json::json;

        let channel = self.state.amqp.create_channel().await?;

        let payload = json!({
            "event_type": event_type,
            "product": product,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string();

        channel.basic_publish(
            "products",  // exchange
            event_type,  // routing key
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default(),
        ).await?;

        Ok(())
    }
}
