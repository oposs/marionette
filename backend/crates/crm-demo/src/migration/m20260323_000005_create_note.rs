use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000005_create_note"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE note (
                    note_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    note_contact INTEGER REFERENCES contact(contact_id) ON DELETE CASCADE,
                    note_company INTEGER REFERENCES company(company_id) ON DELETE CASCADE,
                    note_text TEXT NOT NULL,
                    note_user INTEGER NOT NULL REFERENCES user(user_id),
                    note_created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS note")
            .await?;
        Ok(())
    }
}
