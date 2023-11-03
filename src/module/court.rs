use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::appstate::AppState;

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtAddSchema {
    pub name: String,
    pub location: String,
    pub class: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtSchema {
    pub id: Uuid,
    pub admin: Uuid,
    pub name: String,
    pub location: String,
    pub class: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddCourtSchema {
    pub admin: Uuid,
    pub name: String,
    pub location: String,
    pub class: String,
}

impl CourtSchema {
    pub async fn add(
        court: CourtAddSchema,
        admin_id: Uuid,
        state: &AppState,
    ) -> Result<(), String> {
        sqlx::query!(
            "insert into courts (admin,name,location,class) values ($1, $2, $3,$4);",
            admin_id,
            court.name,
            court.location,
            court.class
        )
        .execute(&state.pgpool)
        .await
        .and_then(|_| {
            info!("球场添加成功");
            Ok(())
        })
        .or_else(|err| {
            warn!("球场添加失败: {}", err.to_string());
            Err(err.to_string())
        })
    }
}
