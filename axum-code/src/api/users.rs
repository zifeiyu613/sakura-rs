use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
    extract::{State, Path, Extension, Query},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use std::sync::Arc;

use crate::domain::models::user::User;
use crate::domain::services::user_service::UserService;
use crate::error::AppError;
use crate::server::AppState;
use crate::utils::pagination::{Paginated, PaginationParams};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
        .route("/me", get(get_current_user))
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub name: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Paginated<UserResponse>>, AppError> {
    let user_service = UserService::new(state.clone());

    let paginated_users = user_service.list_users(pagination).await?;

    let users = paginated_users.items
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(Paginated {
        items: users,
        total: paginated_users.total,
        page: paginated_users.page,
        page_size: paginated_users.page_size,
    }))
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user_service = UserService::new(state.clone());

    let user = user_service.get_user(id).await?;

    Ok(Json(UserResponse::from(user)))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // 验证请求
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user_service = UserService::new(state.clone());

    let user = user_service.create_user(&payload.email, &payload.name, &payload.password).await?;

    Ok(Json(UserResponse::from(user)))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
    Extension(current_user_id): Extension<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    // 验证请求
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 检查权限
    if id != current_user_id {
        return Err(AppError::Forbidden("You can only update your own profile".to_string()));
    }

    let user_service = UserService::new(state.clone());

    let user = user_service.update_user(id, payload.name, payload.email).await?;

    Ok(Json(UserResponse::from(user)))
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(current_user_id): Extension<Uuid>,
) -> Result<(), AppError> {
    // 检查权限
    if id != current_user_id {
        return Err(AppError::Forbidden("You can only delete your own account".to_string()));
    }

    let user_service = UserService::new(state.clone());

    user_service.delete_user(id).await?;

    Ok(())
}

async fn get_current_user(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user_service = UserService::new(state.clone());

    let user = user_service.get_user(user_id).await?;

    Ok(Json(UserResponse::from(user)))
}
