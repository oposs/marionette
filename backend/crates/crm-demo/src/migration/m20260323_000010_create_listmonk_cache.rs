use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000010_create_listmonk_cache"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE listmonk_cache (
                    listmonk_cache_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    listmonk_cache_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
                    listmonk_cache_data TEXT NOT NULL CHECK (json_valid(listmonk_cache_data)),
                    listmonk_cache_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE(listmonk_cache_contact)
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS listmonk_cache")
            .await?;
        Ok(())
    }
}
