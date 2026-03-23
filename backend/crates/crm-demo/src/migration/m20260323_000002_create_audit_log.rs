use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000002_create_audit_log"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE audit_log (
                    audit_log_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    audit_log_user INTEGER NOT NULL,
                    audit_log_table TEXT NOT NULL,
                    audit_log_record_id INTEGER NOT NULL,
                    audit_log_action TEXT NOT NULL CHECK (audit_log_action IN ('create', 'update', 'delete')),
                    audit_log_changes TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(audit_log_changes)),
                    audit_log_timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (audit_log_user) REFERENCES user(user_id)
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS audit_log")
            .await?;
        Ok(())
    }
}
