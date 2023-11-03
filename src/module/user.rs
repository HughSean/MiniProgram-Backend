use crate::{appstate::AppState, utils::passwd};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UserRegisterSchema {
    pub name: String,
    pub pwd: String,
    pub phone: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLoginSchema {
    pub name: String,
    pub pwd: String,
}

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
        schema: &UserRegisterSchema,
    ) -> Result<(), String> {
        let password_hash = passwd::passwd_encode(&schema.pwd)?;
        let _n = sqlx::query!(
            "insert into users (name, pwd, phone, role) values ($1, $2, $3, $4);",
            schema.name,
            password_hash,
            schema.phone,
            schema.role
        )
        .execute(&state.pgpool)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();
        info!("用户({})注册成功", schema.name);
        Ok(())
    }

    pub async fn fetch_optional_by_name(name: &str, state: &AppState) -> Option<UserSchema> {
        sqlx::query_as!(UserSchema, "select * from users where name = $1", name)
            .fetch_optional(&state.pgpool)
            .await
            .unwrap_or(None)
    }
}
