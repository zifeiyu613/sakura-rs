use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::AppState;
use crate::utils::jwt::verify_token;

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // 从请求头中获取 Authorization token
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // 验证 JWT 令牌
    let jwt_secret = state.app_config.get_extension::<String>("jwt_secret").unwrap();
    let claims = match verify_token(&token, jwt_secret.as_bytes()) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // 将用户信息添加到请求扩展中
    let mut request = request;
    request.extensions_mut().insert(claims);

    // 继续处理请求
    Ok(next.run(request).await)
}
