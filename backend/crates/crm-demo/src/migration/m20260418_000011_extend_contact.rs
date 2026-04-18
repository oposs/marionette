use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260418_000011_extend_contact"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE contact ADD COLUMN contact_country TEXT",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE contact ADD COLUMN contact_notes TEXT",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE contact ADD COLUMN contact_opt_in INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE contact DROP COLUMN contact_opt_in",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE contact DROP COLUMN contact_notes",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE contact DROP COLUMN contact_country",
        )
        .await?;
        Ok(())
    }
}
