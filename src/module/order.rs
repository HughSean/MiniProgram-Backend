use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct OrderSchema {
    pub id: Uuid,
    pub userid: Uuid,
    pub courtid: Uuid,
    pub order_time: chrono::NaiveDateTime,
    pub apt_start: chrono::NaiveDateTime,
    pub apt_end: chrono::NaiveDateTime,
    pub remark: String,
}
