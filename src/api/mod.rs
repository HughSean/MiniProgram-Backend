use crate::appstate::AppState;
use axum::Router;
use std::sync::Arc;
use tracing::info;
pub mod admin;
pub mod open;
pub mod test;
pub mod user;

pub fn router(state: Arc<AppState>) -> Router<std::sync::Arc<crate::appstate::AppState>> {
    info!("/api/* 挂载中");
    Router::new()
        .nest("/user", user::router())
        .nest("/admin", admin::router())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::utils::auth::auth,
        ))
        .merge(open::router())
}
