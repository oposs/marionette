//! Modal demo handlers — `gallery-demo/modal-open` opens an overlay by
//! rendering a Container body (NOT a `type: "modal"` component — see G-01)
//! into the `modal` sub-surface; `close-modal` (frontend-hardcoded) renders
//! an empty Container sentinel that `ModalSurface.svelte` treats as closed
//! (see frontend/src/lib/components/popup/ModalSurface.svelte's isOpen check).

use std::collections::HashMap;

use marionette::builders::{Container, Heading, Text};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::messages::RenderMessage;
use marionette_protocol::{Component, ProtocolMessage};

#[allow(clippy::unused_async)]
pub async fn handle_modal_open(ctx: HandlerContext) -> ActionResult {
    // IMPORTANT: do NOT use Modal::new() here. The Modal builder emits
    // `type: "modal"`, which the frontend registry maps to ModalSurface.svelte
    // (frontend/src/lib/registry/defaults.ts:57). Rendering a `type: "modal"`
    // Component into the modal sub-surface causes ModalSurface-inside-
    // ModalSurface infinite recursion — tab lockup (G-01).
    //
    // ModalSurface.svelte already supplies the Dialog.Root + Dialog.Content
    // chrome; the tree.root it renders is the INNER body only. So we emit
    // a plain Container with the body children (Heading title + Text).
    let modal_title = Heading::new("Example modal")
        .id("demo-modal-title")
        .build();
    let modal_body = Text::new("Clicking X or the backdrop dismisses this dialog.")
        .id("demo-modal-body")
        .build();
    let modal_nodes = Container::new()
        .id("demo-modal-root")
        .children(vec![modal_title, modal_body])
        .build_with_children();

    let mut map: HashMap<String, Component> = HashMap::new();
    for (id, c) in modal_nodes {
        map.insert(id, c);
    }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: "demo-modal-root".into(),
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
