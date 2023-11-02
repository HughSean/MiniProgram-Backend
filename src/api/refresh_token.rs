use crate::{appstate::AppState, module::user::UserSchema, utils::token};
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/refresh_token", get(refresh_access_token))
}

async fn refresh_access_token(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    debug!("get refresh token");
    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or((
            StatusCode::FORBIDDEN,
            Json(json!({
                "code": -1,
                "msg": "请在cookie中提供refresh token"
            })),
        ))?;

    debug!("verify refresh token");
    let refresh_token_details =
        match token::jwt_token_verify(&refresh_token, &state.cfg.tokencfg.refresh_pubkey) {
            Ok(token_details) => token_details,
            Err(err) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "code": -1,
                        "msg": format!("验证错误：{}",err.to_string())
                    })),
                ));
            }
        };

    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Redis error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    debug!("query redis");
    let redis_token_user_id = redis::AsyncCommands::get::<_, String>(
        &mut redis_client,
        refresh_token_details.token_uuid.to_string(),
    )
    .await
    .or(Err((
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "code": -1,
            "msg": "token非法或已过期"
        })),
    )))?;

    let user_id_uuid = uuid::Uuid::parse_str(&redis_token_user_id).or(Err((
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "code": -1,
            "msg": "token非法或已过期"
        })),
    )))?;

    debug!("query postgreSQL");
    let user = sqlx::query_as!(
        UserSchema,
        "SELECT * FROM users WHERE id = $1",
        user_id_uuid
    )
    .fetch_optional(&state.pgpool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": -1,
                "msg": format!("数据库查询错误: {}", e)
            })),
        )
    })?;

    let user = user.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "code": -1,
            "msg": "用户不存在"
        })),
    ))?;

    debug!("token gen");
    let access_token_details = token::jwt_token_gen(
        user.id,
        state.cfg.tokencfg.access_token_ttl,
        &state.cfg.tokencfg.access_prikey,
    )
    .or(Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"code":-1,"msg":"token 生成失败"})), //todo
    )))?;

    debug!("save new token to redis");
    token::save_token_data_to_redis(
        &state,
        &access_token_details,
        state.cfg.tokencfg.access_token_ttl,
    )
    .await
    .or(Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"code":-1,"msg":"token 生成失败"})),
    )))?;

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

    // let logged_in_cookie = Cookie::build("logged_in", "true")
    //     .path("/")
    //     .max_age(time::Duration::minutes(data.cfg.tokencfg.access_token_ttl * 60))
    //     .same_site(SameSite::Lax)
    //     .http_only(false)
    //     .finish();
    let mut response = Response::new(
        json!({"code": 0,"msg":"刷新成功", "access_token": access_token_details.token.unwrap()})
            .to_string(),
    );
    let mut headers = HeaderMap::new();

    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    // headers.append(
    //     header::SET_COOKIE,
    //     logged_in_cookie.to_string().parse().unwrap(),
    // );
    debug!("give response");
    response.headers_mut().extend(headers);
    Ok(response)
}
