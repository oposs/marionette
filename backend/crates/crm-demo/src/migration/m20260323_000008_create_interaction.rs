use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000008_create_interaction"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE interaction (
                    interaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    interaction_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
                    interaction_type TEXT NOT NULL CHECK(interaction_type IN ('call', 'email', 'meeting')),
                    interaction_subject TEXT NOT NULL,
                    interaction_notes TEXT,
                    interaction_user INTEGER NOT NULL REFERENCES user(user_id),
                    interaction_date TEXT NOT NULL,
                    interaction_created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS interaction")
            .await?;
        Ok(())
    }
}
