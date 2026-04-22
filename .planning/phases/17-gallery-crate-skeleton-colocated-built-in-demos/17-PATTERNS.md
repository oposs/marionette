# Phase 17: Gallery Crate Skeleton + Colocated Built-in Demos — Pattern Map

**Mapped:** 2026-04-22
**Phase directory:** `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/`
**Files analyzed:** ~34 new/modified files (1 new crate, 24 builder splits, 2 refactors, 2 new tests, 2 docs, 3 config/manifest edits)
**Analogs found:** 28 files with strong analogs / 34 total (6 net-new files — `GALLERY-DEMOS.md`, the composite‑demo pattern, `handle_gallery_show`, `handle_noop`, the flat-alphabetical nav iteration, and Phase 16.5 `Vec<Node>` signature — have no prior shape in the repo).

---

## File Classification

### Cluster 1 — New `gallery-demo` binary crate (workspace member #6)

| File | Role | Data Flow | Closest Analog | Match Quality |
|------|------|-----------|----------------|---------------|
| `backend/crates/gallery-demo/Cargo.toml` | manifest (bin) | pure-compile-time | `backend/crates/crm-demo/Cargo.toml` (reduced: no sea-orm-migration/bcrypt/chrono) + `backend/crates/gallery-smoke/Cargo.toml` (for `features = ["gallery"]` line) | exact + hybrid |
| `backend/crates/gallery-demo/src/main.rs` | binary main / router wiring | request-response + pub-sub (WS) | `backend/crates/crm-demo/src/main.rs` | role-match (copy-and-simplify) |
| `backend/crates/gallery-demo/src/state.rs` | state container | bidirectional | part of `ws::AppState` usage in CRM + `crm-demo/tests/integration_test.rs:80-85` (MockDatabase pattern) | partial |
| `backend/crates/gallery-demo/src/home.rs` | pure builder module | producer-only | `crm-demo/src/main.rs:342-388` `build_login_form()` | role-match |
| `backend/crates/gallery-demo/src/handlers/mod.rs` | module index | pure-compile-time | `crm-demo/src/handlers/mod.rs` | exact |
| `backend/crates/gallery-demo/src/handlers/navigate.rs` | handler | request-response | `crm-demo/src/main.rs:130-335` `handle_navigate` | exact (simplified — no auth) |
| `backend/crates/gallery-demo/src/handlers/show.rs` | handler (registry-driven) | request-response | no direct analog; `crm-demo/src/handlers/contact.rs handle_contact_list` is nearest for Render shape | role-match |
| `backend/crates/gallery-demo/src/handlers/noop.rs` | handler (toast emitter) | event-driven (patch) | `crm-demo/src/handlers/contact.rs:1623-1677` toast-emit subsection of `contact_country_change` | role-match |
| `backend/crates/gallery-demo/src/handlers/modal.rs` | handler (open/close overlay) | request-response | no direct analog; pattern derived from `ModalSurface.svelte` semantics + CRM `handle_navigate`'s sub-surface Render | partial |
| `backend/crates/gallery-demo/src/handlers/confirm.rs` | handler (dialog flow) | request-response | same as modal.rs | partial |
| `backend/crates/gallery-demo/src/handlers/toast.rs` | handler (dispatch + dismiss) | event-driven (patch) | `crm-demo/src/handlers/contact.rs:1687-1711` `handle_dismiss_toast` | exact |
| `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` | handler (DataTable rows) | request-response | `crm-demo/src/handlers/fetch_rows.rs:100-158` | exact (simplified — single source `"demo-rows"`) |
| `backend/crates/gallery-demo/tests/smoke_boot.rs` | integration test | bidirectional (WS) | `crm-demo/tests/integration_test.rs` (first 110 lines for start_server + MockDatabase) | role-match |
| `backend/crates/gallery-demo/tests/nav_auto_discovery.rs` | integration test | request-response | `crm-demo/tests/integration_test.rs` start_server pattern + `gallery-smoke/tests/registry_roundtrip.rs` registry-iteration pattern | hybrid |

### Cluster 2 — Per-component builder file refactor (D-B3) on `backend/crates/marionette/src/builders/`

All 24 new files use the same mechanical split pattern. The analog is `standard.rs` itself — excerpts already in situ.

| File | Role | Data Flow | Source lines in `standard.rs` | In-scope `gallery_demo`? | Colocated props |
|------|------|-----------|-------------------------------|---------------------------|-----------------|
| `button.rs` | builder + demo | producer-only | 11–21 | yes (`key = "button"`) | — |
| `text_input.rs` | builder + demo | producer-only | 23–46 | yes (`key = "text-input"`) | — |
| `select.rs` | builder + demo | producer-only | 69–97 (struct) + 49–53 (`SelectOption`) | yes (`key = "select"`) | `SelectOption` |
| `checkbox.rs` | builder + demo | producer-only | 99–116 | yes (`key = "checkbox"`) | — |
| `container.rs` | builder only (skip) | producer-only | 120–125 | **no** | — |
| `grid.rs` | builder + demo | producer-only | 127–134 | yes (`key = "grid"`) | — |
| `heading.rs` | builder + demo | producer-only | 138–144 | yes (`key = "heading"`) | — |
| `text.rs` | builder + demo | producer-only | 146–150 | yes (`key = "text"`) | — |
| `side_nav.rs` | builder only (skip) | producer-only | 154–156 | **no** | — |
| `nav_item.rs` | builder only (skip) | producer-only | 158–165 | **no** | — |
| `nav_group.rs` | builder only (skip) | producer-only | 167–171 | **no** | — |
| `surface_mount.rs` | builder only (skip) | producer-only | 181–185 | **no** | — |
| `form.rs` | builder + demo (composite) | producer-only | 189–194 | yes (`key = "form"`) | — |
| `textarea.rs` | builder + demo | producer-only | 202–226 | yes (`key = "textarea"`) | — |
| `radio_group.rs` | builder + demo | producer-only | 235–253 (struct) + 61–67 (`RadioOption`) | yes (`key = "radio-group"`) | `RadioOption` |
| `switch.rs` | builder + demo | producer-only | 262–277 | yes (`key = "switch"`) | — |
| `field_set.rs` | builder + demo (composite) | producer-only | 292–309 | yes (`key = "field-set"`) | — |
| `field_separator.rs` | builder only (skip) | producer-only | 315–317 | **no** | — |
| `data_table.rs` | builder + demo (composite) | producer-only | 488–532 (struct) + 320–370 (`TableColumn`) + 374–388 (`ColumnKind`) + 394–486 (`Filter`) | yes (`key = "data-table"`) | `TableColumn`, `ColumnKind`, `Filter` |
| `modal.rs` | builder + demo (composite) | producer-only | 536–542 | yes (`key = "modal"`) | — |
| `toast.rs` | builder + demo | producer-only | 544–552 | yes (`key = "toast"`) | — |
| `confirm_dialog.rs` | builder + demo (composite) | producer-only | 554–559 | yes (`key = "confirm-dialog"`) | — |
| `spinner.rs` | builder + demo | producer-only | 561–566 | yes (`key = "spinner"`) | — |
| `error_display.rs` | builder + demo | producer-only | 568–572 | yes (`key = "error-display"`) | — |
| `composites.rs` | helper fn | producer-only | 620–664 (`form_shell`) | — (internal helper) | — |
| `builders/mod.rs` | re-export hub | pure-compile-time | existing (3-line stub) | — | — |
| `builders/standard.rs` | re-export shim (Option A) or retired | pure-compile-time | retained as `pub use` shim | — | — |

### Cluster 3 — In-place additions to existing `builders/app_shell.rs`

| File | Role | Data Flow | Analog | Match Quality |
|------|------|-----------|--------|---------------|
| `backend/crates/marionette/src/builders/app_shell.rs` | append `gallery_demo()` at bottom | producer-only | `gallery-smoke/src/lib.rs` (shape) + this file's existing `AppShell::new()...build_with_children()` invocation in `crm-demo/src/main.rs:252-261` (content to put in demo body) | hybrid |

### Cluster 4 — Phase 16.5 `DemoEntry.render: fn() -> Vec<Node>` micro-refactor (§D-Z1)

| File | Role | Data Flow | Analog | Match Quality |
|------|------|-----------|--------|---------------|
| `backend/crates/marionette/src/gallery.rs` | signature change on `DemoEntry.render` field | pure-compile-time | file itself (edit in place at line 32) | in-place edit |
| `backend/crates/marionette-macros/src/gallery_demo.rs` | `return_type_is_node` → `return_type_is_vec_node` | pure-compile-time | file itself (edit at line 133-162) | in-place edit |
| `backend/crates/gallery-smoke/src/lib.rs` | `smoke()` returns `vec![Text::new(...).build()]` | producer-only | file itself (edit line 25-27) | in-place edit |
| `backend/crates/gallery-smoke/tests/ui/fail_wrong_return.stderr` | expected error message refers to `Vec<Node>` | pure-compile-time | file itself (edit line 1) | in-place edit |
| `backend/crates/gallery-smoke/tests/ui/fail_wrong_return.rs` | trybuild fixture input (may need no change if `Vec<u32>` stays wrong) | pure-compile-time | file itself | in-place edit |

### Cluster 5 — Workspace + docs + tooling

| File | Role | Data Flow | Analog | Match Quality |
|------|------|-----------|--------|---------------|
| `backend/Cargo.toml` | workspace manifest | pure-compile-time | file itself (edit at `members = [...]`) | in-place edit |
| `backend/crates/marionette/GALLERY-DEMOS.md` | authoring contract doc | — | no analog; top-level `CONCEPT.md` / `TOOLING.md` for writing-style reference only | net-new |
| `Makefile` | build-system target | — | file itself (add `gallery-dev` target paralleling `dev` at line 3) | in-place edit |
| `.planning/REQUIREMENTS.md` | ordinal reconciliation (§CRATE-01 "5th" → "6th") | — | file itself (edit at line 19) | in-place edit |

---

## Pattern Assignments

### 1. `backend/crates/gallery-demo/src/main.rs` (binary main / Axum + ActionRouter + static serve)

**Analog:** `backend/crates/crm-demo/src/main.rs` (gallery-demo is this minus auth/DB/migrations/Listmonk).

**Imports pattern** (from `crm-demo/src/main.rs:1-28`):
```rust
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod handlers;  // gallery-demo: mod handlers; mod home; mod state;
// (gallery-demo drops: entities, listmonk, migration, seed, audit)

use std::sync::Arc;
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use marionette::builders::app_shell::AppShell;
use marionette::builders::standard::{
    Button, Container, Heading, NavItem, SideNav, SurfaceMount,
};
use marionette::extractors::HandlerContext;
use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};
```

**Axum Router + static-serve fallback pattern** (copy from `crm-demo/src/main.rs:609-631`):
```rust
// Static files with SPA fallback
let serve_dir = ServeDir::new("../frontend/build")
    .fallback(ServeFile::new("../frontend/build/index.html"));

let app = Router::new()
    .route("/ws", axum::routing::any(ws_handler))
    .route("/api/health", axum::routing::get(health))
    .fallback_service(serve_dir)
    .with_state(state);

let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")  // port 3002 for gallery
    .await
    .unwrap();
tracing::info!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, app).await.unwrap();
```

**ActionRouter registration pattern** (shape from `crm-demo/src/main.rs:451-600`; gallery actions from RESEARCH.md §Example 1):
```rust
let action_router = ActionRouter::new()
    .action("navigate", box_handler(handle_navigate), AuthRequirement::None)
    .action("gallery-show", box_handler(handle_gallery_show), AuthRequirement::None)
    .action("gallery-demo/noop", box_handler(handle_noop), AuthRequirement::None)
    .action("gallery-demo/modal-open", box_handler(handle_modal_open), AuthRequirement::None)
    .action("close-modal", box_handler(handle_modal_close), AuthRequirement::None) // frontend hardcode
    .action("gallery-demo/confirm-open", box_handler(handle_confirm_open), AuthRequirement::None)
    .action("gallery-demo/confirm-accept", box_handler(handle_confirm_accept), AuthRequirement::None)
    .action("gallery-demo/confirm-reject", box_handler(handle_confirm_reject), AuthRequirement::None)
    .action("gallery-demo/toast-fire", box_handler(handle_toast_fire), AuthRequirement::None)
    .action("dismiss-toast", box_handler(handle_dismiss_toast), AuthRequirement::None)
    .action("fetch-rows", box_handler(handle_demo_fetch_rows), AuthRequirement::None);
```

**MockDatabase + AppState construction** (from `crm-demo/tests/integration_test.rs:80-85`, used at compile rather than in tests):
```rust
use sea_orm::{DatabaseBackend, MockDatabase};

let state = Arc::new(AppState {
    router: action_router,
    db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
    login_form: None,
    listmonk: None,
});
```

**Why this pattern:** `AppState.db: Arc<sea_orm::DatabaseConnection>` is mandatory at `ws.rs` (Pitfall #2 in RESEARCH.md). `MockDatabase` satisfies the type with zero real-DB setup — this is a production-equivalent of the CRM test pattern used in-binary.

**Helper-fn shape (recommended, CONTEXT.md §Claude's Discretion):**
```rust
fn register_gallery_actions(router: ActionRouter) -> ActionRouter {
    router
        .action("navigate", box_handler(handle_navigate), AuthRequirement::None)
        // ...10 more actions...
}
```

---

### 2. `backend/crates/gallery-demo/src/handlers/navigate.rs` (handler, request-response — AppShell slot wiring)

**Analog:** `backend/crates/crm-demo/src/main.rs:130-335` (`handle_navigate`).

**AppShell slot-builder chain pattern** (copy structure from `crm-demo/src/main.rs:192-261`):
```rust
// -- Sidebar: SideNav with NavItems (one per registered demo, D-C1 alphabetical) --
let nav_items: Vec<(String, Component)> = registered_demos()
    .map(|entry| {
        NavItem::new(entry.display_name, format!("/gallery/{}", entry.key))
            .id(format!("nav-{}", entry.key))
            .bind(format!("/nav/active/{}", entry.key))
            .action(
                ComponentAction::click("gallery-show")
                    .with_payload(serde_json::json!({ "key": entry.key })),
            )
            .build()
    })
    .collect();
let (sidebar_root, sidebar_desc) = SideNav::new()
    .id("shell-side-nav")
    .children(nav_items)
    .build_tree();

// -- Header: title only (no user menu — no auth) --
let header_title = Heading::new("Marionette Gallery").id("header-title").build();
let (header_root, header_desc) = Container::new()
    .id("shell-header")
    .children(vec![header_title])
    .build_tree();

// -- Footer: version + connection-status binding (D-B6 pattern from CRM:215-233) --
let footer_version = Heading::new("Marionette Gallery · v1.2").id("footer-version").build();
let footer_status = Heading::new("connected")
    .id("footer-connection-status")
    .bind("/system/connectionStatus")
    .build();
let (footer_root, footer_desc) = Container::new()
    .id("shell-footer")
    .children(vec![footer_version, footer_status])
    .build_tree();

// -- Three sub-surface mounts (D-B8) --
let content_mount = SurfaceMount::new("content").id("shell-content-mount").build();
let modal_mount = SurfaceMount::new("modal").id("shell-modal-mount").build();
let toasts_mount = SurfaceMount::new("toasts").id("shell-toasts-mount").build();

// -- Assemble: collect all descendants, then AppShell::new()...build_with_children() --
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
```

**Render composition** (pattern from `crm-demo/src/main.rs:262-334`):
```rust
let mut shell_map = HashMap::new();
for (id, component) in shell_nodes {
    shell_map.insert(id, component);
}
let shell_data = serde_json::json!({
    "system": { "connectionStatus": "connected" },
    "nav": { "active": {} },
});

// Toasts seed — the Container root must exist before any InsertChild targets "toasts-root"
let (toasts_root_tuple, _) = Container::new().id("toasts-root").build_tree();
let mut toasts_map = HashMap::new();
toasts_map.insert(toasts_root_tuple.0.clone(), toasts_root_tuple.1);

Ok(vec![
    ProtocolMessage::Render(RenderMessage {
        id: None, surface: "main".into(),
        root: "app-shell-root".into(),
        nodes: shell_map, data: shell_data,
    }),
    // Emit home page for the content sub-surface (build via home::build_home_page).
    ProtocolMessage::Render(RenderMessage {
        id: None, surface: "content".into(),
        root: home_root_id, nodes: home_nodes_map,
        data: serde_json::json!({}),
    }),
    ProtocolMessage::Render(RenderMessage {
        id: None, surface: "toasts".into(),
        root: toasts_root_tuple.0, nodes: toasts_map,
        data: serde_json::json!({}),
    }),
])
```

---

### 3. `backend/crates/gallery-demo/src/home.rs` (pure builder, producer-only — Home page)

**Analog:** `crm-demo/src/main.rs:342-388` (`build_login_form`) — same shape (pure fn returns `(root_id, HashMap, data)`).

**Imports + core pattern (from analog lines 342-388):**
```rust
use marionette::builders::standard::{Button, Container, Grid, Heading, Text};
use marionette::gallery::registered_demos;
use marionette_protocol::{Component, ComponentAction};

pub fn build_home_page() -> (String, std::collections::HashMap<String, Component>, serde_json::Value) {
    let welcome = Heading::new("Marionette Gallery").id("home-welcome").level(1).build();
    let intro = Text::new("Visual-iteration harness + SDUI-frontend exerciser.")
        .id("home-intro").build();

    // Tile per registered demo (Claude's discretion: registry-derived; recommended).
    let tiles: Vec<(String, Component)> = registered_demos()
        .map(|entry| {
            Button::new(entry.display_name)
                .id(format!("home-tile-{}", entry.key))
                .variant("outline")
                .action(
                    ComponentAction::click("gallery-show")
                        .with_payload(serde_json::json!({ "key": entry.key })),
                )
                .build()
        })
        .collect();

    let (grid_root, grid_desc) = Grid::new().id("home-grid").cols(3).gap("1rem")
        .children(tiles).build_tree();

    let root_id = "home-root".to_string();
    // Pattern from CRM login-form (crm-demo/src/main.rs:367-380):
    let container_nodes = Container::new()
        .id(&root_id)
        .children(vec![welcome, intro, grid_root])
        .build_with_children();

    let mut nodes = std::collections::HashMap::new();
    for (id, c) in container_nodes { nodes.insert(id, c); }
    for (id, c) in grid_desc { nodes.insert(id, c); }

    (root_id, nodes, serde_json::json!({}))
}
```

---

### 4. `backend/crates/gallery-demo/src/handlers/show.rs` (handler, request-response — registry-driven Render)

**Analog:** no exact match. Closest behavior is `crm-demo/src/handlers/contact.rs handle_contact_list` (renders into `content` sub-surface). Registry-lookup pattern is net-new for Phase 17.

**Core pattern** (derived from RESEARCH.md Pattern 3, adapted for `Vec<Node>` after §D-Z1):
```rust
use std::collections::HashMap;
use marionette::error::{ActionError, ActionResult};
use marionette::extractors::HandlerContext;
use marionette::gallery::registered_demos;
use marionette_protocol::{ProtocolMessage, RenderMessage};

pub async fn handle_gallery_show(ctx: HandlerContext) -> ActionResult {
    let key = ctx.action.payload.as_ref()
        .and_then(|p| p.get("key"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ActionError::BadPayload("missing 'key'".into()))?;

    let entry = registered_demos()
        .find(|e| e.key == key)
        .ok_or_else(|| ActionError::BadPayload(format!("unknown gallery demo '{key}'")))?;

    // Post-§D-Z1: render returns Vec<Node>; index 0 is root, remaining are descendants.
    let nodes_vec = (entry.render)();
    let root_id = nodes_vec[0].0.clone();
    let nodes_map: HashMap<_, _> = nodes_vec.into_iter().collect();

    let data = seed_for_key(key);

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        root: root_id,
        nodes: nodes_map,
        data,
    })])
}
```

**Seed lookup table** (RESEARCH.md §Pattern 3 seed_for_key):
```rust
fn seed_for_key(key: &str) -> serde_json::Value {
    match key {
        "text-input" => serde_json::json!({ "demo": { "text-input": { "value": "" } } }),
        "select"     => serde_json::json!({ "demo": { "select":    { "value": "" } } }),
        "switch"     => serde_json::json!({ "demo": { "switch":    { "checked": false } } }),
        "form"       => serde_json::json!({ "demo": { "form": { "email": "", "name": "" } } }),
        "data-table" => serde_json::json!({ "demo": { "data-table": { "rows": seed_table_rows() } } }),
        _            => serde_json::json!({}),
    }
}
```

---

### 5. `backend/crates/gallery-demo/src/handlers/noop.rs` (handler, event-driven — toast emitter)

**Analog:** `backend/crates/crm-demo/src/handlers/contact.rs:1645-1674` (toast build inside `contact_country_change`).

**Toast-emit pattern** (copy directly from analog lines 1645-1669):
```rust
use marionette::builders::standard::Button;
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::{ComponentAction, ProtocolMessage};

pub async fn handle_noop(ctx: HandlerContext) -> ActionResult {
    let source = ctx.action.name.as_str();  // or ctx.action.payload lookup
    let toast_id = format!("toast-noop-{}", uuid::Uuid::new_v4());
    let toast_label = format!("Demo action from {source}");
    let (_, toast_node) = Button::new(&toast_label)
        .id(&toast_id)
        .action(ComponentAction::click("dismiss-toast"))
        .build();

    let ops = vec![
        PatchOperation::SetNode { id: toast_id.clone(), component: toast_node },
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
```

---

### 6. `backend/crates/gallery-demo/src/handlers/toast.rs::handle_dismiss_toast`

**Analog:** `backend/crates/crm-demo/src/handlers/contact.rs:1687-1711` (`handle_dismiss_toast`).

**Exact pattern** (copy wholesale, adjust default id):
```rust
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;

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
```

---

### 7. `backend/crates/gallery-demo/src/handlers/fetch_rows.rs`

**Analog:** `backend/crates/crm-demo/src/handlers/fetch_rows.rs:100-158` (generic handler).

**Simplified pattern** (single-source `"demo-rows"`, no per-source auth):
```rust
pub async fn handle_demo_fetch_rows(ctx: HandlerContext) -> ActionResult {
    // Single-source gallery handler — no auth check, no pagination cap.
    // Seeded rows for the DataTable demo (5-10 rows per D-D1).
    let rows = vec![
        serde_json::json!({"id": 1, "name": "Alice Baker", "email": "alice@example.com"}),
        serde_json::json!({"id": 2, "name": "Bob Chen",    "email": "bob@example.com"}),
        // ... 3-8 more rows ...
    ];

    let mut ops: Vec<PatchOperation> = Vec::with_capacity(rows.len());
    for row in rows {
        let row_id = row.get("id").and_then(|v| v.as_i64()).unwrap().to_string();
        ops.push(PatchOperation::Set {
            path: format!("/demo/data-table/rows/{row_id}"),
            value: row,
        });
    }

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}
```

---

### 8. `backend/crates/gallery-demo/tests/smoke_boot.rs` (integration test, bidirectional)

**Analog:** `backend/crates/crm-demo/tests/integration_test.rs:1-110` (start_server + MockDatabase + WS connect).

**Imports + start_server pattern** (copy lines 1-110, strip the CRM-specific handlers):
```rust
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use sea_orm::{DatabaseBackend, MockDatabase};
use tokio_tungstenite::connect_async;
use tower_http::services::{ServeDir, ServeFile};

use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;

async fn start_server() -> (String, u16) {
    // Use gallery-demo's real register_gallery_actions + handlers.
    let action_router = gallery_demo::handlers::register_gallery_actions(ActionRouter::new());

    let state = Arc::new(AppState {
        router: action_router,
        db: Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection()),
        login_form: None,
        listmonk: None,
    });

    let frontend_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../frontend/build");
    let serve_dir = ServeDir::new(&frontend_dir)
        .fallback(ServeFile::new(frontend_dir.join("index.html")));

    let app = axum::Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
    (format!("ws://127.0.0.1:{port}/ws"), port)
}

#[tokio::test]
async fn gallery_demo_boots_and_serves_hello() {
    let (url, _) = start_server().await;
    let (mut ws, _) = connect_async(&url).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap();
    // First message is server hello — same assertion as CRM test.
    assert!(msg.into_text().unwrap().contains("\"type\":\"hello\""));
}
```

---

### 9. `backend/crates/gallery-demo/tests/nav_auto_discovery.rs`

**Analogs:**
- `gallery-smoke/tests/registry_roundtrip.rs` (lines 17-51) — registry iteration + alphabetical ordering assertions.
- `crm-demo/tests/integration_test.rs:112-120` — WS send `navigate` + read response pattern.

**Pattern (hybrid of the two):**
```rust
use marionette::gallery::registered_demos;

#[tokio::test]
async fn navigate_includes_one_nav_item_per_registered_demo() {
    let (url, _) = start_server().await;
    let (mut ws, _) = connect_async(&url).await.unwrap();
    // Skip hello frame
    let _hello = ws.next().await.unwrap().unwrap();
    // Send navigate action
    ws.send(Message::Text(r#"{"type":"action","action":{"name":"navigate"}}"#.into()))
        .await.unwrap();

    // Read the shell Render; it must contain one NavItem per registered demo.
    let msg = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
    let nodes = v["nodes"].as_object().unwrap();

    let expected_keys: Vec<&'static str> = registered_demos().map(|e| e.key).collect();
    for key in expected_keys {
        let nav_id = format!("nav-{key}");
        assert!(nodes.contains_key(&nav_id), "missing NavItem for key '{key}'");
    }
}
```

---

### 10. Per-component builder files (24 files — `button.rs` through `error_display.rs`)

**Analog for file-level shape:** `backend/crates/marionette/src/builders/app_shell.rs:1-20` (imports + lint comments + module doc).

**Analog for struct definitions:** each target file's line range in `standard.rs` (see "File Classification → Cluster 2" table above) is copied verbatim — same imports, same `#[derive(ComponentBuilder)]`, same `#[component(type = "...")]`.

**Template (leaf, with demo — e.g. `button.rs`):**
```rust
//! `Button` component builder + canonical `#[gallery_demo]` sibling.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "button")]
pub struct Button {
    pub label: String,
    #[builder(optional)]
    pub variant: Option<String>,
    #[builder(optional)]
    pub size: Option<String>,
    #[builder(optional)]
    pub disabled: Option<bool>,
}

// ---- gallery_demo sibling (Phase 17 DEMO-01) ----

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "button")]
#[must_use]
pub fn gallery_demo() -> crate::gallery::Node {
    use marionette_protocol::ComponentAction;
    // Post-§D-Z1: return type is Vec<Node>. Leaves return vec![one_tuple].
    // Three variants stacked in a Container: default / disabled / destructive (D-A1).
    // ... body emits vec![root_tuple, ...descendants] ...
}

#[cfg(test)]
mod tests {
    // Imported tests from standard.rs lines 671-699 that exercise Button.
}
```

**Gallery demo sibling pattern (leaf)** — derived from `gallery-smoke/src/lib.rs:23-27`:
```rust
// Minimal shape from gallery-smoke:
#[gallery_demo(key = "smoke", name = "Smoke Check")]
#[must_use]
pub fn smoke() -> Node {
    Text::new("gallery-smoke").build()
}
```

Post-§D-Z1 leaf (Phase 17 actual shape):
```rust
#[cfg(feature = "gallery")]
#[gallery_demo(key = "button")]
pub fn gallery_demo() -> crate::gallery::Node {
    let a = Button::new("Primary")
        .action(ComponentAction::submit("gallery-demo/noop"))
        .build();
    let b = Button::new("Disabled").disabled(true).build();
    let c = Button::new("Destructive").variant("destructive")
        .action(ComponentAction::submit("gallery-demo/noop"))
        .build();
    // Phase 16.5: Node = Vec<(String, Component)>, so build_with_children fits directly.
    crate::builders::container::Container::new()
        .children(vec![a, b, c])
        .build_with_children()
}
```

**Gallery demo sibling pattern (composite — e.g. `form.rs`)** — no direct prior analog; new in Phase 17:
```rust
#[cfg(feature = "gallery")]
#[gallery_demo(key = "form")]
pub fn gallery_demo() -> crate::gallery::Node {
    // Per D-A1: composite demos nest other gallery_demo() calls.
    let email_nodes = crate::builders::text_input::gallery_demo();
    let submit = Button::new("Submit")
        .action(ComponentAction::submit("gallery-demo/noop"))
        .build();
    // email_nodes is Vec<Node> (post-§D-Z1); email_nodes[0] is root, rest are desc.
    let (form_root, form_desc) = Form::new()
        .children(vec![email_nodes[0].clone(), submit])
        .build_tree();
    let mut all = vec![form_root];
    all.extend(email_nodes.into_iter().skip(1));  // descendants of text-input demo
    all.extend(form_desc);
    all
}
```

---

### 11. `backend/crates/marionette/src/builders/mod.rs` (re-export hub — replaces current 3-line stub)

**Current shape (file itself, 7 lines):**
```rust
pub mod node;
pub mod standard;
pub mod app_shell;

pub use node::*;
pub use standard::*;
pub use app_shell::*;
```

**Target shape (Option A — shim; recommended per RESEARCH.md §Pattern 5):**
```rust
pub mod node;
pub mod app_shell;

// Per-component builder modules (Phase 17 D-B3 refactor).
pub mod button;
pub mod text_input;
pub mod select;
pub mod checkbox;
pub mod container;
pub mod grid;
pub mod heading;
pub mod text;
pub mod side_nav;
pub mod nav_item;
pub mod nav_group;
pub mod surface_mount;
pub mod form;
pub mod textarea;
pub mod radio_group;
pub mod switch;
pub mod field_set;
pub mod field_separator;
pub mod data_table;
pub mod modal;
pub mod toast;
pub mod confirm_dialog;
pub mod spinner;
pub mod error_display;
pub mod composites;

// Preserved re-export shim (Option A from RESEARCH.md §Pattern 5).
// Keeps `marionette::builders::standard::*` import paths working for all
// 10 external callers enumerated in RESEARCH.md §Pattern 5 audit.
pub mod standard;

pub use node::*;
pub use app_shell::*;
pub use button::*;
pub use text_input::*;
pub use select::*;
pub use checkbox::*;
pub use container::*;
pub use grid::*;
pub use heading::*;
pub use text::*;
pub use side_nav::*;
pub use nav_item::*;
pub use nav_group::*;
pub use surface_mount::*;
pub use form::*;
pub use textarea::*;
pub use radio_group::*;
pub use switch::*;
pub use field_set::*;
pub use field_separator::*;
pub use data_table::*;
pub use modal::*;
pub use toast::*;
pub use confirm_dialog::*;
pub use spinner::*;
pub use error_display::*;
pub use composites::*;
```

**`standard.rs` reduces to** (pattern from the shim idiom):
```rust
//! Re-export shim for pre-Phase-17 import paths.
//! Use `marionette::builders::Button` directly; this alias remains for
//! the 10 external callers still using `builders::standard::Button`.

pub use super::{
    button::*, text_input::*, select::*, checkbox::*,
    container::*, grid::*, heading::*, text::*,
    side_nav::*, nav_item::*, nav_group::*, surface_mount::*,
    form::*, textarea::*, radio_group::*, switch::*,
    field_set::*, field_separator::*, data_table::*,
    modal::*, toast::*, confirm_dialog::*,
    spinner::*, error_display::*,
    composites::*,
};
```

---

### 12. `backend/crates/marionette/src/builders/app_shell.rs` (append `gallery_demo()` at bottom)

**Analog for file layout:** the file itself — unchanged structure.
**Analog for `gallery_demo()` body content:** `crm-demo/src/main.rs:192-261` (the full AppShell construction), *curated* per D-A2 (hand-picked content, NOT auto-nested).

**Append at end of file:**
```rust
#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "app-shell")]
#[must_use]
pub fn gallery_demo() -> crate::gallery::Node {
    // D-A2: hand-designed curated shell. Renders INSIDE the `content`
    // sub-surface as a nested example — not as the outer shell.
    use crate::builders::{Container, Heading, NavItem, SideNav, SurfaceMount, Text};
    use marionette_protocol::ComponentAction;

    let nav_a = NavItem::new("Dashboard", "/demo/app-shell/dashboard")
        .id("demo-app-shell-nav-a").build();
    let nav_b = NavItem::new("Reports", "/demo/app-shell/reports")
        .id("demo-app-shell-nav-b").build();
    let nav_c = NavItem::new("Settings", "/demo/app-shell/settings")
        .id("demo-app-shell-nav-c").build();
    let (sidebar_root, sidebar_desc) = SideNav::new()
        .id("demo-app-shell-sidebar")
        .children(vec![nav_a, nav_b, nav_c])
        .build_tree();

    let header = Heading::new("Demo App")
        .id("demo-app-shell-header-title").build();
    let main = Text::new("This AppShell demo is hand-curated per D-A2.")
        .id("demo-app-shell-main-text").build();

    AppShell::new()
        .id("demo-app-shell")
        .sidebar(sidebar_root)
        .header(header)
        .main(main)
        .with_descendants(sidebar_desc)
        .build_with_children()
}
```

---

### 13. `backend/crates/marionette/src/gallery.rs` (Phase 16.5 signature change)

**In-place edit at line 32:**

Before:
```rust
pub render: fn() -> Node,
```

After:
```rust
pub render: fn() -> Vec<Node>,
```

Also update the doc comment (currently lines 29-31) to say "returns a flat `Vec<Node>` — index 0 is the root, remaining are descendants. Per the pure-fn contract …".

And update the test helper `minimal_node` + `leak_entry` at lines 115-131 to wrap in `vec![...]`.

---

### 14. `backend/crates/marionette-macros/src/gallery_demo.rs` (Phase 16.5 validator change)

**In-place edit at lines 134-162.** Rename `return_type_is_node` → `return_type_is_vec_node`. New check logic:

Before (current):
```rust
fn return_type_is_node(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty
        && let Some(last) = p.path.segments.last()
    {
        return last.ident == "Node";
    }
    false
}
```

After:
```rust
fn return_type_is_vec_node(ty: &syn::Type) -> bool {
    // Accept `Vec<Node>`, `std::vec::Vec<Node>`, etc.
    let syn::Type::Path(p) = ty else { return false; };
    let Some(last) = p.path.segments.last() else { return false; };
    if last.ident != "Vec" { return false; }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else { return false; };
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else { return false; };
    let syn::Type::Path(inner_p) = inner else { return false; };
    inner_p.path.segments.last().map(|s| s.ident == "Node").unwrap_or(false)
}
```

Also update the error message at line 143-147:
```rust
"#[gallery_demo] fn must return `Vec<Node>` (an alias for \
 `Vec<(String, marionette_protocol::Component)>`)"
```

Update the call site at line 140: `if return_type_is_vec_node(ty)`.

---

### 15. `backend/crates/gallery-smoke/src/lib.rs` (Phase 16.5 `smoke` body update)

**In-place edit at line 25-27:**

Before:
```rust
pub fn smoke() -> Node {
    Text::new("gallery-smoke").build()
}
```

After:
```rust
pub fn smoke() -> Vec<Node> {
    vec![Text::new("gallery-smoke").build()]
}
```

Also update `tests/registry_roundtrip.rs` at line 22:
```rust
let _render: fn() -> Vec<marionette::gallery::Node> = smoke;
```

And update `tests/ui/fail_wrong_return.stderr` (line 1) to reference `Vec<Node>`.

---

### 16. `backend/crates/gallery-demo/Cargo.toml` (new manifest)

**Analog (primary):** `backend/crates/crm-demo/Cargo.toml` (reduced set).
**Analog (feature toggle):** `backend/crates/gallery-smoke/Cargo.toml:9` (`features = ["gallery"]`).

**Target shape:**
```toml
[package]
name = "gallery-demo"
version = "0.1.0"
edition.workspace = true
license.workspace = true
publish = false

[dependencies]
marionette = { path = "../marionette", features = ["gallery"] }
marionette-protocol = { path = "../marionette-protocol" }
marionette-macros = { path = "../marionette-macros" }
axum.workspace = true
tokio.workspace = true
tower-http.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
serde.workspace = true
serde_json.workspace = true
sea-orm.workspace = true           # MockDatabase only
uuid.workspace = true

[dev-dependencies]
tokio-tungstenite.workspace = true
futures.workspace = true
```

**Removed (vs CRM):** `sea-orm-migration`, `bcrypt`, `chrono`, `axum-extra`, `time`, `reqwest`, `wiremock`.

---

### 17. `backend/Cargo.toml` (workspace member addition)

**In-place edit** at `members = [...]` block (currently lines 3-9). Add `"crates/gallery-demo"` as the 6th member:
```toml
members = [
    "crates/marionette-protocol",
    "crates/marionette-macros",
    "crates/marionette",
    "crates/crm-demo",
    "crates/gallery-smoke",
    "crates/gallery-demo",        # Phase 17 CRATE-01
]
```

---

### 18. `Makefile` (add `gallery-dev` target)

**Analog:** the existing `dev` target (lines 3-8 of the root `Makefile`).

**Append after `dev`:**
```makefile
gallery-dev:
	@echo "Starting gallery-demo..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p gallery-demo & \
	wait
```

**Rationale (RESEARCH.md §Port Selection):** Gallery runs on 3002, serves `../frontend/build` directly (no Vite proxy). If the user wants frontend hot-reload alongside, document a second target (`gallery-dev-full`) with `npm run dev` — recommendation is defer; keep `gallery-dev` single-service for simplicity.

---

### 19. `backend/crates/marionette/GALLERY-DEMOS.md` (net-new doc)

**No analog in repo.** Use `.planning/CONCEPT.md` and `.planning/TOOLING.md` for writing-style reference only. Content is net-new per CONTEXT.md §code_context + RESEARCH.md §GALLERY-DEMOS.md contract.

**Required sections (per CONTEXT.md D-B4 + RESEARCH.md):**
1. **Contract** — `fn() -> Vec<Node>`, no args, no state, no I/O. Feature-gated by `"gallery"`. Explicit `key = "..."` required.
2. **Bind-path convention** — `/demo/{key}/...` with examples: `/demo/text-input/value`, `/demo/data-table/rows`.
3. **Action namespace** — `gallery-demo/*` for demo-fired actions. `close-modal` and `dismiss-toast` use frontend-hardcoded names.
4. **Skip list + rationale** — 7 skipped builders (SurfaceMount, NavItem, NavGroup, FieldSeparator, SideNav, Container, TableColumn-is-not-a-builder).
5. **Coverage matrix** — table of all ComponentBuilder structs with yes/no + rationale.
6. **AppShell exception** — D-A2 hand-design note.
7. **Composite-nesting rule** — D-A1 (nest other `gallery_demo()` calls where leaf shape fits).
8. **Recipe: adding a new built-in** — 3-step recipe.

---

## Shared Patterns

### Pattern A — Lint-block preamble (all new Rust files)

**Source:** `crm-demo/src/main.rs:1-2` + `gallery-smoke/src/lib.rs:13-14`.
**Apply to:** `gallery-demo/src/main.rs` and every `handlers/*.rs` module.

```rust
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
```

The per-component builder files in `marionette/src/builders/*.rs` inherit lint config from `marionette/src/lib.rs` and do NOT repeat this block.

---

### Pattern B — HandlerContext → ActionResult shape

**Source:** `crm-demo/src/handlers/contact.rs handle_dismiss_toast` (lines 1687-1711), `crm-demo/src/handlers/fetch_rows.rs handle_fetch_rows` (lines 100-158).
**Apply to:** every file in `gallery-demo/src/handlers/`.

```rust
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;

pub async fn handle_X(ctx: HandlerContext) -> ActionResult {
    // 1. Parse payload (propagate ActionError::BadPayload on missing fields)
    // 2. Do domain work (pure Vec<PatchOperation> or Vec<ProtocolMessage> construction)
    // 3. Return Ok(vec![ProtocolMessage::Patch(...)]) or Ok(vec![ProtocolMessage::Render(...)])
}
```

---

### Pattern C — Surface-target routing

**Source:** `crm-demo/src/main.rs:263-332` (shell vs content vs toasts).
**Apply to:** every gallery-demo handler that emits messages.

| Handler | Surface target | Message type |
|---------|----------------|--------------|
| `handle_navigate` | `main`, then `content`, then `toasts` | 3× `Render` (shell + home + toasts seed) |
| `handle_gallery_show` | `content` | `Render` |
| `handle_noop` | `toasts` | `Patch` (SetNode + InsertChild) |
| `handle_modal_open` / `handle_modal_close` | `modal` | `Render` |
| `handle_confirm_*` | `modal` or `content` (accept/reject clear modal) | `Render` |
| `handle_toast_fire` | `toasts` | `Patch` |
| `handle_dismiss_toast` | `toasts` | `Patch` (RemoveChild + DeleteNode) |
| `handle_demo_fetch_rows` | `content` | `Patch` (Set ops on `/demo/data-table/rows/{id}`) |

---

### Pattern D — Builder sub-tree flatten idiom

**Source:** `crm-demo/src/main.rs:360-379` (`build_login_form` — canonical flatten).
**Apply to:** `home.rs::build_home_page`, `handlers/navigate.rs`, every composite gallery_demo body.

```rust
// Every composite builder call returns two shapes that need merging into
// a single HashMap<String, Component>:
//   .build_tree()  → (root_tuple, descendants_vec)
//   .build_with_children() → Vec<(String, Component)>   ← root at index 0
//
// Merge idiom:
let (child_root, child_desc) = SubBuilder::new().children(...).build_tree();
let outer_nodes = Container::new().children(vec![..., child_root, ...])
    .build_with_children();
let mut nodes = HashMap::new();
for (id, c) in outer_nodes  { nodes.insert(id, c); }
for (id, c) in child_desc   { nodes.insert(id, c); }
```

Post-§D-Z1 (cluster 4), every `gallery_demo()` body terminates by returning `build_with_children()`'s `Vec<Node>` directly — the handler does the HashMap conversion in one `.into_iter().collect()` call.

---

### Pattern E — Gallery demo sibling annotation (all 19 in-scope builder files)

**Source:** `gallery-smoke/src/lib.rs:23-27`.
**Apply to:** every `backend/crates/marionette/src/builders/<component>.rs` file in the in-scope set (19 of 24).

**Canonical shape:**
```rust
#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "<type-string>")]
#[must_use]
pub fn gallery_demo() -> crate::gallery::Node {
    // body returns Vec<Node> (post-§D-Z1)
}
```

**Mandatory: explicit `key = "..."` (CONTEXT.md §D-C1, Phase 16 lock).** Without it, every fn (all named `gallery_demo`) would register under key `"gallery_demo"` and panic in debug builds. Use the builder's `#[component(type = "...")]` string verbatim.

**Verification grep** (Pitfall #1):
```bash
grep -rn '#\[gallery_demo' backend/crates/marionette/src/builders/ \
    | grep -v 'key =' \
    # Must be empty
```

---

### Pattern F — Default shape for builder file split (all 24 files)

**Source:** `backend/crates/marionette/src/builders/app_shell.rs:1-20` (module doc + imports + lints).

Each new per-component file uses this skeleton:
```rust
//! `<Component>` component builder + canonical `#[gallery_demo]` sibling.
//!
//! Split from `standard.rs` in Phase 17 D-B3.

// Re-use imports the struct originally needed in standard.rs (most need only
// ComponentBuilder; Select + DataTable also pull serde; composites pull neighbors.)
use marionette_macros::ComponentBuilder;
// ... selective additional uses per component ...

// [struct definition: copy verbatim from standard.rs using the line range above]

// [gallery_demo sibling if in-scope]

#[cfg(test)]
mod tests {
    use super::*;
    // [tests moved from standard.rs lines 666-1398 split by subject]
}
```

---

## No Analog Found

Files with no close match in the codebase (planner must derive from research + decisions):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `backend/crates/gallery-demo/src/handlers/show.rs` (the registry-lookup pattern) | handler | request-response | No prior handler iterates `registered_demos()` + renders per-key. Pattern is net-new in Phase 17. |
| `backend/crates/gallery-demo/src/handlers/modal.rs` (modal-open/close flow) | handler | request-response | CRM has no demo-style modal flow. Derived from `ModalSurface.svelte` semantics + RESEARCH.md §Pattern 4. |
| `backend/crates/gallery-demo/src/handlers/confirm.rs` (3-action dialog flow) | handler | request-response | Same as modal — net-new behavior. |
| Composite `gallery_demo()` fn bodies (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast) | pure builder | producer-only | Nested-call composition via other `gallery_demo()` fns is new in Phase 17 D-A1. Only the leaf shape exists in `gallery-smoke`. |
| `backend/crates/marionette/GALLERY-DEMOS.md` | authoring doc | — | No prior author-facing contract doc in the crate. |
| Phase 16.5 `DemoEntry.render: fn() -> Vec<Node>` signature change | framework micro-refactor | pure-compile-time | Phase 16 locked `fn() -> Node`; this phase changes it. |

---

## Metadata

**Analog search scope:**
- `backend/crates/crm-demo/src/` (main.rs + handlers/) — primary analog for the binary crate
- `backend/crates/crm-demo/tests/integration_test.rs` — primary analog for `tests/smoke_boot.rs`
- `backend/crates/gallery-smoke/` — template for `#[gallery_demo]` sibling shape + registry tests
- `backend/crates/marionette/src/builders/standard.rs` (all 1398 lines) — source for the per-component split
- `backend/crates/marionette/src/builders/app_shell.rs` — template for hand-written builder shape + AppShell demo content
- `backend/crates/marionette/src/gallery.rs` — Phase 16.5 signature-change target
- `backend/crates/marionette-macros/src/gallery_demo.rs` — macro validator edit site
- `backend/Cargo.toml` + `backend/crates/marionette/Cargo.toml` + `backend/crates/gallery-smoke/Cargo.toml` + `backend/crates/crm-demo/Cargo.toml` — manifest references

**Files read during mapping (10):** CONTEXT.md, RESEARCH.md (partial — ~500 of 1268 lines), crm-demo/src/main.rs (partial), crm-demo/tests/integration_test.rs (partial), gallery-smoke/src/lib.rs, gallery-smoke/tests/registry_roundtrip.rs, marionette/src/gallery.rs, marionette-macros/src/gallery_demo.rs, marionette/src/builders/standard.rs (partial), marionette/src/builders/app_shell.rs (partial), marionette/src/builders/mod.rs, marionette/src/builders/node.rs, Makefile, Cargo.toml ×4.

**Pattern extraction date:** 2026-04-22

---

## PATTERN MAPPING COMPLETE

**Phase:** 17 - Gallery Crate Skeleton + Colocated Built-in Demos
**Files classified:** 34 (14 gallery-demo crate + 24 builder-split + 4 Phase-16.5 edits + 4 workspace/doc/Makefile)
**Analogs found:** 28 / 34

### Coverage
- Files with exact analog: 19 (all 24 builder-split files with verbatim line ranges + 5 CRM copy-and-simplify sites)
- Files with role-match analog: 9 (gallery-demo main.rs, handlers, tests)
- Files with no analog: 6 (show.rs handler, modal/confirm flows, composite demo pattern, GALLERY-DEMOS.md, §D-Z1 signature change)

### Key Patterns Identified
- **AppShell slot-builder chain** — sidebar/header/footer/main/popups/toasts wired via `AppShell::new().sidebar(...).build_with_children()`; descendants collected from each slot's `build_tree()` and passed via `with_descendants()`. Exact shape from `crm-demo/src/main.rs:192-261`.
- **Axum `Router::new().route("/ws", ...).fallback_service(ServeDir::new(...))` with `MockDatabase`** — gallery-demo is a verbatim strip of CRM main.rs minus auth/DB/migrations.
- **`#[gallery_demo(key = "<type-string>")]` on every in-scope builder** — the single pattern repeated 19 times; MUST include explicit `key` to avoid collision (Phase 16 §D-C1).
- **`/demo/{key}/...` bind-path convention + `gallery-demo/*` action namespace** — new in Phase 17; documented in GALLERY-DEMOS.md.
- **Toast emit via `Button::new(...).action(dismiss-toast).build()` + `PatchOperation::{SetNode, InsertChild}` against `toasts-root`** — exact copy from `crm-demo/src/handlers/contact.rs:1645-1669`.
- **§D-Z1 `DemoEntry.render: fn() -> Vec<Node>`** — flat tree return simplifies composite demos; 4-file ripple (gallery.rs, gallery_demo.rs macro, gallery-smoke lib.rs, trybuild stderr).

### File Created
`.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can now reference analog patterns in PLAN.md files for every touched file. Six net-new patterns (listed in "No Analog Found") require the planner to derive from RESEARCH.md §Pattern 3-6 rather than copying from an analog.
