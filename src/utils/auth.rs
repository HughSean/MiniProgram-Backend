use super::{error::BaseError, token};
use crate::{appstate::AppState, module::user::UserSchema};
use axum::{
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: UserSchema,
    pub access_token_uuid: uuid::Uuid,
}

pub async fn auth<B>(
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, BaseError<&'static str>> {
    debug!("提取token");
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
        .ok_or_else(|| {
            warn!("未发现token");
            BaseError::BadRequest(-1, "需要koen")
        })?;

    debug!("正在验证token");
    //校验token
    let tokendetils = token::jwt_token_verify(&access_token, &state.cfg.tokencfg.access_pubkey)
        .or_else(|err| {
            warn!("token 验证错误: {}", err.to_string());
            Err(BaseError::BadRequest(-1, "无效token"))
        })?;

    debug!("查询用户({})信息", tokendetils.user_id.to_string());
    //查询用户数据库确定该用户是否存在
    let user = sqlx::query_as!(
        UserSchema,
        "select * from users where id =$1",
        tokendetils.user_id
    )
    .fetch_optional(&state.pgpool)
    .await
    .or_else(|err| {
        warn!("数据库查询错误: {}", err.to_string());
        Err(BaseError::ServerInnerErr)
    })?
    .ok_or_else(|| {
        warn!("token所属用户不存在");
        BaseError::BadRequest(-1, "用户不存在")
    })?;
    //token签名校验/数据库用户校验, 全部完成用过认证, 为合法用户
    info!("用户({})身份验证通过", user.name);
    req.extensions_mut().insert(JWTAuthMiddleware {
        user,
        access_token_uuid: tokendetils.token_uuid,
    });
    Ok(next.run(req).await)
}

pub async fn admin_auth<B>(
    Extension(jwt_guard): Extension<JWTAuthMiddleware>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, BaseError<&'static str>> {
    if jwt_guard.user.role == "admin" {
        Ok(next.run(req).await)
    } else {
        warn!("用户({})被拒绝访问/admin", jwt_guard.user.name);
        Err(BaseError::BadRequest(-1, "非管理员用户, 禁止通行"))
    }
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
