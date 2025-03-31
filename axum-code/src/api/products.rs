use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
    extract::{State, Path, Query},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use std::sync::Arc;

use crate::domain::models::product::Product;
use crate::domain::services::product_service::ProductService;
use crate::error::AppError;
use crate::server::AppState;
use crate::utils::pagination::{Paginated, PaginationParams};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_products))
        .route("/", post(create_product))
        .route("/:id", get(get_product))
        .route("/:id", put(update_product))
        .route("/:id", delete(delete_product))
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            name: product.name,
            description: product.description,
            price: product.price,
            stock: product.stock,
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    pub description: String,
    #[validate(range(min = 0.01, message = "Price must be positive"))]
    pub price: f64,
    pub stock: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 0.01, message = "Price must be positive"))]
    pub price: Option<f64>,
    pub stock: Option<i32>,
}

async fn list_products(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Paginated<ProductResponse>>, AppError> {
    let product_service = ProductService::new(state.clone());

    let paginated_products = product_service.list_products(pagination).await?;

    let products = paginated_products.items
        .into_iter()
        .map(ProductResponse::from)
        .collect();

    Ok(Json(Paginated {
        items: products,
        total: paginated_products.total,
        page: paginated_products.page,
        page_size: paginated_products.page_size,
    }))
}

async fn get_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, AppError> {
    let product_service = ProductService::new(state.clone());

    let product = product_service.get_product(id).await?;

    Ok(Json(ProductResponse::from(product)))
}

async fn create_product(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    // 验证请求
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let product_service = ProductService::new(state.clone());

    let product = product_service.create_product(
        &payload.name,
        &payload.description,
        payload.price,
        payload.stock
    ).await?;

    Ok(Json(ProductResponse::from(product)))
}

async fn update_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    // 验证请求
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let product_service = ProductService::new(state.clone());

    let product = product_service.update_product(
        id,
        payload.name,
        payload.description,
        payload.price,
        payload.stock
    ).await?;

    Ok(Json(ProductResponse::from(product)))
}

async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    let product_service = ProductService::new(state.clone());

    product_service.delete_product(id).await?;

    Ok(())
}
