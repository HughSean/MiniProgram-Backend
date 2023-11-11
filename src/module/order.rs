use sea_orm::prelude::DateTime;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TryIntoModel};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::utils::error::HandleErr;
use crate::{appstate::AppState, module::db::orders};

use super::db::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum OrderStatus {
    Done,
    Going,
    Waiting,
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

pub struct OrderOp;
impl OrderOp {
    pub async fn save<T>(
        user_id: Uuid,
        order: SaveOrder,
        state: &AppState,
    ) -> Result<orders::Model, HandleErr<T>> {
        orders::ActiveModel {
            order_id: order.order_id.map(Set).unwrap_or(NotSet),
            user_id: Set(user_id),
            court_id: order.court_id.map(Set).unwrap_or(NotSet),
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
            HandleErr::ServerInnerErr(id)
        })?
        .try_into_model()
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })
    }

    pub async fn hasClash(
        start: DateTime,
        end: DateTime,
        order_id: Option<Uuid>,
        court_id: Uuid,

        state: &AppState,
    ) -> Result<bool, HandleErr<&'static str>> {
        Ok((end.date() - start.date()).num_days() >= 1
            || Courts::find_by_id(court_id)
                .one(&state.db)
                .await
                .map(|e| match e {
                    Some(e) => Some(start.time() < e.open_time || end.time() > e.close_time),
                    None => None,
                })
                .map_err(|err| {
                    let id = Uuid::new_v4();
                    error!("{} >>>> {}", id, err.to_string());
                    HandleErr::ServerInnerErr(id)
                })?
                .ok_or(HandleErr::BadRequest(-1, "court_id无效"))?
            || Orders::find()
                .filter(
                    orders::Column::CourtId
                        .eq(court_id)
                        .and(orders::Column::OrderId.ne(order_id.unwrap_or(Uuid::nil()))),
                )
                .all(&state.db)
                .await
                .map_err(|err| {
                    let id = Uuid::new_v4();
                    error!("{} >>>> {}", id, err.to_string());
                    HandleErr::ServerInnerErr(id)
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
    ) -> Result<OrderStatus, HandleErr<&'static str>> {
        let order = Orders::find()
            .filter(
                orders::Column::OrderId
                    .eq(order_id)
                    .and(orders::Column::UserId.eq(user_id)),
            )
            .one(&state.db)
            .await
            .map_err(|err| {
                let id = Uuid::new_v4();
                error!("{} >>>> {}", id, err.to_string());
                HandleErr::ServerInnerErr(id)
            })?
            .ok_or(HandleErr::BadRequest(-1, "订单信息不存在"))?;
        let now = chrono::Utc::now().naive_utc();

        if order.apt_start > now {
            Ok(OrderStatus::Waiting)
        } else if order.apt_end > now {
            Ok(OrderStatus::Going)
        } else {
            Ok(OrderStatus::Done)
        }
    }
}
