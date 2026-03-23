use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000003_create_company"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE company (
                    company_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    company_name TEXT NOT NULL UNIQUE,
                    company_website TEXT,
                    company_address TEXT,
                    company_created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    company_updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS company")
            .await?;
        Ok(())
    }
}
