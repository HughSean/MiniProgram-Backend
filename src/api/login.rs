use crate::appstate::AppState;
use crate::module::user::{UserLoginSchema, UserSchema};
use crate::utils::{passwd, token};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde_json::json;
use std::sync::Arc;
use tracing::info;
use tracing::{debug, warn};

pub fn router() -> Router<Arc<AppState>> {
    info!("/login 挂载");
    Router::new().route("/login", post(login))
}

#[derive(Debug, serde::Deserialize)]
pub enum LoginError {
    RequestErr(String),
    ServerErr,
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        match self {
            LoginError::RequestErr(err) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "code":-1,
                    "msg":&err
                })),
            )
                .into_response(),
            LoginError::ServerErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":-2,"msg":"服务器内部错误"})),
            )
                .into_response(),
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(schema): Json<UserLoginSchema>,
) -> Result<impl IntoResponse, LoginError> {
    //先根据用户名查询用户信息
    debug!("查询用户信息");
    let user_schema = UserSchema::fetch_optional_by_name(&schema.name, &state)
        .await
        .ok_or_else(|| {
            warn!("用户不存在");
            LoginError::RequestErr("用户不存在".to_string())
        })?;

    debug!("校验密码");
    passwd::passwd_verify(&schema.pwd, &user_schema.pwd)
        .or(Err(LoginError::RequestErr("密码错误".to_string())))?;
    //生成access_token
    debug!("生成token");
    let access_token_details = token::jwt_token_gen(
        user_schema.id,
        state.cfg.tokencfg.access_token_ttl,
        &state.cfg.tokencfg.access_prikey,
    )
    .or(Err(LoginError::ServerErr))?;

    //设置cookie
    let access_cookie = Cookie::build(
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    )
    .path("/")
    .max_age(time::Duration::minutes(
        state.cfg.tokencfg.access_token_ttl * 60,
    ))
    .same_site(SameSite::Lax)
    .http_only(true)
    .finish();

    let mut response = Response::new(
        json!({
            "code": 0,
            "msg":"登录成功",
            "access_token":  access_token_details.token.unwrap(),
        })
        .to_string(),
    );

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );

    info!("用户({})登录成功", schema.name);
    response.headers_mut().extend(headers);
    Ok(response)
}
