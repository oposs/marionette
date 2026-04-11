use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use marionette::builders::standard::{
    ColumnKind, Container, DataTable, Filter, Heading, SelectOption, TableColumn,
};
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload};
use marionette_protocol::{ProtocolMessage, RenderMessage};

use crate::entities::{audit_log, user};

/// Date-range filter payload shape produced by the Phase 13 DataTable
/// `date-range` filter kind. Sent as `{from, to}` inside the filter values
/// object (see D-C3).
#[derive(Deserialize, Default)]
pub struct DateRange {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

/// Filter payload for the audit log query screen.
///
/// Matches the new Phase 13 filter shape: a flat map keyed by filter id
/// where date-range filters use a nested `{from, to}` object.
#[derive(Deserialize, Default)]
pub struct AuditFilterPayload {
    /// `select` filter value; comes in as a string from the frontend
    /// `Select.Root` component and is parsed to i32 inside the handler.
    user_id: Option<String>,
    /// `text` filter value for the audit_log_table column.
    table: Option<String>,
    /// `date-range` filter value for audit_log_timestamp.
    date: Option<DateRange>,
}

/// Handle the `audit_list` action: render a filterable audit log DataTable.
pub async fn handle_audit_list(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;

    // Extract optional filter payload (default if absent)
    let filter = Payload::<AuditFilterPayload>::from_context(&ctx)
        .map(|p| p.0)
        .unwrap_or_default();

    // Parse the (string) user_id once, so we can reuse it in both the page
    // query and the count query.
    let user_id_int: Option<i32> = filter
        .user_id
        .as_deref()
        .and_then(|s| s.parse::<i32>().ok());

    // Build query with conditional filters
    let mut query = audit_log::Entity::find();

    if let Some(uid) = user_id_int {
        query = query.filter(audit_log::Column::AuditLogUser.eq(uid));
    }
    if let Some(ref tbl) = filter.table {
        if !tbl.is_empty() {
            query = query.filter(audit_log::Column::AuditLogTable.eq(tbl.as_str()));
        }
    }
    if let Some(ref dr) = filter.date {
        if let Some(ref from) = dr.from {
            if !from.is_empty() {
                query = query.filter(audit_log::Column::AuditLogTimestamp.gte(from.as_str()));
            }
        }
        if let Some(ref to) = dr.to {
            if !to.is_empty() {
                query = query.filter(audit_log::Column::AuditLogTimestamp.lte(to.as_str()));
            }
        }
    }

    let entries = query
        .order_by_desc(audit_log::Column::AuditLogTimestamp)
        .all(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    // Limit to 100 results
    let entries: Vec<_> = entries.into_iter().take(100).collect();

    // Compute total_rows with the SAME WHERE clauses as the page query (D-H2).
    // SeaORM consumes the query in `.all(...)`, so we rebuild the filter chain.
    let mut count_query = audit_log::Entity::find();
    if let Some(uid) = user_id_int {
        count_query = count_query.filter(audit_log::Column::AuditLogUser.eq(uid));
    }
    if let Some(ref tbl) = filter.table {
        if !tbl.is_empty() {
            count_query = count_query.filter(audit_log::Column::AuditLogTable.eq(tbl.as_str()));
        }
    }
    if let Some(ref dr) = filter.date {
        if let Some(ref from) = dr.from {
            if !from.is_empty() {
                count_query =
                    count_query.filter(audit_log::Column::AuditLogTimestamp.gte(from.as_str()));
            }
        }
        if let Some(ref to) = dr.to {
            if !to.is_empty() {
                count_query =
                    count_query.filter(audit_log::Column::AuditLogTimestamp.lte(to.as_str()));
            }
        }
    }
    let total_rows: u64 = count_query
        .count(&*db.0)
        .await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

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

    let table = DataTable::new(vec![
        TableColumn::new("timestamp", "When")
            .sortable()
            .kind(ColumnKind::Date),
        TableColumn::new("user", "Who").sortable(),
        TableColumn::new("table", "Table").sortable(),
        TableColumn::new("recordId", "Record"),
        TableColumn::new("action", "Action").sortable(),
        TableColumn::new("changes", "Changes").hidden_default(true),
    ])
    .filter(Filter::select("user_id", user_options).label("User"))
    .filter(
        Filter::text("table")
            .label("Table")
            .placeholder("e.g. user"),
    )
    .filter(Filter::date_range("date").label("Date range"))
    .total_rows(total_rows)
    .source("audit_list")
    .row_id_key("id")
    .page_size(50u32)
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
    let all_children = vec![heading, table];

    let container_nodes = Container::new()
        .id("audit-root")
        .children(all_children)
        .build_with_children();

    let mut nodes = HashMap::new();
    for (id, component) in container_nodes {
        nodes.insert(id, component);
    }

    // Filter state is owned locally by the frontend DataTable component per
    // D-C4; the backend no longer pre-populates initial filter values.
    let data = serde_json::json!({
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
