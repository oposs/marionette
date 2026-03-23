use sea_orm_migration::prelude::*;

mod m20260323_000001_create_user;
mod m20260323_000002_create_audit_log;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260323_000001_create_user::Migration),
            Box::new(m20260323_000002_create_audit_log::Migration),
        ]
    }
}
