use super::db;
use crate::{appstate::AppState, utils::error::BaseError};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;
pub struct CourtOp;
impl CourtOp {
    pub async fn save<T>(schema: SaveCourt, state: &AppState) -> Result<Uuid, BaseError<T>> {
        Ok(db::courts::ActiveModel {
            court_id: schema
                .court_id
                .and_then(|e| Some(Set(e)))
                .or(Some(NotSet))
                .unwrap(),
            court_name: Set(schema.court_name),
            location: Set(schema.location),
            label: Set(schema.label),
            ..Default::default()
        }
        .save(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            BaseError::ServerInnerErr(id)
        })?
        .court_id
        .unwrap())
    }
}

//update/insert
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveCourt {
    pub court_id: Option<Uuid>,
    pub court_name: String,
    pub location: String,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddCourt {
    pub court_name: String,
    pub location: String,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DelCourt {
    pub court_id: Uuid,
}
