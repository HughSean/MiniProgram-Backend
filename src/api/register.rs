use crate::appstate::AppState;
use crate::module::user::UserSchema;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/register", post(register))
}

#[derive(Debug, Deserialize)]
pub struct RegisterSchema {
    pub name: String,
    pub pwd: String,
    pub phone: String,
    pub role: String,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(info): Json<RegisterSchema>,
) -> impl IntoResponse {
    let query: Option<UserSchema> = sqlx::query_as("select * from users where name = $1")
        .bind(&info.name)
        .fetch_optional(&state.pgpool)
        .await
        .unwrap_or(None);

    let code;
    let msg: String;

    if query.is_none() {
        match UserSchema::register_new_user(&state, info).await {
            Ok(_) => {
                code = 0;
                msg = "OK".to_string();
            }
            Err(err) => {
                code = -1;
                msg = err;
            }
        }
    } else {
        code = -1;
        msg = "用户名已存在".to_string();
    }
    return Json(json!({
        "code":code,
        "msg":msg
    }));
}
