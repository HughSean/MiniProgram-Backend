use super::db;
use crate::{appstate::AppState, error::HandleErr};
use sea_orm::prelude::Time;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    TryIntoModel,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;
//update/insert
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CourtSave {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub court_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_id: Option<Uuid>,
    pub court_name: String,
    pub location: String,
    pub label: String,
    pub price_per_hour: f64,
    pub open_time: Time,
    pub close_time: Time,
}

pub type CourtAdd = CourtSave;
pub type CourtUpdate = CourtSave;
pub type CourtAdminSchema = CourtSave;
pub type CourtUserSchema = CourtSave;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CourtDel {
    pub court_id: Uuid,
}

pub struct CourtOp;
impl CourtOp {
    pub async fn save<T>(
        schema: CourtSave,
        state: &AppState,
    ) -> Result<db::courts::Model, HandleErr<T>> {
        db::courts::ActiveModel {
            court_id: schema.court_id.map(Set).unwrap_or(NotSet),
            admin_id: schema.admin_id.map(Set).unwrap_or(NotSet),
            court_name: Set(schema.court_name),
            location: Set(schema.location),
            label: Set(schema.label),
            price_per_hour: Set(schema.price_per_hour),
            open_time: Set(schema.open_time),
            close_time: Set(schema.close_time),
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
}
