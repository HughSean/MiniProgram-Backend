use crate::{
    appstate::AppState,
    error::HandleErr,
    module::{
        db::{courts, orders, prelude::*, users},
        order::OrderAdminSchema,
    },
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;
pub fn router() -> Router<Arc<AppState>> {
    info!("/order/* 挂载中");
    Router::new().route("/:id", get(ordersOfcourt))
}

async fn ordersOfcourt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    debug!("into admin order");
    let orders = Orders::find()
        .filter(orders::Column::CourtId.eq(id))
        .join(JoinType::InnerJoin, orders::Relation::Users.def())
        .column_as(users::Column::UserName, "user_name")
        .column_as(users::Column::Phone, "user_phone")
        .join(JoinType::InnerJoin, orders::Relation::Courts.def())
        .column_as(courts::Column::CourtName, "court_name")
        .into_model::<OrderAdminSchema>()
        .all(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr::<String>(id)
        })?;
    debug!("pass admin order");
    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":orders
    })))
}
