//! `gallery-demo/noop` handler — catch-all fire-and-toast for leaf-demo
//! components. Every leaf-demo Button/Switch/etc. dispatches this action;
//! the handler fires a `toast` event naming the source action so the demo
//! feels alive. The client renders via svelte-sonner (see docs/SDUI-CONCEPT.md
//! §"Where the Client Is Smart").

use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::messages::EventMessage;
use marionette_protocol::ProtocolMessage;

#[allow(clippy::unused_async)]
pub async fn handle_noop(ctx: HandlerContext) -> ActionResult {
    let source = ctx.action.name.clone();
    Ok(vec![ProtocolMessage::Event(EventMessage {
        id: ctx.action.id.clone(),
        name: "toast".into(),
        surface: None,
        hint: Some(serde_json::json!({
            "message": format!("Demo action from {source}"),
            "severity": "info",
            "duration": 3000,
        })),
    })])
}
