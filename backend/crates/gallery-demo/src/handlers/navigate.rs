//! `navigate` handler — WS-connect entry point.
//!
//! Emits three Renders in canonical order:
//! 1. `main` sub-surface — the AppShell with sidebar iterating `registered_demos()` (CRATE-02)
//! 2. `content` sub-surface — the Home page (D-C2)
//! 3. `toasts` sub-surface — an empty `toasts-root` Container so subsequent
//!    `PatchOperation::InsertChild` calls have a valid parent (D-B15 parallel).

use std::collections::HashMap;

use marionette::builders::{AppShell, Container, Heading, NavItem, SideNav, SurfaceMount, Text};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette::gallery::registered_demos;
use marionette_protocol::messages::RenderMessage;
use marionette_protocol::{Component, ComponentAction, ProtocolMessage};

use crate::home::build_home_page;

#[allow(clippy::unused_async)] // HandlerContext requires async fn
pub async fn handle_navigate(_ctx: HandlerContext) -> ActionResult {
    // -- Sidebar: one NavItem per registered demo (flat alphabetical per D-C1) --
    let nav_items: Vec<(String, Component)> = registered_demos()
        .map(|entry| {
            let mut action = ComponentAction::click("gallery-show");
            action.extra.insert(
                "payload".into(),
                serde_json::json!({ "key": entry.key }),
            );
            NavItem::new(entry.display_name, format!("/gallery/{}", entry.key))
                .id(format!("nav-{}", entry.key))
                .action(action)
                .build()
        })
        .collect();
    let (sidebar_root, sidebar_desc) = SideNav::new()
        .id("shell-side-nav")
        .children(nav_items)
        .build_tree();

    // -- Header: title only (no user menu — no auth) --
    let header_title = Heading::new("Marionette Gallery")
        .id("header-title")
        .build();
    let (header_root, header_desc) = Container::new()
        .id("shell-header")
        .children(vec![header_title])
        .build_tree();

    // -- Footer: version + connection-status binding (mirrors CRM's D-B6) --
    //
    // G-06 (Phase 17 Plan 17-05 Task 1): Footer must render as small muted
    // text. `Heading::new(...)` renders as <h2 class="text-xl font-semibold">,
    // overriding the footer wrapper's `text-xs text-muted-foreground`. Swap to
    // `Text::new(...)` so inherited footer typography takes effect.
    let footer_version = Text::new("Marionette Gallery · v1.2")
        .id("footer-version")
        .build();
    let footer_status = Text::new("connected")
        .id("footer-connection-status")
        .bind("/system/connectionStatus")
        .build();
    let (footer_root, footer_desc) = Container::new()
        .id("shell-footer")
        .children(vec![footer_version, footer_status])
        .build_tree();

    // -- Three sub-surface mounts (D-B8) --
    let content_mount = SurfaceMount::new("content")
        .id("shell-content-mount")
        .build();
    let modal_mount = SurfaceMount::new("modal").id("shell-modal-mount").build();
    let toasts_mount = SurfaceMount::new("toasts")
        .id("shell-toasts-mount")
        .build();

    // -- Assemble AppShell --
    let mut descendants: Vec<(String, Component)> = Vec::new();
    descendants.extend(sidebar_desc);
    descendants.extend(header_desc);
    descendants.extend(footer_desc);

    let shell_nodes = AppShell::new()
        .id("app-shell-root")
        .sidebar(sidebar_root)
        .header(header_root)
        .footer(footer_root)
        .main(content_mount)
        .popups(modal_mount)
        .toasts(toasts_mount)
        .with_descendants(descendants)
        .build_with_children();

    let mut shell_map: HashMap<String, Component> = HashMap::new();
    for (id, c) in shell_nodes {
        shell_map.insert(id, c);
    }
    let shell_data = serde_json::json!({
        "system": { "connectionStatus": "connected" },
    });

    // -- Home page --
    let (home_root_id, home_nodes_map, home_data) = build_home_page();

    // -- Toasts root seed (Container so InsertChild has a parent) --
    let (toasts_root_tuple, _) = Container::new().id("toasts-root").build_tree();
    let mut toasts_map: HashMap<String, Component> = HashMap::new();
    toasts_map.insert(toasts_root_tuple.0.clone(), toasts_root_tuple.1);

    // -- Modal root seed (Container so ModalSurface does not render LoadingSkeleton) --
    //
    // G-07 (Phase 17 Plan 17-05 Task 1): Without this Render, SurfaceMount("modal")
    // has no tree on initial page load, so the frontend renders a grey
    // `LoadingSkeleton` placeholder below the footer on Home. Mirrors the toasts
    // seed pattern used by CRM (crm-demo/src/main.rs:293-309).
    //
    // The "modal-empty" id is the canonical "closed" sentinel — ModalSurface.svelte
    // treats an empty Container root as the closed state (see G-04 fix in
    // frontend/src/lib/components/popup/ModalSurface.svelte). handle_modal_close
    // and confirm_close_with_toast both emit the same sentinel.
    let (modal_root_tuple, _) = Container::new().id("modal-empty").build_tree();
    let mut modal_map: HashMap<String, Component> = HashMap::new();
    modal_map.insert(modal_root_tuple.0.clone(), modal_root_tuple.1);

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
            root: home_root_id,
            nodes: home_nodes_map,
            data: home_data,
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None,
            surface: "toasts".into(),
            root: toasts_root_tuple.0,
            nodes: toasts_map,
            data: serde_json::json!({}),
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None,
            surface: "modal".into(),
            root: modal_root_tuple.0,
            nodes: modal_map,
            data: serde_json::json!({}),
        }),
    ])
}
