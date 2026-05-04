//! CAT-02 blur-validate handlers.
//!
//! Each handler reads `ctx.action.payload.value` (boolean for Checkbox / Switch;
//! string otherwise), applies the input-specific rule (UI-SPEC §CAT-02
//! §Copywriting Error messages), and emits the Phase 12 component-tree op
//! mix locked in UI-SPEC §CAT-02 lines 371-382.
//!
//! The error-slot Containers these ops target are pre-mounted in the
//! catalog-forms render tree (`catalog/forms.rs`) so SetNode / SetChildren
//! always address existing node ids.

use marionette::builders::{Container, ErrorDisplay};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::{Component, ProtocolMessage};

// ---------- shared helpers ----------

/// Construct an inline ErrorDisplay `Component` with a stable id + a bind path
/// that the frontend resolves to the error message string. We build via the
/// standard `ErrorDisplay` builder and pull out the `Component` half of the
/// returned tuple — only that value goes into `SetNode::component`.
fn error_display_component(id: &str, bind_path: &str) -> Component {
    let (_id, comp) = ErrorDisplay::new("error")
        .id(id)
        .bind(bind_path)
        .build();
    comp
}

/// An empty `Container` used on VALID blur for Checkbox / Switch — the
/// error-slot id is SetNode-replaced with a node that renders nothing.
fn empty_container_component(id: &str) -> Component {
    let (_id, comp) = Container::new().id(id).build();
    comp
}

/// Extract a string `value` from the action payload, defaulting to `""` on
/// any missing / type-mismatched field (defense-in-depth per threat model
/// T-18-05-03).
fn payload_string(ctx: &HandlerContext) -> String {
    ctx.action
        .payload
        .as_ref()
        .and_then(|p| p.get("value"))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// Extract a boolean `value` from the action payload, defaulting to `false`.
fn payload_bool(ctx: &HandlerContext) -> bool {
    ctx.action
        .payload
        .as_ref()
        .and_then(|p| p.get("value"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

// `ActionResult` is the public handler contract; the helper shares that
// signature so each handler can `return patch_response(...)` directly. The
// `Result` wrap is therefore intentional even though this helper itself
// never fails — silence `clippy::unnecessary_wraps`.
#[allow(clippy::unnecessary_wraps)]
fn patch_response(ctx: &HandlerContext, patch: Vec<PatchOperation>) -> ActionResult {
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch,
    })])
}

// ---------- TextInput (email) ----------
// Invalid: SetNode + SetChildren (add slot) + Set(error).
// Valid:   SetChildren (remove slot) + Set("").
//
// Demonstrates `set-children` both as an add-child and as a remove-child op —
// the error-slot node itself persists in the graph; the card's children list
// is what changes.

#[allow(clippy::unused_async)]
pub async fn validate_text_input(ctx: HandlerContext) -> ActionResult {
    let value = payload_string(&ctx);
    let valid = value.contains('@') && value.contains('.');
    if valid {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetChildren {
                    id: "catalog-forms-text-card".into(),
                    children: vec![
                        "catalog-forms-text-heading".into(),
                        "catalog-forms-text-state-grid".into(),
                        "catalog-forms-text-sep".into(),
                        "catalog-forms-text-interactive".into(),
                    ],
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/text-value".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        )
    } else {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-text-error-slot".into(),
                    component: error_display_component(
                        "catalog-forms-text-error-slot",
                        "/_errors/demo/catalog-forms/text-value",
                    ),
                },
                PatchOperation::SetChildren {
                    id: "catalog-forms-text-card".into(),
                    children: vec![
                        "catalog-forms-text-heading".into(),
                        "catalog-forms-text-state-grid".into(),
                        "catalog-forms-text-sep".into(),
                        "catalog-forms-text-interactive".into(),
                        "catalog-forms-text-error-slot".into(),
                    ],
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/text-value".into(),
                    value: serde_json::Value::String("Enter a valid email address.".into()),
                },
            ],
        )
    }
}

// ---------- Select (required) ----------
// Invalid: SetNode (mount ErrorDisplay with its OWN stable id inside the slot)
//          + SetChildren (add to card) + Set(error).
// Valid:   DeleteNode (target the ErrorDisplay's stable id) + Set("").
//
// The ErrorDisplay node carries id `catalog-forms-select-error` so a later
// DeleteNode op can target THE NODE (not the slot). SetNode then addresses
// the slot id; on first invalid blur SetNode REPLACES the pre-mounted empty
// Container at that id with an ErrorDisplay whose own id is the slot's id —
// BUT we need a separate id for DeleteNode. Solved by emitting SetNode at
// the slot id with a component whose id is the ErrorDisplay's stable id.
// Frontend node-patch semantics: the SetNode payload's Component.id supplies
// the new node id; the slot's old id is replaced. On valid blur we DeleteNode
// by the ErrorDisplay's id.

#[allow(clippy::unused_async)]
pub async fn validate_select(ctx: HandlerContext) -> ActionResult {
    let value = payload_string(&ctx);
    let valid = !value.is_empty();
    if valid {
        patch_response(
            &ctx,
            vec![
                PatchOperation::DeleteNode {
                    id: "catalog-forms-select-error".into(),
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/select-value".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        )
    } else {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-select-error-slot".into(),
                    component: error_display_component(
                        "catalog-forms-select-error",
                        "/_errors/demo/catalog-forms/select-value",
                    ),
                },
                PatchOperation::SetChildren {
                    id: "catalog-forms-select-card".into(),
                    children: vec![
                        "catalog-forms-select-heading".into(),
                        "catalog-forms-select-state-grid".into(),
                        "catalog-forms-select-sep".into(),
                        "catalog-forms-select-interactive".into(),
                        "catalog-forms-select-error-slot".into(),
                    ],
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/select-value".into(),
                    value: serde_json::Value::String("Please make a selection.".into()),
                },
            ],
        )
    }
}

// ---------- Checkbox (must-agree) ----------
// Invalid: SetNode (slot → ErrorDisplay) + Set(error).
// Valid:   SetNode (slot → empty Container) + Set("").
//
// Demonstrates set-node as a REPLACE op (no SetChildren needed — the slot is
// already a direct child of the card, and its id stays the same).

#[allow(clippy::unused_async)]
pub async fn validate_checkbox(ctx: HandlerContext) -> ActionResult {
    let value = payload_bool(&ctx);
    if value {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-checkbox-error-slot".into(),
                    component: empty_container_component("catalog-forms-checkbox-error-slot"),
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/checkbox-value".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        )
    } else {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-checkbox-error-slot".into(),
                    component: error_display_component(
                        "catalog-forms-checkbox-error-slot",
                        "/_errors/demo/catalog-forms/checkbox-value",
                    ),
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/checkbox-value".into(),
                    value: serde_json::Value::String("You must agree to continue.".into()),
                },
            ],
        )
    }
}

// ---------- Switch (must-enable) ----------
// Mirrors Checkbox — set-node as a replace op in both directions.

#[allow(clippy::unused_async)]
pub async fn validate_switch(ctx: HandlerContext) -> ActionResult {
    let value = payload_bool(&ctx);
    if value {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-switch-error-slot".into(),
                    component: empty_container_component("catalog-forms-switch-error-slot"),
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/switch-value".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        )
    } else {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-switch-error-slot".into(),
                    component: error_display_component(
                        "catalog-forms-switch-error-slot",
                        "/_errors/demo/catalog-forms/switch-value",
                    ),
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/switch-value".into(),
                    value: serde_json::Value::String("Notifications must be enabled.".into()),
                },
            ],
        )
    }
}

// ---------- Radio (required) ----------
// Invalid: SetNode + SetChildren (add) + Set(error).
// Valid:   SetChildren (remove) + Set("").
//
// Mirror of TextInput — set-children both as add and as remove.

#[allow(clippy::unused_async)]
pub async fn validate_radio(ctx: HandlerContext) -> ActionResult {
    let value = payload_string(&ctx);
    let valid = !value.is_empty();
    if valid {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetChildren {
                    id: "catalog-forms-radio-card".into(),
                    children: vec![
                        "catalog-forms-radio-heading".into(),
                        "catalog-forms-radio-state-grid".into(),
                        "catalog-forms-radio-sep".into(),
                        "catalog-forms-radio-interactive".into(),
                    ],
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/radio-value".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        )
    } else {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-radio-error-slot".into(),
                    component: error_display_component(
                        "catalog-forms-radio-error-slot",
                        "/_errors/demo/catalog-forms/radio-value",
                    ),
                },
                PatchOperation::SetChildren {
                    id: "catalog-forms-radio-card".into(),
                    children: vec![
                        "catalog-forms-radio-heading".into(),
                        "catalog-forms-radio-state-grid".into(),
                        "catalog-forms-radio-sep".into(),
                        "catalog-forms-radio-interactive".into(),
                        "catalog-forms-radio-error-slot".into(),
                    ],
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/radio-value".into(),
                    value: serde_json::Value::String("Please pick one option.".into()),
                },
            ],
        )
    }
}

// ---------- Textarea (min-length 20) ----------
// Invalid: SetNode (slot → ErrorDisplay w/ stable id) + SetChildren (add) + Set(error).
// Valid:   DeleteNode (by ErrorDisplay's stable id) + Set("").
//
// Mirror of Select — delete-node target needs its own id, distinct from slot id.

#[allow(clippy::unused_async)]
pub async fn validate_textarea(ctx: HandlerContext) -> ActionResult {
    let value = payload_string(&ctx);
    let valid = value.chars().count() >= 20;
    if valid {
        patch_response(
            &ctx,
            vec![
                PatchOperation::DeleteNode {
                    id: "catalog-forms-textarea-error".into(),
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/textarea-value".into(),
                    value: serde_json::Value::String(String::new()),
                },
            ],
        )
    } else {
        patch_response(
            &ctx,
            vec![
                PatchOperation::SetNode {
                    id: "catalog-forms-textarea-error-slot".into(),
                    component: error_display_component(
                        "catalog-forms-textarea-error",
                        "/_errors/demo/catalog-forms/textarea-value",
                    ),
                },
                PatchOperation::SetChildren {
                    id: "catalog-forms-textarea-card".into(),
                    children: vec![
                        "catalog-forms-textarea-heading".into(),
                        "catalog-forms-textarea-state-grid".into(),
                        "catalog-forms-textarea-sep".into(),
                        "catalog-forms-textarea-interactive".into(),
                        "catalog-forms-textarea-error-slot".into(),
                    ],
                },
                PatchOperation::Set {
                    path: "/_errors/demo/catalog-forms/textarea-value".into(),
                    value: serde_json::Value::String(
                        "Bio must be at least 20 characters.".into(),
                    ),
                },
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    //! 12 tests: 6 handlers × {invalid, valid}. Each test asserts the handler
    //! emits the exact locked op sequence (count, order, ids, paths, values)
    //! per UI-SPEC §CAT-02 (lines 371-382) and the plan's <interfaces> block.

    use super::*;
    use marionette::extractors::Session;
    use marionette_protocol::ActionMessage;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::sync::Arc;

    fn mock_db() -> Arc<sea_orm::DatabaseConnection> {
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())
    }

    fn anon_session() -> Session {
        Session {
            user_id: None,
            roles: vec![],
        }
    }

    fn make_ctx(action_name: &str, payload: serde_json::Value) -> HandlerContext {
        HandlerContext {
            action: ActionMessage {
                id: Some("t1".into()),
                name: action_name.into(),
                source: None,
                payload: Some(payload),
                optimistic: None,
            },
            db: mock_db(),
            session: anon_session(),
            extensions: marionette::Extensions::new(),
        }
    }

    fn unwrap_patch(msgs: &[ProtocolMessage]) -> &PatchMessage {
        match &msgs[0] {
            ProtocolMessage::Patch(p) => p,
            _ => panic!("expected Patch message"),
        }
    }

    // ---- TextInput ----

    #[tokio::test]
    async fn text_input_invalid_emits_set_node_then_set_children_then_set() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-text-input",
            serde_json::json!({ "value": "not-an-email" }),
        );
        let msgs = validate_text_input(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 3);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, .. } => {
                assert_eq!(id, "catalog-forms-text-error-slot");
            }
            _ => panic!("op 0 must be SetNode"),
        }
        match &patch.patch[1] {
            PatchOperation::SetChildren { id, children } => {
                assert_eq!(id, "catalog-forms-text-card");
                assert_eq!(children.len(), 5);
                assert_eq!(children.last().map(String::as_str), Some("catalog-forms-text-error-slot"));
            }
            _ => panic!("op 1 must be SetChildren"),
        }
        match &patch.patch[2] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/text-value");
                assert_eq!(value, &serde_json::json!("Enter a valid email address."));
            }
            _ => panic!("op 2 must be Set"),
        }
    }

    #[tokio::test]
    async fn text_input_valid_emits_set_children_then_set_empty() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-text-input",
            serde_json::json!({ "value": "a@b.com" }),
        );
        let msgs = validate_text_input(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::SetChildren { id, children } => {
                assert_eq!(id, "catalog-forms-text-card");
                assert_eq!(children.len(), 4);
                assert!(!children.contains(&"catalog-forms-text-error-slot".to_string()));
            }
            _ => panic!("op 0 must be SetChildren"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/text-value");
                assert_eq!(value, &serde_json::json!(""));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    // ---- Select ----

    #[tokio::test]
    async fn select_invalid_emits_set_node_set_children_set() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-select",
            serde_json::json!({ "value": "" }),
        );
        let msgs = validate_select(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 3);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, component } => {
                assert_eq!(id, "catalog-forms-select-error-slot");
                // The ErrorDisplay component payload carries its own stable id
                // used by the valid-path DeleteNode below.
                let v = serde_json::to_value(component).expect("serialize");
                assert_eq!(v["type"], "error-display");
            }
            _ => panic!("op 0 must be SetNode"),
        }
        match &patch.patch[2] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/select-value");
                assert_eq!(value, &serde_json::json!("Please make a selection."));
            }
            _ => panic!("op 2 must be Set"),
        }
    }

    #[tokio::test]
    async fn select_valid_emits_delete_node_then_set_empty() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-select",
            serde_json::json!({ "value": "USA" }),
        );
        let msgs = validate_select(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::DeleteNode { id } => {
                assert_eq!(id, "catalog-forms-select-error");
            }
            _ => panic!("op 0 must be DeleteNode"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/select-value");
                assert_eq!(value, &serde_json::json!(""));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    // ---- Checkbox ----

    #[tokio::test]
    async fn checkbox_invalid_emits_set_node_then_set() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-checkbox",
            serde_json::json!({ "value": false }),
        );
        let msgs = validate_checkbox(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, component } => {
                assert_eq!(id, "catalog-forms-checkbox-error-slot");
                let v = serde_json::to_value(component).expect("serialize");
                assert_eq!(v["type"], "error-display");
            }
            _ => panic!("op 0 must be SetNode"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/checkbox-value");
                assert_eq!(value, &serde_json::json!("You must agree to continue."));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    #[tokio::test]
    async fn checkbox_valid_emits_set_node_empty_container_then_set_empty() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-checkbox",
            serde_json::json!({ "value": true }),
        );
        let msgs = validate_checkbox(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, component } => {
                assert_eq!(id, "catalog-forms-checkbox-error-slot");
                let v = serde_json::to_value(component).expect("serialize");
                assert_eq!(v["type"], "container");
            }
            _ => panic!("op 0 must be SetNode (container)"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/checkbox-value");
                assert_eq!(value, &serde_json::json!(""));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    // ---- Switch ----

    #[tokio::test]
    async fn switch_invalid_emits_set_node_then_set() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-switch",
            serde_json::json!({ "value": false }),
        );
        let msgs = validate_switch(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, component } => {
                assert_eq!(id, "catalog-forms-switch-error-slot");
                let v = serde_json::to_value(component).expect("serialize");
                assert_eq!(v["type"], "error-display");
            }
            _ => panic!("op 0 must be SetNode"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/switch-value");
                assert_eq!(value, &serde_json::json!("Notifications must be enabled."));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    #[tokio::test]
    async fn switch_valid_emits_set_node_empty_container_then_set_empty() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-switch",
            serde_json::json!({ "value": true }),
        );
        let msgs = validate_switch(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, component } => {
                assert_eq!(id, "catalog-forms-switch-error-slot");
                let v = serde_json::to_value(component).expect("serialize");
                assert_eq!(v["type"], "container");
            }
            _ => panic!("op 0 must be SetNode (container)"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/switch-value");
                assert_eq!(value, &serde_json::json!(""));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    // ---- Radio ----

    #[tokio::test]
    async fn radio_invalid_emits_set_node_set_children_set() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-radio",
            serde_json::json!({ "value": "" }),
        );
        let msgs = validate_radio(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 3);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, .. } => {
                assert_eq!(id, "catalog-forms-radio-error-slot");
            }
            _ => panic!("op 0 must be SetNode"),
        }
        match &patch.patch[1] {
            PatchOperation::SetChildren { id, children } => {
                assert_eq!(id, "catalog-forms-radio-card");
                assert_eq!(children.last().map(String::as_str), Some("catalog-forms-radio-error-slot"));
            }
            _ => panic!("op 1 must be SetChildren"),
        }
        match &patch.patch[2] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/radio-value");
                assert_eq!(value, &serde_json::json!("Please pick one option."));
            }
            _ => panic!("op 2 must be Set"),
        }
    }

    #[tokio::test]
    async fn radio_valid_emits_set_children_then_set_empty() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-radio",
            serde_json::json!({ "value": "pro" }),
        );
        let msgs = validate_radio(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::SetChildren { id, children } => {
                assert_eq!(id, "catalog-forms-radio-card");
                assert!(!children.contains(&"catalog-forms-radio-error-slot".to_string()));
            }
            _ => panic!("op 0 must be SetChildren"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/radio-value");
                assert_eq!(value, &serde_json::json!(""));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    // ---- Textarea ----

    #[tokio::test]
    async fn textarea_invalid_emits_set_node_set_children_set() {
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-textarea",
            serde_json::json!({ "value": "short" }),
        );
        let msgs = validate_textarea(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 3);
        match &patch.patch[0] {
            PatchOperation::SetNode { id, .. } => {
                assert_eq!(id, "catalog-forms-textarea-error-slot");
            }
            _ => panic!("op 0 must be SetNode"),
        }
        match &patch.patch[1] {
            PatchOperation::SetChildren { id, children } => {
                assert_eq!(id, "catalog-forms-textarea-card");
                assert_eq!(
                    children.last().map(String::as_str),
                    Some("catalog-forms-textarea-error-slot")
                );
            }
            _ => panic!("op 1 must be SetChildren"),
        }
        match &patch.patch[2] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/textarea-value");
                assert_eq!(value, &serde_json::json!("Bio must be at least 20 characters."));
            }
            _ => panic!("op 2 must be Set"),
        }
    }

    #[tokio::test]
    async fn textarea_valid_emits_delete_node_then_set_empty() {
        let long = "x".repeat(20);
        let ctx = make_ctx(
            "gallery-demo/catalog-forms/validate-textarea",
            serde_json::json!({ "value": long }),
        );
        let msgs = validate_textarea(ctx).await.expect("ok");
        let patch = unwrap_patch(&msgs);
        assert_eq!(patch.patch.len(), 2);
        match &patch.patch[0] {
            PatchOperation::DeleteNode { id } => {
                assert_eq!(id, "catalog-forms-textarea-error");
            }
            _ => panic!("op 0 must be DeleteNode"),
        }
        match &patch.patch[1] {
            PatchOperation::Set { path, value } => {
                assert_eq!(path, "/_errors/demo/catalog-forms/textarea-value");
                assert_eq!(value, &serde_json::json!(""));
            }
            _ => panic!("op 1 must be Set"),
        }
    }

    // ---- Invariants ----

    #[tokio::test]
    async fn every_handler_writes_to_errors_prefix_path() {
        // Spot-check all 6 handlers' invalid paths to confirm `/_errors/...`
        // is the data-set path (NOT `/demo/...`, which is the value store).
        let cases = [
            (
                "text",
                validate_text_input(make_ctx(
                    "validate-text-input",
                    serde_json::json!({ "value": "" }),
                ))
                .await
                .expect("ok"),
            ),
            (
                "select",
                validate_select(make_ctx(
                    "validate-select",
                    serde_json::json!({ "value": "" }),
                ))
                .await
                .expect("ok"),
            ),
            (
                "checkbox",
                validate_checkbox(make_ctx(
                    "validate-checkbox",
                    serde_json::json!({ "value": false }),
                ))
                .await
                .expect("ok"),
            ),
            (
                "switch",
                validate_switch(make_ctx(
                    "validate-switch",
                    serde_json::json!({ "value": false }),
                ))
                .await
                .expect("ok"),
            ),
            (
                "radio",
                validate_radio(make_ctx(
                    "validate-radio",
                    serde_json::json!({ "value": "" }),
                ))
                .await
                .expect("ok"),
            ),
            (
                "textarea",
                validate_textarea(make_ctx(
                    "validate-textarea",
                    serde_json::json!({ "value": "" }),
                ))
                .await
                .expect("ok"),
            ),
        ];
        for (name, msgs) in cases {
            let msg = unwrap_patch(&msgs);
            let set_op = msg.patch.iter().find_map(|op| match op {
                PatchOperation::Set { path, .. } => Some(path.clone()),
                _ => None,
            });
            let set_path = set_op.unwrap_or_else(|| panic!("{name}: missing Set op"));
            assert!(
                set_path.starts_with("/_errors/demo/catalog-forms/"),
                "{name} writes to {set_path} — must be under /_errors/demo/catalog-forms/"
            );
        }
    }
}
