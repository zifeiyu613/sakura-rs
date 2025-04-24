pub mod models;
pub mod repositories;
pub mod services;
pub mod handlers;

use axum::{handler, Router};
use axum::routing::{get, Route};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/activity", get(|| async {"activity is OK "}))

}