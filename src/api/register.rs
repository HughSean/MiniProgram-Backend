use crate::appstate::AppState;
use crate::module::user::UserSchema;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/register", post(register))
}

#[derive(Debug, Deserialize)]
pub enum RegisterError {
    UserNameClashErr,
    InnerErr,
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        match self {
            RegisterError::UserNameClashErr => (
                StatusCode::BAD_REQUEST,
                Json(json!({"code":-1,"msg":"用户名已存在"})),
            )
                .into_response(),
            RegisterError::InnerErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":-1,"msg":"服务器端错误，注册失败"})),
            )
                .into_response(),
        }
    }
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
) -> Result<impl IntoResponse, RegisterError> {
    debug!("query user from postgreSQL");
    let query = sqlx::query_as!(UserSchema, "select * from users where name = $1", info.name)
        .fetch_optional(&state.pgpool)
        .await
        .unwrap_or(None);

    if query.is_none() {
        debug!("registering");
        UserSchema::register_new_user(&state, info)
            .await
            .or(Err(RegisterError::InnerErr))?;
    } else {
        debug!("user name({}) exits", info.name);
        return Err(RegisterError::UserNameClashErr);
    }
    debug!("register success");
    Ok(Json(json!({"code":0,"msg":"注册成功"})))
}
