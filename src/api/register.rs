use crate::appstate::AppState;
use crate::module::user::{UserRegisterSchema, UserSchema};
use axum::http::StatusCode;
use axum::response::Response;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info};

pub fn router() -> Router<Arc<AppState>> {
    info!("/register 挂载");
    Router::new().route("/register", post(register))
}

#[derive(Debug, Deserialize)]
pub enum RegisterError {
    RequestErr(String),
    ServerErr,
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        match self {
            RegisterError::RequestErr(err) => {
                (StatusCode::BAD_REQUEST, Json(json!({"code":-1,"msg":&err}))).into_response()
            }
            RegisterError::ServerErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":-1,"msg":"服务器端错误，注册失败"})),
            )
                .into_response(),
        }
    }
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(schema): Json<UserRegisterSchema>,
) -> Result<impl IntoResponse, RegisterError> {
    debug!("查询用户数据信息");
    if (schema.role != "admin") && (schema.role != "user") {
        return Err(RegisterError::RequestErr(
            "role必须为'admin'或者'user'".to_string(),
        ));
    };
    let query = UserSchema::fetch_optional_by_name(&schema.name, &state).await;
    if query.is_none() {
        debug!("注册中");
        UserSchema::register_new_user(&state, &schema)
            .await
            .or(Err(RegisterError::ServerErr))?;
    } else {
        debug!("用户({})已存在", schema.name);
        return Err(RegisterError::RequestErr("用户已存在".to_string()));
    }
    info!("注册成功");
    Ok(Json(json!({"code":0,"msg":"注册成功"})))
}
