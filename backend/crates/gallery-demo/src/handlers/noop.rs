//! `gallery-demo/noop` handler — catch-all fire-and-toast for leaf-demo
//! components. Every leaf-demo Button/Switch/etc. dispatches this action;
//! the handler enqueues a toast naming the source action so the demo feels alive.

use marionette::builders::Button;
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::{ComponentAction, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_noop(ctx: HandlerContext) -> ActionResult {
    let source = ctx.action.name.clone();
    let toast_id = format!("toast-noop-{}", uuid::Uuid::new_v4());
    let toast_label = format!("Demo action from {source}");
    let (_toast_id_built, toast_node) = Button::new(toast_label)
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
