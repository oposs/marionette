//! `fetch-rows` handler — delivers DataTable demo rows for `source: "demo-rows"`.
//!
//! Gallery-scoped: no per-source auth check, no pagination. 5 synthetic rows
//! per CONTEXT.md §D-D1. Phase 18 CAT-03 will extract a shared generator
//! (~500+ rows); Phase 19 EXER-03 will push to 10 000+.

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::ProtocolMessage;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;

#[allow(clippy::unused_async)]
pub async fn handle_demo_fetch_rows(ctx: HandlerContext) -> ActionResult {
    let rows = vec![
        serde_json::json!({"id": 1, "name": "Alice Baker", "email": "alice@example.com", "created": "2026-01-05"}),
        serde_json::json!({"id": 2, "name": "Bob Chen",    "email": "bob@example.com",   "created": "2026-01-08"}),
        serde_json::json!({"id": 3, "name": "Carol Davis", "email": "carol@example.com", "created": "2026-01-12"}),
        serde_json::json!({"id": 4, "name": "Dan Evans",   "email": "dan@example.com",   "created": "2026-01-15"}),
        serde_json::json!({"id": 5, "name": "Eva Frost",   "email": "eva@example.com",   "created": "2026-01-20"}),
    ];

    let mut ops: Vec<PatchOperation> = Vec::with_capacity(rows.len());
    for row in &rows {
        let row_id = row
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0)
            .to_string();
        ops.push(PatchOperation::Set {
            path: format!("/demo/data-table/rows/{row_id}"),
            value: row.clone(),
        });
    }

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}
