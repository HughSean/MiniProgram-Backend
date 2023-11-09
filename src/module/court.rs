use super::db;
use crate::{appstate::AppState, utils::error::BaseError};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, Set, TryIntoModel};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;
pub struct CourtOp;
impl CourtOp {
    pub async fn save<T>(
        schema: SaveCourt,
        state: &AppState,
    ) -> Result<db::courts::Model, BaseError<T>> {
        Ok(db::courts::ActiveModel {
            court_id: schema
                .court_id
                .and_then(|e| Some(Set(e)))
                .or(Some(NotSet))
                .unwrap(),
            admin_id: Set(schema.admin_id),
            court_name: Set(schema.court_name),
            location: Set(schema.location),
            label: Set(schema.label),
            price_per_hour: Set(schema.price_per_hour),
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
}

//update/insert
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveCourt {
    pub court_id: Option<Uuid>,
    pub admin_id: Uuid,
    pub court_name: String,
    pub location: String,
    pub label: String,
    pub price_per_hour: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CourtSchema {
    pub court_id: Option<Uuid>,
    pub admin_id: Uuid,
    pub court_name: String,
    pub location: String,
    pub label: String,
    pub price_per_hour: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddCourt {
    pub court_name: String,
    pub location: String,
    pub label: String,
    pub price_per_hour: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DelCourt {
    pub court_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateCourt {
    pub court_id: Uuid,
    pub court_name: String,
    pub location: String,
    pub label: String,
    pub price_per_hour: f64,
}