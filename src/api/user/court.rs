use crate::{
    appstate::AppState,
    module::{court::CourtUserSchema, db::prelude::Courts},
    utils::error::HandleErr,
};
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use sea_orm::EntityTrait;
use serde_json::json;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/all", get(all))
}

async fn all(
    // Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    let courts: Vec<_> = Courts::find()
        .all(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?
        .into_iter()
        .map(|e| CourtUserSchema {
            court_id: Some(e.court_id),
            admin_id: None,
            court_name: e.court_name,
            location: e.location,
            label: e.label,
            price_per_hour: e.price_per_hour,
            open_time: e.open_time,
            close_time: e.close_time,
        })
        .collect();

    Ok(Json(json!({
        "code":0,
        "msg":"OK",
        "data":courts
    })))
}
