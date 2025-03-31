use axum::{
    routing::{post, get},
    Router,
    Json,
    extract::{State, Extension},
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::services::auth_service::AuthService;
use crate::error::AppError;
use crate::server::AppState;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    pub name: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 验证请求
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 创建认证服务
    let auth_service = AuthService::new(state.clone());

    // 处理登录
    let auth_result = auth_service.login(&payload.email, &payload.password).await?;

    Ok(Json(AuthResponse {
        access_token: auth_result.access_token,
        refresh_token: auth_result.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: auth_result.expires_in,
    }))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 验证请求
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 创建认证服务
    let auth_service = AuthService::new(state.clone());

    // 处理注册
    let auth_result = auth_service.register(&payload.email, &payload.password, &payload.name).await?;

    Ok(Json(AuthResponse {
        access_token: auth_result.access_token,
        refresh_token: auth_result.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: auth_result.expires_in,
    }))
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 创建认证服务
    let auth_service = AuthService::new(state.clone());

    // 处理令牌刷新
    let auth_result = auth_service.refresh_token(&payload.refresh_token).await?;

    Ok(Json(AuthResponse {
        access_token: auth_result.access_token,
        refresh_token: auth_result.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: auth_result.expires_in,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<uuid::Uuid>,
) -> Result<(), AppError> {
    // 创建认证服务
    let auth_service = AuthService::new(state.clone());

    // 处理登出
    auth_service.logout(user_id).await?;

    Ok(())
}
