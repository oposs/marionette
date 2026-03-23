use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_tag_contact: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_tag_tag: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contact::Entity",
        from = "Column::ContactTagContact",
        to = "super::contact::Column::ContactId"
    )]
    Contact,
    #[sea_orm(
        belongs_to = "super::tag::Entity",
        from = "Column::ContactTagTag",
        to = "super::tag::Column::TagId"
    )]
    Tag,
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
