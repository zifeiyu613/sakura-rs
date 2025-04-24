use crate::server::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Extension, Json, Router};
use std::sync::Arc;
use tracing::log::info;
use crate::errors::ApiError;

pub fn routes() -> Router {
    Router::new().route("/", get(landing_page_info).post(landing_page_info))
}

async fn landing_page_info(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<String>, ApiError> {
    info!("Landing page state: {:?}", state);

    Ok(Json("landing_page_info API Success".to_string()))
}
