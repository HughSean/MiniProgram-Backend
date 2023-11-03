use crate::{
    appstate::AppState,
    module::court::{CourtAddSchema, CourtSchema},
    utils::auth::JWTAuthMiddleware,
};
use axum::{extract::State, response::IntoResponse, Extension, Json};
use serde_json::json;
use std::sync::Arc;
use tracing::warn;

pub async fn addcourt(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtAddSchema>,
) -> Result<impl IntoResponse, Json<serde_json::Value>> {
    CourtSchema::add(schema, jwt_guard.user.id, &state)
        .await
        .map_err(|err| {
            warn!("球场信息添加失败: {}", err.to_string());
            Json(json!({"code":-1,"msg":&err}))
        })?;
    Ok(Json(json!({"code":0,"msg":"球场添加成功"})))
}
