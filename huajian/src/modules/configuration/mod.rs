mod handler;
mod service;
mod repository;
mod model;

use axum::Router;
use axum::routing::{get, post};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/config", get(|| async {"Configuration is OK "}))

}