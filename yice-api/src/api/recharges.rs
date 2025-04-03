use crate::app::AppState;
use crate::error::YiceError;
use axum::routing::post;
use axum::{Extension, Json, Router};
use std::sync::Arc;
use tracing::log::info;

pub(crate) fn routes() -> Router {
    let recharge_routes = Router::new().route("/getPayManageList", post(get_pay_manage_list));

    recharge_routes
}

async fn get_pay_manage_list(Extension(state): Extension<Arc<AppState>>) -> Result<Json<String>, YiceError> {
    info!(
        "Got a request to get pay manage list, with state: {:?}",
        state
    );

    Ok(Json("get_pay_manage_list".to_string()))
}

pub struct Recharges {
    pub path: String,
}

impl Recharges {
    pub fn new<P: Into<String>>(path: P) -> Self {
        Self { path: path.into() }
    }
}
