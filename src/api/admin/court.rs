use crate::{
    appstate::AppState,
    module::court::{CourtAddSchema, CourtAllSchema, CourtDelSchema, CourtSchema, CourtUpSchema},
    utils::{auth::JWTAuthMiddleware, error::BaseError},
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn};

pub fn router() -> Router<Arc<AppState>> {
    info!("/court/* 挂载中");
    Router::new()
        .route("/add", post(add))
        .route("/del", delete(del))
        .route("/all", get(all))
        .route("/update", post(update))
}

async fn add(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtAddSchema>,
) -> Result<impl IntoResponse, BaseError<String>> {
    CourtSchema::add(&schema, &jwt_guard.user.id, &state)
        .await
        .map_err(|err| {
            warn!("球场添加失败: {}", err.to_string());
            BaseError::BadRequest(-1, err.to_string())
        })?;
    info!("admin({})添加球场({})", jwt_guard.user.name, schema.name);
    Ok(Json(json!({"code":0,"msg":"球场添加成功"})))
}

async fn del(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtDelSchema>,
) -> Result<impl IntoResponse, BaseError<String>> {
    CourtSchema::del(&schema, &jwt_guard.user.id, &state)
        .await
        .map_err(|err| {
            warn!("球场删除失败: {}", err);
            BaseError::BadRequest(-1, err)
        })?;
    info!("admin({})删除球场({})", jwt_guard.user.name, schema.id);
    Ok(Json(json!({"code":0,"msg":"球场删除成功"})))
}

async fn all(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, BaseError<String>> {
    let courts = CourtSchema::all(&jwt_guard.user.id, &state)
        .await
        .or(Err(BaseError::ServerInnerErr))?;
    let courts: Vec<_> = courts
        .into_iter()
        .map(|c| CourtAllSchema {
            id: c.id,
            name: c.name,
            location: c.location,
            class: c.class,
        })
        .collect();
    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":courts
    })))
}

async fn update(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtUpSchema>,
) -> Result<impl IntoResponse, BaseError<String>> {
    CourtSchema::update(&schema, &jwt_guard.user.id, &state)
        .await
        .map_err(|err| BaseError::BadRequest(-1, err))?;
    Ok(Json(json!({
        "code":0,
        "msg":"操作成功",
    })))
}
