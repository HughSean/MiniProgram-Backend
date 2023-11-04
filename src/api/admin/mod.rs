mod court;
mod order;
use crate::appstate::AppState;
use axum::{middleware, Router};
use std::sync::Arc;
use tracing::info;

pub fn router() -> Router<Arc<AppState>> {
    info!("/admin/* 挂载中");
    let court = Router::new().nest("/court", court::router());

    let router = Router::new().merge(court);
    Router::new()
        .nest("/admin", router)
        .layer(middleware::from_fn(crate::utils::auth::admin_auth))
}
