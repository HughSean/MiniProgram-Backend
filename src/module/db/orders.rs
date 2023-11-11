//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub court_id: Uuid,
    pub create_time: DateTime,
    pub apt_start: DateTime,
    pub apt_end: DateTime,
    #[sea_orm(column_type = "Double")]
    pub cost: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::courts::Entity",
        from = "Column::CourtId",
        to = "super::courts::Column::CourtId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Courts,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::UserId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

impl Related<super::courts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Courts.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
