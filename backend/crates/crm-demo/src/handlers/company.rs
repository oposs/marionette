use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use marionette::builders::standard::{
    form_shell, Button, ColumnKind, Container, DataTable, FieldSet, Form, Heading, TableColumn,
    Text, Textarea, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette::validation::validation_error_patch;
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

/// Payload for identifying a company by ID.
#[derive(Deserialize)]
struct CompanyIdPayload {
    company_id: i32,
}

/// Inner form data for a company (create or update).
#[derive(Deserialize)]
struct CompanyFormData {
    id: Option<i32>,
    name: String,
    website: Option<String>,
    address: Option<String>,
}

/// Payload wrapper: the frontend sends all surface data, with form fields
/// nested under the form's bind prefix (e.g. `companyForm`).
#[derive(Deserialize)]
struct CompanySavePayload {
    #[serde(rename = "companyForm")]
    company_form: CompanyFormData,
}

/// Shared helper: build a rendered company list from the database.
async fn render_company_list(ctx: &HandlerContext) -> ActionResult {
    let db = Db::from_context(ctx)?;
    let companies = company::Entity::find()
        .order_by_asc(company::Column::CompanyName)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // D-H2: total_rows reflects the full (unfiltered) collection.
    let company_count: u64 = company::Entity::find()
        .count(&*db.0)
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
        TableColumn::new("name", "Name").sortable(),
        TableColumn::new("website", "Website").sortable(),
        TableColumn::new("contactCount", "Contacts")
            .sortable()
            .kind(ColumnKind::Number),
        TableColumn::new("created", "Created")
            .sortable()
            .kind(ColumnKind::Date),
        TableColumn::new("actions", "").kind(ColumnKind::Actions),
    ])
    .total_rows(company_count)
    .source("company_list")
    .row_id_key("id")
    .page_size(50u32)
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

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root: "company-list-root".into(),
            nodes,
            data,
        }),
        nav_active_patch("companies"),
    ])
}

/// Handle the `company_list` action: render a DataTable of all companies.
pub async fn handle_company_list(ctx: HandlerContext) -> ActionResult {
    render_company_list(&ctx).await
}

/// Handle the `company_new` / `company_edit` action: render a create/edit form.
///
/// PHASE 15 MIGRATION (D-A1 + D-B1 + D-B2 + D-D1 + D-E3 + D-I): built via
/// `form_shell()` + `FieldSet` per the Phase 14 canonical composition
/// (15-PATTERNS.md SP-1) and the 15-UI-SPEC §Per-Screen §1 contract:
/// - `FieldSet("Company details")` wraps `[name, website, address]`.
/// - `name` carries the locked description string from 15-UI-SPEC
///   §Description Copy Contract ("Will appear on invoices and contact
///   details.").
/// - Action row `flex gap-2 justify-end` wraps `[Cancel, Save company]`.
/// - Inline note-add form uses UI-SPEC §6 layout (`flex flex-col
///   gap-2 items-end` wrapper, Textarea with `rows(3)`, "+ Add note").
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

    // -- Heading + back button --

    let heading = Heading::new(form_title)
        .id("company-form-heading")
        .build();

    let back_button = Button::new("← Back")
        .id("company-form-back")
        .variant("outline")
        .action(ComponentAction::click("company_list"))
        .build();

    // -- FieldSet 1: Company details --

    let name_input = TextInput::new("Name")
        .id("company-form-name")
        .bind("/companyForm/name")
        .required(true)
        // 15-UI-SPEC §Description Copy Contract (D-E3)
        .description("Will appear on invoices and contact details.")
        .build();

    let website_input = TextInput::new("Website")
        .id("company-form-website")
        .bind("/companyForm/website")
        .input_type("url")
        .placeholder("https://example.com")
        .build();

    let address_input = TextInput::new("Address")
        .id("company-form-address")
        .bind("/companyForm/address")
        .build();

    let (details_set, details_descendants) = FieldSet::new()
        .id("company-details-set")
        .legend("Company details")
        .children(vec![name_input, website_input, address_input])
        .build_tree();

    // -- Action row (D-D1 Option A: flex gap-2 justify-end) --

    let cancel_button = Button::new("Cancel")
        .id("company-form-cancel")
        .variant("outline")
        .action(ComponentAction::click("company_list"))
        .build();

    let save_button = Button::new("Save company")
        .id("company-form-save")
        .variant("default")
        .action(ComponentAction::submit("company_save"))
        .build();

    let (action_row, action_row_descendants) = Container::new()
        .id("company-form-actions")
        .class("flex gap-2 justify-end")
        .children(vec![cancel_button, save_button])
        .build_tree();

    // -- Compose the form --

    let (form_child, form_descendants) = Form::new()
        .id("company-form")
        .children(vec![details_set, action_row])
        .build_tree();

    // Collect all descendants in a single flat vec.
    let mut all_descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
    all_descendants.extend(details_descendants);
    all_descendants.extend(action_row_descendants);
    all_descendants.extend(form_descendants);

    // -- Post-form content (edit mode only): linked contacts table + notes --
    //
    // These are appended to the outer Container AFTER the form. We build an
    // auxiliary node vec and fold it into the form_shell output at the end.
    let mut post_form_nodes: Vec<(String, marionette_protocol::Component)> = Vec::new();
    let mut post_form_descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
    let mut merged_data = form_data;

    if let Some(cid) = company_id {
        // --- Linked contacts sub-table ---
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
                    ..Default::default()
                },
                TableColumn {
                    key: "email".into(),
                    label: "Email".into(),
                    sortable: Some(true),
                    ..Default::default()
                },
                TableColumn {
                    key: "phone".into(),
                    label: "Phone".into(),
                    sortable: None,
                    ..Default::default()
                },
                TableColumn {
                    key: "actions".into(),
                    label: "Actions".into(),
                    sortable: None,
                    ..Default::default()
                },
            ])
            .id("company-contacts-table")
            .bind("/linkedContacts")
            .build();

            post_form_nodes.push(contacts_heading);
            post_form_nodes.push(contacts_table);

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

        // --- Notes section ---
        let notes = note::Entity::find()
            .filter(note::Column::NoteCompany.eq(cid))
            .order_by_desc(note::Column::NoteCreatedAt)
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;

        let notes_heading = Heading::new("Notes").id("notes-heading").build();
        post_form_nodes.push(notes_heading);

        // Inline note-add form — 15-UI-SPEC §6 locked layout:
        // `flex flex-col gap-2 items-end` wrapper, Textarea with rows(3),
        // primary Button labeled "+ Add note".
        let note_input = Textarea::new("Add a note...")
            .id("note-input")
            .bind("/noteForm/text")
            .rows(3u32)
            .full_width(true)
            .build();

        let note_submit = Button::new("+ Add note")
            .id("note-submit")
            .variant("default")
            .action(ComponentAction::submit("note_save"))
            .build();

        let (note_row, note_row_descendants) = Container::new()
            .id("company-note-form-row")
            .class("flex flex-col gap-2 items-end")
            .children(vec![note_input, note_submit])
            .build_tree();

        let (note_form_child, note_form_descendants) = Form::new()
            .id("note-form")
            .children(vec![note_row])
            .build_tree();
        post_form_nodes.push(note_form_child);
        post_form_descendants.extend(note_row_descendants);
        post_form_descendants.extend(note_form_descendants);

        // Render existing notes as plain text rows.
        for n in &notes {
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
            post_form_nodes.push(note_component);
        }

        if let Some(obj) = merged_data.as_object_mut() {
            obj.insert(
                "noteForm".into(),
                serde_json::json!({ "text": "", "company_id": cid }),
            );
        }
    }

    // -- Outer shell via form_shell (D-B1) --
    //
    // form_shell wraps [heading, back_button, form_child] in the outer
    // Container. Post-form content (linked contacts, notes, note-form) is
    // appended to that Container's children below.
    let (root, mut nodes) = form_shell(
        "company-form-root",
        heading,
        back_button,
        form_child,
        all_descendants,
    );

    // Append post-form content to the outer Container's children list.
    if !post_form_nodes.is_empty() {
        if let Some(container) = nodes.get_mut(&root) {
            let existing: Vec<String> = container
                .children
                .clone()
                .unwrap_or_default();
            let appended_ids: Vec<String> = post_form_nodes
                .iter()
                .map(|(id, _)| id.clone())
                .collect();
            let mut merged_children = existing;
            merged_children.extend(appended_ids);
            container.children = Some(merged_children);
        }
        for (id, component) in post_form_nodes {
            nodes.insert(id, component);
        }
        for (id, component) in post_form_descendants {
            nodes.insert(id, component);
        }
    }

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root,
            nodes,
            data: merged_data,
        }),
        nav_active_patch("companies"),
    ])
}

/// Build a `PatchMessage` that marks `<active_slug>` as the active nav item and
/// clears all others. Emitted alongside every screen Render so the sidebar's
/// `NavItem` active indicators (bound to `/nav/active/<slug>`) stay in sync
/// with the currently-visible screen. Per D-B13.
fn nav_active_patch(active_slug: &str) -> marionette_protocol::ProtocolMessage {
    use marionette_protocol::data::PatchOperation;
    use marionette_protocol::messages::PatchMessage;
    let slugs = ["home", "contacts", "companies", "users", "audit"];
    let ops: Vec<PatchOperation> = slugs
        .iter()
        .map(|s| PatchOperation::Set {
            path: format!("/nav/active/{s}"),
            value: serde_json::json!(*s == active_slug),
        })
        .collect();
    marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "main".into(),
        patch: ops,
    })
}

/// Phase 15 D-D1: Collect per-field validation errors for a company save
/// payload in form-display order (name → website). Returns a flat
/// `Vec<(bind_path, message)>` that the caller feeds into
/// `validation_error_patch`.
///
/// Bind paths are server-derived string literals — T-15-03-PLAN03-a
/// mitigation (no user input interpolated into bind paths).
#[must_use]
fn collect_company_save_errors(
    name: &str,
    website: Option<&str>,
    _address: Option<&str>,
) -> Vec<(String, String)> {
    let mut errors: Vec<(String, String)> = Vec::new();
    if name.trim().is_empty() {
        errors.push((
            "/companyForm/name".into(),
            "Name is required.".into(),
        ));
    }
    if let Some(w) = website {
        if !w.is_empty() && !w.starts_with("http://") && !w.starts_with("https://") {
            errors.push((
                "/companyForm/website".into(),
                "Website must start with http:// or https://.".into(),
            ));
        }
    }
    errors
}

/// Handle the `company_save` action: create or update a company.
///
/// PHASE 15 D-D1: per-field validation emits `/_errors/{bind}` patches via
/// `validation_error_patch()` instead of `Err(ActionError::BadPayload(...))`.
/// `ActionError::BadPayload` stays reserved for protocol-layer failures
/// (JSON parse, missing `form_bind`) per D-D4.
pub async fn handle_company_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::{NotSet, Set};

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<CompanySavePayload>::from_context(&ctx)?;
    let data = payload.0.company_form;

    // Phase 15 D-D1 — per-field validation via /_errors{bind} patches.
    let errors = collect_company_save_errors(
        &data.name,
        data.website.as_deref(),
        data.address.as_deref(),
    );
    if !errors.is_empty() {
        return Ok(vec![validation_error_patch("content", errors)]);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Phase 15 D-D1 / T-15-03-PLAN03-a: required-field validation pushes
    /// a `/companyForm/name` error when name is empty or whitespace.
    #[test]
    fn collect_company_save_errors_flags_empty_name() {
        let errors = collect_company_save_errors("", Some("https://ex.com"), None);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "/companyForm/name");
        assert_eq!(errors[0].1, "Name is required.");

        let errors_ws = collect_company_save_errors("   ", None, None);
        assert_eq!(errors_ws.len(), 1, "whitespace-only name should fail");
        assert_eq!(errors_ws[0].0, "/companyForm/name");
    }

    /// Phase 15 D-D1: website URL prefix check is enforced when a
    /// non-empty string is supplied; empty / None are permitted.
    #[test]
    fn collect_company_save_errors_flags_bad_website() {
        let errors = collect_company_save_errors("Acme", Some("acme.com"), None);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "/companyForm/website");
        assert!(errors[0].1.contains("http://"));
        assert!(errors[0].1.contains("https://"));

        // Empty website string OK.
        assert!(
            collect_company_save_errors("Acme", Some(""), None).is_empty(),
            "empty website should not produce an error"
        );
        // None OK.
        assert!(
            collect_company_save_errors("Acme", None, None).is_empty(),
            "missing website should not produce an error"
        );
        // http:// OK.
        assert!(
            collect_company_save_errors("Acme", Some("http://ex.com"), None).is_empty()
        );
        // https:// OK.
        assert!(
            collect_company_save_errors("Acme", Some("https://ex.com"), None).is_empty()
        );
    }

    /// Phase 15 D-D1: multiple invalid fields are reported together in
    /// form-display order (name first, then website).
    #[test]
    fn collect_company_save_errors_preserves_field_order() {
        let errors = collect_company_save_errors("", Some("not-a-url"), None);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].0, "/companyForm/name");
        assert_eq!(errors[1].0, "/companyForm/website");
    }

    /// Phase 15 D-D1: valid inputs return an empty vec — the caller short-
    /// circuits and proceeds to DB write.
    #[test]
    fn collect_company_save_errors_valid_input_empty() {
        let errors = collect_company_save_errors(
            "Acme",
            Some("https://acme.example"),
            Some("1 Market St"),
        );
        assert!(errors.is_empty());
    }
}
