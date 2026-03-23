use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter,
};
use sea_orm::ActiveValue::NotSet;

use crate::entities::{company, contact, user};

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

/// Seed demo companies if the company table is empty.
///
/// # Errors
///
/// Returns `DbErr` if the database query or insert fails.
pub async fn seed_companies(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = company::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    let companies = vec![
        ("Acme Corp", Some("https://acme.example.com"), Some("123 Main St")),
        ("Globex Inc", Some("https://globex.example.com"), Some("456 Oak Ave")),
        ("Initech", Some("https://initech.example.com"), Some("789 Tech Blvd")),
    ];

    for (name, website, address) in companies {
        let model = company::ActiveModel {
            company_id: NotSet,
            company_name: Set(name.into()),
            company_website: Set(website.map(String::from)),
            company_address: Set(address.map(String::from)),
            company_created_at: NotSet,
            company_updated_at: NotSet,
        };
        model.insert(db).await?;
    }

    tracing::info!("Seeded 3 demo companies");
    Ok(())
}

/// Seed demo contacts if the contact table is empty.
///
/// Companies must be seeded first (FK dependency).
///
/// # Errors
///
/// Returns `DbErr` if the database query or insert fails.
pub async fn seed_contacts(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = contact::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    // Look up company IDs by name
    let acme = company::Entity::find()
        .filter(company::Column::CompanyName.eq("Acme Corp"))
        .one(db)
        .await?;
    let globex = company::Entity::find()
        .filter(company::Column::CompanyName.eq("Globex Inc"))
        .one(db)
        .await?;

    #[allow(clippy::type_complexity)]
    let contacts: Vec<(&str, &str, Option<&str>, Option<&str>, Option<i32>)> = vec![
        ("Alice Johnson", "alice@acme.example.com", Some("+1-555-0101"), Some("CEO"), acme.map(|c| c.company_id)),
        ("Bob Smith", "bob@globex.example.com", Some("+1-555-0102"), Some("CTO"), globex.map(|c| c.company_id)),
        ("Carol Williams", "carol@example.com", None, Some("Freelancer"), None),
    ];

    for (name, email, phone, title, company_id) in contacts {
        let model = contact::ActiveModel {
            contact_id: NotSet,
            contact_name: Set(name.into()),
            contact_email: Set(email.into()),
            contact_phone: Set(phone.map(String::from)),
            contact_title: Set(title.map(String::from)),
            contact_company: Set(company_id),
            contact_created_at: NotSet,
            contact_updated_at: NotSet,
        };
        model.insert(db).await?;
    }

    tracing::info!("Seeded 3 demo contacts");
    Ok(())
}
