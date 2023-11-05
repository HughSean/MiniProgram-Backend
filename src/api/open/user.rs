use crate::module::db;
use crate::module::user::{UserLoginSchema, UserOP, UserRegisterSchema};
use crate::utils::error::BaseError;
use crate::utils::{passwd, token};
use crate::{appstate::AppState, module::db::prelude::Users};
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::{IntoResponse, Response};
use axum::{routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};
use tracing::{error, info};
use uuid::Uuid;

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
    let user_schema = Users::find()
        .filter(db::users::Column::UserName.eq(&schema.name))
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr(id)
        })?
        .ok_or(BaseError::BadRequest(-1, "用户不存在"))?;
    debug!("校验密码");
    passwd::passwd_verify(&schema.pwd, &user_schema.user_pwd)?;
    //生成access_token
    debug!("生成token");
    let access_token_details = token::jwt_token_gen(
        user_schema.user_id,
        state.cfg.tokencfg.access_token_ttl,
        &state.cfg.tokencfg.access_prikey,
    )
    .map_err(|err| {
        let id = Uuid::new_v4();
        error!("{} >>>> {}", id, err.to_string());
        BaseError::ServerInnerErr(id)
    })?;

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
    let user = Users::find()
        .filter(
            crate::module::db::users::Column::UserName
                .eq(&schema.name)
                .or(crate::module::db::users::Column::Phone.eq(&schema.phone)),
        )
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr(id)
        })?;
    if user.is_some() {
        warn!(
            "注册失败: 用户名({})已存在, 或者号码({})已存在",
            schema.name, schema.phone
        );
        return Err(BaseError::BadRequest(-1, "用户名已存在, 或者号码已存在"));
    }

    debug!("注册中");
    let name = UserOP::register_new_user(schema, &state).await?;
    info!("用户({})注册成功", name);
    Ok(Json(json!({"code":0,"msg":"注册成功"})))
}
