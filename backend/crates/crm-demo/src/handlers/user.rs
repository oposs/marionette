use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait};
use serde::Deserialize;

use marionette::builders::standard::{
    Button, Container, DataTable, Form, Heading, Select, SelectOption, TableColumn, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
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

    let heading = Heading::new("User Management").id("user-heading").build();

    let new_button = Button::new("New User")
        .id("btn-new-user")
        .action(ComponentAction::click("user_new"))
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
            key: "role".into(),
            label: "Role".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "lastLogin".into(),
            label: "Last Login".into(),
            sortable: Some(true),
        },
        TableColumn {
            key: "actions".into(),
            label: "Actions".into(),
            sortable: None,
        },
    ])
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

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "user-list-root".into(),
        nodes,
        data,
    })])
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

/// Payload for saving a user (create or update).
#[derive(Deserialize)]
struct UserSavePayload {
    id: Option<i32>,
    name: String,
    email: String,
    password: String,
    role: String,
}

/// Handle the `user_new` / `user_edit` action: render a create/edit form.
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
                    "role": found.user_role
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
                    "role": "user"
                }
            }),
            "New User",
        )
    };

    let heading = Heading::new(form_title).id("user-form-heading").build();

    let name_input = TextInput::new("Name")
        .id("user-form-name")
        .bind("/userForm/name")
        .build();

    let email_input = TextInput::new("Email")
        .id("user-form-email")
        .bind("/userForm/email")
        .build();

    let password_input = TextInput::new("Password")
        .id("user-form-password")
        .input_type("password")
        .bind("/userForm/password")
        .build();

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

    let save_button = Button::new("Save")
        .id("user-form-save")
        .action(ComponentAction::submit("user_save"))
        .build();

    let cancel_button = Button::new("Cancel")
        .id("user-form-cancel")
        .action(ComponentAction::click("user_list"))
        .build();

    let form = Form::new()
        .id("user-form")
        .children(vec![
            name_input,
            email_input,
            password_input,
            role_select,
            save_button,
            cancel_button,
        ])
        .build_with_children();

    let mut all_nodes = Vec::new();
    all_nodes.push(heading);
    all_nodes.extend(form);

    let container_nodes = Container::new()
        .id("user-form-root")
        .children(all_nodes)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "main".into(),
        root: "user-form-root".into(),
        nodes,
        data: form_data,
    })])
}

/// Handle the `user_save` action: create or update a user.
pub async fn handle_user_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::Set;

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<UserSavePayload>::from_context(&ctx)?;
    let data = payload.0;

    // Validate required fields
    if data.name.trim().is_empty() {
        return Err(ActionError::BadPayload("Name is required".into()));
    }
    if data.email.trim().is_empty() {
        return Err(ActionError::BadPayload("Email is required".into()));
    }
    if data.role != "admin" && data.role != "user" {
        return Err(ActionError::BadPayload(
            "Role must be 'admin' or 'user'".into(),
        ));
    }

    match data.id {
        None => {
            // Create mode: password required and minimum 8 chars
            if data.password.len() < 8 {
                return Err(ActionError::BadPayload(
                    "Password must be at least 8 characters".into(),
                ));
            }

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

            // Only update password if provided
            if !data.password.is_empty() {
                if data.password.len() < 8 {
                    return Err(ActionError::BadPayload(
                        "Password must be at least 8 characters".into(),
                    ));
                }
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
