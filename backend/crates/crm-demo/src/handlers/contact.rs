use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use marionette::builders::standard::{
    Button, Container, DataTable, Form, Heading, Select, SelectOption, TableColumn, Text, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

use crate::entities::{company, contact, note, user};

/// Format current UTC time as SQLite datetime string.
fn now_sqlite() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Payload for identifying a contact by ID.
#[derive(Deserialize)]
struct ContactIdPayload {
    contact_id: i32,
}

/// Payload for saving a contact (create or update).
#[derive(Deserialize)]
struct ContactSavePayload {
    id: Option<i32>,
    name: String,
    email: String,
    phone: Option<String>,
    title: Option<String>,
    company: Option<String>,
}

/// Shared helper: build a rendered contact list from the database.
async fn render_contact_list(ctx: &HandlerContext) -> ActionResult {
    let db = Db::from_context(ctx)?;
    let contacts = contact::Entity::find()
        .find_also_related(company::Entity)
        .order_by_asc(contact::Column::ContactName)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let heading = Heading::new("Contact Management")
        .id("contact-heading")
        .build();

    let new_button = Button::new("New Contact")
        .id("btn-new-contact")
        .action(ComponentAction::click("contact_new"))
        .build();

    let table = DataTable::new(vec![
        TableColumn {
            key: "name".into(),
            label: "Name".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "email".into(),
            label: "Email".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "phone".into(),
            label: "Phone".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "company".into(),
            label: "Company".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "created".into(),
            label: "Created".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "actions".into(),
            label: "Actions".into(),
            sortable: None,
        },
    ])
    .id("contact-table")
    .bind("/contacts")
    .build();

    let container_nodes = Container::new()
        .id("contact-list-root")
        .children(vec![heading, new_button, table])
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    // Build row data with joined company name and per-row edit/delete actions
    let rows: Vec<serde_json::Value> = contacts
        .iter()
        .map(|(c, co)| {
            serde_json::json!({
                "id": c.contact_id,
                "name": c.contact_name,
                "email": c.contact_email,
                "phone": c.contact_phone.as_deref().unwrap_or("-"),
                "company": co.as_ref().map(|comp| comp.company_name.as_str()).unwrap_or("-"),
                "created": c.contact_created_at,
                "actions": [
                    { "label": "Edit", "action": { "type": "click", "name": "contact_edit", "payload": { "contact_id": c.contact_id } } },
                    { "label": "Delete", "action": { "type": "click", "name": "contact_delete", "payload": { "contact_id": c.contact_id } } }
                ]
            })
        })
        .collect();

    let data = serde_json::json!({ "contacts": rows });

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "contact-list-root".into(),
        nodes,
        data,
    })])
}

/// Handle the `contact_list` action: render a DataTable of all contacts.
pub async fn handle_contact_list(ctx: HandlerContext) -> ActionResult {
    render_contact_list(&ctx).await
}

/// Handle the `contact_new` / `contact_edit` action: render a create/edit form.
pub async fn handle_contact_form(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    // Try to extract contact_id from payload (edit mode) -- if missing, it's create mode
    let contact_id = Payload::<ContactIdPayload>::from_context(&ctx)
        .ok()
        .map(|p| p.0.contact_id);

    let (form_data, form_title) = if let Some(cid) = contact_id {
        let found = contact::Entity::find_by_id(cid)
            .one(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?
            .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;
        (
            serde_json::json!({
                "contactForm": {
                    "id": found.contact_id,
                    "name": found.contact_name,
                    "email": found.contact_email,
                    "phone": found.contact_phone.as_deref().unwrap_or(""),
                    "title": found.contact_title.as_deref().unwrap_or(""),
                    "company": found.contact_company.map(|id| id.to_string()).unwrap_or_default()
                }
            }),
            "Edit Contact",
        )
    } else {
        (
            serde_json::json!({
                "contactForm": {
                    "id": null,
                    "name": "",
                    "email": "",
                    "phone": "",
                    "title": "",
                    "company": ""
                }
            }),
            "New Contact",
        )
    };

    // Build company select dropdown options
    let companies = company::Entity::find()
        .order_by_asc(company::Column::CompanyName)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let mut options = vec![SelectOption {
        value: String::new(),
        label: "No Company".into(),
    }];
    for co in &companies {
        options.push(SelectOption {
            value: co.company_id.to_string(),
            label: co.company_name.clone(),
        });
    }

    let heading = Heading::new(form_title)
        .id("contact-form-heading")
        .build();

    let name_input = TextInput::new("Name")
        .id("contact-form-name")
        .bind("/contactForm/name")
        .build();

    let email_input = TextInput::new("Email")
        .id("contact-form-email")
        .bind("/contactForm/email")
        .build();

    let phone_input = TextInput::new("Phone")
        .id("contact-form-phone")
        .bind("/contactForm/phone")
        .build();

    let title_input = TextInput::new("Title")
        .id("contact-form-title")
        .bind("/contactForm/title")
        .build();

    let company_select = Select::new("Company", options)
        .id("contact-form-company")
        .bind("/contactForm/company")
        .build();

    let save_button = Button::new("Save")
        .id("contact-form-save")
        .action(ComponentAction::submit("contact_save"))
        .build();

    let cancel_button = Button::new("Cancel")
        .id("contact-form-cancel")
        .action(ComponentAction::click("contact_list"))
        .build();

    let form = Form::new()
        .id("contact-form")
        .children(vec![
            name_input,
            email_input,
            phone_input,
            title_input,
            company_select,
            save_button,
            cancel_button,
        ])
        .build_with_children();

    let mut all_nodes = Vec::new();
    all_nodes.push(heading);
    all_nodes.extend(form);

    let mut merged_data = form_data;

    // In edit mode, append notes section below the form
    if let Some(cid) = contact_id {
        let notes = note::Entity::find()
            .filter(note::Column::NoteContact.eq(cid))
            .order_by_desc(note::Column::NoteCreatedAt)
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        // Notes heading
        let notes_heading = Heading::new("Notes")
            .id("notes-heading")
            .build();
        all_nodes.push(notes_heading);

        // Add-note form: text input + submit button wrapped in a Form
        let note_input = TextInput::new("Add a note...")
            .id("note-input")
            .bind("/noteForm/text")
            .build();

        let note_submit = Button::new("Add Note")
            .id("note-submit")
            .action(ComponentAction::submit("note_save"))
            .build();

        let note_form = Form::new()
            .id("note-form")
            .children(vec![note_input, note_submit])
            .build_with_children();
        all_nodes.extend(note_form);

        // Render existing notes as Text components
        for n in &notes {
            // Look up author name (N+1 acceptable at demo scale)
            let author_name = user::Entity::find_by_id(n.note_user)
                .one(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .map(|u| u.user_name)
                .unwrap_or_else(|| "Unknown".into());

            let note_text = format!(
                "[{}] {}: {}",
                n.note_created_at, author_name, n.note_text
            );
            let note_component = Text::new(&note_text)
                .id(&format!("note-{}", n.note_id))
                .build();
            all_nodes.push(note_component);
        }

        // Merge noteForm data with contact_id for the note_save handler
        if let Some(obj) = merged_data.as_object_mut() {
            obj.insert(
                "noteForm".into(),
                serde_json::json!({ "text": "", "contact_id": cid }),
            );
        }
    }

    let container_nodes = Container::new()
        .id("contact-form-root")
        .children(all_nodes)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "contact-form-root".into(),
        nodes,
        data: merged_data,
    })])
}

/// Handle the `contact_save` action: create or update a contact.
pub async fn handle_contact_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::{NotSet, Set};

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<ContactSavePayload>::from_context(&ctx)?;
    let data = payload.0;

    // Validate required fields
    if data.name.trim().is_empty() {
        return Err(ActionError::BadPayload(
            "Contact name is required".into(),
        ));
    }
    if data.email.trim().is_empty() {
        return Err(ActionError::BadPayload("Email is required".into()));
    }
    if !data.email.contains('@') {
        return Err(ActionError::BadPayload(
            "Invalid email format".into(),
        ));
    }

    // Parse optional company FK
    let company_id: Option<i32> = data
        .company
        .as_deref()
        .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });

    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);

    match data.id {
        None => {
            // Create mode
            let contact_json = serde_json::json!({
                "name": &data.name,
                "email": &data.email,
                "phone": &data.phone,
                "title": &data.title,
                "company": company_id,
            });

            let new_contact = contact::ActiveModel {
                contact_id: NotSet,
                contact_name: Set(data.name),
                contact_email: Set(data.email),
                contact_phone: Set(data.phone),
                contact_title: Set(data.title),
                contact_company: Set(company_id),
                contact_created_at: NotSet,
                contact_updated_at: NotSet,
            };
            let inserted = new_contact
                .insert(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            crate::audit::record_audit(
                &*db.0,
                caller_id,
                "contact",
                inserted.contact_id,
                "create",
                contact_json,
            )
            .await?;
        }
        Some(cid) => {
            // Edit mode: fetch existing contact
            let found = contact::Entity::find_by_id(cid)
                .one(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;

            let old_json = serde_json::json!({
                "name": found.contact_name,
                "email": found.contact_email,
                "phone": found.contact_phone,
                "title": found.contact_title,
                "company": found.contact_company,
            });

            let mut active: contact::ActiveModel = found.into();
            active.contact_name = Set(data.name.clone());
            active.contact_email = Set(data.email.clone());
            active.contact_phone = Set(data.phone.clone());
            active.contact_title = Set(data.title.clone());
            active.contact_company = Set(company_id);
            active.contact_updated_at = Set(now_sqlite());

            active
                .update(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            let new_json = serde_json::json!({
                "name": &data.name,
                "email": &data.email,
                "phone": &data.phone,
                "title": &data.title,
                "company": company_id,
            });
            let changes = crate::audit::compute_changes(&old_json, &new_json);
            crate::audit::record_audit(&*db.0, caller_id, "contact", cid, "update", changes)
                .await?;
        }
    }

    // After save, re-render the contact list
    render_contact_list(&ctx).await
}

/// Handle the `contact_delete` action: delete a contact by ID and re-render the list.
pub async fn handle_contact_delete(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let payload = Payload::<ContactIdPayload>::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;

    // Find contact
    let found = contact::Entity::find_by_id(payload.0.contact_id)
        .one(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .ok_or_else(|| ActionError::Internal("Contact not found".into()))?;

    let deleted_json = serde_json::json!({
        "name": found.contact_name,
        "email": found.contact_email,
        "phone": found.contact_phone,
        "title": found.contact_title,
        "company": found.contact_company,
    });

    // Delete
    let deleted_id = found.contact_id;
    found
        .delete(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Record audit entry
    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    crate::audit::record_audit(
        &*db.0,
        caller_id,
        "contact",
        deleted_id,
        "delete",
        deleted_json,
    )
    .await?;

    // Re-render the list
    render_contact_list(&ctx).await
}
