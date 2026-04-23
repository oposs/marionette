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
        // switch demo binds /demo/switch/checked-1 + /demo/switch/checked-2 (see
        // marionette/src/builders/switch.rs::gallery_demo). Pre-17-06 seed wrote
        // only `/demo/switch/checked` (G-05 path mismatch). Seed both now, with
        // Wifi on and Bluetooth off for a visually distinct initial state.
        "switch" => serde_json::json!({
            "demo": { "switch": { "checked-1": true, "checked-2": false } }
        }),
        "radio-group" => serde_json::json!({ "demo": { "radio-group":{ "value": "" } } }),
        // textarea demo binds /demo/textarea/value + /demo/textarea/value-desc
        // (see marionette/src/builders/textarea.rs::gallery_demo). Pre-17-06
        // seed only wrote `/demo/textarea/value` (G-05 path mismatch on the
        // second textarea). Seed both empty so the Field.Label stays bound.
        "textarea" => serde_json::json!({
            "demo": { "textarea": { "value": "", "value-desc": "" } }
        }),
        // error-display demo binds /demo/error-display/errors-a +
        // /demo/error-display/errors-b (added in 17-06 to fix G-05: the
        // pre-17-06 demo omitted .bind(...) entirely, so the frontend's
        // `{#if errors.length > 0}` guard failed and nothing rendered).
        // ErrorEntry = { path?: string, message: string } per frontend
        // ErrorDisplay.svelte:26-41.
        "error-display" => serde_json::json!({
            "demo": {
                "error-display": {
                    "errors-a": [
                        { "path": "/contact/email", "message": "Email is required" },
                        { "path": "/contact/phone", "message": "Phone number is invalid" },
                    ],
                    "errors-b": [
                        { "message": "A system-level error (no path)" },
                    ],
                }
            }
        }),
        "form" => serde_json::json!({ "demo": { "form":       { "email": "", "name": "" } } }),
        "field-set" => serde_json::json!({ "demo": { "field-set":  { "a": "", "b": "" } } }),
        "data-table" => serde_json::json!({ "demo": { "data-table": { "rows": seed_table_rows() } } }),
        // Phase 18 Plan 04 (CAT-01): pure-visual screen; no bind paths are read
        // by catalog/buttons.rs (the matrix fires `gallery-demo/noop` on click
        // but reads no surface data). Empty seed is the correct zero-state.
        "catalog-buttons" => serde_json::json!({}),
        _ => serde_json::json!({}),
    }
}

fn seed_table_rows() -> serde_json::Value {
    // Object-map keyed by stringified id — matches the frontend contract in
    // DataTable.svelte:113-119 (`Object.entries(rawData)` iteration) AND the
    // CRM per-row Set pattern in crm-demo/src/handlers/fetch_rows.rs:136-149.
    // (G-03 fix, Phase 17 Plan 17-05 Task 3.) Phase 18 CAT-03 will extract a
    // shared generator (≥500 rows); Phase 19 EXER-03 pushes to ≥10 000.
    serde_json::json!({
        "1": {"id": 1, "name": "Alice Baker", "email": "alice@example.com", "created": "2026-01-05"},
        "2": {"id": 2, "name": "Bob Chen",    "email": "bob@example.com",   "created": "2026-01-08"},
        "3": {"id": 3, "name": "Carol Davis", "email": "carol@example.com", "created": "2026-01-12"},
        "4": {"id": 4, "name": "Dan Evans",   "email": "dan@example.com",   "created": "2026-01-15"},
        "5": {"id": 5, "name": "Eva Frost",   "email": "eva@example.com",   "created": "2026-01-20"},
    })
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
        // Object-map keyed by stringified id (matches frontend Object.entries
        // contract in DataTable.svelte:113). See seed_table_rows comment.
        assert_eq!(rows.as_object().unwrap().len(), 5);
        assert_eq!(rows["1"]["name"], "Alice Baker");
        assert_eq!(rows["5"]["name"], "Eva Frost");
    }
}
