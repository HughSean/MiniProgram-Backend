use std::sync::Arc;

use crate::{appstate::AppState, utils::token};
use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use redis::AsyncCommands;
use serde_json::json;
use tracing::debug;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/logout", get(logout))
}

async fn logout(
    cookie_jar: CookieJar,
    Extension(auth_guard): Extension<crate::utils::auth::JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Json<serde_json::Value>> {
    debug!("get refresh token");
    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or(Json(json!({
            "code":-1,
            "msg":"没有refresh token"
        })))?;

        debug!("verify refresh token");
    let refresh_token_details =
        token::jwt_token_verify(&refresh_token, &state.cfg.tokencfg.refresh_pubkey).or(Err(
            Json(json!({
                "code":-1,
                "msg":"token非法"
            })),
        ))?;

    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .or(Err(Json(json!({"code":-1,"msg":"服务器内部错误"}))))?;

    debug!("remove refresh/access token from redis");
    redis_client
        .del(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.access_token_uuid.to_string(),
        ])
        .await
        .or(Err(Json(json!({
            "code":-1,
            "msg":"服务器内部错误"
        }))))?;

    let access_cookie = Cookie::build("access_token", "")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    // let logged_in_cookie = Cookie::build("logged_in", "true")
    //     .path("/")
    //     .max_age(time::Duration::minutes(-1))
    //     .same_site(SameSite::Lax)
    //     .http_only(false)
    //     .finish();

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

    let mut response = Response::new(json!({"code":0,"msg":"注销成功"}).to_string());
    response.headers_mut().extend(headers);
    Ok(response)
}
