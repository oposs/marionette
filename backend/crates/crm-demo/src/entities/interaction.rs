use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "interaction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub interaction_id: i32,
    pub interaction_contact: i32,
    pub interaction_type: String,
    pub interaction_subject: String,
    pub interaction_notes: Option<String>,
    pub interaction_user: i32,
    pub interaction_date: String,
    pub interaction_created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contact::Entity",
        from = "Column::InteractionContact",
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
