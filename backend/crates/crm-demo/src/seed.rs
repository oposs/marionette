use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter,
};
use sea_orm::ActiveValue::NotSet;

use crate::entities::{company, contact, contact_tag, interaction, note, tag, user};

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
    // Phase 13: the infinite-scroll E2E test requires > page_size contacts
    // (120 total — 3 named + 117 generated). Seeding is idempotent AND
    // top-up-aware: if the table already has the named contacts but fewer
    // than 120 total (e.g. a stale DB from a pre-Phase-13 run), we top up
    // the generated contacts without re-inserting the named ones.
    let count = contact::Entity::find().count(db).await?;
    if count >= 120 {
        return Ok(());
    }
    let needs_named_seed = count == 0;

    // Look up company IDs by name
    let acme = company::Entity::find()
        .filter(company::Column::CompanyName.eq("Acme Corp"))
        .one(db)
        .await?;
    let globex = company::Entity::find()
        .filter(company::Column::CompanyName.eq("Globex Inc"))
        .one(db)
        .await?;

    // Named contacts kept stable so seed_tags/seed_notes/seed_interactions
    // lookups by contact_name continue to work.
    #[allow(clippy::type_complexity)]
    let named_contacts: Vec<(&str, &str, Option<&str>, Option<&str>, Option<i32>)> = vec![
        ("Alice Johnson", "alice@acme.example.com", Some("+1-555-0101"), Some("CEO"), acme.as_ref().map(|c| c.company_id)),
        ("Bob Smith", "bob@globex.example.com", Some("+1-555-0102"), Some("CTO"), globex.as_ref().map(|c| c.company_id)),
        ("Carol Williams", "carol@example.com", None, Some("Freelancer"), None),
    ];

    if needs_named_seed {
        for (name, email, phone, title, company_id) in named_contacts {
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
    }

    // Bulk-seed additional contacts so Phase 13's infinite-scroll E2E has
    // > 2 × page_size (50) rows. Deterministic naming for test assertions.
    // Only insert generated contacts that aren't already present — this
    // makes the seed idempotent for partial DBs (e.g., a stale DB from a
    // pre-Phase-13 run that only has the 3 named contacts).
    let titles = ["Engineer", "Manager", "Analyst", "Designer", "Director"];
    let company_ids: Vec<Option<i32>> = vec![
        acme.as_ref().map(|c| c.company_id),
        globex.as_ref().map(|c| c.company_id),
        None,
    ];
    let mut inserted: u32 = 0;
    for i in 0..117 {
        let name = format!("Seed Contact {i:03}");
        // Skip if this generated contact already exists (idempotent top-up).
        let existing = contact::Entity::find()
            .filter(contact::Column::ContactName.eq(name.clone()))
            .one(db)
            .await?;
        if existing.is_some() {
            continue;
        }
        let email = format!("seed{i:03}@example.com");
        let title = titles[i as usize % titles.len()];
        let company_id = company_ids[i as usize % company_ids.len()];
        let phone = format!("+1-555-{:04}", 1000 + i);
        let model = contact::ActiveModel {
            contact_id: NotSet,
            contact_name: Set(name),
            contact_email: Set(email),
            contact_phone: Set(Some(phone)),
            contact_title: Set(Some(title.into())),
            contact_company: Set(company_id),
            contact_created_at: NotSet,
            contact_updated_at: NotSet,
        };
        model.insert(db).await?;
        inserted += 1;
    }

    tracing::info!(
        "Seeded {} generated demo contacts (top-up to 120, {} existed already)",
        inserted,
        117 - inserted,
    );
    Ok(())
}

/// Seed demo tags and contact-tag links if the tag table is empty.
///
/// # Errors
///
/// Returns `DbErr` if the database query or insert fails.
pub async fn seed_tags(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = tag::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    let tag_names = vec!["VIP", "Lead", "Partner", "Inactive", "Newsletter"];
    for name in &tag_names {
        let model = tag::ActiveModel {
            tag_id: NotSet,
            tag_name: Set((*name).into()),
        };
        model.insert(db).await?;
    }

    // Look up contacts and tags by name for linking
    let alice = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Alice Johnson"))
        .one(db)
        .await?;
    let bob = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Bob Smith"))
        .one(db)
        .await?;
    let carol = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Carol Williams"))
        .one(db)
        .await?;

    let vip = tag::Entity::find()
        .filter(tag::Column::TagName.eq("VIP"))
        .one(db)
        .await?;
    let lead = tag::Entity::find()
        .filter(tag::Column::TagName.eq("Lead"))
        .one(db)
        .await?;
    let partner = tag::Entity::find()
        .filter(tag::Column::TagName.eq("Partner"))
        .one(db)
        .await?;
    let inactive = tag::Entity::find()
        .filter(tag::Column::TagName.eq("Inactive"))
        .one(db)
        .await?;
    let newsletter = tag::Entity::find()
        .filter(tag::Column::TagName.eq("Newsletter"))
        .one(db)
        .await?;

    // Alice: VIP + Partner
    // Bob: Lead + Newsletter
    // Carol: Inactive
    let links: Vec<(Option<&contact::Model>, Option<&tag::Model>)> = vec![
        (alice.as_ref(), vip.as_ref()),
        (alice.as_ref(), partner.as_ref()),
        (bob.as_ref(), lead.as_ref()),
        (bob.as_ref(), newsletter.as_ref()),
        (carol.as_ref(), inactive.as_ref()),
    ];

    for (c, t) in links {
        if let (Some(c), Some(t)) = (c, t) {
            let model = contact_tag::ActiveModel {
                contact_tag_contact: Set(c.contact_id),
                contact_tag_tag: Set(t.tag_id),
            };
            model.insert(db).await?;
        }
    }

    tracing::info!("Seeded 5 tags and 5 contact-tag links");
    Ok(())
}

/// Seed demo notes if the note table is empty.
///
/// # Errors
///
/// Returns `DbErr` if the database query or insert fails.
pub async fn seed_notes(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = note::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    let alice = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Alice Johnson"))
        .one(db)
        .await?;
    let bob = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Bob Smith"))
        .one(db)
        .await?;
    let acme = company::Entity::find()
        .filter(company::Column::CompanyName.eq("Acme Corp"))
        .one(db)
        .await?;

    let notes_data: Vec<(Option<i32>, Option<i32>, &str)> = vec![
        (alice.as_ref().map(|c| c.contact_id), None, "Met at the industry conference. Very interested in our enterprise plan."),
        (bob.as_ref().map(|c| c.contact_id), None, "Needs follow-up on the technical proposal by end of week."),
        (None, acme.as_ref().map(|c| c.company_id), "Key account — renewing contract Q2 2026."),
    ];

    for (contact_id, company_id, text) in notes_data {
        let model = note::ActiveModel {
            note_id: NotSet,
            note_contact: Set(contact_id),
            note_company: Set(company_id),
            note_text: Set(text.into()),
            note_user: Set(1),
            note_created_at: NotSet,
        };
        model.insert(db).await?;
    }

    tracing::info!("Seeded 3 demo notes");
    Ok(())
}

/// Seed demo interactions if the interaction table is empty.
///
/// # Errors
///
/// Returns `DbErr` if the database query or insert fails.
pub async fn seed_interactions(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = interaction::Entity::find().count(db).await?;
    if count > 0 {
        return Ok(());
    }

    let alice = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Alice Johnson"))
        .one(db)
        .await?;
    let bob = contact::Entity::find()
        .filter(contact::Column::ContactName.eq("Bob Smith"))
        .one(db)
        .await?;

    let interactions_data: Vec<(Option<&contact::Model>, &str, &str, Option<&str>, &str)> = vec![
        (alice.as_ref(), "call", "Initial outreach", Some("Discussed product features and pricing."), "2026-03-20 10:00:00"),
        (bob.as_ref(), "email", "Follow-up proposal", Some("Sent detailed proposal with pricing tiers."), "2026-03-21 14:00:00"),
        (alice.as_ref(), "meeting", "Contract review", Some("Reviewed contract terms with legal team."), "2026-03-22 09:00:00"),
    ];

    for (contact_ref, itype, subject, notes, date) in interactions_data {
        if let Some(c) = contact_ref {
            let model = interaction::ActiveModel {
                interaction_id: NotSet,
                interaction_contact: Set(c.contact_id),
                interaction_type: Set(itype.into()),
                interaction_subject: Set(subject.into()),
                interaction_notes: Set(notes.map(String::from)),
                interaction_user: Set(1),
                interaction_date: Set(date.into()),
                interaction_created_at: NotSet,
            };
            model.insert(db).await?;
        }
    }

    tracing::info!("Seeded 3 demo interactions");
    Ok(())
}
