use crate::appstate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/login", post(login))
}

#[derive(Debug, Deserialize)]
struct LoginSchema {
    name: String,
    pwd: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(info): Json<LoginSchema>,
) -> Result<impl IntoResponse, Json<serde_json::Value>> {
    let user_schema = sqlx::query_as!(
        crate::module::user::UserSchema,
        "select * from users where name=$1",
        info.name
    )
    .fetch_optional(&state.pgpool)
    .await
    .map_err(|_| {
        Json(json!(
            {
                "code":-1,
                "msg":format!{"未查出用户：{}",info.name}
            }
        ))
    })?;

    

    Ok("".into_response())
}
