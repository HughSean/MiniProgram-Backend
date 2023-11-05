use crate::{
    appstate::AppState,
    module::{
        db::{courts, orders, prelude::*, users},
        order::OrderAdminSchema,
    },
    utils::{auth::JWTAuthMiddleware, error::BaseError},
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

async fn ordersOfcourt<T>(
    State(state): State<AppState>,
    Path(court_id): Path<uuid::Uuid>,
    Extension(auth): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, BaseError<T>> {
    let orders = Orders::find()
        .filter(orders::Column::CourtId.eq(court_id))
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
            BaseError::ServerInnerErr(id)
        })?;

    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":orders
    })))
}
