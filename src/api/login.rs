use crate::appstate::AppState;
use crate::utils::{passwd, token};
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};
use axum::{routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

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
    //先根据用户名查询用户信息
    debug!("query user from postgreSQL");
    let user_schema = sqlx::query_as!(
        crate::module::user::UserSchema,
        "select * from users where name=$1",
        info.name
    )
    .fetch_optional(&state.pgpool)
    .await
    .unwrap_or(None)
    .ok_or(Json(json!({
        "code":-1,
        "msg":"用户不存在"
    })))?;

    debug!("verify passwd");
    passwd::passwd_verify(&info.pwd, &user_schema.pwd).or(Err(Json(json!({
        "code":-2,
        "msg":"密码错误"
    }))))?;
    //生成access_token
    debug!("generate token");
    let access_token_details = token::jwt_token_gen(
        user_schema.id,
        state.cfg.tokencfg.access_token_ttl,
        &state.cfg.tokencfg.access_prikey,
    )
    .or(Err(Json(json!({
        "code":-3,
        "msg":"服务器内部错误"
    }))))?;
    //生成refresh_token
    let refresh_token_details = token::jwt_token_gen(
        user_schema.id,
        state.cfg.tokencfg.refresh_token_ttl,
        &state.cfg.tokencfg.refresh_prikey,
    )
    .or(Err(Json(json!({
        "code":-3,
        "msg": "服务器内部错误"
    }))))?;
    //保存token到redis
    debug!("save token to redis");
    token::save_token_data_to_redis(
        &state,
        &access_token_details,
        state.cfg.tokencfg.access_token_ttl,
    )
    .await
    .or(Err(Json(json!({
    "code":-3,
    "msg":"服务器内部错误"
    }))))?;
    token::save_token_data_to_redis(
        &state,
        &refresh_token_details,
        state.cfg.tokencfg.refresh_token_ttl,
    )
    .await
    .or(Err(Json(json!({
    "code":-3,
    "msg":"服务器内部错误"
    }))))?;
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
    let refresh_cookie = Cookie::build(
        "refresh_token",
        refresh_token_details.token.clone().unwrap_or_default(),
    )
    .path("/")
    .max_age(time::Duration::minutes(
        state.cfg.tokencfg.refresh_token_ttl * 60,
    ))
    .same_site(SameSite::Lax)
    .http_only(true)
    .finish();
    // let logged_in_cookie = Cookie::build("logged_in", "true")
    //     .path("/")
    //     .max_age(time::Duration::minutes(
    //         state.cfg.tokencfg.access_token_ttl * 60,
    //     ))
    //     .same_site(SameSite::Lax)
    //     .http_only(false)
    //     .finish();

    let mut response = Response::new(
        json!({
            "code": 0,
            "msg":"登录成功",
            "access_token":  access_token_details.token.unwrap(),
            "refresh_token": refresh_token_details.token.unwrap(),
        })
        .to_string(),
    );
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    // headers.append(
    //     header::SET_COOKIE,
    //     logged_in_cookie.to_string().parse().unwrap(),
    // );

    debug!("give response");
    response.headers_mut().extend(headers);
    Ok(response)
}
