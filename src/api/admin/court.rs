use crate::{
    appstate::AppState,
    module::court::{AddCourt, CourtOp, DelCourt, SaveCourt},
    module::{
        db,
        db::prelude::{self, Courts},
    },
    utils::{auth::JWTAuthMiddleware, error::BaseError},
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use prelude::Orders;
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait, Set};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
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
    Json(schema): Json<AddCourt>,
) -> Result<impl IntoResponse, BaseError<String>> {
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
            BaseError::ServerInnerErr::<String>(id)
        })?
        .map_or(
            Err(BaseError::BadRequest(-1, "球场不存在".to_string())),
            |_| Ok(()),
        )?;
    let id = CourtOp::save::<String>(
        SaveCourt {
            court_id: None,
            court_name: schema.court_name.clone(),
            location: schema.location,
            label: schema.label,
        },
        &state,
    )
    .await?;

    info!("admin({})添加球场({})", auth.user.name, schema.court_name);
    Ok(Json(json!({"code":0,"msg":"球场添加成功","court_id":id})))
}

async fn del(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<DelCourt>,
) -> Result<impl IntoResponse, BaseError<String>> {
    let now = chrono::Utc::now().naive_utc();
    Orders::find()
        .filter(db::orders::Column::CourtId.eq(schema.court_id))
        .join(JoinType::InnerJoin, db::courts::Relation::Orders.def())
        .filter(db::orders::Column::AptEnd.gte(now))
        .one(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr::<String>(id)
        })?
        .map_or(Ok(()), |_| {
            Err(BaseError::BadRequest(-1, "球场仍有未完成的订单"))
        })?;

    if Courts::delete(db::courts::ActiveModel {
        admin_id: Set(auth.user.user_id),
        court_id: Set(schema.court_id),
        ..Default::default()
    })
    .exec(&state.db)
    .await
    .map_err(|err| {
        let id = Uuid::new_v4();
        error!("{} >>>> {}", id, err.to_string());
        BaseError::ServerInnerErr::<String>(id)
    })?
    .rows_affected
        == 0
    {
        Err(BaseError::BadRequest(
            -1,
            format!("没有球场({})", schema.court_id).to_string(),
        ))
    } else {
        info!("admin({})删除球场({})", auth.user.name, schema.court_id);
        Ok(Json(json!({"code":0,"msg":"球场删除成功"})))
    }
}

async fn all(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, BaseError<String>> {
    let courts: Vec<_> = Courts::find()
        .filter(db::courts::Column::AdminId.eq(auth.user.user_id))
        .all(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr::<String>(id)
        })?
        .into_iter()
        .map(|e| SaveCourt {
            court_id: Some(e.court_id),
            court_name: e.court_name,
            location: e.location,
            label: e.label,
        })
        .collect();
    Ok(Json(json!({
        "code":0,
        "msg":"查询成功",
        "data":courts
    })))
}

async fn update(
    Extension(auth): Extension<JWTAuthMiddleware>,
    State(state): State<Arc<AppState>>,
    Json(schema): Json<SaveCourt>,
) -> Result<impl IntoResponse, BaseError<String>> {
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
            BaseError::ServerInnerErr::<String>(id)
        })?
        .ok_or(BaseError::BadRequest(-1, "球场不存在".to_string()))?;
    CourtOp::save::<String>(schema, &state).await?;
    Ok(Json(json!({
        "code":0,
        "msg":"操作成功",
    })))
}
