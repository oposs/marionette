//! Modal demo handlers — `gallery-demo/modal-open` and the frontend-hardcoded
//! `close-modal` (NOT `gallery-demo/modal-close`, per RESEARCH.md §Pitfall 3
//! which verifies that `frontend/src/lib/components/popup/ModalSurface.svelte:15`
//! dispatches the literal string `"close-modal"`).

use std::collections::HashMap;

use marionette::builders::{Container, Heading, Modal, Text};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::messages::RenderMessage;
use marionette_protocol::{Component, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_modal_open(ctx: HandlerContext) -> ActionResult {
    let modal_root_id = "demo-modal-root".to_string();
    let modal_title = Heading::new("Example modal")
        .id("demo-modal-title")
        .build();
    let modal_body = Text::new("Clicking X or the backdrop dismisses this dialog.")
        .id("demo-modal-body")
        .build();
    let modal_nodes = Modal::new("Example modal")
        .id(&modal_root_id)
        .children(vec![modal_title, modal_body])
        .build_with_children();

    let mut map: HashMap<String, Component> = HashMap::new();
    for (id, c) in modal_nodes {
        map.insert(id, c);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: modal_root_id,
        nodes: map,
        data: serde_json::json!({}),
    })])
}

#[allow(clippy::unused_async)]
pub async fn handle_modal_close(ctx: HandlerContext) -> ActionResult {
    // Empty Container at "modal-empty" root clears the modal sub-surface
    // (RESEARCH.md §Pattern 4 — best-effort approximation until Open Q #2
    //  settles on a definitive close semantics; Chrome MCP UAT in Plan 04
    //  catches any residual open-state).
    let (root_id, root_comp) = Container::new().id("modal-empty").build();
    let mut map: HashMap<String, Component> = HashMap::new();
    map.insert(root_id.clone(), root_comp);

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: root_id,
        nodes: map,
        data: serde_json::json!({}),
    })])
}
