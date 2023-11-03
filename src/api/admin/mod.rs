pub mod addcourt;
use crate::appstate::AppState;
use axum::{routing::post, Router};
use std::sync::Arc;
use tracing::info;

pub fn router() -> Router<Arc<AppState>> {
    info!("/admin/* 挂载");
    Router::new().nest(
        "/admin",
        Router::new().route("/addcourt", post(addcourt::addcourt)),
    )
}
