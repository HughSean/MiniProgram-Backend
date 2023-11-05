use crate::{
    appstate::AppState,
    module::db::{self, prelude::*},
    module::order::{DelOrder, OrderOp, OrderStatus, OrderUserSchema, SaveOrder, UpdateOrder},
    utils::{auth::JWTAuthMiddleware, error::BaseError},
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
    Json(schema): Json<SaveOrder>,
) -> Result<impl IntoResponse, BaseError<String>> {
    if !OrderOp::hasClash::<String>(
        schema.apt_start,
        schema.apt_end,
        schema.court_id.unwrap_or(Uuid::nil()),
        &state,
    )
    .await?
    {
        let id = OrderOp::orderSave::<String>(auth.user.user_id, schema, &state).await?;
        Ok(Json(json!({
            "code":0,
            "msg":"预定成功",
            "order_id":id
        })))
    } else {
        Err(BaseError::BadRequest(-1, "时间段冲突".to_string()))
    }
}

async fn all(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, BaseError<()>> {
    let data = Orders::find()
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
            BaseError::ServerInnerErr(id)
        })?;
    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":data
    })))
}

async fn del(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<DelOrder>,
) -> Result<impl IntoResponse, BaseError<&'static str>> {
    if OrderStatus::Done == OrderOp::orderStatus(schema.order_id, auth.user.user_id, &state).await?
    {
        return Err(BaseError::BadRequest(-1, "当前订单已完成"));
    }
    if Orders::delete_by_id(schema.order_id)
        .exec(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr(id)
        })?
        .rows_affected
        == 0
    {
        Err(BaseError::BadRequest(-1, "没有数据行被删除"))
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
) -> Result<impl IntoResponse, BaseError<String>> {
    if OrderOp::orderStatus(schema.order_id, auth.user.user_id, &state).await?
        == OrderStatus::Waiting
    {
        let id = OrderOp::orderSave::<String>(
            auth.user.user_id,
            SaveOrder {
                order_id: Some(schema.order_id),
                court_id: None,
                apt_start: schema.apt_start,
                apt_end: schema.apt_end,
            },
            &state,
        )
        .await?;
        Ok(Json(json!({
            "code":0,
            "msg":format!("订单({})已修改",id)
        })))
    } else {
        Err(BaseError::BadRequest(-1, "订单已完成或正在进行, 无法修改").into())
    }
}