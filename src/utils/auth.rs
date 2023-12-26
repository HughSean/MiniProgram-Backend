use super::token;
use crate::{
    appstate::AppState,
    error::HandleErr,
    module::{db::prelude::Users, user::UserSchema},
};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: UserSchema,
}

pub async fn auth(
    cookie_jar: axum_extra::extract::CookieJar,
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HandleErr<&'static str>> {
    debug!("提取token");
    //从cookie或者头部取得token
    let access_token = cookie_jar
        .get("access_token")
        .map(|e| e.value().to_string())
        .or(req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|e| e.to_str().ok())
            .and_then(|auth_value| auth_value.strip_prefix("Bearer ").map(|e| e.to_string())))
        .ok_or_else(|| {
            warn!("未发现token");
            HandleErr::UnAuthorized
        })?;

    debug!("正在验证token");
    //校验token
    let user_id =
        token::verify(&access_token, &state.cfg.tokencfg.access_pubkey).map_err(|err| {
            warn!("token 验证错误: {}", err.to_string());
            HandleErr::UnAuthorized
        })?;

    debug!("查询用户({})信息", user_id.to_string());
    //查询用户数据库确定该用户是否存在
    let id = Uuid::new_v4();
    let user = Users::find()
        .filter(crate::module::db::users::Column::UserId.eq(user_id))
        .one(&state.db)
        .await
        .map_err(|err| {
            error!("id({}): {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?
        .ok_or_else(|| {
            warn!("token所属用户不存在");
            HandleErr::UnAuthorized
        })?;

    let user = UserSchema {
        user_id: user.user_id,
        user_name: user.user_name,
        // pwd: user.user_pwd,
        phone: user.phone,
        is_admin: user.is_admin,
    };
    //token签名校验/数据库用户校验, 全部完成用过认证, 为合法用户
    info!("用户({})身份验证通过", user.user_name);
    req.extensions_mut().insert(JWTAuthMiddleware { user });
    Ok(next.run(req).await)
}

pub async fn admin_auth(
    Extension(auth): Extension<JWTAuthMiddleware>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, HandleErr<&'static str>> {
    if auth.user.is_admin {
        Ok(next.run(req).await)
    } else {
        warn!("用户({})被拒绝访问 /admin/*", auth.user.user_name);
        Err(HandleErr::BadRequest(401, "权限不足"))
    }
}
