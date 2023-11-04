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

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct GetOrderSchema {
    pub id: Uuid,
    pub user_name: String,
    pub court_name: String,
    pub order_time: chrono::NaiveDateTime,
    pub apt_start: chrono::NaiveDateTime,
    pub remark: String,
}
