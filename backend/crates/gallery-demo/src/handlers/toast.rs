//! Toast demo handler.
//!
//! `gallery-demo/toast-fire` dispatches a `toast` event — the client
//! renders via svelte-sonner (stacking / fade / countdown); the server
//! owns content. See docs/OpenSDUI-CONCEPT.md §"Where the Client Is Smart".

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::messages::EventMessage;
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
