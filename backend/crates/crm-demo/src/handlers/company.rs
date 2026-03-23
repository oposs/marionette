use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use marionette::builders::standard::{
    Button, Container, DataTable, Form, Heading, TableColumn, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

use crate::entities::{company, contact};

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

/// Payload for identifying a company by ID.
#[derive(Deserialize)]
struct CompanyIdPayload {
    company_id: i32,
}

/// Payload for saving a company (create or update).
#[derive(Deserialize)]
struct CompanySavePayload {
    id: Option<i32>,
    name: String,
    website: Option<String>,
    address: Option<String>,
}

/// Shared helper: build a rendered company list from the database.
async fn render_company_list(ctx: &HandlerContext) -> ActionResult {
    let db = Db::from_context(ctx)?;
    let companies = company::Entity::find()
        .order_by_asc(company::Column::CompanyName)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let heading = Heading::new("Company Management")
        .id("company-heading")
        .build();

    let new_button = Button::new("New Company")
        .id("btn-new-company")
        .action(ComponentAction::click("company_new"))
        .build();

    let table = DataTable::new(vec![
        TableColumn {
            key: "name".into(),
            label: "Name".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "website".into(),
            label: "Website".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "contactCount".into(),
            label: "Contacts".into(),
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
    .id("company-table")
    .bind("/companies")
    .build();

    let container_nodes = Container::new()
        .id("company-list-root")
        .children(vec![heading, new_button, table])
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    // Build row data with per-row contact count and edit/delete actions
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for c in &companies {
        let contact_count = contact::Entity::find()
            .filter(contact::Column::ContactCompany.eq(c.company_id))
            .count(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        rows.push(serde_json::json!({
            "id": c.company_id,
            "name": c.company_name,
            "website": c.company_website.as_deref().unwrap_or("-"),
            "contactCount": contact_count,
            "created": c.company_created_at,
            "actions": [
                { "label": "Edit", "action": { "type": "click", "name": "company_edit", "payload": { "company_id": c.company_id } } },
                { "label": "Delete", "action": { "type": "click", "name": "company_delete", "payload": { "company_id": c.company_id } } }
            ]
        }));
    }

    let data = serde_json::json!({ "companies": rows });

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "company-list-root".into(),
        nodes,
        data,
    })])
}

/// Handle the `company_list` action: render a DataTable of all companies.
pub async fn handle_company_list(ctx: HandlerContext) -> ActionResult {
    render_company_list(&ctx).await
}

/// Handle the `company_new` / `company_edit` action: render a create/edit form.
pub async fn handle_company_form(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    // Try to extract company_id from payload (edit mode) -- if missing, it's create mode
    let company_id = Payload::<CompanyIdPayload>::from_context(&ctx)
        .ok()
        .map(|p| p.0.company_id);

    let (form_data, form_title) = if let Some(cid) = company_id {
        let found = company::Entity::find_by_id(cid)
            .one(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?
            .ok_or_else(|| ActionError::Internal("Company not found".into()))?;
        (
            serde_json::json!({
                "companyForm": {
                    "id": found.company_id,
                    "name": found.company_name,
                    "website": found.company_website.as_deref().unwrap_or(""),
                    "address": found.company_address.as_deref().unwrap_or("")
                }
            }),
            "Edit Company",
        )
    } else {
        (
            serde_json::json!({
                "companyForm": {
                    "id": null,
                    "name": "",
                    "website": "",
                    "address": ""
                }
            }),
            "New Company",
        )
    };

    let heading = Heading::new(form_title)
        .id("company-form-heading")
        .build();

    let name_input = TextInput::new("Name")
        .id("company-form-name")
        .bind("/companyForm/name")
        .build();

    let website_input = TextInput::new("Website")
        .id("company-form-website")
        .bind("/companyForm/website")
        .build();

    let address_input = TextInput::new("Address")
        .id("company-form-address")
        .bind("/companyForm/address")
        .build();

    let save_button = Button::new("Save")
        .id("company-form-save")
        .action(ComponentAction::submit("company_save"))
        .build();

    let cancel_button = Button::new("Cancel")
        .id("company-form-cancel")
        .action(ComponentAction::click("company_list"))
        .build();

    let form = Form::new()
        .id("company-form")
        .children(vec![
            name_input,
            website_input,
            address_input,
            save_button,
            cancel_button,
        ])
        .build_with_children();

    let mut all_nodes = Vec::new();
    all_nodes.push(heading);
    all_nodes.extend(form);

    let mut merged_data = form_data;

    // In edit mode, add linked contacts sub-table
    if let Some(cid) = company_id {
        let linked_contacts = contact::Entity::find()
            .filter(contact::Column::ContactCompany.eq(cid))
            .order_by_asc(contact::Column::ContactName)
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        if !linked_contacts.is_empty() {
            let contacts_heading = Heading::new("Linked Contacts")
                .id("company-contacts-heading")
                .build();

            let contacts_table = DataTable::new(vec![
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
                    sortable: None,
                },
                TableColumn {
                    key: "actions".into(),
                    label: "Actions".into(),
                    sortable: None,
                },
            ])
            .id("company-contacts-table")
            .bind("/linkedContacts")
            .build();

            all_nodes.push(contacts_heading);
            all_nodes.push(contacts_table);

            let contact_rows: Vec<serde_json::Value> = linked_contacts
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.contact_id,
                        "name": c.contact_name,
                        "email": c.contact_email,
                        "phone": c.contact_phone.as_deref().unwrap_or("-"),
                        "actions": [
                            { "label": "Edit", "action": { "type": "click", "name": "contact_edit", "payload": { "contact_id": c.contact_id } } }
                        ]
                    })
                })
                .collect();

            if let Some(obj) = merged_data.as_object_mut() {
                obj.insert(
                    "linkedContacts".into(),
                    serde_json::json!(contact_rows),
                );
            }
        }
    }

    let container_nodes = Container::new()
        .id("company-form-root")
        .children(all_nodes)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "company-form-root".into(),
        nodes,
        data: merged_data,
    })])
}

/// Handle the `company_save` action: create or update a company.
pub async fn handle_company_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::{NotSet, Set};

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<CompanySavePayload>::from_context(&ctx)?;
    let data = payload.0;

    // Validate required fields
    if data.name.trim().is_empty() {
        return Err(ActionError::BadPayload(
            "Company name is required".into(),
        ));
    }

    let caller_id: i32 = session
        .user_id
        .as_ref()
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);

    match data.id {
        None => {
            // Create mode
            let company_json = serde_json::json!({
                "name": &data.name,
                "website": &data.website,
                "address": &data.address,
            });

            let new_company = company::ActiveModel {
                company_id: NotSet,
                company_name: Set(data.name),
                company_website: Set(data.website),
                company_address: Set(data.address),
                company_created_at: NotSet,
                company_updated_at: NotSet,
            };
            let inserted = new_company
                .insert(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            crate::audit::record_audit(
                &*db.0,
                caller_id,
                "company",
                inserted.company_id,
                "create",
                company_json,
            )
            .await?;
        }
        Some(cid) => {
            // Edit mode: fetch existing company
            let found = company::Entity::find_by_id(cid)
                .one(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .ok_or_else(|| ActionError::Internal("Company not found".into()))?;

            let old_json = serde_json::json!({
                "name": found.company_name,
                "website": found.company_website,
                "address": found.company_address,
            });

            let mut active: company::ActiveModel = found.into();
            active.company_name = Set(data.name.clone());
            active.company_website = Set(data.website.clone());
            active.company_address = Set(data.address.clone());
            active.company_updated_at = Set(now_sqlite());

            active
                .update(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            let new_json = serde_json::json!({
                "name": &data.name,
                "website": &data.website,
                "address": &data.address,
            });
            let changes = crate::audit::compute_changes(&old_json, &new_json);
            crate::audit::record_audit(&*db.0, caller_id, "company", cid, "update", changes)
                .await?;
        }
    }

    // After save, re-render the company list
    render_company_list(&ctx).await
}

/// Handle the `company_delete` action: delete a company by ID and re-render the list.
pub async fn handle_company_delete(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let payload = Payload::<CompanyIdPayload>::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;

    // Find company
    let found = company::Entity::find_by_id(payload.0.company_id)
        .one(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .ok_or_else(|| ActionError::Internal("Company not found".into()))?;

    let deleted_json = serde_json::json!({
        "name": found.company_name,
        "website": found.company_website,
        "address": found.company_address,
    });

    // Delete (ON DELETE SET NULL handles contact FK)
    let deleted_id = found.company_id;
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
        "company",
        deleted_id,
        "delete",
        deleted_json,
    )
    .await?;

    // Re-render the list
    render_company_list(&ctx).await
}
