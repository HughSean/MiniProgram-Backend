use crate::appstate::AppState;
use axum::Router;
use std::sync::Arc;
use tracing::info;
pub mod admin;
pub mod open;
pub mod order;
pub mod test;

pub fn router(state: Arc<AppState>) -> Router<std::sync::Arc<crate::appstate::AppState>> {
    info!("/api/* 挂载中");
    Router::new()
        .nest("/order", order::router())
        .nest("/admin", admin::router())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::utils::auth::auth,
        ))
        .nest("/open", open::user::router())
}
