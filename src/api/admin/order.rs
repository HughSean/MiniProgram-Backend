use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    appstate::AppState,
    module::{
        db::{courts, orders, prelude::*, users},
        order::{OrderAdminSchema, OrdersOfCourt},
    },
    utils::{auth::JWTAuthMiddleware, error::HandleErr},
};

pub fn router() -> Router<Arc<AppState>> {
    info!("/order/* 挂载中");
    Router::new().route("/orders_of_court", post(ordersOfcourt))
}

async fn ordersOfcourt(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<OrdersOfCourt>,
) -> Result<impl IntoResponse, HandleErr<String>> {
    let orders = Orders::find()
        .filter(orders::Column::CourtId.eq(schema.court_id))
        .join(JoinType::InnerJoin, orders::Relation::Users.def())
        .filter(users::Column::UserId.eq(auth.user.user_id))
        .column_as(users::Column::UserName, "user_name")
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

    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":orders
    })))
}
