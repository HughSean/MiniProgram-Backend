use crate::appstate::AppState;
use crate::module::user::{UserLoginSchema, UserRegisterSchema, UserSchema};
use crate::utils::error::BaseError;
use crate::utils::{passwd, token};
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};
use axum::{routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde_json::json;
use std::sync::Arc;
use tracing::info;
use tracing::{debug, warn};

pub fn router() -> Router<Arc<AppState>> {
    info!("/login 挂载中");
    info!("/register 挂载中");
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(schema): Json<UserLoginSchema>,
) -> Result<impl IntoResponse, BaseError<&'static str>> {
    //先根据用户名查询用户信息
    debug!("查询用户信息");
    let user_schema = UserSchema::fetch_optional_by_name(&schema.name, &state)
        .await
        .ok_or_else(|| {
            warn!("用户不存在");
            BaseError::BadRequest(-1, "用户不存在")
        })?;

    debug!("校验密码");
    passwd::passwd_verify(&schema.pwd, &user_schema.pwd)
        .or(Err(BaseError::BadRequest(-1, "密码错误")))?;
    //生成access_token
    debug!("生成token");
    let access_token_details = token::jwt_token_gen(
        user_schema.id,
        state.cfg.tokencfg.access_token_ttl,
        &state.cfg.tokencfg.access_prikey,
    )
    .or(Err(BaseError::ServerInnerErr))?;

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

async fn register(
    State(state): State<Arc<AppState>>,
    Json(schema): Json<UserRegisterSchema>,
) -> Result<impl IntoResponse, BaseError<&'static str>> {
    debug!("查询用户数据信息");
    if (schema.role != "admin") && (schema.role != "user") {
        return Err(BaseError::BadRequest(-1, "role必须为'admin'或者'user'"));
    };

    let query = UserSchema::fetch_optional_by_name(&schema.name, &state).await;
    if query.is_none() {
        debug!("注册中");
        UserSchema::register_new_user(&state, &schema)
            .await
            .or(Err(BaseError::ServerInnerErr))?;
    } else {
        debug!("用户({})已存在", schema.name);
        return Err(BaseError::BadRequest(-1, "用户已存在"));
    }
    info!("用户({})注册成功", schema.name);
    Ok(Json(json!({"code":0,"msg":"注册成功"})))
}
