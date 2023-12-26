use crate::{
    appstate::AppState,
    error::HandleErr,
    module::db::{self, prelude::*},
    module::order::{
        DelOrder, OrderOp, OrderStatus, OrderUserSchema, SaveOrder, SubmitOrder, UpdateOrder,
    },
    utils::auth::JWTAuthMiddleware,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, RelationTrait};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
pub fn router() -> Router<Arc<AppState>> {
    info!("/order/* 挂载中");
    Router::new()
        .route("/submit", post(submit))
        .route("/all", get(all))
        .route("/del", delete(del))
        .route("/update", post(update))
}

async fn submit(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<SubmitOrder>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    if !OrderOp::hasClash(
        schema.apt_start,
        schema.apt_end,
        None,
        schema.court_id,
        &state,
    )
    .await
    .map_err(|err| err.into())?
    {
        let court = Courts::find_by_id(schema.court_id)
            .one(&state.db)
            .await
            .map_err(|err| {
                let id = Uuid::new_v4();
                error!("{} >>>> {}", id, err.to_string());
                HandleErr::ServerInnerErr(id)
            })?
            .unwrap();

        let cost =
            (schema.apt_end - schema.apt_start).num_minutes() as f64 / 60.0 * court.price_per_hour;
        let order = OrderOp::save(
            auth.user.user_id,
            SaveOrder {
                order_id: None,
                court_id: Some(schema.court_id),
                apt_start: schema.apt_start,
                apt_end: schema.apt_end,
                cost,
            },
            &state,
        )
        .await?;
        let order = OrderUserSchema {
            order_id: order.order_id,
            court_id: order.court_id,
            court_name: court.court_name,
            create_time: order.create_time,
            apt_start: order.apt_start,
            apt_end: order.apt_end,
            cost,
        };

        Ok(Json(json!({
            "code":0,
            "msg":"预定成功",
            "data":order
        })))
    } else {
        Err(HandleErr::BadRequest(-1, "时间冲突".to_string()))
    }
}

async fn all(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    let orders = Orders::find()
        .filter(db::orders::Column::UserId.eq(auth.user.user_id))
        .join(
            sea_orm::JoinType::InnerJoin,
            db::orders::Relation::Courts.def(),
        )
        .column_as(db::courts::Column::CourtName, "court_name")
        .into_model::<OrderUserSchema>()
        .all(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?;
    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":orders
    })))
}

async fn del(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<DelOrder>,
) -> Result<impl IntoResponse, HandleErr<&'static str>> {
    if OrderStatus::Done == OrderOp::orderStatus(schema.order_id, auth.user.user_id, &state).await?
    {
        return Err(HandleErr::BadRequest(-1, "当前订单已完成"));
    }
    let rows_affected = Orders::delete_by_id(schema.order_id)
        .exec(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?
        .rows_affected;
    if rows_affected == 0 {
        Err(HandleErr::BadRequest(-1, "没有数据行被删除"))
    } else {
        Ok(Json(json!({
            "code":0,
            "msg":"操作成功"
        })))
    }
}

async fn update(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<UpdateOrder>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    let court_id = Orders::find_by_id(schema.order_id)
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?
        .ok_or(HandleErr::BadRequest(-1, "order_id无效").into())?
        .court_id;

    if OrderOp::orderStatus(schema.order_id, auth.user.user_id, &state)
        .await
        .map_err(|err| err.into())?
        == OrderStatus::Waiting
        && !OrderOp::hasClash(
            schema.apt_start,
            schema.apt_end,
            Some(schema.order_id),
            court_id,
            &state,
        )
        .await
        .map_err(|err| err.into())?
    {
        let price = Courts::find()
            .join_rev(
                sea_orm::JoinType::InnerJoin,
                db::orders::Relation::Courts.def(),
            )
            .filter(db::orders::Column::OrderId.eq(schema.order_id))
            .one(&state.db)
            .await
            .map_err(|err| {
                let id = Uuid::new_v4();
                error!("{} >>>> {}", id, err.to_string());
                HandleErr::ServerInnerErr(id)
            })?
            .unwrap()
            .price_per_hour;

        let order = OrderOp::save(
            auth.user.user_id,
            SaveOrder {
                order_id: Some(schema.order_id),
                court_id: None,
                apt_start: schema.apt_start,
                apt_end: schema.apt_end,
                cost: (schema.apt_end - schema.apt_start).num_minutes() as f64 / 60.0 * price,
            },
            &state,
        )
        .await?;
        Ok(Json(json!({
            "code":0,
            "msg":"订单已修改",
            "data":order
        })))
    } else {
        Err(HandleErr::BadRequest(-1, "无法修改").into())
    }
}
