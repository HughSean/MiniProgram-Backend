mod court;
mod order;
use crate::appstate::AppState;
use axum::{middleware, Router};
use std::sync::Arc;
use tracing::info;

pub fn router() -> Router<Arc<AppState>> {
    info!("/admin/* 挂载中");

    Router::new()
        .nest("/court", court::router())
        .layer(middleware::from_fn(crate::utils::auth::admin_auth))
}
