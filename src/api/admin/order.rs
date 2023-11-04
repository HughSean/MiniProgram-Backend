use crate::{
    appstate::AppState,
    utils::{auth::JWTAuthMiddleware, error::BaseError},
};
use axum::{extract::State, response::IntoResponse, Extension};

async fn get_many(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, BaseError<&'static str>> {
    Ok("()")
}
