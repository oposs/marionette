# Phase 17: Gallery Crate Skeleton + Colocated Built-in Demos — Research

**Researched:** 2026-04-22
**Domain:** Rust workspace scaffolding + AppShell composition + per-component-file refactor + `linkme`-backed registry iteration
**Confidence:** HIGH (all critical findings verified against the codebase; external library versions pinned in workspace manifest)

## Summary

Phase 17 is mechanically tractable: the framework spine (Phase 16) already ships `registered_demos()`, the `#[gallery_demo]` macro, and the `gallery` feature gate verbatim. What Phase 17 adds is (a) a new 6th workspace crate `gallery-demo` that boots a thin axum server, (b) one `gallery_demo()` fn per in-scope builder, delivered via a per-component-file refactor of `builders/standard.rs`, (c) `GALLERY-DEMOS.md`, and (d) Makefile target.

Three non-obvious findings shape the plan:

1. **`AppState.db` is a hard-required `Arc<DatabaseConnection>`.** Every existing handler dispatch path assumes it exists (ws.rs:28, ws.rs:67–97 session lookup, extractors.rs:34 HandlerContext). The crm-demo integration test at `crm-demo/tests/integration_test.rs:82` already demonstrates the tested workaround: `Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection())`. Gallery-demo adopts the same pattern — it adds `sea-orm` as a dep ONLY for the mock DB constructor (no schema, no migrations, no queries). This is the minimum-invasive path; refactoring `AppState` to make `db` optional is scope-flex Phase 16.5 work and not necessary here.

2. **The frontend dispatches `navigate` on WebSocket connect** (`frontend/src/lib/routing/router.svelte.ts:27`). This gives gallery-demo the trigger for its Home render: the `navigate` handler emits the shell + the Home-page content Render. A second click of a nav tile fires `gallery-show` which patches just the `content` sub-surface. No synthetic trigger needed.

3. **The frontend hardcodes `close-modal` as the dismiss action** (`frontend/src/lib/components/popup/ModalSurface.svelte:15` and `ConfirmDialog.svelte:34`). Gallery-demo MUST register a backend handler named `close-modal` (not `gallery-demo/modal-close`) OR the gallery's Modal/Confirm X-buttons will dispatch to an unregistered action and surface an error. CONTEXT.md §D-C4 lists `gallery-demo/modal-close` as the designed name — the planner must reconcile this: either (a) register BOTH names pointing at the same handler, or (b) drop `gallery-demo/modal-close` and wire the close flow through `close-modal`. Option (a) with `close-modal` as primary is recommended — it aligns with frontend contract and reserves the `gallery-demo/*` namespace for the demo-triggered opens.

**Primary recommendation:** Treat Phase 17 as four task clusters — (1) workspace + crate skeleton + mock-DB AppState, (2) per-component file refactor with re-export shim, (3) `gallery_demo()` sibling sweep with explicit `key = "…"` annotations, (4) gallery-demo binary (nav iteration + Home page + handler set) + GALLERY-DEMOS.md + Makefile. Cluster 2 must land before Cluster 3 (sibling functions need their home files to exist); Cluster 1 and Cluster 2 can parallelize.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Area A — Demo content density:**

- **D-A1: Hybrid density — canonical leaf, substantive composite.** Leaves (Button, TextInput, Checkbox, Switch, Textarea, Select, RadioGroup, Heading, Text, Grid, Spinner, ErrorDisplay) emit 2–3 representative instances stacked in a Container. Composites (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast) emit a meaningful mini-composition via nested `gallery_demo()` calls.
- **D-A2: `AppShell::gallery_demo()` is hand-designed, not an automatic nest.** Sidebar entries + main content are hand-picked. Other composites follow DEMO-02's nested-call rule. Exception documented in GALLERY-DEMOS.md.
- **D-A3: Demo fns emit canonical action names in the `gallery-demo/*` namespace.** Buttons carry `.action(ComponentAction::submit("gallery-demo/noop"))`. TextInputs carry `.bind("/demo/{key}/value")`.
- **D-A4: Modal and ConfirmDialog render real behavior (trigger button + closed overlay).** `Modal::gallery_demo()` returns `Button("Open modal")` + a closed Modal; clicking fires `gallery-demo/modal-open` which dispatches a surface patch to open. Same shape for ConfirmDialog.

**Area B — Sweep scope + per-component-file refactor:**

- **D-B1: Visible-standalone coverage rule.** Every ComponentBuilder struct that renders meaningfully on its own gets a `#[gallery_demo]` sibling. Structural pieces are demoed transitively.
- **D-B2: Skip list** (documented in GALLERY-DEMOS.md): `SurfaceMount`, `NavItem`, `NavGroup`, `FieldSeparator`, `SideNav`, `Container`, `TableColumn`. All other ComponentBuilder structs + the hand-written `AppShell` ship demos. **In-scope list: 19 demos** — Button, TextInput, Select, Checkbox, Grid, Heading, Text, Form, Textarea, RadioGroup, Switch, FieldSet, DataTable, Modal, Toast, ConfirmDialog, Spinner, ErrorDisplay, AppShell.
- **D-B3: Per-component file refactor of `builders/`.** `builders/standard.rs` is broken into one file per `#[derive(ComponentBuilder)]` struct. Each file hosts: the struct + related props types + the `gallery_demo()` sibling (when in-scope). Public API preserved via `pub use` re-exports in `builders/mod.rs`. Structural-skip components also move to their own files.
- **D-B4: Coverage is documented, not CI-enforced.** GALLERY-DEMOS.md ships a coverage matrix; GALLERY-LINT is v1.3+.

**Area C — Gallery-demo binary:**

- **D-C1: Flat alphabetical nav by `DemoEntry.key`.** No grouping metadata.
- **D-C2: Curated Home page on first visit.** Hand-authored welcome Heading + explanatory Text + Grid of tiles derived from `registered_demos()`.
- **D-C3: Single `gallery-show` action routes every nav click.** Payload `{ key: "<demo-key>" }`. Handler: extract key → `registered_demos().find(|e| e.key == key)` → invoke `e.render()` → seed `/demo/{key}/...` state → Render targeting `content`.
- **D-C4: Small set of purpose-built demo actions registered at gallery-demo startup**: `gallery-show`, `gallery-demo/noop`, `gallery-demo/modal-open`, `gallery-demo/modal-close`, `gallery-demo/confirm-open`, `gallery-demo/confirm-accept`, `gallery-demo/confirm-reject`, `gallery-demo/toast-fire`.

**Area D — Stateful fixtures:**

- **D-D1: Minimal per-demo seeds.** DataTable 5–10 rows (Phase 18 CAT-03 takes to ≥500; Phase 19 EXER-03 to ≥10 000). Form 2–3 bound fields. Modal/Confirm closed; Toast empty queue on visit.
- **D-D2: Bind-path convention `/demo/{key}/...`** for all demo fns.
- **D-D3: Demos render real interactive behavior where it's cheap.** TextInput updates `/demo/text-input/value`, Switch `/demo/switch/checked`, etc.
- **D-D4: Toast demo seeds an initial queue entry** + exposes "Fire toast" Button.

### Claude's Discretion

- Port number (3002 recommended), Makefile target name.
- `GALLERY-DEMOS.md` exact location (`backend/crates/marionette/GALLERY-DEMOS.md` recommended).
- `standard.rs` disposition after refactor (retire vs re-export shim).
- Related props-struct placement (colocate with parent recommended).
- Home page tile rendering shape.
- Whether Home tile list is registry-derived or hand-curated (registry-derived recommended).
- Exact noop-handler toast message shape.
- Title-casing / `name = "..."` overrides per demo.
- DataTable seed-row helper (local vs shared).
- Action-registration boilerplate style (inline vs helper fn; helper fn recommended at ~10 actions).
- Modal/ConfirmDialog target sub-surface (`modal` sub-surface recommended, matching real usage).
- Deep-link URL hash behavior.
- Whether `gallery-smoke`'s Cargo.toml needs adjustment (default: no).

### Deferred Ideas (OUT OF SCOPE)

- Grouping metadata on `DemoEntry` — deferred to v1.3+ if Phase 18/19 UX reveals need.
- `GALLERY-LINT` CI lint — v1.3+.
- Deep-link URL hash handling — scope-flex if time permits.
- Shared synthetic-data generator across Phases 17/18/19.
- `ActionRouter` fallback/wildcard handler capability — rejected.
- Framework-level composition machinery for composite demos — rejected.
- Auto-generated screenshots / documentation from demos — v1.3+.
- Theme editor — Phase 20.
- Live reactivity of the gallery on new demo addition during `cargo run` — impossible by design.
- Third-party (non-marionette) demo-registering crates — v1.3+.
- Standalone demos for `SideNav` / `Container` / structural pieces.
- Noop handler toast message richness.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRATE-01 | New `backend/crates/gallery-demo/` workspace member with thin backend scaffolding: no auth, no DB, no migrations — in-memory `Arc<RwLock<_>>` state only. `cargo run -p gallery-demo` boots on its own port. | §Gallery-demo binary skeleton (§Architecture Patterns); §AppState Compatibility — use `MockDatabase` to satisfy the required `db: Arc<DatabaseConnection>` field without schema/migrations. Port 3002. |
| CRATE-02 | Gallery `main.rs` builds AppShell nav by iterating the auto-discovered demo registry — no hand-maintained menu list. Adding a new `#[gallery_demo]` anywhere in the workspace automatically surfaces it on next build. | §Gallery AppShell Construction — concrete sketch iterates `registered_demos()` producing one NavItem per entry, flat alphabetical. |
| DEMO-01 | Every existing built-in component in `backend/crates/marionette/src/builders/` ships a sibling `pub fn gallery_demo() -> Node` annotated with `#[gallery_demo]`. | §Per-Component File Refactor — enumerates all 19 in-scope builders with line ranges and refactor mechanics. |
| DEMO-02 | Demo contract enforced by convention + `GALLERY-DEMOS.md`: pure `fn() -> Node`, no external state, no I/O, no fixtures. Composite demos built by calling other `gallery_demo()` functions directly. | §GALLERY-DEMOS.md Contract — canonical doc shape; §Per-Demo Content Design — exact signatures per builder. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

No `./CLAUDE.md` file in `/home/oetiker/checkouts/marionette`. Global `~/.claude/CLAUDE.md` adds one relevant rule: `NEVER run find /home/oetiker` — not applicable to Phase 17 (planner uses targeted paths).

Auto-memory feedback that applies:

- `feedback_pre_deployment_no_backcompat.md` — no back-compat shims; fix root causes. **Applies to the per-component refactor**: import paths change cleanly. The `standard.rs` re-export shim recommendation below is a clean-path choice, not a deprecation alias — it preserves a well-chosen public API surface (see Per-Component File Refactor §Option A).
- `feedback_options_need_reasoning.md` — options get pros/cons/rationale. Applied throughout §Architecture Patterns.
- `feedback_no_handrolling_ui.md` — adopt framework recipes. Applies to the Home page: use existing `Grid` / `Heading` / `Container` / `Button` primitives, don't invent new patterns.
- `feedback_use_chrome_for_uat.md` — Chrome MCP drives UAT checkpoints. Applies to Success Criterion #5 verification — see §Validation Architecture §UAT.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Nav iteration from registry | Backend (gallery-demo binary) | — | `registered_demos()` is a backend API; nav tree is built into the Render payload. |
| Home page composition | Backend (gallery-demo `navigate` handler) | — | Emitted as a Render into the `content` sub-surface. No frontend code changes. |
| Per-demo state seeding | Backend (`gallery-show` handler) | In-memory `Arc<RwLock<_>>` AppState | Seed written to the `data` field of the Render payload so frontend data store hydrates `/demo/{key}/...` paths. |
| Demo rendering | `gallery_demo()` fns (marionette crate, feature-gated) | Backend registry | Pure Rust fn emitting `(String, Component)`. |
| Modal open/close | Backend (`gallery-demo/modal-open` + `close-modal` handlers) | Frontend (ModalSurface reads `modal` sub-surface) | Open = Render into `modal` sub-surface; close = Render with empty-tree or `ClearSurface` semantics. Frontend's ModalSurface.svelte:15 hardcodes dispatch of `close-modal`. |
| Toast dispatch | Backend (`gallery-demo/toast-fire` handler) | Frontend (ToastSurface) | Pattern from `crm-demo/src/handlers/contact.rs:1626–1675` — `PatchMessage` on `toasts` surface with `SetNode` + `InsertChild` against `toasts-root`. |
| Per-component file refactor | Marionette crate (builders module) | — | Pure code-organization; no tier crossing. |
| Static file serving | Backend (axum + `tower-http::ServeDir`) | — | Reuses `frontend/build/` identical to crm-demo. |

## Standard Stack

### Core (workspace-locked; no changes)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | 0.8 (workspace) | HTTP routing + WebSocket upgrade | [VERIFIED: `backend/Cargo.toml:20`] Same version crm-demo uses. |
| `tokio` | 1 (full features) | Async runtime + `RwLock` | [VERIFIED: `backend/Cargo.toml:19`] Required for axum. `RwLock` is `tokio::sync::RwLock` (not `std::sync::RwLock`) — async-aware. |
| `tower-http` | 0.6 (fs, cors) | `ServeDir` + `ServeFile` SPA fallback | [VERIFIED: `backend/Cargo.toml:21`] Reused verbatim from `crm-demo/src/main.rs:609–611`. |
| `tracing` + `tracing-subscriber` | 0.1 / 0.3 | Structured logging | [VERIFIED: workspace deps] Pattern: `tracing_subscriber::fmt::init()` in `main()`. |
| `serde_json` | 1 | Payload + seed data | [VERIFIED: workspace deps]. |
| `linkme` | 0.3 (optional) | Registry backbone — transitively through `marionette/features=gallery` | [VERIFIED: Phase 16 landed]. Not a direct dep of gallery-demo. |
| `sea-orm` | 1.1 (sqlx-sqlite, mock) | `MockDatabase` for `AppState.db` placeholder | [VERIFIED: `backend/Cargo.toml:25`, `crm-demo/tests/integration_test.rs:82`]. **Unusual but locked by AppState shape** — see §AppState Compatibility. Gallery-demo uses `DatabaseBackend::Sqlite` mock only; zero SQL queries. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `marionette` | path dep, features=["gallery"] | Registry + builders + ws/router | Mandatory. |
| `marionette-protocol` | path dep | `ProtocolMessage`, `PatchOperation`, `Component` | Mandatory. |
| `marionette-macros` | path dep | `#[gallery_demo]` macro | Used by every `gallery_demo()` fn (imported via re-export from `marionette`). |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `MockDatabase::new(DatabaseBackend::Sqlite).into_connection()` | Refactor `AppState::db` to `Option<Arc<DatabaseConnection>>` | Framework-level refactor with ws.rs session-auth path changes; scope creep into marionette-core. Mock DB is a 2-line workaround with zero runtime overhead (no queries issued). |
| `MockDatabase` | In-memory SQLite (`sqlite::memory:`) | Wastes time initializing a real DB for no reason. Mock DB returns immediately without any query plan. |
| Register `close-modal` in gallery-demo | Change frontend to dispatch `gallery-demo/modal-close` | Frontend change is out of scope per CONTEXT.md §domain. Backend-only path is cleaner. |
| Direct `Arc<RwLock<SomeState>>` in handler closures | Pass state via axum's `with_state` extension | `with_state` is already attached to AppState; adding a parallel state-plumbing layer is redundant. Store gallery-local mutable state as a field inside a custom `GalleryState` wrapper cloned into a closure at registration time. |

**Installation (gallery-demo Cargo.toml, recommended shape):**

```toml
[package]
name = "gallery-demo"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
marionette = { path = "../marionette", features = ["gallery"] }
marionette-protocol = { path = "../marionette-protocol" }
marionette-macros = { path = "../marionette-macros" }
axum.workspace = true
tokio.workspace = true
tower-http.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
serde_json.workspace = true
sea-orm.workspace = true  # for MockDatabase only — zero queries
```

**Version verification:**

All versions are locked in `backend/Cargo.toml` workspace deps [VERIFIED: file read at 2026-04-22]. No new versions to pick.

## Architecture Patterns

### System Architecture Diagram

```
 Browser (frontend/build/)                Backend (gallery-demo :3002)
 ──────────────────────                   ────────────────────────────
                                            main()
    WebSocket connect                         ├─ MockDatabase (stub)
    ───────────────────► /ws ─────────►      ├─ GalleryState (Arc<RwLock<..>>)
                                              ├─ ActionRouter
    navigate action ─────────────────►        │    ├─ "navigate"       ─► handle_navigate
                                              │    ├─ "gallery-show"   ─► handle_gallery_show
                                              │    ├─ "gallery-demo/noop"       ─► handle_noop
                                              │    ├─ "gallery-demo/modal-open" ─► handle_modal_open
                                              │    ├─ "close-modal"    ─► handle_modal_close   (frontend contract)
                                              │    ├─ "gallery-demo/confirm-*" ─► handle_confirm_*
                                              │    └─ "gallery-demo/toast-fire" ─► handle_toast_fire
                                              │
    ◄─── Render(main: AppShell)              handle_navigate ─┐
    ◄─── Render(content: Home)                                │
                                                              │  uses:
    User clicks nav tile ─────────────►                       ▼
    gallery-show { key: "button" } ─►    handle_gallery_show
                                              ├─ find entry: registered_demos().find(|e| e.key == key)
                                              ├─ invoke: entry.render()   ◄── calls gallery_demo() fn
                                              │                                (linkme-registered, from marionette/builders/{X}.rs)
                                              ├─ seed /demo/{key}/... state
                                              └─ return Render(content: ...)
    ◄─── Render(content: demo tree)
                                              ┌─────────────────────────────────┐
    User clicks "Open modal" ───►          handle_modal_open                     │
    gallery-demo/modal-open { key } ►      ├─ build Modal component        ◄─────┤
                                              ├─ return Render(modal: ...)       │
    ◄─── Render(modal: Modal tree)                                                │
                                              ┌─────────────────────────────────┐
    User clicks X / Cancel ─────────►     handle_modal_close                     │
    close-modal ─────────────────►         └─ return Render(modal: empty-tree)   │
                                                                                  │
 Static file fallback:                                                            │
    GET /                    ──► tower-http::ServeDir("../frontend/build")        │
    GET /app.css             ──► same                                             │
    GET /anything-else       ──► ServeFile("../frontend/build/index.html") (SPA) ─┘
```

### Recommended Project Structure

```
backend/crates/gallery-demo/
├── Cargo.toml
└── src/
    ├── main.rs              # tokio::main, AppState, Router, ActionRouter wiring
    ├── state.rs             # GalleryState (Arc<RwLock<_>>) struct
    ├── home.rs              # Home-page construction fn build_home_page()
    ├── handlers.rs          # mod handlers; re-exports
    └── handlers/
        ├── navigate.rs      # handle_navigate (emits shell + home)
        ├── show.rs          # handle_gallery_show (registry lookup + render)
        ├── noop.rs          # handle_noop (toast fires naming source)
        ├── modal.rs         # handle_modal_open + handle_modal_close (close-modal)
        ├── confirm.rs       # handle_confirm_{open,accept,reject}
        └── toast.rs         # handle_toast_fire + handle_dismiss_toast
```

One file per handler family is preferred over a monolithic `handlers.rs` at ~10 handlers. Mirrors `crm-demo/src/handlers/` layout (contact.rs, company.rs, etc.).

### Pattern 1: Gallery AppShell Construction (answers research focus §3)

**What:** Iterate `registered_demos()` inside `handle_navigate` to build nav items; mirror crm-demo's slot wiring.

**Where:** `backend/crates/gallery-demo/src/handlers/navigate.rs`

**Code sketch:**

```rust
// Source: adapted verbatim from crm-demo/src/main.rs:130–335 (handle_navigate)
// + registered_demos() iteration from marionette::gallery

use marionette::gallery::registered_demos;
use marionette::builders::{AppShell, Container, Grid, Heading, NavItem, SideNav, SurfaceMount, Text};
use marionette_protocol::{ComponentAction, ProtocolMessage, RenderMessage};

pub async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    // -- Sidebar: one NavItem per registered demo (flat alphabetical; D-C1) --
    let nav_items: Vec<(String, marionette_protocol::Component)> = registered_demos()
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

    // -- Header: app title only (no user menu in gallery per domain §NOT) --
    let header_title = Heading::new("Marionette Gallery").id("header-title").build();
    let (header_root, header_desc) = Container::new()
        .id("shell-header")
        .children(vec![header_title])
        .build_tree();

    // -- Footer: version literal + connection status (mirrors crm-demo D-B6) --
    let footer_version = Heading::new("Marionette Gallery · v1.2").id("footer-version").build();
    let footer_status = Heading::new("connected")
        .id("footer-connection-status")
        .bind("/system/connectionStatus")
        .build();
    let (footer_root, footer_desc) = Container::new()
        .id("shell-footer")
        .children(vec![footer_version, footer_status])
        .build_tree();

    // -- Sub-surface mounts: content, modal, toasts (mirrors crm-demo) --
    let content_mount = SurfaceMount::new("content").id("shell-content-mount").build();
    let modal_mount = SurfaceMount::new("modal").id("shell-modal-mount").build();
    let toasts_mount = SurfaceMount::new("toasts").id("shell-toasts-mount").build();

    // -- Assemble AppShell --
    let mut descendants = Vec::new();
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

    // -- Shell Render + Home-page Render + toasts-root seed Render --
    let (shell_map, shell_data) = flatten_and_seed(shell_nodes);
    let (home_root_id, home_nodes_map, home_data) = build_home_page();
    let toasts_seed = build_toasts_seed();

    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: None, surface: "main".into(),
            root: "app-shell-root".into(),
            nodes: shell_map, data: shell_data,
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None, surface: "content".into(),
            root: home_root_id, nodes: home_nodes_map, data: home_data,
        }),
        ProtocolMessage::Render(RenderMessage {
            id: None, surface: "toasts".into(),
            root: "toasts-root".into(), nodes: toasts_seed,
            data: serde_json::json!({}),
        }),
    ])
}
```

**Question (a) — SideNav + NavItems or direct AppShell.sidebar slot?** Answer: **use SideNav**. The CRM does this at `crm-demo/src/main.rs:192–195`; `AppShell::sidebar()` accepts any `(id, Component)` tuple but the frontend's `AppShell.svelte` mounts the sidebar node and delegates child rendering. SideNav is the documented container for NavItems.

**Question (b) — header/footer minimums:** CRM's header uses title + user menu (`crm-demo/src/main.rs:198–205`). Gallery skips user menu (no auth). Footer mirrors CRM's version + connection-status pattern but drops legal. `footer-connection-status` Heading bound to `/system/connectionStatus` is required if the frontend's connection-banner mechanism expects it (Phase 12 D-B6; defensive keep).

**Question (c) — content / modal / toasts mount:** Exact copy of CRM pattern. No variations.

### Pattern 2: Home Page Construction (answers research focus §4)

**What:** The `navigate` handler emits the Home page as the initial content Render. No synthetic trigger needed — the frontend already dispatches `navigate` on WS connect (`frontend/src/lib/routing/router.svelte.ts:27`).

**Trigger path:** `navigate` fires once on connect → handler emits shell + home content. Subsequent `gallery-show` actions replace ONLY the content surface (shell stays).

**Code sketch:**

```rust
// backend/crates/gallery-demo/src/home.rs
use marionette::builders::{Button, Container, Grid, Heading, Text};
use marionette::gallery::registered_demos;
use marionette_protocol::{Component, ComponentAction};

pub fn build_home_page() -> (String, std::collections::HashMap<String, Component>, serde_json::Value) {
    let welcome = Heading::new("Marionette Gallery")
        .id("home-welcome")
        .level(1)
        .build();
    let intro = Text::new(
        "Visual-iteration harness and SDUI-frontend exerciser. \
         Pick a component from the sidebar to see its gallery demo.",
    ).id("home-intro").build();

    // Tile per registered demo (Claude's discretion: registry-derived).
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

    let (grid_root, grid_desc) = Grid::new()
        .id("home-grid").cols(3).gap("1rem")
        .children(tiles)
        .build_tree();

    let root_id = "home-root".to_string();
    let outer_nodes = Container::new()
        .id(&root_id)
        .children(vec![welcome, intro, grid_root])
        .build_with_children();

    let mut map = std::collections::HashMap::new();
    for (id, c) in outer_nodes { map.insert(id, c); }
    for (id, c) in grid_desc { map.insert(id, c); }

    (root_id, map, serde_json::json!({}))
}
```

### Pattern 3: `gallery-show` Handler (answers research focus §5)

**What:** Single entry-point for every nav click. Payload extraction + registry lookup + state seed + Render.

**Payload shape:** Inline `{"key": "button"}`, dispatched via `ComponentAction::click("gallery-show").with_payload(...)`. Follows CRM's `contact_country_change` payload shape — top-level payload keys, not wrapped in an `action-data` envelope.

**Seed strategy:** **Central seed registry is recommended** over match arms in the handler. Rationale: (1) one look-up table keeps seeds discoverable, (2) the handler stays ≤20 lines, (3) empty seeds are expressed as `json!({})` rather than handler branching.

**Code sketch:**

```rust
// backend/crates/gallery-demo/src/handlers/show.rs
use marionette::gallery::registered_demos;
use marionette_protocol::{ProtocolMessage, RenderMessage};

pub async fn handle_gallery_show(ctx: HandlerContext) -> ActionResult {
    let key = ctx.action.payload.as_ref()
        .and_then(|p| p.get("key")).and_then(|v| v.as_str())
        .ok_or_else(|| ActionError::BadPayload("missing 'key'".into()))?;

    let entry = registered_demos().find(|e| e.key == key)
        .ok_or_else(|| ActionError::NotFound(format!("gallery demo '{key}'")))?;

    let (root_id, node) = (entry.render)();
    let mut nodes_map = std::collections::HashMap::new();
    // Leaves return `(id, Component)` directly; composites may return a root
    // node whose children references are already collected inside the nodes
    // map. Unified path: always insert the returned tuple; when composites
    // use `.build_with_children()`, they collapse inside their own gallery_demo
    // body and return just the root — but the returned `node` has its
    // `children` populated. Research open question: does `gallery_demo()`
    // return ONLY the root or the flattened tree? See Open Question #1 below.
    nodes_map.insert(root_id.clone(), node);

    let data = seed_for_key(key);

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        root: root_id,
        nodes: nodes_map,
        data,
    })])
}

fn seed_for_key(key: &str) -> serde_json::Value {
    match key {
        "button" | "heading" | "text" | "spinner" | "error-display" => serde_json::json!({}),
        "text-input" => serde_json::json!({ "demo": { "text-input": { "value": "" } } }),
        "select" => serde_json::json!({ "demo": { "select": { "value": "" } } }),
        "checkbox" => serde_json::json!({ "demo": { "checkbox": { "checked": false } } }),
        "switch" => serde_json::json!({ "demo": { "switch": { "checked": false } } }),
        "radio-group" => serde_json::json!({ "demo": { "radio-group": { "value": "" } } }),
        "textarea" => serde_json::json!({ "demo": { "textarea": { "value": "" } } }),
        "form" => serde_json::json!({ "demo": { "form": { "email": "", "name": "" } } }),
        "field-set" => serde_json::json!({ "demo": { "field-set": { "a": "", "b": "" } } }),
        "grid" => serde_json::json!({}),
        "data-table" => serde_json::json!({ "demo": { "data-table": { "rows": seed_table_rows() } } }),
        "modal" => serde_json::json!({}),
        "toast" => serde_json::json!({}),
        "confirm-dialog" => serde_json::json!({}),
        "app-shell" => serde_json::json!({}),
        _ => serde_json::json!({}),
    }
}

fn seed_table_rows() -> serde_json::Value {
    // 5-10 synthetic rows per D-D1
    serde_json::json!([
        {"id": 1, "name": "Alice Baker", "email": "alice@example.com", "created": "2026-01-05"},
        {"id": 2, "name": "Bob Chen",    "email": "bob@example.com",   "created": "2026-01-08"},
        {"id": 3, "name": "Carol Davis", "email": "carol@example.com", "created": "2026-01-12"},
        {"id": 4, "name": "Dan Evans",   "email": "dan@example.com",   "created": "2026-01-15"},
        {"id": 5, "name": "Eva Frost",   "email": "eva@example.com",   "created": "2026-01-20"},
    ])
}
```

**CRITICAL open question about `gallery_demo()` return shape — see Open Question #1 below. The `handle_gallery_show` sketch above assumes `gallery_demo()` returns a single flat `(String, Component)` where composites use `build_with_children()` internally to produce a root whose children references are embedded via the `children` prop. If composites need a flat node map, the handler and the `gallery_demo()` signatures both need to adapt.**

### Pattern 4: Modal / ConfirmDialog / Toast Handlers (answers research focus §6)

**What the CRM shows:** CRM opens/closes modal by rendering/clearing the `modal` sub-surface tree. Toasts use `PatchMessage` with `SetNode` + `InsertChild` against a pre-seeded `toasts-root`.

**Model for gallery-demo:**

```rust
// handlers/modal.rs — gallery-demo/modal-open
pub async fn handle_modal_open(ctx: HandlerContext) -> ActionResult {
    // Build a concrete Modal component to render into the `modal` sub-surface.
    let modal_root_id = "demo-modal-root".to_string();
    let modal_title = Heading::new("Example modal").id("demo-modal-title").build();
    let modal_body = Text::new("Clicking X or outside dismisses via close-modal.")
        .id("demo-modal-body").build();
    let modal_nodes = Modal::new("Example modal")
        .id(&modal_root_id)
        .children(vec![modal_title, modal_body])
        .build_with_children();

    let mut map = HashMap::new();
    for (id, c) in modal_nodes { map.insert(id, c); }

    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: modal_root_id,
        nodes: map,
        data: serde_json::json!({}),
    })])
}

// handlers/modal.rs — close-modal  (⚠ NAME MATTERS: frontend hardcodes it)
pub async fn handle_modal_close(ctx: HandlerContext) -> ActionResult {
    // Clear the modal sub-surface by rendering an empty container as root.
    // Frontend's ModalSurface detects tree !== undefined to decide open/closed —
    // the cleanest close is to render a container with no children (isOpen will
    // remain true unless we can clear the tree; see Risk §R-3 below).
    //
    // Alternative: emit a PatchMessage DeleteNode for the modal root id AND
    // the children, but surfaces.svelte.ts has no "clearSurfaceTree"
    // semantics via the wire protocol — only Render replaces a tree, and
    // the frontend never sees an "empty" Render used as "no tree".
    //
    // Simplest working path: render an empty Container at a stable "empty"
    // root. Frontend reads `rootProps` from the tree's root; with a plain
    // empty Container the Dialog still shows briefly but with no content.
    // Chrome-MCP UAT catches this — see Risk §R-3.
    let empty_root = Container::new().id("demo-modal-empty").build();
    let (root_id, component) = empty_root;
    let mut map = HashMap::new();
    map.insert(root_id.clone(), component);
    Ok(vec![ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: root_id,
        nodes: map,
        data: serde_json::json!({}),
    })])
}

// handlers/toast.rs — gallery-demo/toast-fire
// Mirrors crm-demo/src/handlers/contact.rs:1626–1675 exactly.
pub async fn handle_toast_fire(_ctx: HandlerContext) -> ActionResult {
    let toast_id = format!("demo-toast-{}", uuid::Uuid::new_v4());
    let (_id, toast_node) = Button::new("Demo toast from gallery-demo/toast-fire")
        .id(&toast_id)
        .action(ComponentAction::click("dismiss-toast"))
        .build();
    let ops = vec![
        PatchOperation::SetNode { id: toast_id.clone(), component: toast_node },
        PatchOperation::InsertChild {
            parent: "toasts-root".into(), index: 0, child_id: toast_id,
        },
    ];
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: None, surface: "toasts".into(), patch: ops,
    })])
}
```

**Answer to research focus §6 "how does CRM currently open a Modal":** CRM does NOT have a Modal-demo use case; it has only the country-change toast demo. But the mechanism for opening a Modal (from domain reasoning + `crm-demo/src/handlers/contact.rs:1626–1675`) is **render a tree into the `modal` sub-surface**. Frontend's `ModalSurface.svelte:8` derives `isOpen` from `tree !== undefined`. The `Dialog.Root` shadcn component wrapping the sub-surface respects `open` prop; sending a Render with a populated tree makes it appear.

### Pattern 5: Per-Component File Refactor (answers research focus §2)

**Struct inventory in `backend/crates/marionette/src/builders/standard.rs` (verified 2026-04-22):**

| Struct | Lines | `#[component(type = "...")]` | Target file | Related props structs | In-scope demo? |
|--------|-------|-----------------------------|-------------|-----------------------|----------------|
| `Button` | 11–21 | `"button"` | `button.rs` | — | yes |
| `TextInput` | 23–46 | `"text-input"` | `text_input.rs` | — | yes |
| `SelectOption` | 49–53 | (not a builder — plain struct) | `select.rs` | colocate with Select | — |
| `RadioOption` | 61–67 | (plain struct) | `radio_group.rs` | colocate with RadioGroup | — |
| `Select` | 69–97 | `"select"` | `select.rs` | hosts `SelectOption` | yes |
| `Checkbox` | 99–116 | `"checkbox"` | `checkbox.rs` | — | yes |
| `Container` | 120–125 | `"container"` | `container.rs` | — | **no (skip list)** |
| `Grid` | 127–134 | `"grid"` | `grid.rs` | — | yes |
| `Heading` | 138–144 | `"heading"` | `heading.rs` | — | yes |
| `Text` | 146–150 | `"text"` | `text.rs` | — | yes |
| `SideNav` | 154–156 | `"side-nav"` | `side_nav.rs` | — | **no (skip list)** |
| `NavItem` | 158–165 | `"nav-item"` | `nav_item.rs` | — | **no (skip list)** |
| `NavGroup` | 167–171 | `"nav-group"` | `nav_group.rs` | — | **no (skip list)** |
| `SurfaceMount` | 181–185 | `"surface-mount"` | `surface_mount.rs` | — | **no (skip list)** |
| `Form` | 189–194 | `"form"` | `form.rs` | — | yes |
| `Textarea` | 202–226 | `"textarea"` | `textarea.rs` | — | yes |
| `RadioGroup` | 235–253 | `"radio-group"` | `radio_group.rs` | hosts `RadioOption` | yes |
| `Switch` | 262–277 | `"switch"` | `switch.rs` | — | yes |
| `FieldSet` | 292–309 | `"field-set"` | `field_set.rs` | — | yes |
| `FieldSeparator` | 315–317 | `"field-separator"` | `field_separator.rs` | — | **no (skip list)** |
| `TableColumn` + `impl` | 320–370 | (plain struct + impl) | `data_table.rs` | colocate with DataTable | — |
| `ColumnKind` (enum) | 374–388 | — | `data_table.rs` | colocate | — |
| `Filter` (enum) + `impl` | 394–486 | — | `data_table.rs` | colocate | — |
| `DataTable` + `impl DataTableBuilder` | 488–532 | `"data-table"` | `data_table.rs` | hosts TableColumn/ColumnKind/Filter | yes |
| `Modal` | 536–542 | `"modal"` | `modal.rs` | — | yes |
| `Toast` | 544–552 | `"toast"` | `toast.rs` | — | yes |
| `ConfirmDialog` | 554–559 | `"confirm-dialog"` | `confirm_dialog.rs` | — | yes |
| `Spinner` | 561–566 | `"spinner"` | `spinner.rs` | — | yes |
| `ErrorDisplay` | 568–572 | `"error-display"` | `error_display.rs` | — | yes |
| `form_shell()` fn | 620–664 | — | `composites.rs` (new file) | — | — (composite helper) |
| `tests` mod | 666–1398 | — | Split per-module into `#[cfg(test)] mod tests` inside each new file | — | — |

**Total new files: 24** (19 component files + `composites.rs` for `form_shell()` + `mod.rs` rewrite — plus the existing `app_shell.rs` and `node.rs` untouched except for the `gallery_demo()` addition to `app_shell.rs`).

**Test disposition:** The 700+ lines of tests at lines 666–1398 of `standard.rs` split naturally by subject — `button_builder` test belongs in `button.rs`, `text_input_*` in `text_input.rs`, etc. The `all_19_standard_types` test at lines 1079–1110 stays in a new `builders/mod.rs` `#[cfg(test)] mod tests { ... }` block as a "one place that proves every builder still works" meta-test (update the asserted list if any builder is renamed — none should be).

**External callers audit (verified 2026-04-22 via `grep -rn 'builders::standard' backend/`):**

| Caller | File:Line | Import |
|--------|-----------|--------|
| crm-demo | `src/main.rs:19` | `use marionette::builders::standard::{Button, Container, Form, Heading, NavItem, SideNav, SurfaceMount, TextInput};` |
| crm-demo | `src/handlers/audit.rs:6` | `use marionette::builders::standard::{...}` |
| crm-demo | `src/handlers/company.rs:6` | `use marionette::builders::standard::{...}` |
| crm-demo | `src/handlers/contact.rs:9` | `use marionette::builders::standard::{...}` |
| crm-demo | `src/handlers/interaction.rs:4` | `use marionette::builders::standard::{...}` |
| crm-demo | `src/handlers/user.rs:6` | `use marionette::builders::standard::{...}` |
| crm-demo | `tests/integration_test.rs:11` | `use marionette::builders::standard::{Button, Container, Heading, Text};` |
| gallery-smoke | `src/lib.rs:16` | `use marionette::builders::standard::Text;` |
| gallery-smoke | `tests/ui/fail_wrong_signature.rs:1` | `use marionette::builders::standard::Text;` |
| gallery-smoke | `tests/ui/fail_not_pub.rs:1` | `use marionette::builders::standard::Text;` |
| marionette (internal) | `src/builders/app_shell.rs:244` | `use crate::builders::standard::{Container, Heading, SideNav, SurfaceMount};` |

**Refactor option A (recommended): Keep `standard.rs` as a re-export shim.** `builders/mod.rs` declares `pub mod button; pub mod text_input; ...` (all 24 files) and keeps `pub mod standard;` where `standard.rs` reduces to `pub use super::{button::*, text_input::*, select::*, ...};`. Pros: zero external caller changes; pattern permits future deprecation at chosen pace. Cons: two import paths (`builders::Button` and `builders::standard::Button`) both work — cosmetically redundant. **This matches `feedback_pre_deployment_no_backcompat.md` as a clean choice, NOT a back-compat shim**, because the re-exports are the intended public API; the old `standard` name is being kept by design.

**Refactor option B: Retire `standard.rs`, update all callers.** `builders/mod.rs` declares per-component modules and `pub use` everything into `builders::`. Update all 10 external call-sites + `app_shell.rs:244` to `use marionette::builders::{Button, ...};`. Pros: clean-cut, single import path. Cons: 10 caller edits for no functional gain. **Planner's choice per CONTEXT.md §Claude's Discretion.**

**Recommendation: Option A.** Lower change surface, zero risk of test regressions outside the refactor, and the `standard` name is not stigmatized — it's a description of "the standard 19 components". Phase 17 is already the biggest refactor of v1.2; don't chain an import-churn phase onto it.

**Cross-struct dependencies inside `standard.rs` (relevant for placement):**

- `Select` uses `Vec<SelectOption>` — both in `select.rs`.
- `RadioGroup` uses `Vec<RadioOption>` — both in `radio_group.rs`.
- `DataTable` uses `Vec<TableColumn>`, `Vec<Filter>`; `Filter::Select` uses `SelectOption`; `TableColumn` uses `ColumnKind` — `data_table.rs` imports `SelectOption` from `select.rs` via `use super::select::SelectOption;`.
- The `form_shell()` helper fn (lines 620–664) imports `Container`, `Heading`, `Button`, `Form` — move to `composites.rs` with `use super::{button::Button, container::Container, form::Form, heading::Heading};`.
- No `impl` blocks on one struct that reference another struct's types beyond the above.

**No shared internal helper fns** exist in `standard.rs` other than `form_shell()`; the split is cleanly delineated.

### Pattern 6: `gallery_demo()` Sibling Fn Shape

**Canonical shape (leaf example: Button):**

```rust
// backend/crates/marionette/src/builders/button.rs
use marionette_macros::ComponentBuilder;
// ... struct Button definition unchanged ...

#[cfg(feature = "gallery")]
use marionette_macros::gallery_demo;
#[cfg(feature = "gallery")]
use crate::gallery::Node;
#[cfg(feature = "gallery")]
use crate::builders::container::Container;

/// Canonical demo for `Button` — shows default + disabled + destructive.
#[cfg(feature = "gallery")]
#[gallery_demo(key = "button")]
pub fn gallery_demo() -> Node {
    let a = Button::new("Primary")
        .action(marionette_protocol::ComponentAction::submit("gallery-demo/noop"))
        .build();
    let b = Button::new("Disabled").disabled(true).build();
    let c = Button::new("Destructive")
        .variant("destructive")
        .action(marionette_protocol::ComponentAction::submit("gallery-demo/noop"))
        .build();
    let nodes = Container::new()
        .children(vec![a, b, c])
        .build_with_children();
    // Container::build_with_children() returns Vec<(String, Component)> with
    // the root at index 0. We need a single (id, Component) — return the root.
    // The root's Component.children is populated with the 3 child ids, but
    // the children's Component definitions themselves are lost from the
    // caller's view unless we pre-flatten them into the returned Component.
    // ⚠ THIS IS THE OPEN QUESTION — see Open Question #1 below.
    let (root_id, root_comp) = nodes.into_iter().next().expect("container has root");
    (root_id, root_comp)
}
```

**Why explicit `key = "button"` matters (Phase 16 §D-C1 lock):** Without the override, every `fn gallery_demo()` would register under `key = "gallery_demo"` and collide at runtime. The macro panics in debug builds on duplicate keys (`gallery.rs:62–67`) — uncaught, this bites at `cargo test` time. Planner MUST set the explicit key on every one of the 19 annotations. Recommended convention: key matches `#[component(type = "…")]` string verbatim (e.g., `"text-input"`, `"data-table"`, `"confirm-dialog"`) so nav links are predictable.

**Canonical shape (composite example: Form, answers D-A1):**

```rust
// backend/crates/marionette/src/builders/form.rs
#[cfg(feature = "gallery")]
#[gallery_demo(key = "form")]
pub fn gallery_demo() -> Node {
    let email = crate::builders::text_input::gallery_demo();   // nested call per D-A1
    let name = crate::builders::text_input::gallery_demo();    // ⚠ collision: both demos return the same ids (UUID-generated) so this actually works, but the two calls return structurally identical sub-trees
    let (form_root, form_desc) = Form::new()
        .children(vec![email, name])  // This only works if nested gallery_demo returns a single (id, Component) — see Open Question #1
        .build_tree();
    // ... collapse descendants into root ...
    form_root
}
```

**Demo fn placement — CRITICAL issue:** `crate::builders::text_input::gallery_demo()` fn is `#[cfg(feature = "gallery")]`-gated. For a sibling in `form.rs` (also gallery-gated) to call it at compile-time, BOTH sites must have the feature active. Under `cargo build -p gallery-demo` this is fine (gallery-demo enables the feature transitively). Under `cargo build -p marionette` (default, gallery off) nothing compiles — matches the spec.

### Anti-Patterns to Avoid

- **DO NOT default-derive keys.** Every `#[gallery_demo]` in Phase 17 MUST set `key = "..."` explicitly. Debug builds of any consumer will panic at registry initialization on duplicates.
- **DO NOT add grouping metadata to `DemoEntry`.** CONTEXT.md §deferred. Phase 17 consumes Phase 16 shape verbatim.
- **DO NOT build a `gallery-demo` AppState by hand-rolling a new type.** Reuse `marionette::ws::AppState` with the mock-DB workaround; otherwise ws-handler integration diverges.
- **DO NOT use `std::sync::RwLock`.** Use `tokio::sync::RwLock` — async-aware, no deadlocks with the tokio runtime.
- **DO NOT hand-roll the Home page layout beyond existing builders.** `feedback_no_handrolling_ui.md` — Container + Grid + Button tiles is the recipe.
- **DO NOT register `close-modal` under the `gallery-demo/*` namespace.** Frontend hardcodes `"close-modal"` at `ModalSurface.svelte:15` and `ConfirmDialog.svelte:34`. Either name doesn't survive both contracts.
- **DO NOT create a `gallery` cargo feature on gallery-demo itself.** It's an application, not a lib; its sole purpose is enabling `marionette/features=["gallery"]`. `gallery-smoke` has one, mirror that shape if feature propagation is desired.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AppState without DB | `pub struct GalleryAppState { router, state }` | `marionette::ws::AppState` with `MockDatabase` | ws.rs session-auth path assumes `AppState.db` exists; refactoring it is scope creep. |
| Custom Modal show/hide transport | `Open`/`Close` protocol message types | Render → `modal` sub-surface to open; `close-modal` action wired to render empty root | Frontend already reads `modal` sub-surface tree; no new wire protocol needed. |
| Nav groupings | `DemoEntry { ..., group: Option<&'static str> }` | Flat alphabetical | CONTEXT.md §deferred; ~25 entries reads fine flat. |
| Synthetic row generator | `gallery-demo/src/fixtures.rs` with Faker-style generators | 5–10 hand-written rows in `handlers/show.rs::seed_table_rows` | Phase 17 seeds are tiny; Phase 18 will need a shared generator — extract then. |
| Home tile list | Hand-maintained vec of `("button", "Button")` tuples | `registered_demos().map(...)` | Registry-derived stays in sync automatically. |
| `fn() -> Node` return shape for composites | `fn() -> Vec<(String, Component)>` | Keep `fn() -> Node` + put composite nesting into the `Component.children` prop pointing at pre-embedded ids | Protocol contract locked Phase 16. |

**Key insight:** Every demo-handler primitive already has a proven pattern in the CRM codebase. Phase 17 is copy-and-simplify, not new-design.

## Runtime State Inventory

Phase 17 is **a greenfield crate + a file refactor**, not a rename/migration. No runtime state survives to bite the refactor. For completeness:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — gallery-demo has no database; state is in-memory `Arc<RwLock<_>>`. | None. |
| Live service config | None — the Makefile adds a new target; no existing service re-registers. | None. |
| OS-registered state | None — gallery-demo is a new binary; no pm2 / launchd / Task Scheduler entries exist yet. | None (fresh binary, launched via `cargo run -p gallery-demo`). |
| Secrets/env vars | None — gallery-demo reads no environment variables (no `LISTMONK_URL`, no `DATABASE_URL`). | None. |
| Build artifacts | None to migrate — `gallery-demo` is new, nothing stale exists. `marionette`'s `target/` will rebuild on `gallery` feature toggle. | None. |

**Nothing found in any category** — verified by inspecting `gallery-smoke`'s Cargo.toml (which defines the crate shape being mirrored) and confirming `backend/Cargo.toml` holds no secrets or env hints. Refactor in cluster 2 only moves source files; no persistence, no registered state.

## Per-Demo Content Design (answers research focus §8)

For each in-scope component, the `gallery_demo()` body outline:

| Component | Demo body (2–3 lines each) | Sub-surface target |
|-----------|---------------------------|-------------------|
| `Button` | 3 Buttons (default / disabled / destructive) stacked in Container. Each carries `gallery-demo/noop`. | `content` |
| `TextInput` | 3 TextInputs (default, disabled, with-description) stacked in Container. All bind `/demo/text-input/value`. | `content` |
| `Select` | 2 Selects (default, disabled) stacked in Container. Options hardcoded fruit list. Binds `/demo/select/value`. | `content` |
| `Checkbox` | 3 Checkboxes (unchecked, checked, disabled). Each binds `/demo/checkbox/agree-N`. | `content` |
| `Grid` | 2×3 Grid of Heading placeholders showing the layout. | `content` |
| `Heading` | 3 Headings (h1, h2, h3) stacked in Container. | `content` |
| `Text` | 3 Text blocks (short, paragraph, technical) stacked in Container. | `content` |
| `Form` | Form containing nested `TextInput::gallery_demo()` + `Select::gallery_demo()` + Submit Button → `gallery-demo/noop`. Per D-A1 nested pattern. | `content` |
| `Textarea` | 2 Textareas (default, with description). Binds `/demo/textarea/value`. | `content` |
| `RadioGroup` | RadioGroup with 3 options. Binds `/demo/radio-group/value`. | `content` |
| `Switch` | 2 Switches (off, on). Binds `/demo/switch/checked-N`. | `content` |
| `FieldSet` | FieldSet with 2-col grid containing nested `TextInput::gallery_demo()` + `Select::gallery_demo()`. Per D-A1. | `content` |
| `DataTable` | DataTable with 3–5 columns + 5–10 rows from `seed_table_rows()`. **NOTE: rows arrive via the `data` field in the Render payload, NOT embedded in the Component props.** | `content` |
| `Modal` | **"Open modal" Button (gallery-demo/modal-open) + closed Modal node.** Click opens via Render to `modal` sub-surface. | trigger on `content`, opens into `modal` |
| `Toast` | Fire toast Button (`gallery-demo/toast-fire`) + static Heading "Example toast demo". | `content` — dispatches to `toasts` sub-surface |
| `ConfirmDialog` | "Open confirm" Button (`gallery-demo/confirm-open`) + closed ConfirmDialog. Click opens. | trigger on `content`, opens into `modal` |
| `Spinner` | 3 Spinners (sm, md, lg). | `content` |
| `ErrorDisplay` | 2 ErrorDisplays with example messages. | `content` |
| `AppShell` | Hand-designed curated shell — see D-A2. Mini-nav with 3 fake NavItems, hand-picked title in header, Text block in main. **NOT rendered as the outer shell — this demo shows AppShell composed inside `content` as a nested example.** | `content` (nested) |

**⚠ DataTable critical note (answers research focus §8 row-fixture question):** Reading `standard.rs:488–515`, `DataTable` has no `rows` prop — its rows come from `fetch-rows` action dispatched by the frontend via the `source` prop. For a demo without a backend `fetch-rows` handler implementation for `"demo-rows"`, the DataTable would render empty or spin forever.

**Three resolution paths:**

1. **Seed `/demo/data-table/rows`** in the `gallery-show` handler's `data` field. If the frontend's DataTable reads rows from a bind-path, this works. **VERIFY:** check `frontend/src/lib/components/table/DataTable.svelte` — it uses `source` + `fetch-rows`, not bind-path. So this path doesn't work without frontend changes.
2. **Register a `fetch-rows` handler in gallery-demo** that responds to `source = "demo-rows"` with the seeded rows. Mirrors `crm-demo/src/handlers/fetch_rows.rs` pattern; ~30 lines of code.
3. **DataTable demo shows headers + empty body** and an explanatory Text note beside it ("Rows load via fetch-rows in real use — Phase 18 CAT-03 seeds ≥500 rows"). Lowest-cost demo path.

**Recommendation:** **Option 2** — register a gallery-demo-scoped `fetch-rows` handler. Completes the demo's "feels alive" bar at low cost (~30 LOC), matches how real DataTables work, and is directly reusable by Phase 18 CAT-03. Planner confirms.

**Modal / ConfirmDialog sub-surface answer (research focus §7):** Modal and ConfirmDialog demos render a trigger-Button into `content` (as part of the `gallery-show` Render). The Button's `gallery-demo/modal-open` handler emits a Render targeting `modal`. The two sub-surfaces are independent — no AppShell slot rebuilding. When a different demo is shown next, `gallery-show` Renders `content` (not `modal`), so any stale Modal tree persists. Planner may add a defensive `close-modal` Render at the start of every `gallery-show` handler to clear leftovers. **Recommended: yes, include defensive close** — cost is negligible, prevents Modal ghosts.

## Port Selection + Makefile Target (answers research focus §9)

- **Port 3002.** CRM is 3001; gallery-demo 3002. No conflict; easy mental model.
- **Makefile target:** `gallery-dev` (recommended name; mirrors `dev`). Shape:

```makefile
gallery-dev:
	@echo "Starting gallery-demo + frontend..."
	@trap 'kill 0' EXIT; \
	cd backend && cargo run -p gallery-demo & \
	cd frontend && npm run dev & \
	wait
```

- **Frontend-build dependency:** Frontend `npm run dev` runs Vite; Vite proxies `/api` and `/ws` to port 3001 by default (STACK.md line 51). Gallery-demo's port 3002 requires either (a) Vite proxy reconfigure, or (b) running gallery-demo without the Vite dev server, serving only the prebuilt `frontend/build/`. **Option (b) is simplest** — `gallery-dev` target runs only `cargo run -p gallery-demo`; `frontend/build/` is already prebuilt and served by `ServeDir`. Document in GALLERY-DEMOS.md: "To hack frontend + gallery simultaneously, run `cd frontend && npm run build` after each change; or set up a parallel Vite proxy (follow-up concern)."

## GALLERY-DEMOS.md Contract (answers DEMO-02)

**Location recommendation:** `backend/crates/marionette/GALLERY-DEMOS.md` (crate-level doc, sibling to `Cargo.toml`). This makes it discoverable via `cargo doc` and `cargo metadata`, and matches crate-top-level docs convention.

**Structure (sections the planner writes):**

1. **Contract** — pure `fn() -> Node`, no args, no state, no I/O. Composites nest other `gallery_demo()` calls. AppShell is hand-designed (exception).
2. **Bind-path convention** — `/demo/{key}/...` for all demo fns. Examples.
3. **Action namespace** — `gallery-demo/*` for demo-fired actions. Registered in `gallery-demo/src/main.rs`. The `close-modal` action uses the frontend-hardcoded name.
4. **Skip list + rationale** — the 7 skipped builders (SurfaceMount, NavItem, NavGroup, FieldSeparator, SideNav, Container, TableColumn) and why.
5. **Coverage matrix** — table: component → demo yes/no → rationale-if-skipped.
6. **Adding a new built-in component** — 3-step recipe: (1) add ComponentBuilder struct in `builders/{new_component}.rs`, (2) add `pub fn gallery_demo() -> Node` sibling with `#[gallery_demo(key = "new-component")]`, (3) re-export from `builders/mod.rs` if needed. `cargo run -p gallery-demo` auto-surfaces it in nav.

## Common Pitfalls

### Pitfall 1: Default-key collision across sibling `gallery_demo` fns

**What goes wrong:** If any of the 19 `#[gallery_demo]` annotations omits `key = "..."`, the macro defaults to `fn_ident.to_string()` — every sibling's fn is named `gallery_demo`, so they all register under the key `"gallery_demo"` and the registry panics in debug builds with "duplicate #[gallery_demo] key" at `gallery.rs:62`.

**Why it happens:** Phase 16's macro design uses `fn_ident` as the default key. The Phase 17 convention of colocation forces every demo fn to be named `gallery_demo` (sibling convention). The mismatch is the trap.

**How to avoid:** **MECHANICAL CHECK:** every `#[gallery_demo]` annotation in `backend/crates/marionette/src/builders/` MUST have `key = "..."`. The key value MUST match the builder's `#[component(type = "...")]` string. A one-line grep after the sweep verifies: `grep -rn '#\[gallery_demo' backend/crates/marionette/src/builders/ | grep -v 'key ='` MUST return empty.

**Warning signs:** `cargo test -p marionette --features gallery` panics at `sort_entries_duplicate_panics_in_debug` path. Panic message names both display_names at the colliding key.

### Pitfall 2: `AppState.db` is required, not optional

**What goes wrong:** Naive attempt to instantiate `AppState { router, db: None, ... }` fails to compile because `db: Arc<sea_orm::DatabaseConnection>` is mandatory at `ws.rs:28`.

**Why it happens:** Framework assumed every app has a DB. Gallery-demo breaks the assumption.

**How to avoid:** Use `MockDatabase::new(DatabaseBackend::Sqlite).into_connection()` wrapped in `Arc::new(...)`. Pattern from `crm-demo/tests/integration_test.rs:82`. Add `sea-orm.workspace = true` to gallery-demo's `Cargo.toml` (feature `"mock"` is already in workspace config).

**Warning signs:** Compile error "expected `Arc<DatabaseConnection>`, found `()`".

### Pitfall 3: Frontend `close-modal` hardcode

**What goes wrong:** Planner registers `gallery-demo/modal-close` only. User clicks the X on the Modal or the Cancel button on ConfirmDialog. Frontend dispatches `close-modal`. Backend router returns `ActionError::NotFound("close-modal")`. Error banner appears in UI.

**Why it happens:** Frontend pops don't read CONTEXT.md. They hardcode the action name in `ModalSurface.svelte:15` and `ConfirmDialog.svelte:34`.

**How to avoid:** Register `close-modal` as a backend action that clears the `modal` sub-surface. Optionally also register `gallery-demo/modal-close` as an alias (same handler) if CONTEXT.md §D-C4 naming must be honored literally.

**Warning signs:** Clicking Cancel on ConfirmDialog demo produces an error toast; Chrome-MCP UAT catches it on Success Criterion #5.

### Pitfall 4: `build_tree()` vs `build_with_children()` mismatch in `gallery_demo()` bodies

**What goes wrong:** `gallery_demo()` returns `fn() -> Node` (a single tuple). Composite builders like `Grid::new().children(...).build_with_children()` return `Vec<Node>`. A naïve `.into_iter().next()` grabs only the root — the child Components are lost, and the `handle_gallery_show` handler inserts only the root into the nodes map. Frontend renders a Grid with no cells.

**Why it happens:** The `Node` type alias fights the composite protocol shape.

**How to avoid:** Three candidate resolutions — see **Open Question #1** below.

**Warning signs:** Demo renders as an empty Container; Chrome MCP UAT catches "no visible child nodes".

### Pitfall 5: Hyphenated keys through the macro

**What goes wrong:** `#[gallery_demo(key = "data-table")]` — hyphen in the string literal. Macro parses via `darling::FromMeta` which accepts any string; the emitted `static __GALLERY_DEMO_gallery_demo: DemoEntry { key: "data-table", ... }` — OK at macro level, but the static ident `__GALLERY_DEMO_gallery_demo` is unique per fn, so this is fine. Darling's string parsing handles it verbatim (not an identifier).

**Why it happens (or doesn't):** Macro verified: `opts.key.unwrap_or_else(|| fn_ident.to_string())` at `gallery_demo.rs:54` — key is stored as a plain string, no ident validation.

**How to avoid:** Non-issue; macro handles hyphens correctly. Documented here for Phase 17 planner confidence.

**Warning signs:** None expected. If it ever fails, error would be `key is not a valid rust identifier` at macro-expand time — doesn't happen per `darling`'s behavior.

## Code Examples

Verified patterns from the codebase:

### Example 1: Gallery-demo main.rs skeleton

```rust
// Source: adapted from crm-demo/src/main.rs:390–632 (minus auth/DB/seed/migration paths)
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod handlers;
mod home;
mod state;

use std::sync::Arc;
use axum::Router;
use sea_orm::{Database, DatabaseBackend, MockDatabase};
use tower_http::services::{ServeDir, ServeFile};
use marionette::router::{box_handler, ActionRouter};
use marionette::ws::{ws_handler, AppState};
use marionette_protocol::common::AuthRequirement;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // MockDatabase satisfies AppState.db without real DB (Pitfall #2).
    let db: Arc<sea_orm::DatabaseConnection> =
        Arc::new(MockDatabase::new(DatabaseBackend::Sqlite).into_connection());

    let action_router = register_gallery_actions(ActionRouter::new());

    let state = Arc::new(AppState {
        router: action_router,
        db,
        login_form: None,
        listmonk: None,
    });

    let serve_dir = ServeDir::new("../frontend/build")
        .fallback(ServeFile::new("../frontend/build/index.html"));

    let app = Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .fallback_service(serve_dir)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    tracing::info!("gallery-demo listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn register_gallery_actions(router: ActionRouter) -> ActionRouter {
    router
        .action("navigate", box_handler(handlers::navigate::handle_navigate), AuthRequirement::None)
        .action("gallery-show", box_handler(handlers::show::handle_gallery_show), AuthRequirement::None)
        .action("gallery-demo/noop", box_handler(handlers::noop::handle_noop), AuthRequirement::None)
        .action("gallery-demo/modal-open", box_handler(handlers::modal::handle_modal_open), AuthRequirement::None)
        .action("gallery-demo/modal-close", box_handler(handlers::modal::handle_modal_close), AuthRequirement::None)
        .action("close-modal", box_handler(handlers::modal::handle_modal_close), AuthRequirement::None) // frontend contract
        .action("gallery-demo/confirm-open", box_handler(handlers::confirm::handle_confirm_open), AuthRequirement::None)
        .action("gallery-demo/confirm-accept", box_handler(handlers::confirm::handle_confirm_accept), AuthRequirement::None)
        .action("gallery-demo/confirm-reject", box_handler(handlers::confirm::handle_confirm_reject), AuthRequirement::None)
        .action("gallery-demo/toast-fire", box_handler(handlers::toast::handle_toast_fire), AuthRequirement::None)
        .action("dismiss-toast", box_handler(handlers::toast::handle_dismiss_toast), AuthRequirement::None)
        // Optional: fetch-rows for DataTable demo (Pattern 5 option 2)
        .action("fetch-rows", box_handler(handlers::fetch_rows::handle_demo_fetch_rows), AuthRequirement::None)
}
```

### Example 2: Minimum `gallery_demo()` sibling — canonical leaf

```rust
// backend/crates/marionette/src/builders/spinner.rs  (leaf, trivial)
use marionette_macros::ComponentBuilder;

#[derive(ComponentBuilder)]
#[component(type = "spinner")]
pub struct Spinner {
    #[builder(optional)]
    pub size: Option<String>,
}

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "spinner")]
pub fn gallery_demo() -> crate::gallery::Node {
    let a = Spinner::new().size("sm").build();
    let b = Spinner::new().size("md").build();
    let c = Spinner::new().size("lg").build();
    // Stack in Container via build_with_children → then hoist root.
    let nodes = crate::builders::container::Container::new()
        .children(vec![a, b, c])
        .build_with_children();
    let mut iter = nodes.into_iter();
    let root = iter.next().expect("container root exists");
    // ⚠ descendants are lost here — see Open Question #1
    root
}
```

### Example 3: Noop handler — fires toast naming source demo

```rust
// backend/crates/gallery-demo/src/handlers/noop.rs
use marionette_protocol::{Component, ComponentAction, PatchMessage, PatchOperation, ProtocolMessage};

pub async fn handle_noop(ctx: HandlerContext) -> ActionResult {
    let source = ctx.action.source.as_deref().unwrap_or("unknown");
    let toast_label = format!("Demo action from {source}");
    let toast_id = format!("toast-noop-{}", uuid::Uuid::new_v4());
    let (_, toast_node) = marionette::builders::Button::new(&toast_label)
        .id(&toast_id)
        .action(ComponentAction::click("dismiss-toast"))
        .build();
    let ops = vec![
        PatchOperation::SetNode { id: toast_id.clone(), component: toast_node },
        PatchOperation::InsertChild {
            parent: "toasts-root".into(), index: 0, child_id: toast_id,
        },
    ];
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(), surface: "toasts".into(), patch: ops,
    })])
}
```

### Example 4: Flattening composite descendants into nodes map (Pitfall #4 + Open Question #1)

```rust
// Helper: flatten build_with_children() return into (root_id, HashMap) for
// Render payload. Accepts Vec<(String, Component)> where index 0 is root.
fn flatten_root(nodes: Vec<(String, marionette_protocol::Component)>)
    -> (String, std::collections::HashMap<String, marionette_protocol::Component>)
{
    let mut iter = nodes.into_iter();
    let (root_id, root_comp) = iter.next().expect("root exists");
    let mut map = std::collections::HashMap::new();
    map.insert(root_id.clone(), root_comp);
    for (id, c) in iter { map.insert(id, c); }
    (root_id, map)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| All components in one `standard.rs` | One file per ComponentBuilder | Phase 17 (this phase) | Per-component demos colocate naturally; file grows linearly with components. |
| Hand-maintained nav lists | `registered_demos()` iteration | Phase 17 (CRATE-02) | New components auto-surface. |
| `inventory` vs `linkme` tradeoff | `linkme` winner | Phase 16 (Phase 17 consumes) | Type-safe, zero-runtime-cost registry. |
| One Cargo workspace member per concept | 6 members (5th is `gallery-smoke`, 6th is `gallery-demo`) | Phase 16 + 17 | REQUIREMENTS.md §CRATE-01 "5th" wording requires reconciliation. |

**Deprecated/outdated:**

- `backend/crates/marionette/src/builders/standard.rs` — split into per-component files; file may be retained as a re-export shim (Option A) or removed (Option B). Planner's call.
- CRM-style monolithic `handlers.rs` in `crm-demo` — gallery-demo adopts per-handler-family files from day one (Pattern 5 recommends a cleaner layout).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build | ✓ | pinned via `mise.toml` | — |
| `rustc` | Compile | ✓ | Rust 1.93+ | — |
| `npm` + `frontend/build/` | Static file serving | ✓ | committed in repo | — |
| `linkme` | Registry (transitive) | ✓ | workspace 0.3 | — |
| `sea-orm` | MockDatabase | ✓ | workspace 1.1 (feature `mock` included) | — |
| `tokio` (full features) | Runtime + `sync::RwLock` | ✓ | workspace 1 | — |
| `tower-http` (fs features) | `ServeDir` / `ServeFile` | ✓ | workspace 0.6 | — |
| Chrome / claude-in-chrome MCP | UAT verification of SC #5 | assumed ✓ (per `feedback_use_chrome_for_uat.md`) | — | manual click-through if MCP unavailable |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None — every dep is already in the workspace.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (workspace-native). Frontend: vitest for component tests; Playwright for E2E. |
| Config file | `backend/Cargo.toml` (test targets auto-discovered). No `cargo test` config file. |
| Quick run command | `cd backend && cargo test -p gallery-demo` |
| Full suite command | `cd backend && cargo test --workspace --features gallery` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| CRATE-01 | `gallery-demo` workspace member exists; `cargo run -p gallery-demo` compiles and boots on :3002 | Integration | `cargo build -p gallery-demo --release && cargo test -p gallery-demo --test smoke_boot` | ❌ Wave 0 — new `tests/smoke_boot.rs` |
| CRATE-01 | No auth / no DB — thin backend | Grep assertion | `grep -L 'sea_orm::Migrator\|bcrypt' backend/crates/gallery-demo/src/main.rs` | ❌ Wave 0 |
| CRATE-01 | In-memory `Arc<RwLock<_>>` state only | Code inspection (plan-check) | `grep -rn 'Arc<RwLock' backend/crates/gallery-demo/src/state.rs` | ❌ Wave 0 |
| CRATE-02 | Nav built at runtime from `registered_demos()` | Integration | `cargo test -p gallery-demo --test nav_auto_discovery` — spins up the app, dispatches `navigate`, asserts the returned shell Render's sidebar contains one NavItem per `registered_demos()` entry | ❌ Wave 0 |
| CRATE-02 | Adding new `#[gallery_demo]` auto-surfaces | Manual + integration | Manual: add a scratch demo in a feature branch, rebuild, observe. Automated: `#[test] fn new_demo_auto_surfaces` in gallery-demo adds a demo via a test-only crate, re-runs nav assert | ❌ Wave 0 (integration test) |
| DEMO-01 | Every in-scope builder has a `gallery_demo` sibling | Static assertion | `cargo test -p marionette --features gallery --lib gallery::builtin_coverage` — a test that iterates expected keys from a const list (19 of them) against `registered_demos()` | ❌ Wave 0 |
| DEMO-01 | Skipped builders have no `gallery_demo` | Static assertion (same test, negative check) | Same test asserts skipped keys NOT present in registry | ❌ Wave 0 |
| DEMO-02 | Pure-fn contract (enforced by macro) | Macro-level (already enforced) | `cargo test -p gallery-smoke --test ui_errors` — trybuild fixtures catch async / args / generics already | ✅ Phase 16 provides |
| DEMO-02 | `GALLERY-DEMOS.md` exists and documents contract | File-exists + content check | `[ -f backend/crates/marionette/GALLERY-DEMOS.md ] && grep -q 'pure fn\|no I/O' backend/crates/marionette/GALLERY-DEMOS.md` | ❌ Wave 0 |
| SC #5 | Every nav entry produces a screen, not an error surface | Chrome MCP UAT | Manual-automated: Chrome MCP drives `navigate` → for each `registered_demos()` key, clicks the tile, asserts no `ErrorMessage` in console, asserts content-surface non-empty | manual (Chrome MCP) |

### Sampling Rate

- **Per task commit:** `cd backend && cargo test -p gallery-demo` (quick)
- **Per wave merge:** `cd backend && cargo test --workspace --features gallery`
- **Phase gate:** Full suite green + Chrome MCP walks every nav entry with zero errors

### Wave 0 Gaps

- [ ] `backend/crates/gallery-demo/tests/smoke_boot.rs` — covers CRATE-01 (boot path, port binding, static file serving)
- [ ] `backend/crates/gallery-demo/tests/nav_auto_discovery.rs` — covers CRATE-02 (nav iteration from registry)
- [ ] `backend/crates/marionette/src/gallery.rs` — add a `#[cfg(all(test, feature = "gallery"))] mod builtin_coverage_tests` that asserts the expected set of built-in keys is registered (DEMO-01). Separate from the `gallery-smoke` tests to keep gallery-smoke's 1-entry assumption intact.
- [ ] `backend/crates/marionette/GALLERY-DEMOS.md` — covers DEMO-02 docs
- [ ] Chrome MCP walk script / session-captured record — documents SC #5 coverage (not automated yet; Chrome MCP UAT per `feedback_use_chrome_for_uat.md`)

### UAT

**Method:** Chrome MCP / claude-in-chrome tools drive the browser through each nav entry. For each key in `registered_demos()`:

1. Click the NavItem for `key`
2. Wait for content-surface Render
3. Assert no `ErrorMessage` appears in dispatcher console
4. Screenshot for visual review

**Why Chrome MCP over automation:** Success Criterion #5 is visual — "produces a screen, not an error surface" — and Phase 17 is the first interactive Gallery UI. Automating this with Playwright would require either (a) harness setup for the full WebSocket+frontend loop, or (b) smoke-level boot-only testing that doesn't actually render demos. Chrome MCP lets us tick every nav entry with visual confirmation in one pass; Phases 18/19 will add per-screen Playwright assertions.

## Security Domain

`security_enforcement` presumed enabled (absent in `.planning/config.json`). The gallery has no auth (out-of-scope per REQUIREMENTS.md) and no DB — the attack surface is minimal.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Gallery has no auth (per REQUIREMENTS.md §Out of Scope). |
| V3 Session Management | no | `WsSession::user_id` is always `None`; no session state is stored or retrieved. |
| V4 Access Control | yes (framework-level) | All gallery actions registered `AuthRequirement::None`. No admin-only or authenticated-only handlers. |
| V5 Input Validation | yes | `gallery-show` payload extracts `key` as `&str`; `registered_demos().find(...)` returns `None` for unknown keys → `ActionError::NotFound`. Zero trust that the key is valid — the match-failure path is the validator. |
| V6 Cryptography | no | No secrets, no tokens, no crypto in gallery-demo. |

### Known Threat Patterns for axum + tokio + sea-orm (gallery)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Arbitrary action name dispatch | Spoofing | `ActionRouter` looks up by exact name; unknown actions return `ActionError::NotFound`. Verified at `router.rs:51–54`. |
| Registry poisoning via 3rd-party crate registering demos | Tampering | `linkme` is compile-time; only crates with `dep: marionette-macros` can register. 3rd-party registration is out-of-scope per REQUIREMENTS.md. |
| DoS via large payload in `gallery-show` | DoS | axum default body-size limits apply (~2 MB). Gallery payloads are tiny `{key: "..."}` — well within. |
| Path traversal via `../frontend/build` ServeDir | Info Disclosure | `tower-http::ServeDir` normalizes paths; traversal to sibling paths is blocked. Verified by CRM's identical usage. |
| Concurrent write to `Arc<RwLock<_>>` state (race) | DoS | `tokio::sync::RwLock` serializes writers; concurrent readers are safe. Pattern-standard. |

## Open Questions

### Open Question #1 — Composite `gallery_demo()` return shape [HIGH priority]

**What we know:** The `Node` type alias is `(String, Component)` — a single tuple. `build_with_children()` returns `Vec<(String, Component)>`. The `#[gallery_demo]` macro enforces `fn() -> Node` return type (macro `return_type_is_node` at `gallery_demo.rs:155`).

**What's unclear:** How does a composite gallery demo like `Grid` — which needs 6 nested cells — return a single `Node` that the Render handler can use?

**Three candidate answers:**

1. **A single root Component with children embedded in `children: Vec<String>` + the full flat tree built by calling another demo-internal helper that the handler uses.** Requires the composite demo to return a flat *Vec* internally, then pack everything except the root into the children's `Component.children` by id-reference, hoping the Render handler constructs the full nodes map by traversal. **Doesn't work with current `fn() -> Node` signature.**

2. **Relax the contract: demo return type is `impl IntoNodeBundle` or similar trait.** Requires Phase 16 macro surgery + API changes. **Out of scope per CONTEXT.md §domain.**

3. **Build the full sub-tree inside the `gallery_demo()` body, flatten, and return only the root `(String, Component)` with descendant refs in `children`.** Descendants are *lost* from the caller's view unless the caller *reconstructs* them. This is how leaf + composite demos already work at `gallery-smoke/src/lib.rs:25–27` (leaf case, `Text::new(...).build()` returns one tuple with children = None).

**Recommendation:** **Path 3 with a registered helper.** The composite `gallery_demo()` body calls a separate `pub fn gallery_demo_tree() -> Vec<Node>` (feature-gated, exported from the same file) that returns the full flat tree. The `gallery_demo()` fn itself calls `gallery_demo_tree()`, grabs the first tuple as root, and returns it — **but the handler must also call `gallery_demo_tree()` to get the full nodes map.**

This has one big issue: the macro-emitted `DemoEntry::render: fn() -> Node` field stores only a `Node`-returning pointer. The handler can't access `gallery_demo_tree()` without knowing the demo's key-specific function name.

**Better resolution: extend `DemoEntry` to carry a `tree: fn() -> Vec<Node>` field.** This IS a Phase 16 framework change. Out of scope.

**Workable resolution (Planner's call):** Every composite `gallery_demo()` body internally flattens descendants and returns a single Container root whose `Component.children` references ids that ARE present in the containing Component's `children` array — BUT because only the root Component is returned, the descendant Components are lost. The handler's `nodes` HashMap would contain only the root. **This is broken.**

**THIS IS A BLOCKER. The planner MUST resolve before implementation.** Candidates:

1. **Change `DemoEntry` to carry `render_tree: fn() -> Vec<Node>`** — requires Phase 16 macro edit. Planner re-opens CONTEXT.md §domain.
2. **Embed full descendants in the Component via a new transport shape** — out of scope (framework change).
3. **Leaf-only demos:** composites (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast, AppShell) skip the `Container` wrapping and return ONLY THEIR ROOT NODE. All children are authored as sub-bindings at runtime. Drops D-A1 composite density.
4. **Change the `Node` alias to `(String, Component, Vec<Node>)`** — breaks Phase 16 signature.
5. **Add a `pub fn <name>_tree() -> Vec<Node>` convention** — bolted-on pattern not enforced by the macro; handler reads it via key-to-fn mapping (type-erased lookup in a separate `phf_map!` or hashmap).

**Recommendation for planner**: Raise with user. Most promising path without framework change is **(3) reduced density for composites** (only leaf demos stay full-fidelity) OR adopt **(5) with a pre-Wave-0 task** to land a `DemoEntry { render: fn() -> Vec<Node> }` signature change in Phase 16.5 (micro-refactor). The current `registered_demos()` consumers (gallery-smoke + yet-to-be-built gallery-demo) are the only affected code sites.

**⚠ THIS IS THE LARGEST OPEN QUESTION. The planner CANNOT implement DEMO-01 for composites until this is resolved. Leaf demos (12 of 19) compile and ship under the current contract.**

### Open Question #2 — Modal close semantics [MEDIUM priority]

**What we know:** `ModalSurface.svelte:8` reads `isOpen = tree !== undefined`. Clearing the tree requires either (a) a Render with an empty tree (ambiguous — does Frontend treat `root: "some-id"` with one empty Component as "open" or "closed"?) or (b) a new `ClearSurface` wire op that doesn't exist.

**What's unclear:** Does rendering a Container with no children at a "closed" root make `isOpen` false, or does any Render keep the tree defined and thus `isOpen` true?

**Recommendation:** The planner tests this early. Proposed test: in `backend/crates/gallery-demo/tests/modal_close.rs`, spin up the app, dispatch `gallery-demo/modal-open`, then `close-modal`, and assert the frontend `modal` surface state is empty (via Playwright browser test). If `close-modal` can't clear the tree, planner opens a Phase 16.5 side-task to add a `ClearSurface` op or equivalent. Alternative workaround: close-modal renders a zero-height invisible Container — visually closed, logically present.

### Open Question #3 — DataTable rows via bind-path [MEDIUM priority]

**What we know:** `DataTable`'s frontend uses `source: string` + dispatches `fetch-rows` actions for data. The demo path option (Pattern 5 option 2) requires registering a `fetch-rows` handler.

**What's unclear:** Does the frontend's DataTable also accept inline `rows: []` in its Component props as a fallback, allowing pure-fn demos?

**Recommendation:** Inspect `frontend/src/lib/components/table/DataTable.svelte` early in Phase 17 Wave 0. If inline rows are supported (even experimentally), demos can embed fixture rows in the Component props directly. Otherwise, gallery-demo ships a `fetch-rows` handler for source `"demo-rows"` returning the seeded row set.

### Open Question #4 — REQUIREMENTS.md §CRATE-01 wording reconciliation [LOW priority, cosmetic]

**What we know:** REQUIREMENTS.md §CRATE-01 says "5th Cargo workspace entry"; Phase 16's `gallery-smoke` took slot 5; gallery-demo is the 6th.

**What's unclear:** Update REQUIREMENTS.md or accept gallery-smoke as a "test-fixture crate not counted in the ordinal"?

**Recommendation:** Planner updates REQUIREMENTS.md line 19 from "5th" to "6th" with a brief annotation: "(the 5th is `gallery-smoke`, a permanent test-fixture crate)". Trivial doc edit.

## Assumptions Log

Every claim in this research is [VERIFIED] against the codebase or [CITED] from an existing planning document. No unverified training-knowledge claims are load-bearing in this research. **Assumptions Log: empty — no user confirmation needed for research content.**

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | (none) | — | — |

## Sources

### Primary (HIGH confidence)

- `backend/crates/marionette/src/ws.rs:23–49` — AppState struct + ws_handler (verified 2026-04-22)
- `backend/crates/marionette/src/gallery.rs:25–104` — DemoEntry + registered_demos (Phase 16 landed)
- `backend/crates/marionette-macros/src/gallery_demo.rs:22–151` — proc macro implementation
- `backend/crates/marionette/src/builders/standard.rs:1–1398` — full builder inventory with line ranges
- `backend/crates/marionette/src/builders/app_shell.rs:1–379` — AppShell builder + slot patterns
- `backend/crates/crm-demo/src/main.rs:130–335` — handle_navigate template for gallery-demo
- `backend/crates/crm-demo/src/handlers/contact.rs:1470–1711` — Toast + Modal patterns
- `backend/crates/crm-demo/tests/integration_test.rs:67–102` — MockDatabase AppState shape
- `backend/crates/gallery-smoke/src/lib.rs:16–27` — toy demo template + cross-crate registration
- `backend/crates/gallery-smoke/tests/registry_roundtrip.rs:1–52` — test-shape precedent
- `frontend/src/lib/components/popup/ModalSurface.svelte:1–25` — close-modal hardcode
- `frontend/src/lib/components/popup/ConfirmDialog.svelte:34` — close-modal hardcode
- `frontend/src/lib/routing/router.svelte.ts:10–30` — initial navigate dispatch
- `frontend/src/lib/store/surfaces.svelte.ts:28–44` — surface tree lifecycle
- `backend/Cargo.toml:3–41` — workspace deps (axum 0.8, tokio 1, tower-http 0.6, sea-orm 1.1, linkme 0.3)
- `Makefile:3–8` — existing `dev` target
- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-CONTEXT.md` — full Phase 17 decision set
- `.planning/phases/16-framework-hooks/16-CONTEXT.md` §D-A1/C1/C2/D3 — Phase 16 locks
- `.planning/STATE.md` §Phase 17 hand-off — explicit-key requirement
- `.planning/codebase/STRUCTURE.md` / `CONVENTIONS.md` / `STACK.md` — architectural context
- `.planning/REQUIREMENTS.md` §CRATE-01/02, §DEMO-01/02 — phase requirements
- `.planning/ROADMAP.md` §Phase 17 — success criteria (5)

### Secondary (MEDIUM confidence)

- `ctx7@latest library axum` Context7 lookup — confirmed axum 0.8.x is current with `Router` + `with_state` + `fallback_service` APIs. Used only for cross-check; codebase pinning already canonical.

### Tertiary (LOW confidence)

- None — this research intentionally avoided training-knowledge claims. All patterns and versions come from the codebase or existing planning docs.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — every version is pinned in `backend/Cargo.toml`
- Architecture: HIGH for boot + nav + Home; MEDIUM for Modal close semantics (Open Q #2); MEDIUM for DataTable row delivery (Open Q #3); **LOW for composite demo return shape (Open Q #1 — blocker for planner)**
- Pitfalls: HIGH — each is grounded in a specific line of existing code
- Validation: HIGH — test shapes mirror Phase 16's existing `gallery-smoke/tests/`

**Research date:** 2026-04-22
**Valid until:** 2026-05-22 (30 days — stable codebase, no external-library churn expected)

---

## RESEARCH COMPLETE

**Phase:** 17 — Gallery Crate Skeleton + Colocated Built-in Demos
**Confidence:** HIGH for skeleton + per-component refactor; MEDIUM for composite demo body shape (resolved by Open Q #1 answer)

### Key Findings

- **AppState shape forces a `MockDatabase` workaround** — pattern proven in `crm-demo/tests/integration_test.rs:82`. Zero SQL queries; ~2 LOC.
- **Frontend hardcodes `close-modal`** — gallery-demo MUST register this action name, not just `gallery-demo/modal-close`.
- **Frontend dispatches `navigate` on WS connect** — gives gallery-demo the natural Home-page trigger; no synthetic bootstrap needed.
- **Per-component refactor is 24 new files**: 19 component files + `composites.rs` (for `form_shell`) + `mod.rs` rewrite + 3 file renames in tests. 10 external call-sites use `builders::standard::...` — the `pub use` re-export shim (Option A) preserves every import with zero caller edits.
- **Default-key collision is the #1 pitfall**: 19 sibling fns all named `gallery_demo` → mandatory `key = "..."` on every annotation, matching `#[component(type = "...")]`.
- **Open Question #1 (composite demo return shape)** is a potential **blocker** for DEMO-01's composite demos. Research recommends the planner raise it with the user before implementation; leaf demos (12 of 19) are safe regardless.

### File Created

`.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Gallery-demo skeleton | HIGH | Line-by-line analog in `crm-demo/src/main.rs`; mock-DB pattern in existing test |
| Per-component file refactor | HIGH | Full struct inventory with line ranges; 10 external caller sites identified; re-export shim pattern is trivial |
| `gallery_demo()` leaf bodies | HIGH | Phase 16 `gallery-smoke/src/lib.rs` already demonstrates the shape |
| `gallery_demo()` composite bodies | MEDIUM | Depends on Open Q #1; workable path 3 (reduced density) is always viable |
| Nav iteration + Home page | HIGH | `registered_demos()` API is stable; Grid/Container/Button primitives are proven |
| Modal/Toast/Confirm handlers | MEDIUM | Toast pattern verbatim from CRM; Modal close semantics verified open question (Open Q #2) |
| DataTable demo | MEDIUM | Need 1 of 3 row-delivery paths (Open Q #3); each is viable |
| Validation & tests | HIGH | Test shapes mirror Phase 16 `gallery-smoke/tests/` pattern |

### Open Questions (blocking or near-blocking)

1. **Composite demo return shape** (`fn() -> Node` vs flat tree) — planner MUST resolve before implementing Form / FieldSet / DataTable / Modal / Toast / ConfirmDialog / AppShell demos. Leaves are safe. Recommendations in Open Q #1 offer three viable paths including a framework-stable fallback (reduced composite density).
2. **Modal close tree semantics** — early test in Wave 0 clarifies; workaround (invisible Container) always works.
3. **DataTable row delivery** — cheap early check against `frontend/src/lib/components/table/DataTable.svelte`.

### Ready for Planning

Research complete. Planner can now create PLAN.md files, holding the composite-demo question open for user confirmation at the phase-planning checkpoint.
