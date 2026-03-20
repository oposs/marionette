use sea_orm_migration::MigratorTrait;

use crate::migration::Migrator;

/// `SeaORM` entity for the `session` table.
///
/// Follows project SQL conventions: singular table name, `table_field` columns.
pub mod session {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "session")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub session_id: i32,
        #[sea_orm(unique)]
        pub session_token: String,
        pub session_user: Option<i32>,
        /// JSON array of role strings, e.g. `["admin", "user"]`.
        pub session_roles: String,
        pub session_created: String,
        pub session_expires: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/// Connect to the database and run all pending migrations.
///
/// # Errors
///
/// Returns `sea_orm::DbErr` if connection or migration fails.
pub async fn init_db(database_url: &str) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    let db = sea_orm::Database::connect(database_url).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

/// Create an in-memory `SQLite` database with migrations applied.
///
/// Intended for test use only.
///
/// # Panics
///
/// Panics if database connection or migration fails.
pub async fn test_db() -> sea_orm::DatabaseConnection {
    init_db("sqlite::memory:")
        .await
        .expect("failed to initialize test database")
}
