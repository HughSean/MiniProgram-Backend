use crate::{api::register, appstate::AppState, utils::passwd};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UserSchema {
    pub id: Uuid,
    pub name: String,
    pub pwd: String,
    pub phone: String,
    pub role: String,
    pub create_time: chrono::NaiveDateTime,
}

impl UserSchema {
    pub async fn register_new_user(
        state: &AppState,
        info: register::RegisterSchema,
    ) -> Result<(), String> {
        let password_hash = passwd::passwd_encode(&info.pwd)?;

        let n = sqlx::query!(
            "INSERT INTO users (name, pwd, phone, role) VALUES ($1, $2, $3, $4);",
            info.name,
            password_hash,
            info.phone,
            info.role
        )
        .execute(&state.pgpool)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();

        warn!("in fn register_new_user, query rows_affected: {}", n);
        Ok(())
    }
}
