//! `gallery-show` handler — single entry point for every nav click.
//!
//! Payload `{ "key": "<demo-key>" }`. Handler looks up the DemoEntry,
//! invokes `(entry.render)()` returning `Vec<Node>` (post-D-Z1), packs
//! into a HashMap, seeds any needed /demo/{key}/... state into the
//! Render's data field, and emits a Render to the `content` sub-surface.

use std::collections::HashMap;

use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette::gallery::registered_demos;
use marionette_protocol::messages::RenderMessage;
use marionette_protocol::{Component, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_gallery_show(ctx: HandlerContext) -> ActionResult {
    let key = ctx
        .action
        .payload
        .as_ref()
        .and_then(|p| p.get("key"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ActionError::BadPayload("gallery-show requires `key` field".into()))?
        .to_string();

    let entry = registered_demos()
        .find(|e| e.key == key)
        .ok_or_else(|| ActionError::BadPayload(format!("unknown gallery demo '{key}'")))?;

    let nodes_vec = (entry.render)();
    if nodes_vec.is_empty() {
        return Err(ActionError::BadPayload(format!(
            "gallery demo '{key}' returned empty Vec<Node>"
        )));
    }
    let root_id = nodes_vec[0].0.clone();
    let nodes_map: HashMap<String, Component> = nodes_vec.into_iter().collect();

    let data = seed_for_key(&key);

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        root: root_id,
        nodes: nodes_map,
        data,
    })])
}

/// Per-demo state seeds (CONTEXT.md §D-D1 + §D-D2).
///
/// Paths under `/demo/{key}/...` seed the data store so bindings in the
/// demo's Component tree resolve to sensible initial values. Unknown keys
/// yield an empty seed — fine for pure-visual leaves (Heading, Text, Spinner).
fn seed_for_key(key: &str) -> serde_json::Value {
    match key {
        "text-input" => serde_json::json!({ "demo": { "text-input": { "value": "" } } }),
        "select" => serde_json::json!({ "demo": { "select":     { "value": "" } } }),
        "checkbox" => serde_json::json!({ "demo": { "checkbox":   { "checked": false } } }),
        "switch" => serde_json::json!({ "demo": { "switch":     { "checked": false } } }),
        "radio-group" => serde_json::json!({ "demo": { "radio-group":{ "value": "" } } }),
        "textarea" => serde_json::json!({ "demo": { "textarea":   { "value": "" } } }),
        "form" => serde_json::json!({ "demo": { "form":       { "email": "", "name": "" } } }),
        "field-set" => serde_json::json!({ "demo": { "field-set":  { "a": "", "b": "" } } }),
        "data-table" => serde_json::json!({ "demo": { "data-table": { "rows": seed_table_rows() } } }),
        _ => serde_json::json!({}),
    }
}

fn seed_table_rows() -> serde_json::Value {
    // 5-10 synthetic rows per CONTEXT.md §D-D1 (Phase 18 CAT-03 takes this to ≥500).
    serde_json::json!([
        {"id": 1, "name": "Alice Baker", "email": "alice@example.com", "created": "2026-01-05"},
        {"id": 2, "name": "Bob Chen",    "email": "bob@example.com",   "created": "2026-01-08"},
        {"id": 3, "name": "Carol Davis", "email": "carol@example.com", "created": "2026-01-12"},
        {"id": 4, "name": "Dan Evans",   "email": "dan@example.com",   "created": "2026-01-15"},
        {"id": 5, "name": "Eva Frost",   "email": "eva@example.com",   "created": "2026-01-20"},
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_for_unknown_key_is_empty() {
        assert_eq!(seed_for_key("bogus"), serde_json::json!({}));
    }

    #[test]
    fn seed_for_form_has_email_and_name() {
        let seed = seed_for_key("form");
        assert_eq!(seed["demo"]["form"]["email"], "");
        assert_eq!(seed["demo"]["form"]["name"], "");
    }

    #[test]
    fn seed_table_rows_has_five_rows() {
        let rows = seed_table_rows();
        assert_eq!(rows.as_array().unwrap().len(), 5);
    }
}
