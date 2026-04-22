//! Toast demo handlers — `gallery-demo/toast-fire` and the frontend-hardcoded
//! `dismiss-toast` (copied from crm-demo/src/handlers/contact.rs:1687-1711).

use marionette::builders::Button;
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::{ComponentAction, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_toast_fire(ctx: HandlerContext) -> ActionResult {
    let toast_id = format!("toast-demo-{}", uuid::Uuid::new_v4());
    let (_, toast_node) = Button::new("Demo toast from gallery-demo/toast-fire")
        .id(&toast_id)
        .action(ComponentAction::click("dismiss-toast"))
        .build();

    let ops = vec![
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

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "toasts".into(),
        patch: ops,
    })])
}

#[allow(clippy::unused_async)]
pub async fn handle_dismiss_toast(ctx: HandlerContext) -> ActionResult {
    let payload = ctx.action.payload.clone().unwrap_or_default();
    let toast_id = payload
        .get("toastId")
        .and_then(|v| v.as_str())
        .unwrap_or("toast-default")
        .to_string();

    let ops = vec![
        PatchOperation::RemoveChild {
            parent: "toasts-root".into(),
            child_id: toast_id.clone(),
        },
        PatchOperation::DeleteNode { id: toast_id },
    ];

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "toasts".into(),
        patch: ops,
    })])
}
