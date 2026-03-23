use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "listmonk_cache")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub listmonk_cache_id: i32,
    pub listmonk_cache_contact: i32,
    pub listmonk_cache_data: String,
    pub listmonk_cache_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contact::Entity",
        from = "Column::ListmonkCacheContact",
        to = "super::contact::Column::ContactId"
    )]
    Contact,
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
