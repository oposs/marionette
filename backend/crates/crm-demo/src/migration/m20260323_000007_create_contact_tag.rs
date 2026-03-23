use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260323_000007_create_contact_tag"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE contact_tag (
                    contact_tag_contact INTEGER NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
                    contact_tag_tag INTEGER NOT NULL REFERENCES tag(tag_id) ON DELETE CASCADE,
                    PRIMARY KEY (contact_tag_contact, contact_tag_tag)
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS contact_tag")
            .await?;
        Ok(())
    }
}
