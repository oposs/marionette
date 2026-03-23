use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "note")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub note_id: i32,
    pub note_contact: Option<i32>,
    pub note_company: Option<i32>,
    pub note_text: String,
    pub note_user: i32,
    pub note_created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contact::Entity",
        from = "Column::NoteContact",
        to = "super::contact::Column::ContactId"
    )]
    Contact,
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::NoteCompany",
        to = "super::company::Column::CompanyId"
    )]
    Company,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::NoteUser",
        to = "super::user::Column::UserId"
    )]
    User,
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
