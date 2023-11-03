use std::sync::Arc;

use axum::{
    http::StatusCode, middleware, response::IntoResponse, routing::get, Extension, Json, Router,
};
use serde_json::json;

pub fn router(state: Arc<crate::appstate::AppState>) -> Router {
    Router::new()
        .nest("/test", Router::new().route("/auth_test", get(auth_test)))
        .layer(middleware::from_fn_with_state(
            state,
            crate::utils::auth::auth,
        ))
}

async fn auth_test(
    Extension(_auth_guard): Extension<crate::utils::auth::JWTAuthMiddleware>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"code":0,"msg":"pass"})))
}
