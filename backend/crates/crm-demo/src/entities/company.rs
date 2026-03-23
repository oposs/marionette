use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "company")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub company_id: i32,
    #[sea_orm(unique)]
    pub company_name: String,
    pub company_website: Option<String>,
    pub company_address: Option<String>,
    pub company_created_at: String,
    pub company_updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::contact::Entity")]
    Contacts,
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contacts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
