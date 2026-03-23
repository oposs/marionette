use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000009_create_listmonk_sync"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE listmonk_sync (
                    listmonk_sync_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    listmonk_sync_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
                    listmonk_sync_status TEXT NOT NULL CHECK (listmonk_sync_status IN ('success', 'error')),
                    listmonk_sync_error TEXT,
                    listmonk_sync_subscriber_id INTEGER,
                    listmonk_sync_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE(listmonk_sync_contact)
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS listmonk_sync")
            .await?;
        Ok(())
    }
}
