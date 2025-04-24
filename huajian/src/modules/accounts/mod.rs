use axum::Router;
use axum::routing::get;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/config", get(|| async {"Accounts is OK "}))

}