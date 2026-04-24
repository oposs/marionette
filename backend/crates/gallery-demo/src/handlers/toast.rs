//! Toast demo handlers.
//!
//! `gallery-demo/toast-fire` dispatches a `toast` event — the client
//! renders via svelte-sonner (stacking / fade / countdown); the server
//! owns content. See CONCEPT.md §"Where the Client Is Smart".
//!
//! `dismiss-toast` is preserved for legacy callers (confirm and noop
//! demos still emit Button toasts into `toasts-root`). Post-sonner,
//! sonner owns dismissal for any toast fired via the event channel; the
//! legacy SurfaceMount-based path continues to use this handler to clear
//! its nodes.

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::{EventMessage, PatchMessage};
use marionette_protocol::ProtocolMessage;

#[allow(clippy::unused_async)]
pub async fn handle_toast_fire(ctx: HandlerContext) -> ActionResult {
    // Showcase the `action` hint shape — sonner renders a "Retry" button
    // inside the toast; clicking it dispatches the named action
    // server-side, same as any SDUI Button's action.
    Ok(vec![ProtocolMessage::Event(EventMessage {
        id: ctx.action.id.clone(),
        name: "toast".into(),
        surface: None,
        hint: Some(serde_json::json!({
            "message": "Demo toast from gallery-demo/toast-fire",
            "severity": "success",
            "duration": 5000,
            "action": {
                "label": "Retry",
                "action": { "name": "gallery-demo/toast-fire" },
            },
        })),
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
