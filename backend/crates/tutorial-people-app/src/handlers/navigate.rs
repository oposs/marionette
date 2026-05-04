//! `navigate` — fired automatically by the frontend on WebSocket connect.
//!
//! Emits three Renders:
//!  1. `main`    — the AppShell.
//!  2. `content` — the People page (form + table) seeded with the current
//!     in-memory rows.
//!  3. `modal`   — an empty Container so `ModalSurface` skips its loading
//!     skeleton on first paint. Required even though this app has no modals.

use std::collections::HashMap;

use marionette::builders::Container;
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette_protocol::messages::RenderMessage;
use marionette_protocol::{Component, ProtocolMessage};

use crate::state::PeopleStore;
use crate::ui;

pub async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let store = ctx
        .extensions
        .get_arc::<PeopleStore>()
        .ok_or_else(|| ActionError::Internal("PeopleStore not registered".into()))?;

    // ---- main: AppShell ----
    let shell_nodes = ui::build_app_shell();
    let mut shell_map: HashMap<String, Component> = HashMap::new();
    for (id, c) in shell_nodes {
        shell_map.insert(id, c);
    }
    let shell_data = serde_json::json!({
        "system": { "connectionStatus": "connected" },
    });

    // ---- content: People page seeded with current rows ----
    let rows = store.snapshot().await;
    let (page_root, page_nodes, page_data) = ui::build_people_page(rows);

    // ---- modal: empty sentinel ----
    let (modal_root, modal_component) = Container::new().id("modal-empty").build();
    let mut modal_nodes: HashMap<String, Component> = HashMap::new();
    modal_nodes.insert(modal_root.clone(), modal_component);

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: None,
            surface: "main".into(),
            root: "app-shell-root".into(),
            nodes: shell_map,
            data: shell_data,
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None,
            surface: "content".into(),
            root: page_root,
            nodes: page_nodes,
            data: page_data,
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None,
            surface: "modal".into(),
            root: modal_root,
            nodes: modal_nodes,
            data: serde_json::json!({}),
        }),
    ])
}
