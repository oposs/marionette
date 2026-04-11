use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use marionette::builders::standard::{
    Button, Container, DataTable, Heading, Select, SelectOption, TableColumn, TextInput,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload};
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

use crate::entities::{audit_log, user};

/// Filter payload for the audit log query screen.
#[derive(Deserialize, Default)]
pub struct AuditFilterPayload {
    user_id: Option<i32>,
    table: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

/// Handle the `audit_list` action: render a filterable audit log DataTable.
pub async fn handle_audit_list(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    // Extract optional filter payload (default if absent)
    let filter = Payload::<AuditFilterPayload>::from_context(&ctx)
        .map(|p| p.0)
        .unwrap_or_default();

    // Build query with conditional filters
    let mut query = audit_log::Entity::find();

    if let Some(uid) = filter.user_id {
        query = query.filter(audit_log::Column::AuditLogUser.eq(uid));
    }
    if let Some(ref tbl) = filter.table {
        if !tbl.is_empty() {
            query = query.filter(audit_log::Column::AuditLogTable.eq(tbl.as_str()));
        }
    }
    if let Some(ref date_from) = filter.date_from {
        if !date_from.is_empty() {
            query = query.filter(audit_log::Column::AuditLogTimestamp.gte(date_from.as_str()));
        }
    }
    if let Some(ref date_to) = filter.date_to {
        if !date_to.is_empty() {
            query = query.filter(audit_log::Column::AuditLogTimestamp.lte(date_to.as_str()));
        }
    }

    let entries = query
        .order_by_desc(audit_log::Column::AuditLogTimestamp)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Limit to 100 results
    let entries: Vec<_> = entries.into_iter().take(100).collect();

    // Fetch users for the filter dropdown
    let users = user::Entity::find()
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Build user options for the select filter
    let mut user_options = vec![SelectOption {
        value: String::new(),
        label: "All Users".into(),
    }];
    for u in &users {
        user_options.push(SelectOption {
            value: u.user_id.to_string(),
            label: u.user_name.clone(),
        });
    }

    // Build UI
    let heading = Heading::new("Audit Log").id("audit-heading").build();

    let user_select = Select::new("User", user_options)
        .id("audit-filter-user")
        .bind("/auditFilter/user_id")
        .build();

    let table_input = TextInput::new("Table")
        .id("audit-filter-table")
        .placeholder("e.g. user")
        .bind("/auditFilter/table")
        .build();

    let date_from_input = TextInput::new("From Date")
        .id("audit-filter-from")
        .placeholder("YYYY-MM-DD")
        .bind("/auditFilter/date_from")
        .build();

    let date_to_input = TextInput::new("To Date")
        .id("audit-filter-to")
        .placeholder("YYYY-MM-DD")
        .bind("/auditFilter/date_to")
        .build();

    let filter_button = Button::new("Filter")
        .id("audit-filter-btn")
        .action(ComponentAction::submit("audit_list"))
        .build();

    let (filter_container_child, filter_container_descendants) = Container::new()
        .id("audit-filter-form")
        .children(vec![
            user_select,
            table_input,
            date_from_input,
            date_to_input,
            filter_button,
        ])
        .build_tree();

    let table = DataTable::new(vec![
        TableColumn {
            key: "timestamp".into(),
            label: "When".into(),
            sortable: Some(true),
            ..Default::default()
        },
        TableColumn {
            key: "user".into(),
            label: "Who".into(),
            sortable: Some(true),
            ..Default::default()
        },
        TableColumn {
            key: "table".into(),
            label: "Table".into(),
            sortable: Some(true),
            ..Default::default()
        },
        TableColumn {
            key: "recordId".into(),
            label: "Record".into(),
            sortable: None,
            ..Default::default()
        },
        TableColumn {
            key: "action".into(),
            label: "Action".into(),
            sortable: Some(true),
            ..Default::default()
        },
        TableColumn {
            key: "changes".into(),
            label: "Changes".into(),
            sortable: None,
            ..Default::default()
        },
    ])
    .id("audit-table")
    .bind("/auditEntries")
    .build();

    // Build user ID -> name lookup
    let user_map: HashMap<i32, &str> = users.iter().map(|u| (u.user_id, u.user_name.as_str())).collect();

    // Build row data
    let rows: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            let user_name = user_map
                .get(&e.audit_log_user)
                .unwrap_or(&"Unknown");
            serde_json::json!({
                "timestamp": e.audit_log_timestamp,
                "user": user_name,
                "table": e.audit_log_table,
                "recordId": e.audit_log_record_id,
                "action": e.audit_log_action,
                "changes": e.audit_log_changes,
            })
        })
        .collect();

    // Combine all nodes
    let all_children = vec![heading, filter_container_child, table];

    let container_nodes = Container::new()
        .id("audit-root")
        .children(all_children)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }
    for (id, component) in filter_container_descendants {
        nodes.insert(id, component);
    }

    let data = serde_json::json!({
        "auditFilter": {
            "user_id": filter.user_id.map(|id| id.to_string()).unwrap_or_default(),
            "table": filter.table.unwrap_or_default(),
            "date_from": filter.date_from.unwrap_or_default(),
            "date_to": filter.date_to.unwrap_or_default(),
        },
        "auditEntries": rows,
    });

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root: "audit-root".into(),
            nodes,
            data,
        }),
        nav_active_patch("audit"),
    ])
}

/// Build a `PatchMessage` that marks `<active_slug>` as the active nav item and
/// clears all others. Emitted alongside every screen Render so the sidebar's
/// `NavItem` active indicators (bound to `/nav/active/<slug>`) stay in sync
/// with the currently-visible screen. Per D-B13.
fn nav_active_patch(active_slug: &str) -> marionette_protocol::ProtocolMessage {
    use marionette_protocol::messages::PatchMessage;
    use marionette_protocol::data::PatchOperation;
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
