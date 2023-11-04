use crate::{appstate::AppState, utils::error::BaseError};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtSchema {
    pub id: Uuid,
    pub admin: Uuid,
    pub name: String,
    pub location: String,
    pub class: String,
}

impl CourtSchema {
    pub async fn add(
        schema: &CourtAddSchema,
        admin_id: &Uuid,
        state: &AppState,
    ) -> Result<(), String> {
        sqlx::query!(
            "insert into courts (admin,name,location,class) values($1, $2, $3, $4);",
            admin_id,
            schema.name,
            schema.location,
            schema.class
        )
        .execute(&state.pgpool)
        .await
        .or_else(|err| Err(err.to_string()))?;
        Ok(())
    }

    pub async fn del(
        schema: &CourtDelSchema,
        admin_id: &Uuid,
        state: &AppState,
    ) -> Result<(), String> {
        if sqlx::query_scalar!(
            "select exists(select * from orders where courtid = $1)",
            schema.id
        )
        .fetch_optional(&state.pgpool)
        .await
        .or(Err("查询错误".to_string()))?
        .flatten()
        .unwrap()
        {
            Err("该球场仍有未完成的订单".to_string())
        } else {
            if sqlx::query!(
                "delete from courts where admin = $1 and id = $2",
                admin_id,
                schema.id
            )
            .execute(&state.pgpool)
            .await
            .or_else(|err| Err(err.to_string()))?
            .rows_affected()
                == 0
            {
                return Err("删除失败, 没有该球场".to_string());
            }
            Ok(())
        }
    }
    pub async fn all(admin_id: &Uuid, state: &AppState) -> Result<Vec<CourtSchema>, String> {
        let courts = sqlx::query_as!(CourtSchema, "select * from courts where admin=$1", admin_id,)
            .fetch_all(&state.pgpool)
            .await
            .map_err(|err| err.to_string())?;
        Ok(courts)
    }
    pub async fn update(
        schema: &CourtUpSchema,
        admin_id: &Uuid,
        state: &AppState,
    ) -> Result<(), String> {
        if sqlx::query!(
            "update courts set name=$1, location=$2, class=$3 where id=$4 and admin=$5",
            schema.name,
            schema.location,
            schema.class,
            schema.id,
            admin_id
        )
        .execute(&state.pgpool)
        .await
        .map_err(|err| err.to_string())?
        .rows_affected()
            == 0
        {
            return Err("没有被更改的数据".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtAddSchema {
    pub name: String,
    pub location: String,
    pub class: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtDelSchema {
    pub id: Uuid,
}
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtAllSchema {
    pub id: Uuid,
    pub name: String,
    pub location: String,
    pub class: String,
}
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct CourtUpSchema {
    pub id: Uuid,
    pub name: String,
    pub location: String,
    pub class: String,
}
