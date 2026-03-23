use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
};

use crate::entities::user;

/// Seed a default admin account if no users exist.
///
/// Credentials are read from environment variables:
/// - `MARIONETTE_ADMIN_NAME` (default: "Admin")
/// - `MARIONETTE_ADMIN_EMAIL` (default: "admin@localhost")
/// - `MARIONETTE_ADMIN_PASSWORD` (default: "admin")
///
/// # Errors
///
/// Returns `DbErr` if the database query or insert fails.
pub async fn seed_admin(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = user::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    let name = std::env::var("MARIONETTE_ADMIN_NAME").unwrap_or_else(|_| "Admin".into());
    let email =
        std::env::var("MARIONETTE_ADMIN_EMAIL").unwrap_or_else(|_| "admin@localhost".into());
    let password =
        std::env::var("MARIONETTE_ADMIN_PASSWORD").unwrap_or_else(|_| "admin".into());

    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password, 10))
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?
        .map_err(|e| DbErr::Custom(e.to_string()))?;

    let admin = user::ActiveModel {
        user_name: Set(name.clone()),
        user_email: Set(email.clone()),
        user_password: Set(hash),
        user_role: Set("admin".into()),
        ..Default::default()
    };
    admin.insert(db).await?;

    tracing::info!(name = %name, email = %email, "Seeded default admin account");
    Ok(())
}
