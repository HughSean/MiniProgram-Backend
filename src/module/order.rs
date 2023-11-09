use super::db::{self, prelude::*};
use crate::utils::error::BaseError;
use crate::{appstate::AppState, module::db::orders};
use sea_orm::prelude::DateTime;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TryIntoModel};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug)]
pub enum OrderStatus {
    Done,
    Going,
    Waiting,
}

pub struct OrderOp;
impl OrderOp {
    pub async fn orderSave<T>(
        user_id: Uuid,
        order: SaveOrder,
        state: &AppState,
    ) -> Result<db::orders::Model, BaseError<T>> {
        Ok(orders::ActiveModel {
            order_id: order
                .order_id
                .and_then(|e| Some(Set(e)))
                .or(Some(NotSet))
                .unwrap(),
            user_id: Set(user_id),
            court_id: order
                .court_id
                .and_then(|e| Some(Set(e)))
                .or(Some(NotSet))
                .unwrap(),
            apt_start: Set(order.apt_start),
            apt_end: Set(order.apt_end),
            cost: Set(order.cost),
            ..Default::default()
        }
        .save(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr(id)
        })?
        .try_into_model()
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr(id)
        })?)
    }

    pub async fn hasClash<T>(
        start: DateTime,
        end: DateTime,
        court_id: Uuid,
        state: &AppState,
    ) -> Result<bool, BaseError<T>> {
        Ok(Orders::find()
            .filter(orders::Column::CourtId.eq(court_id))
            .all(&state.db)
            .await
            .map_err(|err| {
                let id = Uuid::new_v4();
                error!("{} >>>> {}", id, err.to_string());
                BaseError::ServerInnerErr(id)
            })?
            .into_iter()
            .map(|e| {
                (start > e.apt_start && start < e.apt_end)
                    || (end > e.apt_start && end < e.apt_end)
                    || (start == e.apt_start && end == e.apt_end)
            })
            .any(|e| e))
    }

    pub async fn orderStatus(
        order_id: Uuid,
        user_id: Uuid,
        state: &AppState,
    ) -> Result<OrderStatus, BaseError<&'static str>> {
        let order = Orders::find()
            .filter(
                db::orders::Column::OrderId
                    .eq(order_id)
                    .and(db::orders::Column::UserId.eq(user_id)),
            )
            .one(&state.db)
            .await
            .map_err(|err| {
                let id = Uuid::new_v4();
                error!("{} >>>> {}", id, err.to_string());
                BaseError::ServerInnerErr(id)
            })?
            .ok_or(BaseError::BadRequest(-1, "订单信息不存在"))?;
        let now = chrono::naive::NaiveDateTime::from(chrono::Utc::now().naive_utc());

        if order.apt_start > now {
            Ok(OrderStatus::Waiting)
        } else {
            if order.apt_end > now {
                Ok(OrderStatus::Going)
            } else {
                Ok(OrderStatus::Done)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, sea_orm::FromQueryResult)]
pub struct OrderAdminSchema {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub court_id: Uuid,
    pub user_name: String,
    pub court_name: String,
    pub create_time: DateTime,
    pub apt_start: DateTime,
    pub apt_end: DateTime,
    pub cost: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, sea_orm::FromQueryResult)]
pub struct OrderUserSchema {
    pub order_id: Uuid,
    pub court_id: Uuid,
    pub court_name: String,
    pub create_time: DateTime,
    pub apt_start: DateTime,
    pub apt_end: DateTime,
    pub cost: f64,
}
//update/insert
#[derive(Debug, Deserialize, Clone)]
pub struct SaveOrder {
    pub order_id: Option<Uuid>,
    pub court_id: Option<Uuid>,
    pub apt_start: DateTime,
    pub apt_end: DateTime,
    pub cost: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubmitOrder {
    pub court_id: Uuid,
    pub apt_start: DateTime,
    pub apt_end: DateTime,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DelOrder {
    pub order_id: Uuid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateOrder {
    pub order_id: Uuid,
    pub apt_start: DateTime,
    pub apt_end: DateTime,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrdersOfCourt {
    pub court_id: Uuid,
}
