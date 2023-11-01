use super::token;
use crate::{appstate::AppState, module::user::UserSchema};
use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: UserSchema,
    pub access_token_uuid: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub enum AuthError {
    NoToken,
    InvalidToken,
    DBErr,
    RedisErr(String),
    UuidParseErr,
    DeadToken,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::NoToken => (StatusCode::UNAUTHORIZED, "请提供token").into_response(),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "无效token").into_response(),
            AuthError::DBErr => {
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库未查到此用户").into_response()
            }
            AuthError::RedisErr(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
            AuthError::UuidParseErr => {
                (StatusCode::INTERNAL_SERVER_ERROR, "uuid parse error").into_response()
            }
            AuthError::DeadToken => (StatusCode::UNAUTHORIZED, "此token无效").into_response(),
        }
    }
}

pub async fn auth<B>(
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, AuthError> {
    //从cookie或者头部取得token
    let access_token = cookie_jar
        .get("access_token")
        .and_then(|e| Some(e.value().to_string()))
        .or(req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|e| e.to_str().ok())
            .and_then(|auth_value| {
                if auth_value.starts_with("Bearer ") {
                    Some(auth_value[7..].to_owned())
                } else {
                    None
                }
            }))
        .ok_or(AuthError::NoToken)?;

    //校验token
    let tokendetils = token::jwt_token_verify(&access_token, &state.cfg.security.access_pubkey)
        .map_err(|_| AuthError::InvalidToken)?;

    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|_| AuthError::RedisErr("redis 连接错误".to_string()))?;

    //查询当前token的用户是否还在redis数据库中, 已验证会话是否过期
    let redis_token_user_id = redis::AsyncCommands::get::<String, String>(
        &mut redis_client,
        tokendetils.token_uuid.clone().to_string(),
    )
    .await
    .map_err(|_| AuthError::RedisErr("redis 查询错误".to_string()))?;

    let user_uuid =
        uuid::Uuid::parse_str(&redis_token_user_id).map_err(|_| AuthError::UuidParseErr)?;
    //查询用户数据库确定该用户是否存在
    let user = sqlx::query_as!(UserSchema, "select * from users where id =$1", user_uuid)
        .fetch_optional(&state.pgpool)
        .await
        .map_err(|_| AuthError::DBErr)?
        .ok_or_else(|| AuthError::DeadToken)?;
    //token签名校验\redis会话校验\数据库用户校验, 全部完成用过认证, 为合法用户
    req.extensions_mut().insert(JWTAuthMiddleware {
        user,
        access_token_uuid: tokendetils.token_uuid,
    });

    Ok(next.run(req).await)
}

mod test {

    #[test]
    fn f() {
        let e = axum_extra::extract::CookieJar::new()
            .get("name")
            .and_then(|e| Some(e.to_string()));
        println!("{e:?}")
    }
}
