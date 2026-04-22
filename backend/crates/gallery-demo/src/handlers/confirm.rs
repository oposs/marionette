//! ConfirmDialog demo handlers — `gallery-demo/confirm-{open,accept,reject}`.
//!
//! Open: render ConfirmDialog into `modal` sub-surface with accept/reject
//! buttons carrying the matching `confirm-accept`/`confirm-reject` actions.
//! Accept/Reject: clear modal sub-surface + enqueue a toast naming the choice.

use std::collections::HashMap;

use marionette::builders::{Button, ConfirmDialog, Container};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::{PatchMessage, RenderMessage};
use marionette_protocol::{Component, ComponentAction, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_confirm_open(ctx: HandlerContext) -> ActionResult {
    let dialog_root_id = "demo-confirm-root".to_string();
    // ConfirmDialog::new(title, message) — two positional args per
    // backend/crates/marionette/src/builders/confirm_dialog.rs.
    let accept_btn = Button::new("Accept")
        .id("demo-confirm-accept-btn")
        .variant("default")
        .action(ComponentAction::click("gallery-demo/confirm-accept"))
        .build();
    let reject_btn = Button::new("Reject")
        .id("demo-confirm-reject-btn")
        .variant("outline")
        .action(ComponentAction::click("gallery-demo/confirm-reject"))
        .build();
    let dialog_nodes = ConfirmDialog::new("Demo confirm", "Choose an option.")
        .id(&dialog_root_id)
        .children(vec![accept_btn, reject_btn])
        .build_with_children();

    let mut map: HashMap<String, Component> = HashMap::new();
    for (id, c) in dialog_nodes {
        map.insert(id, c);
    }

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

    // 2. Enqueue a toast naming the choice
    let toast_id = format!("toast-confirm-{}", uuid::Uuid::new_v4());
    let toast_label = format!("Confirm {choice}");
    let (_, toast_node) = Button::new(toast_label)
        .id(&toast_id)
        .action(ComponentAction::click("dismiss-toast"))
        .build();
    let toast_ops = vec![
        PatchOperation::SetNode {
            id: toast_id.clone(),
            component: toast_node,
        },
        PatchOperation::InsertChild {
            parent: "toasts-root".into(),
            index: 0,
            child_id: toast_id,
        },
    ];

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "modal".into(),
            root: modal_empty_id,
            nodes: modal_map,
            data: serde_json::json!({}),
        }),
        ProtocolMessage::Patch(PatchMessage {
            id: None,
            surface: "toasts".into(),
            patch: toast_ops,
        }),
    ])
}
