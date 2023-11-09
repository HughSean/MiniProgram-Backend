use crate::{
    appstate::AppState,
    module::{db::prelude::Users, user::UserSchema},
    utils::{auth::JWTAuthMiddleware, error::BaseError},
};
use axum::{extract::State, response::IntoResponse, routing::get, Extension, Json, Router};
use sea_orm::EntityTrait;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};
pub mod court;
pub mod order;

pub fn router() -> Router<Arc<AppState>> {
    info!("/user 挂载中");
    Router::new()
        .nest("/order", order::router())
        .route("/info", get(user_info))
}

async fn user_info(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, BaseError<String>> {
    debug!("{} get user info", auth.user.user_id);
    let user = Users::find_by_id(auth.user.user_id)
        .into_model::<UserSchema>()
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = uuid::Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr::<String>(id)
        })?
        .ok_or(BaseError::BadRequest(-1, "未发现用户"))?;
    Ok(Json(json!({
        "code":0,
        "msg":"OK",
        "data":{
            "user":user
        }
    })))
}
