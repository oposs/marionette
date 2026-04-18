use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, PaginatorTrait};
use serde::Deserialize;

use marionette::builders::standard::{
    form_shell, Button, ColumnKind, Container, DataTable, FieldSeparator, FieldSet, Form, Heading,
    RadioGroup, RadioOption, Select, SelectOption, TableColumn, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
use marionette::validation::validation_error_patch;
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

use crate::entities::user;

/// Payload for identifying a user by ID.
#[derive(Deserialize)]
struct UserIdPayload {
    user_id: i32,
}

/// Shared helper: build a rendered user list from the database.
async fn render_user_list(ctx: &HandlerContext) -> ActionResult {
    let db = Db::from_context(ctx)?;
    let users = user::Entity::find()
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // D-H2: total_rows reflects the full (unfiltered) collection.
    let user_count: u64 = user::Entity::find()
        .count(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let heading = Heading::new("User Management").id("user-heading").build();

    let new_button = Button::new("New User")
        .id("btn-new-user")
        .action(ComponentAction::click("user_new"))
        .build();

    let table = DataTable::new(vec![
        TableColumn::new("name", "Name").sortable(),
        TableColumn::new("email", "Email").sortable(),
        TableColumn::new("role", "Role").sortable(),
        TableColumn::new("lastLogin", "Last Login")
            .sortable()
            .kind(ColumnKind::Date),
        TableColumn::new("actions", "").kind(ColumnKind::Actions),
    ])
    .total_rows(user_count)
    .source("user_list")
    .row_id_key("id")
    .page_size(50u32)
    .id("user-table")
    .bind("/users")
    .build();

    let container_nodes = Container::new()
        .id("user-list-root")
        .children(vec![heading, new_button, table])
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    // Build row data with per-row edit/delete actions
    let rows: Vec<serde_json::Value> = users
        .iter()
        .map(|u| {
            serde_json::json!({
                "id": u.user_id,
                "name": u.user_name,
                "email": u.user_email,
                "role": u.user_role,
                "lastLogin": u.user_last_login.as_deref().unwrap_or("-"),
                "actions": [
                    { "label": "Edit", "action": { "type": "click", "name": "user_edit", "payload": { "user_id": u.user_id } } },
                    { "label": "Delete", "action": { "type": "click", "name": "user_delete", "payload": { "user_id": u.user_id } } }
                ]
            })
        })
        .collect();

    let data = serde_json::json!({ "users": rows });

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root: "user-list-root".into(),
            nodes,
            data,
        }),
        nav_active_patch("users"),
    ])
}

/// Handle the `user_list` action: render a DataTable of all users.
pub async fn handle_user_list(ctx: HandlerContext) -> ActionResult {
    render_user_list(&ctx).await
}

/// Handle the `user_delete` action: delete a user by ID and re-render the list.
pub async fn handle_user_delete(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let payload = Payload::<UserIdPayload>::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;

    // Prevent deleting yourself
    if let Some(ref current_user_id) = session.user_id {
        if payload.0.user_id.to_string() == *current_user_id {
            return Err(ActionError::BadPayload(
                "Cannot delete your own account".into(),
            ));
        }
    }

    // Find user
    let found = user::Entity::find_by_id(payload.0.user_id)
        .one(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?
        .ok_or_else(|| ActionError::Internal("User not found".into()))?;

    let deleted_json = serde_json::json!({
        "name": found.user_name,
        "email": found.user_email,
        "role": found.user_role,
    });

    // Delete
    let deleted_id = found.user_id;
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
    crate::audit::record_audit(&*db.0, caller_id, "user", deleted_id, "delete", deleted_json)
        .await?;

    // Re-render the list
    render_user_list(&ctx).await
}

/// Inner form data for a user (create or update).
#[derive(Deserialize)]
struct UserFormData {
    id: Option<i32>,
    name: String,
    email: String,
    password: String,
    role: String,
    /// Phase 15 D-E2: UI-only demo field. Accepted in the payload so the
    /// RadioGroup's submitted value deserialises cleanly, but discarded
    /// server-side — no DB column, no audit entry. A future phase will
    /// wire persistence if the field graduates out of demo status.
    #[serde(default)]
    #[allow(dead_code)]
    preferred_contact_method: Option<String>,
}

/// Payload wrapper: the frontend sends all surface data, with form fields
/// nested under the form's bind prefix (e.g. `userForm`).
#[derive(Deserialize)]
struct UserSavePayload {
    #[serde(rename = "userForm")]
    user_form: UserFormData,
}

/// Handle the `user_new` / `user_edit` action: render a create/edit form.
///
/// PHASE 15 MIGRATION (D-A1 + D-B1 + D-D1 + D-E2 + D-E3): built via
/// `form_shell()` + two `FieldSet`s (Account, Permissions) separated by
/// a `FieldSeparator` per 15-UI-SPEC §Per-Screen §2 (5 fields → 2 sets).
///
/// - FieldSet "Account" wraps [name, email, password].
/// - FieldSeparator between the two sets (explicit-node path, Phase 14
///   D-C2 preference).
/// - FieldSet "Permissions" wraps [role, preferred_contact_method].
/// - email TextInput carries the locked description "Used for password
///   resets and notifications." (D-E3 + §Description Copy Contract).
/// - preferred_contact_method is a `RadioGroup` with 3 options
///   (email/sms/phone), each carrying a per-option description from
///   15-UI-SPEC §Description Copy Contract. Field is UI-only per D-E2 —
///   the RadioGroup's submitted value is deserialised into
///   `UserFormData::preferred_contact_method` but discarded
///   (`#[allow(dead_code)]`).
pub async fn handle_user_form(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    // Try to extract user_id from payload (edit mode) -- if missing, it's create mode
    let user_id = Payload::<UserIdPayload>::from_context(&ctx)
        .ok()
        .map(|p| p.0.user_id);

    let (form_data, form_title) = if let Some(uid) = user_id {
        let found = user::Entity::find_by_id(uid)
            .one(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?
            .ok_or_else(|| ActionError::Internal("User not found".into()))?;
        (
            serde_json::json!({
                "userForm": {
                    "id": found.user_id,
                    "name": found.user_name,
                    "email": found.user_email,
                    "password": "",
                    "role": found.user_role,
                    "preferred_contact_method": "email"
                }
            }),
            "Edit User",
        )
    } else {
        (
            serde_json::json!({
                "userForm": {
                    "id": null,
                    "name": "",
                    "email": "",
                    "password": "",
                    "role": "user",
                    "preferred_contact_method": "email"
                }
            }),
            "New User",
        )
    };

    // -- Heading + back button --

    let heading = Heading::new(form_title).id("user-form-heading").build();

    let back_button = Button::new("← Back")
        .id("user-form-back")
        .variant("outline")
        .action(ComponentAction::click("user_list"))
        .build();

    // -- FieldSet 1: Account --

    let name_input = TextInput::new("Name")
        .id("user-form-name")
        .bind("/userForm/name")
        .required(true)
        .build();

    let email_input = TextInput::new("Email")
        .id("user-form-email")
        .bind("/userForm/email")
        .input_type("email")
        .required(true)
        // 15-UI-SPEC §Description Copy Contract (D-E3)
        .description("Used for password resets and notifications.")
        .build();

    let password_input = TextInput::new("Password")
        .id("user-form-password")
        .bind("/userForm/password")
        .input_type("password")
        .build();

    let (account_set, account_descendants) = FieldSet::new()
        .id("user-account-set")
        .legend("Account")
        .children(vec![name_input, email_input, password_input])
        .build_tree();

    // -- Separator between Account and Permissions --

    let separator = FieldSeparator::new()
        .id("user-form-separator-1")
        .build();

    // -- FieldSet 2: Permissions --

    let role_select = Select::new(
        "Role",
        vec![
            SelectOption {
                value: "admin".into(),
                label: "Admin".into(),
            },
            SelectOption {
                value: "user".into(),
                label: "User".into(),
            },
        ],
    )
    .id("user-form-role")
    .bind("/userForm/role")
    .build();

    // 15-UI-SPEC §RadioGroup Production Contract + §Description Copy
    // Contract (per-option description strings locked).
    let preferred_radio = RadioGroup::new(
        "Preferred contact method",
        vec![
            RadioOption {
                value: "email".into(),
                label: "Email".into(),
                description: Some("Receive updates by email.".into()),
            },
            RadioOption {
                value: "sms".into(),
                label: "SMS".into(),
                description: Some("Text messages to your phone.".into()),
            },
            RadioOption {
                value: "phone".into(),
                label: "Phone".into(),
                description: Some("A human will call you.".into()),
            },
        ],
    )
    .id("user-form-preferred-contact-method")
    .bind("/userForm/preferred_contact_method")
    .build();

    let (permissions_set, permissions_descendants) = FieldSet::new()
        .id("user-permissions-set")
        .legend("Permissions")
        .children(vec![role_select, preferred_radio])
        .build_tree();

    // -- Action row (D-D1 Option A) --

    let cancel_button = Button::new("Cancel")
        .id("user-form-cancel")
        .variant("outline")
        .action(ComponentAction::click("user_list"))
        .build();

    let save_button = Button::new("Save user")
        .id("user-form-save")
        .variant("default")
        .action(ComponentAction::submit("user_save"))
        .build();

    let (action_row, action_row_descendants) = Container::new()
        .id("user-form-actions")
        .class("flex gap-2 justify-end")
        .children(vec![cancel_button, save_button])
        .build_tree();

    // -- Compose the form --

    let (form_child, form_descendants) = Form::new()
        .id("user-form")
        .children(vec![account_set, separator, permissions_set, action_row])
        .build_tree();

    let mut all_descendants: Vec<(String, marionette_protocol::Component)> = Vec::new();
    all_descendants.extend(account_descendants);
    all_descendants.extend(permissions_descendants);
    all_descendants.extend(action_row_descendants);
    all_descendants.extend(form_descendants);

    // -- Outer shell via form_shell (D-B1) --

    let (root, nodes) = form_shell(
        "user-form-root",
        heading,
        back_button,
        form_child,
        all_descendants,
    );

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root,
            nodes,
            data: form_data,
        }),
        nav_active_patch("users"),
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

/// Phase 15 D-D1: Collect per-field validation errors for a user save
/// payload in form-display order (name → email → password → role).
/// Returns a flat `Vec<(bind_path, message)>` that the caller feeds into
/// `validation_error_patch`.
///
/// Password rules:
/// - On create (`id == None`): password required, must be ≥8 chars.
/// - On edit (`id == Some(_)`): blank password is OK (preserves the
///   existing hash); non-empty but too-short still fails with the
///   same "at least 8 characters" copy.
///
/// Bind paths are server-derived string literals — T-15-03-PLAN03-b
/// mitigation (no user input interpolated into bind paths).
///
/// `preferred_contact_method` is NOT validated (D-E2 — UI-only demo).
#[must_use]
fn collect_user_save_errors(
    id: Option<i32>,
    name: &str,
    email: &str,
    password: &str,
    role: &str,
) -> Vec<(String, String)> {
    let mut errors: Vec<(String, String)> = Vec::new();
    if name.trim().is_empty() {
        errors.push(("/userForm/name".into(), "Name is required.".into()));
    }
    if email.trim().is_empty() {
        errors.push(("/userForm/email".into(), "Email is required.".into()));
    } else if !email.contains('@') {
        errors.push((
            "/userForm/email".into(),
            "Enter a valid email address.".into(),
        ));
    }
    // Password rules depend on create vs. edit (per existing handler contract).
    match id {
        None => {
            // Create mode — password required.
            if password.is_empty() {
                errors.push((
                    "/userForm/password".into(),
                    "Password is required.".into(),
                ));
            } else if password.len() < 8 {
                errors.push((
                    "/userForm/password".into(),
                    "Password must be at least 8 characters.".into(),
                ));
            }
        }
        Some(_) => {
            // Edit mode — blank password preserves existing hash; only
            // reject too-short non-empty passwords.
            if !password.is_empty() && password.len() < 8 {
                errors.push((
                    "/userForm/password".into(),
                    "Password must be at least 8 characters.".into(),
                ));
            }
        }
    }
    if role != "admin" && role != "user" {
        errors.push((
            "/userForm/role".into(),
            "Choose one of the listed roles (admin or user).".into(),
        ));
    }
    errors
}

/// Handle the `user_save` action: create or update a user.
///
/// PHASE 15 D-D1: per-field validation emits `/_errors/{bind}` patches
/// via `validation_error_patch()` on the `content` surface instead of
/// `Err(ActionError::BadPayload(...))`. `ActionError::BadPayload` stays
/// reserved for protocol-layer failures (JSON parse, missing
/// `form_bind`) per D-D4.
///
/// `preferred_contact_method` (D-E2) is deserialised into `UserFormData`
/// but discarded — no DB column, no audit entry.
pub async fn handle_user_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::Set;

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<UserSavePayload>::from_context(&ctx)?;
    let data = payload.0.user_form;

    // Phase 15 D-D1 — per-field validation via /_errors{bind} patches.
    let errors =
        collect_user_save_errors(data.id, &data.name, &data.email, &data.password, &data.role);
    if !errors.is_empty() {
        return Ok(vec![validation_error_patch("content", errors)]);
    }

    match data.id {
        None => {
            // Create mode: password required and minimum 8 chars (validated above).
            let password = data.password.clone();
            let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password, 10))
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            let user_json = serde_json::json!({
                "name": &data.name,
                "email": &data.email,
                "role": &data.role,
            });

            let new_user = user::ActiveModel {
                user_name: Set(data.name),
                user_email: Set(data.email),
                user_password: Set(hash),
                user_role: Set(data.role),
                ..Default::default()
            };
            let inserted = new_user
                .insert(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            let caller_id: i32 = session
                .user_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .unwrap_or(0);
            crate::audit::record_audit(
                &*db.0,
                caller_id,
                "user",
                inserted.user_id,
                "create",
                user_json,
            )
            .await?;
        }
        Some(uid) => {
            // Edit mode: fetch existing user
            let found = user::Entity::find_by_id(uid)
                .one(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?
                .ok_or_else(|| ActionError::Internal("User not found".into()))?;

            let old_json = serde_json::json!({
                "name": found.user_name,
                "email": found.user_email,
                "role": found.user_role,
            });

            let mut active: user::ActiveModel = found.into();
            active.user_name = Set(data.name.clone());
            active.user_email = Set(data.email.clone());
            active.user_role = Set(data.role.clone());

            // Only update password if provided (length already checked above).
            if !data.password.is_empty() {
                let password = data.password.clone();
                let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password, 10))
                    .await
                    .map_err(|e| ActionError::Internal(e.to_string()))?
                    .map_err(|e| ActionError::Internal(e.to_string()))?;
                active.user_password = Set(hash);
            }

            active
                .update(&*db.0)
                .await
                .map_err(|e| ActionError::Internal(e.to_string()))?;

            let new_json = serde_json::json!({
                "name": &data.name,
                "email": &data.email,
                "role": &data.role,
            });
            let changes = crate::audit::compute_changes(&old_json, &new_json);
            let caller_id: i32 = session
                .user_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .unwrap_or(0);
            crate::audit::record_audit(&*db.0, caller_id, "user", uid, "update", changes)
                .await?;
        }
    }

    // After save, re-render the user list
    render_user_list(&ctx).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phase 15 D-D1 / T-15-03-PLAN03-b: required-name validation.
    #[test]
    fn collect_user_save_errors_flags_empty_name() {
        let errors = collect_user_save_errors(
            None,
            "",
            "alice@example.com",
            "correcthorse",
            "admin",
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "/userForm/name");
        assert_eq!(errors[0].1, "Name is required.");
    }

    /// Phase 15 D-D1: email required + must contain `@`.
    #[test]
    fn collect_user_save_errors_flags_bad_email() {
        // Empty email.
        let errors = collect_user_save_errors(
            None,
            "Alice",
            "",
            "correcthorse",
            "admin",
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "/userForm/email");
        assert_eq!(errors[0].1, "Email is required.");

        // Missing @.
        let errors2 = collect_user_save_errors(
            None,
            "Alice",
            "notanemail",
            "correcthorse",
            "admin",
        );
        assert_eq!(errors2.len(), 1);
        assert_eq!(errors2[0].0, "/userForm/email");
        assert!(errors2[0].1.contains("valid email"));
    }

    /// Phase 15 D-D1: enum mismatch on role produces
    /// `/userForm/role` error in the new validation-patch shape.
    #[test]
    fn collect_user_save_errors_flags_bad_role() {
        let errors = collect_user_save_errors(
            None,
            "Alice",
            "alice@example.com",
            "correcthorse",
            "superuser",
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "/userForm/role");
        assert!(errors[0].1.contains("admin") || errors[0].1.contains("listed"));
    }

    /// Phase 15 D-D1: on create (id = None), blank password fails with
    /// the locked copy "Password is required."
    #[test]
    fn collect_user_save_errors_flags_empty_password_on_create() {
        let errors = collect_user_save_errors(
            None,
            "Alice",
            "alice@example.com",
            "",
            "admin",
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "/userForm/password");
        assert_eq!(errors[0].1, "Password is required.");
    }

    /// Phase 15 D-D1: on edit (id = Some), blank password is OK (the
    /// existing hash is preserved server-side). Only short non-empty
    /// passwords fail.
    #[test]
    fn collect_user_save_errors_allows_blank_password_on_edit() {
        // Blank on edit is fine.
        let errors = collect_user_save_errors(
            Some(7),
            "Alice",
            "alice@example.com",
            "",
            "admin",
        );
        assert!(errors.is_empty(), "blank password on edit should be OK");

        // Too-short non-empty on edit still fails.
        let errors2 = collect_user_save_errors(
            Some(7),
            "Alice",
            "alice@example.com",
            "short",
            "admin",
        );
        assert_eq!(errors2.len(), 1);
        assert_eq!(errors2[0].0, "/userForm/password");
        assert!(errors2[0].1.contains("at least 8 characters"));
    }

    /// Phase 15 D-D1: multi-field validation preserves form-display
    /// order (name → email → password → role).
    #[test]
    fn collect_user_save_errors_preserves_field_order() {
        let errors = collect_user_save_errors(
            None,
            "",
            "",
            "",
            "bogus",
        );
        assert_eq!(errors.len(), 4);
        assert_eq!(errors[0].0, "/userForm/name");
        assert_eq!(errors[1].0, "/userForm/email");
        assert_eq!(errors[2].0, "/userForm/password");
        assert_eq!(errors[3].0, "/userForm/role");
    }

    /// Phase 15 D-D1: valid input returns empty vec.
    #[test]
    fn collect_user_save_errors_valid_input_empty() {
        let errors = collect_user_save_errors(
            None,
            "Alice",
            "alice@example.com",
            "correcthorse",
            "admin",
        );
        assert!(errors.is_empty());
    }
}

