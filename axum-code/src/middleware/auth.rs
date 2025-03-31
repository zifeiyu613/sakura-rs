use axum::{extract::{Request, State}, http::{header, StatusCode}, middleware, middleware::Next, response::Response};
use std::sync::Arc;
use axum::middleware::FromFnLayer;
use crate::domain::services::auth_service::AuthService;
use crate::error::AppError;
use crate::server::AppState;

// 导出 layer 函数
// pub fn layer(state: Arc<AppState>) -> FromFnLayer<
//     impl Fn(State<Arc<AppState>>, Request<Body>, Next<Body>) -> _,
//     Arc<AppState>,
//     PhantomData<(State<Arc<AppState>>, Request<Body>)>
// > {
//     middleware::from_fn_with_state(state, auth_middleware)
// }

pub fn layer(state: Arc<AppState>) -> impl tower::Layer<axum::routing::MethodRouter> + Clone {
    middleware::from_fn_with_state(state, auth_middleware)
}

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 检查路径，跳过不需要认证的路由
    let path = request.uri().path();
    if path.starts_with("/api/v1/auth") || path.starts_with("/health") || path.starts_with("/metrics") {
        return Ok(next.run(request).await);
    }

    // 从请求头获取令牌
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Auth("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Auth("Invalid authorization header format".to_string()))?;

    // 验证令牌
    let auth_service = AuthService::new(state.clone());
    let user_id = auth_service.validate_token(token).await?;

    // 将用户 ID 添加到请求扩展中
    request.extensions_mut().insert(user_id);

    // 继续请求流程
    Ok(next.run(request).await)
}
