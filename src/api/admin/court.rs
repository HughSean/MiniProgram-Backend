use crate::{
    appstate::AppState,
    error::HandleErr,
    module::court::{CourtAdd, CourtDel, CourtOp, CourtSave},
    module::{
        court::{CourtAdminSchema, CourtUpdate},
        db,
        db::prelude::{self, Courts},
    },
    utils::auth::JWTAuthMiddleware,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use prelude::Orders;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;
pub fn router() -> Router<Arc<AppState>> {
    info!("/court/* 挂载中");
    Router::new()
        .route("/add", post(add))
        .route("/del", delete(del))
        .route("/all", get(all))
        .route("/update", post(update))
}

async fn add(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtAdd>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    Courts::find()
        .filter(
            db::courts::Column::CourtName
                .eq(&schema.court_name)
                .and(db::courts::Column::AdminId.eq(auth.user.user_id)),
        )
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr::<String>(id)
        })?
        .map_or(Ok(()), |_| {
            Err(HandleErr::BadRequest(-1, "球场名重复".to_string()))
        })?;

    let court = CourtOp::save(
        CourtSave {
            court_id: None,
            admin_id: Some(auth.user.user_id),
            court_name: schema.court_name.clone(),
            location: schema.location,
            label: schema.label,
            price_per_hour: schema.price_per_hour,
            open_time: schema.open_time,
            close_time: schema.close_time,
        },
        &state,
    )
    .await?;

    info!(
        "admin({})添加球场({})",
        auth.user.user_name, schema.court_name
    );

    debug!("pass court add");

    Ok(Json(json!({
        "code":0,
        "msg":"球场添加成功",
        "data":court
    })))
}

async fn del(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtDel>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    let now = chrono::Utc::now().naive_utc();
    Orders::find()
        .filter(
            db::orders::Column::CourtId
                .eq(schema.court_id)
                .and(db::orders::Column::AptEnd.gte(now)),
        )
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?
        .map_or(Ok(()), |_| {
            Err(HandleErr::BadRequest(-1, "球场仍有未完成的订单").into())
        })?;

    let rows_affected = Courts::delete(db::courts::ActiveModel {
        admin_id: Set(auth.user.user_id),
        court_id: Set(schema.court_id),
        ..Default::default()
    })
    .exec(&state.db)
    .await
    .map_err(|err| {
        let id = Uuid::new_v4();
        error!("{} >>>> {}", id, err.to_string());
        HandleErr::ServerInnerErr::<String>(id)
    })?
    .rows_affected;

    if rows_affected == 0 {
        Err(HandleErr::BadRequest(-1, "没有球场被删除").into())
    } else {
        info!(
            "admin({})删除球场({})",
            auth.user.user_name, schema.court_id
        );
        Ok(Json(json!({"code":0,"msg":"球场删除成功"})))
    }
}

async fn all(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    let courts: Vec<_> = Courts::find()
        .filter(db::courts::Column::AdminId.eq(auth.user.user_id))
        .all(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr::<String>(id)
        })?
        .into_iter()
        .map(|e| CourtAdminSchema {
            court_id: Some(e.court_id),
            admin_id: Some(auth.user.user_id),
            court_name: e.court_name,
            location: e.location,
            label: e.label,
            price_per_hour: e.price_per_hour,
            open_time: e.open_time,
            close_time: e.close_time,
        })
        .collect();
    debug!("pass court all");
    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":courts
    })))
}

async fn update(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<CourtUpdate>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    Courts::find()
        .filter(
            db::courts::Column::CourtId
                .eq(schema.court_id)
                .and(db::courts::Column::AdminId.eq(auth.user.user_id)),
        )
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr::<String>(id)
        })?
        .ok_or(HandleErr::BadRequest(-1, "球场不存在".to_string()))?;
    let court = CourtOp::save::<String>(
        CourtSave {
            court_id: schema.court_id,
            admin_id: Some(auth.user.user_id),
            court_name: schema.court_name,
            location: schema.location,
            label: schema.label,
            price_per_hour: schema.price_per_hour,
            open_time: schema.open_time,
            close_time: schema.close_time,
        },
        &state,
    )
    .await?;

    debug!("pass court update");

    Ok(Json(json!({
        "code":0,
        "msg":"操作成功",
        "data":court
    })))
}
