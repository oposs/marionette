use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub contact_id: i32,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub contact_title: Option<String>,
    pub contact_company: Option<i32>,
    pub contact_created_at: String,
    pub contact_updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::company::Entity",
        from = "Column::ContactCompany",
        to = "super::company::Column::CompanyId"
    )]
    Company,
}

impl Related<super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
