use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};

use crate::api::{auth, users, products};
use crate::config::Config;
use crate::error::AppError;
use crate::middleware::{auth as auth_middleware, logging as logging_middleware};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: sqlx::MySqlPool,
    pub redis: redis::Client,
    pub amqp: Arc<lapin::Connection>,
}

pub async fn create_app(state: AppState) -> Result<Router, AppError> {
    let app_state = Arc::new(state);

    // 健康检查路由
    let health_route = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/metrics", get(|| async { "Metrics endpoint" }));

    // API 路由
    let api_routes = Router::new()
        .nest("/auth", auth::routes())
        .nest("/users", users::routes())
        .nest("/products", products::routes())
        .layer(auth_middleware::layer(app_state.clone()));

    // 组合所有路由
    let app = Router::new()
        .nest("/api/v1", api_routes)
        .merge(health_route)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any))
        // .layer(logging_middleware::layer())
        .with_state(app_state);

    Ok(app)
}
