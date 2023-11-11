use super::db::{self, prelude::Users};
use crate::{
    appstate::AppState,
    utils::{error::HandleErr, passwd},
};
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, sea_orm::FromQueryResult)]
pub struct UserSchema {
    pub user_id: Uuid,
    pub user_name: String,
    // pub pwd: String,
    pub phone: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct UserRegisterSchema {
    pub name: String,
    pub pwd: String,
    pub phone: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct UserLoginSchema {
    pub name: String,
    pub pwd: String,
}

pub struct UserOP;

impl UserOP {
    pub async fn register_new_user<T>(
        schema: UserRegisterSchema,
        state: &AppState,
    ) -> Result<Uuid, HandleErr<T>> {
        let password_hash = passwd::hash_password(&schema.pwd)?;
        let id = Users::insert(db::users::ActiveModel {
            user_name: Set(schema.name),
            user_pwd: Set(password_hash),
            phone: Set(schema.phone),
            is_admin: Set(schema.is_admin),
            ..Default::default()
        })
        .exec(&state.db)
        .await
        .map_err(|err| {
            let id = Uuid::new_v4();
            error!("{} >>>> {}", id, err.to_string());
            HandleErr::ServerInnerErr(id)
        })?
        .last_insert_id;
        Ok(id)
    }
}
