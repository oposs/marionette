use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub audit_log_id: i32,
    pub audit_log_user: i32,
    pub audit_log_table: String,
    pub audit_log_record_id: i32,
    pub audit_log_action: String,
    pub audit_log_changes: String,
    pub audit_log_timestamp: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
