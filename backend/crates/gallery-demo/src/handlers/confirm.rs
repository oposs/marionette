//! ConfirmDialog demo handlers — `gallery-demo/confirm-{open,accept,reject}`.
//!
//! Open: render a single structured ConfirmDialog node into the `modal`
//! sub-surface. Labels ("Accept" / "Reject") and the cancel-action name
//! (`gallery-demo/confirm-reject`) are surfaced as props on the dialog
//! itself — NOT as orphan children. The top-level `action` fires the
//! accept handler; the cancel_action prop fires the reject handler.
//! Accept/Reject: clear modal sub-surface + enqueue a toast naming the choice.
//!
//! See Phase 17 Plan 17-05 Task 9 for the G-04 corrective pass that
//! replaced the previous orphan-children shape (which ConfirmDialog.svelte
//! silently ignored, resulting in Confirm=no-op + Cancel=no-toast) with
//! the current prop-driven shape.

use std::collections::HashMap;

use marionette::builders::{ConfirmDialog, Container};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::messages::{EventMessage, RenderMessage};
use marionette_protocol::{Component, ComponentAction, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_confirm_open(ctx: HandlerContext) -> ActionResult {
    let dialog_root_id = "demo-confirm-root".to_string();
    // Phase 17 Plan 17-05 Task 9 (G-04 corrective pass):
    // Emit a single ConfirmDialog node whose props carry the Accept/Reject
    // labels + the cancel_action wiring. The frontend ConfirmDialog.svelte
    // reads these props directly; Accept click dispatches the top-level
    // .action(...) (→ gallery-demo/confirm-accept); Cancel click dispatches
    // props.cancel_action (→ gallery-demo/confirm-reject).
    let (_id, dialog) = ConfirmDialog::new("Demo confirm", "Choose an option.")
        .id(&dialog_root_id)
        .confirm_label("Accept")
        .cancel_label("Reject")
        .cancel_action("gallery-demo/confirm-reject")
        .action(ComponentAction::click("gallery-demo/confirm-accept"))
        .build();

    let mut map: HashMap<String, Component> = HashMap::new();
    map.insert(dialog_root_id.clone(), dialog);

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: dialog_root_id,
        nodes: map,
        data: serde_json::json!({}),
    })])
}

pub async fn handle_confirm_accept(ctx: HandlerContext) -> ActionResult {
    confirm_close_with_toast(ctx, "accepted").await
}

pub async fn handle_confirm_reject(ctx: HandlerContext) -> ActionResult {
    confirm_close_with_toast(ctx, "rejected").await
}

#[allow(clippy::unused_async)]
async fn confirm_close_with_toast(ctx: HandlerContext, choice: &str) -> ActionResult {
    // 1. Clear modal sub-surface (same idiom as handle_modal_close)
    let (modal_empty_id, modal_empty) = Container::new().id("modal-empty").build();
    let mut modal_map: HashMap<String, Component> = HashMap::new();
    modal_map.insert(modal_empty_id.clone(), modal_empty);

    // 2. Fire a `toast` event naming the choice. Severity maps intent:
    //    accept → success (green), reject → info (neutral).
    let severity = if choice == "accepted" { "success" } else { "info" };

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "modal".into(),
            root: modal_empty_id,
            nodes: modal_map,
            data: serde_json::json!({}),
        }),
        ProtocolMessage::Event(EventMessage {
            id: None,
            name: "toast".into(),
            surface: None,
            hint: Some(serde_json::json!({
                "message": format!("Confirm {choice}"),
                "severity": severity,
                "duration": 3000,
            })),
        }),
    ])
}
