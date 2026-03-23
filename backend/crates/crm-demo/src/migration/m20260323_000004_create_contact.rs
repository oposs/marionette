use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000004_create_contact"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE contact (
                    contact_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    contact_name TEXT NOT NULL,
                    contact_email TEXT NOT NULL,
                    contact_phone TEXT,
                    contact_title TEXT,
                    contact_company INTEGER REFERENCES company(company_id) ON DELETE SET NULL,
                    contact_created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    contact_updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS contact")
            .await?;
        Ok(())
    }
}
