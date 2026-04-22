# Phase 17: Gallery Crate Skeleton + Colocated Built-in Demos - Context

**Gathered:** 2026-04-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 17 turns the Phase 16 framework rails into a **running gallery app** and pays the one-time cost of **exhaustive built-in coverage**. It delivers four things:

1. **A new `gallery-demo` binary crate** (6th workspace member — 5th slot is already taken by Phase 16's `gallery-smoke`). Thin backend: no auth, no database, no migrations; only `Arc<RwLock<_>>` in-memory state. `cargo run -p gallery-demo` boots the app on its own port against the shared `frontend/build` assets.

2. **An AppShell whose sidebar nav is built at runtime from `registered_demos()`** — flat alphabetical order, one NavItem per entry, no hand-maintained menu list. Adding a new `#[gallery_demo]` anywhere in the workspace and rebuilding causes the new entry to appear in nav without touching `gallery-demo/src/main.rs`.

3. **A `gallery_demo()` sibling on every qualifying built-in** in `backend/crates/marionette/src/builders/`, each annotated with `#[gallery_demo(key = "...")]` using the explicit-key convention from Phase 16 STATE.md hand-off. The sweep is delivered through a **per-component-file refactor** of `builders/standard.rs` (one file per ComponentBuilder struct).

4. **`GALLERY-DEMOS.md`** under `backend/crates/marionette/` (or equivalent) documents the pure-fn contract, the skip list + rationale, the `/demo/{key}/...` bind-path convention, the `gallery-demo/*` action namespace, and the composite-nesting rule (with the AppShell exception).

**What this phase is NOT:**

- **NOT catalog screens.** Phase 18 (CAT-01 through CAT-05) owns the variant-matrix screens (every Button variant × size × state, every form input × state, DataTable with 500+ rows, Feedback side-by-side, Typography/tokens). Phase 17 demos are intentionally minimal — catalog screens compose via direct builder calls, not via extended `gallery_demo()` fns.
- **NOT exerciser screens.** Phase 19 (EXER-01/02/03) owns Nested AppShell, Rapid Patching, Pathological Scale.
- **NOT the Live Token Editor.** Phase 20 (THEME-01).
- **NOT a CI lint enforcing coverage.** GALLERY-LINT is v1.3+. Phase 17's coverage is delivered by sweep + documentation in GALLERY-DEMOS.md; drift risk is a known deferred concern.
- **NOT a change to the `gallery` cargo feature, the `registered_demos()` iteration semantics, or the `#[gallery_demo]` macro's `key`/`name` arg handling.** Phase 16 locked those; this phase consumes them verbatim. Two in-flight framework-level changes ARE in scope: (1) a **Phase 16.5 micro-refactor** of `DemoEntry.render: fn() -> Node` → `fn() -> Vec<Node>` (per §D-Z1 — surfaced by research as a blocker for composite demos), and (2) the per-component file reorganization of `builders/standard.rs` (per §D-B3). Both touch narrow surfaces — no impact to the linkme backbone, feature gate, explicit-key requirement, pure-fn contract, or alphabetical ordering.
- **NOT adding grouping metadata to `DemoEntry`.** The nav is flat alphabetical; grouping is a deferred idea if Phase 18/19 UX reveals the need.
- **NOT extending `ActionRouter` with wildcard or fallback handlers.** Every demo action is explicitly registered by `gallery-demo/src/main.rs` under the `gallery-demo/*` namespace.
- **NOT frontend work.** The existing `frontend/build` output is served by gallery-demo as-is. No Svelte component changes are expected.

</domain>

<decisions>
## Implementation Decisions

### Area Z — Phase 16.5 micro-refactor: `DemoEntry.render` signature (added 2026-04-22 after research surfaced blocker)

- **D-Z1: `DemoEntry.render: fn() -> Vec<Node>` (flat-tree return shape).** Phase 16's original signature `fn() -> Node` (where `Node = (String, Component)`) cannot carry descendant nodes through the render path — the Render message's `nodes` HashMap needs every descendant, but a single tuple loses them. Composite demos (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast, AppShell — 7 of 19 in-scope demos) therefore cannot satisfy D-A1's hybrid density under the original signature. Phase 17 lands a **Phase 16.5 micro-refactor** as an early wave before the sweep: change `DemoEntry.render` to `fn() -> Vec<Node>`, update the `#[gallery_demo]` macro's signature check, update `gallery-smoke/src/lib.rs`'s `smoke()` fn + trybuild `.stderr` fixtures. Leaf demos return `vec![one_tuple]`; composites return `vec![root_tuple, ...descendants]`. Phase 16's core contract (linkme backbone, `gallery` feature gate, explicit `key = "..."` requirement, pure-fn, alphabetical iteration order) is **unchanged**. The `gallery-show` handler's match-then-Render pipeline consumes `Vec<Node>` directly: `let nodes_vec = entry.render(); let root_id = nodes_vec[0].0.clone(); let nodes_map: HashMap<_, _> = nodes_vec.into_iter().collect();`. This supersedes Phase 16 §D-C2's render-field line.

  **Touched by this refactor (exhaustive):**
  - `backend/crates/marionette/src/gallery.rs` — `DemoEntry.render` field type.
  - `backend/crates/marionette-macros/src/gallery_demo.rs` — `return_type_is_node` check (line ~155 per research) becomes `return_type_is_vec_node`.
  - `backend/crates/gallery-smoke/src/lib.rs` — `smoke()` returns `vec![Text::new("gallery-smoke").build()]`.
  - `backend/crates/gallery-smoke/tests/ui/` — `.stderr` expectations for misapplication cases update to reference `Vec<Node>` rather than `Node`.

  No other consumers of `registered_demos()` exist at Phase 16.5 time (gallery-demo doesn't exist yet). Minimal blast radius.

### Area A — Demo content density (what each `gallery_demo()` emits)

- **D-A1: Hybrid density — canonical leaf, substantive composite.** Leaves (Button, TextInput, Checkbox, Switch, Textarea, Select, RadioGroup, Heading, Text, Grid, Spinner, ErrorDisplay) emit **2–3 representative instances stacked in a Container** — enough to be visually meaningful when clicked directly without turning the demo into a catalog. Button shows default + disabled + destructive; TextInput shows default + disabled + with-description; similar pattern for each leaf. Composites (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast) emit a meaningful mini-composition via nested `gallery_demo()` calls where the leaf-demo shape fits. Phase 18 catalog screens compose the full variant × size × state matrices directly via the builder API (not via extended demo fns).

- **D-A2: `AppShell::gallery_demo()` is hand-designed, not an automatic nest.** Sidebar entries and main content are hand-picked for a curated "this is how you'd really build it" showcase — too many reasonable content combinations to pick automatically from nested demos. Other composites follow DEMO-02 (nested `gallery_demo()` calls where the leaf-demo shape fits): Form nests input demos, FieldSet nests its field demos, ConfirmDialog body nests a Text or Heading demo. The AppShell exception is documented in GALLERY-DEMOS.md.

- **D-A3: Demo fns emit canonical action names in the `gallery-demo/*` namespace.** Demos are "decorative + minimal behavior" — not passive artwork. Buttons carry `.action(ComponentAction::submit("gallery-demo/noop"))` and trigger a toast when clicked. TextInputs carry `.bind("/demo/{key}/value")` so typing updates the surface store. Demos feel alive; wiring is tiny.

- **D-A4: Modal and ConfirmDialog render real behavior (trigger button + closed overlay).** `Modal::gallery_demo()` returns a Button labeled "Open modal" + a closed Modal; clicking fires `gallery-demo/modal-open` which dispatches a surface patch to open the overlay. Same shape for ConfirmDialog with open/accept/reject. This softens D-C4 below — the single noop action is for leaf demos; Modal/ConfirmDialog get purpose-built toggle actions. The `gallery-demo/*` namespace stays consistent.

### Area B — Sweep scope + per-component-file refactor

- **D-B1: Visible-standalone coverage rule.** Every ComponentBuilder struct that renders meaningfully on its own gets a `#[gallery_demo]` sibling. Structural pieces are demoed transitively through their parents' demos.

- **D-B2: Skip list (documented in GALLERY-DEMOS.md):**
  - `SurfaceMount` — mount point with no visual; demoed transitively wherever AppShell hosts it.
  - `NavItem` — single nav entry; demoed transitively via SideNav (inside AppShell::gallery_demo).
  - `NavGroup` — nav subgroup; same as NavItem.
  - `FieldSeparator` — divider inside a FieldSet; demoed transitively via FieldSet::gallery_demo.
  - `SideNav` — standalone outside an AppShell Sidebar context looks contextually wrong; demoed transitively via AppShell::gallery_demo.
  - `Container` — empty Container renders nothing; "wrap some Text" is indistinguishable from the Text demo. Demoed transitively via every composite that wraps content.
  - `TableColumn` — not a component; a DataTable props struct. Excluded by nature.

  All other ComponentBuilder structs in `standard.rs` plus the hand-written `AppShell` in `app_shell.rs` ship demos. Current in-scope list: `Button`, `TextInput`, `Select`, `Checkbox`, `Grid`, `Heading`, `Text`, `Form`, `Textarea`, `RadioGroup`, `Switch`, `FieldSet`, `DataTable`, `Modal`, `Toast`, `ConfirmDialog`, `Spinner`, `ErrorDisplay`, `AppShell` — ~19 demos.

- **D-B3: Per-component file refactor of `builders/`.** `backend/crates/marionette/src/builders/standard.rs` is broken up into one file per `#[derive(ComponentBuilder)]` struct. Each file hosts: the struct + its related props types (Select+SelectOption, RadioGroup+RadioOption, DataTable+TableColumn) + the `gallery_demo()` sibling fn (when the component is on the in-scope list). `app_shell.rs` stays as-is and grows its own `gallery_demo()`. Public API is preserved via `pub use` re-exports in `builders/mod.rs` — existing imports like `marionette::builders::standard::{Button, ...}` continue to resolve (either via a re-export shim or by updating callers, at the planner's discretion). Structural-skip components (SurfaceMount, NavItem, NavGroup, FieldSeparator, SideNav, Container) also move to their own files for consistency, but without a `gallery_demo()` sibling.

- **D-B4: Coverage is documented, not CI-enforced.** GALLERY-DEMOS.md includes a coverage matrix (component → demo yes/no → rationale-if-skipped). Enforcement is aspirational per v1.2 scope; GALLERY-LINT (v1.3+) is the future CI-enforced version.

### Area C — Gallery-demo binary: nav, landing, routing

- **D-C1: Flat alphabetical nav by `DemoEntry.key`.** No grouping metadata added to DemoEntry. `gallery-demo/src/main.rs` iterates `registered_demos()` and emits one NavItem per entry in the returned order (sorted alphabetically by key per Phase 16 D-A2). At ~25 total entries across v1.2 (~19 built-in leaves + Home + 5 catalog + 3 exerciser + 1 theme editor), flat reads fine. Grouping is a deferred idea: Phase 18/19 can re-open if nav becomes unwieldy; the fix is a non-breaking `group: Option<&'static str>` field addition on DemoEntry.

- **D-C2: Curated Home page on first visit.** The gallery's `content` sub-surface initially renders a hand-authored Home page: welcome Heading + explanatory Text explaining the gallery is both a visual-iteration harness and an SDUI-frontend exerciser + a Grid of tiles linking into each demo, derived from `registered_demos()`. The Home page also showcases the framework (Heading + Text + Container + Grid + clickable tiles — each tile is a Button or link-styled element carrying `gallery-show` with the appropriate key payload). No auto-redirect; the gallery's first impression is intentional.

- **D-C3: Single `gallery-show` action routes every nav click.** Demo links fire `ComponentAction::submit("gallery-show")` with payload `{ key: "<demo-key>" }`. The gallery-demo handler:
  1. Extracts `key` from payload.
  2. Looks up `registered_demos().find(|e| e.key == key)`.
  3. Invokes `e.render()` to obtain the `(String, Component)` node tuple.
  4. Seeds any `/demo/{key}/...` state needed by the demo (D-D1).
  5. Wraps the node in a `Render` message targeting the `content` sub-surface.
  6. Returns.

  Auto-extensible — new demos need zero handler changes. The `key` can also flow into the URL hash for deep-linking (router.svelte.ts already supports this); shipping deep-link bootstrap is Claude's discretion.

- **D-C4: Small set of purpose-built demo actions registered at gallery-demo startup.**
  - `gallery-show` — nav routing (D-C3).
  - `gallery-demo/noop` — catch-all fire-and-toast for leaf-demo components (Button clicks, Switch toggles, Select changes, etc.). Handler emits a toast naming the source demo.
  - `gallery-demo/modal-open`, `gallery-demo/modal-close` — Modal demo open/close via surface patch.
  - `gallery-demo/confirm-open`, `gallery-demo/confirm-accept`, `gallery-demo/confirm-reject` — ConfirmDialog demo flow.
  - `gallery-demo/toast-fire` — Toast demo explicit dispatch (fresh toast enqueued).

  Planner may add more if the sweep reveals demos that can't naturally ride the noop action (e.g., a Spinner demo that wants to show "start/stop" toggling); keep the set small and stay inside the `gallery-demo/*` namespace. No wildcard or fallback handlers on `ActionRouter` — every action is explicit.

### Area D — Stateful fixtures (now vs Phase 18/19)

- **D-D1: Minimal per-demo seeds (the "feels alive" bar).** The `gallery-show` handler's match arm seeds just enough state to make the demo visually meaningful:
  - DataTable: 5–10 synthetic rows (Phase 18 CAT-03 takes this to ≥500; Phase 19 EXER-03 takes it to ≥10 000).
  - Form / individual inputs: 2–3 bound fields with seeded default values under `/demo/{key}/...`.
  - Modal / ConfirmDialog: closed by default; trigger opens (D-A4).
  - Toast: empty queue on visit; explicit "fire toast" button dispatches `gallery-demo/toast-fire`.
  - RadioGroup / Switch / Checkbox / Select: seeded initial-value path.

- **D-D2: Bind-path convention `/demo/{key}/...` for all demo fns.** Example paths: `/demo/text-input/value`, `/demo/form/email`, `/demo/data-table/rows`, `/demo/switch/checked`. The `gallery-show` handler's match arm seeds the matching `/demo/{key}` sub-tree. Self-documenting and trivially mappable to seed routines. Documented in GALLERY-DEMOS.md.

- **D-D3: Demos render real interactive behavior where it's cheap.** Typing in a TextInput updates `/demo/text-input/value`; toggling a Switch updates `/demo/switch/checked`; selecting a Select option updates `/demo/select/value`. The state is in-memory (`Arc<RwLock<_>>` in AppState) and reset on restart — no persistence, matching the thin-backend posture.

- **D-D4: Toast demo seeds an initial queue entry plus exposes a "Fire toast" Button.** Visiting the Toast demo shows one baseline toast so the surface isn't empty; the button dispatches `gallery-demo/toast-fire` which enqueues another. Demonstrates both initial render and dispatch.

### Claude's Discretion

- **Port number for `gallery-demo`**: 3002 is the obvious choice (CRM is 3001). Makefile `make dev` / `make gallery-dev` target naming is planner's call.
- **`GALLERY-DEMOS.md` exact location**: `backend/crates/marionette/GALLERY-DEMOS.md` is recommended (crate-level doc sibling to `Cargo.toml`); `backend/crates/marionette/src/gallery/README.md` is also acceptable.
- **`standard.rs` disposition after the refactor**: retire entirely with `pub use` re-exports moved into `builders/mod.rs`, OR keep `standard.rs` as a `pub use` re-export shim that preserves `marionette::builders::standard::Button` import paths. Planner's call; either clears clippy.
- **Related props-struct placement**: colocate `SelectOption` with `select.rs`, `RadioOption` with `radio_group.rs`, `TableColumn` with `data_table.rs` (recommended). A shared `builders/types.rs` is also acceptable if props-structs start cross-referencing.
- **Home page tile rendering shape**: Grid of Button-styled tiles, Grid of NavItems, or a custom Container-of-Headings+Text layout. All cheap; planner picks what reads best visually.
- **Whether the Home page's tile list is derived from `registered_demos()`** or hand-authored with intent-picked featured demos (or both — a "Featured" row plus an "All demos" grid). Recommended: derived from the registry so it stays in sync automatically.
- **Exact noop-handler toast message shape** (e.g., `"Demo action from <key>"` vs `"gallery-demo/noop fired (payload=…)"`).
- **Title-casing / `name = "..."` overrides** on each `#[gallery_demo]` annotation. Default title-case works for most (`"data-table"` → `"Data Table"`); override for any that read awkwardly.
- **Whether DataTable seed rows share the synthetic-data generator Phase 18/19 will use**, or a Phase-17-local quick helper. Recommended: ship a tiny local helper now; if Phase 18/19 adds a shared generator, fold it in then.
- **Exact action-registration boilerplate style** (inline in `main.rs` vs helper fn `register_gallery_actions(router) -> ActionRouter`). Recommended: helper fn for readability at ~10 action registrations.
- **Whether `Modal` and `ConfirmDialog` demos target the gallery's `modal` sub-surface** (matching the AppShell's modal mount) or render the overlay inside `content`. Recommended: the `modal` sub-surface, matching real usage.
- **Deep-link URL hash behavior**: whether a `#button` hash on page load auto-fires `gallery-show` for "button". Cheap to add on top of D-C3; scope-flex.
- **Whether `gallery-smoke`'s Cargo.toml needs any adjustment** as Phase 17 lands. Default assumption: no — it already depends on `marionette --features gallery`, which is unchanged. Planner confirms.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-level intent

- `.planning/ROADMAP.md` §Phase 17 (lines 187–198) — goal (gallery-demo crate + AppShell auto-nav + gallery_demo siblings), depends-on (Phase 16), 5 success criteria, UI hint: yes.
- `.planning/ROADMAP.md` §v1.2 dependency chain (lines 241–242) — Phase 17 must land before Phases 18/19/20; Phases 18/19 can parallelize after Phase 17.
- `.planning/REQUIREMENTS.md` §Gallery crate skeleton — CRATE-01 (new workspace member, thin backend, `cargo run -p gallery-demo` boots), CRATE-02 (AppShell nav built by iterating auto-discovered registry, no hand-maintained menu).
- `.planning/REQUIREMENTS.md` §Built-in demos — DEMO-01 (every existing built-in has a `gallery_demo()` sibling with `#[gallery_demo]`), DEMO-02 (pure-fn contract, composite demos via nested calls, GALLERY-DEMOS.md documents the contract).
- `.planning/REQUIREMENTS.md` §Out of Scope — auth / login in gallery, DB-backed persistence, third-party component demos, backend-driven theme tokens.
- `.planning/REQUIREMENTS.md` §v1.3+ — GALLERY-LINT is deferred; Phase 17 ships coverage by sweep + docs, not by CI enforcement.
- `.planning/PROJECT.md` §Key Decisions — "Gallery app as second demo alongside CRM", "Auto-discoverable demos via `#[gallery_demo]` + inventory/linkme", "Pure `fn() -> Node` demo contract", "linkme over inventory for gallery-demo registry". Phase 17 implements the second and third directly; the first and fourth are framework-level preconditions.

### Phase 16 hand-off (locked, do not re-litigate)

- `.planning/phases/16-framework-hooks/16-CONTEXT.md` — full Phase 16 decision set. Especially:
  - **§D-A1** — `linkme` is the registry backbone. No fallback to `inventory`.
  - **§D-A2 + §D-A4** — Iteration is alphabetical by key, memoized via `OnceLock`.
  - **§D-B1 + §D-B3 + §D-B4** — `gallery` cargo feature gate on `marionette`; both fn body AND registration are cfg-gated; `registered_demos()` stub returns empty iterator when feature is off.
  - **§D-C1** — Phase 17 MUST use explicit `key = "..."` on every annotation, matching each builder's `#[component(type = "…")]` string. Default-derived key would collide because every demo fn is named `gallery_demo`.
  - **§D-C2** — `DemoEntry` = `{ key, render, display_name }`. No `component_type`, no source metadata.
  - **§D-C3** — `display_name` defaults to title-cased key; override via `name = "..."`.
  - **§D-D3** — `gallery-smoke` is permanent (5th workspace member). Phase 17's `gallery-demo` is the **6th** workspace member. REQUIREMENTS.md §CRATE-01 says "5th" — wording will be reconciled (either update REQUIREMENTS.md or accept gallery-smoke as a test-fixture crate that doesn't count toward the ordinal).
- `.planning/STATE.md` §"Phase 17 hand-off (from Phase 16)" — explicit `key = "..."` override requirement re-stated; 6th-crate ordinal shift flagged.
- `.planning/phases/16-framework-hooks/16-01-PLAN.md` through `16-04-PLAN.md` + their SUMMARY files — the actual registry + macro + smoke crate implementation. Read-only reference for Phase 17 planners: these are what the consumers are built on.

### Design foundation

- `.planning/notes/2026-04-21-gallery-demo-architecture.md` — full architectural rationale. §Thin backend (no auth, no DB, in-memory only), §Auto-discoverable component demos — option C (committed), §Demo contract (pure fn, no I/O, composites are nested calls), §Content shape (catalog + exerciser + theme editor), §Likely phase decomposition (Phase B = Phase 17 here). Phase 17 realizes Phase B of this note.

### Prior phase context (inherited stack knowledge)

- `.planning/milestones/v1.1-phases/12-protocol-node-patching-appshell/12-CONTEXT.md` — AppShell as a first-class SDUI component; six slots (sidebar, header, footer, main, popups, toasts); sub-surface semantics; SidebarProvider context. Phase 17 uses the `AppShell` builder verbatim — no changes to the framework component. Phase 19 EXER-01 tests nested composition later.
- `.planning/milestones/v1.1-phases/13-datatable-enhancements/` — DataTable filter bar, virtualized infinite scroll, column visibility, `fetch-rows` generic handler. Phase 17's DataTable demo uses the builder shape established here; Phase 18 CAT-03 exercises virtualization.
- `.planning/milestones/v1.1-phases/14-formscreen-enhancements/14-CONTEXT.md` — Field anatomy, FieldSet grouping, Textarea / RadioGroup / Switch additions, full-width field flag, description rendering. Phase 17's form-family demos follow this shape.
- `.planning/milestones/v1.1-phases/15-crm-migration-validation/15-CONTEXT.md` — pre-deployment posture (no back-compat shims, fix root causes; applies to the per-component refactor), Chrome-MCP UAT pattern (APPLIES to Phase 17 — the gallery is a UI surface; UAT screens via Chrome MCP, not by handing the user a walkthrough).

### Code the phase touches

- **New crate: `backend/crates/gallery-demo/`** (6th workspace member).
  - `Cargo.toml` — depends on `marionette = { path = "../marionette", features = ["gallery"] }`, `marionette-protocol`, `marionette-macros`, `axum`, `tokio`, `tracing`, `tracing-subscriber`, `serde_json`, `tower-http`. No `sea-orm`, no `bcrypt`, no `chrono` (unless demos use them directly — none currently do).
  - `src/main.rs` — tokio main, AppState with `Arc<RwLock<_>>`, ActionRouter wiring (gallery-show + noop + modal/confirm/toast actions), AppShell construction from `registered_demos()`, Home page renderer, axum router with `/ws`, `/api/health`, static file fallback to `../frontend/build`, listener on port 3002.
  - `src/handlers.rs` (or sub-modules) — gallery-show handler, noop handler, modal open/close, confirm open/accept/reject, toast-fire.
- **`backend/Cargo.toml`** — add `"crates/gallery-demo"` to `[workspace] members`. The workspace then holds `marionette-protocol`, `marionette-macros`, `marionette`, `crm-demo`, `gallery-smoke`, `gallery-demo` (6 members).
- **`backend/crates/marionette/src/builders/`** — per-component-file refactor (D-B3):
  - Retire or reshape `standard.rs` per Claude's discretion.
  - New files, one per ComponentBuilder struct: `button.rs`, `text_input.rs`, `select.rs` (with `SelectOption`), `checkbox.rs`, `container.rs`, `grid.rs`, `heading.rs`, `text.rs`, `side_nav.rs`, `nav_item.rs`, `nav_group.rs`, `surface_mount.rs`, `form.rs`, `textarea.rs`, `radio_group.rs` (with `RadioOption`), `switch.rs`, `field_set.rs`, `field_separator.rs`, `data_table.rs` (with `TableColumn`), `modal.rs`, `toast.rs`, `confirm_dialog.rs`, `spinner.rs`, `error_display.rs`.
  - `mod.rs` — declare all modules, re-export public API. Preserve `marionette::builders::*` consumer imports (use `pub use` or callers adapt).
  - `app_shell.rs` — unchanged structure; grows a `gallery_demo()` fn at the bottom.
  - Every in-scope file (all except structural-skip ones: `surface_mount.rs`, `nav_item.rs`, `nav_group.rs`, `field_separator.rs`, `side_nav.rs`, `container.rs`) gains a `#[gallery_demo(key = "<type-string>")] pub fn gallery_demo() -> Node` at the bottom.
- **`backend/crates/marionette/GALLERY-DEMOS.md`** (or equivalent location per Claude's discretion) — new file documenting: pure-fn contract, skip list + rationale, `/demo/{key}/...` bind-path convention, `gallery-demo/*` action namespace, composite-nesting rule + AppShell exception, coverage matrix.
- **`Makefile`** — add `gallery-dev` / `gallery-run` targets paralleling the existing `dev` target for the CRM.
- **`.planning/REQUIREMENTS.md`** — reconcile §CRATE-01 "5th Cargo workspace entry" wording (either update to "6th" or clarify that gallery-smoke is a test-fixture crate not counted in the ordinal). Planner picks resolution + updates.

### External library docs

- https://docs.rs/linkme/latest/linkme/ — Phase 16 reference; Phase 17 consumes, does not re-decide.
- https://docs.axum.rs/axum/latest/axum/ — Axum Router, state management. Same patterns as `crm-demo/src/main.rs` — this is a copy-and-simplify exercise.
- https://docs.rs/tower-http/latest/tower_http/services/ — `ServeDir` + `ServeFile` fallback for SPA. CRM demo's pattern reused verbatim.

### Existing code references (read before planning)

- `backend/crates/crm-demo/src/main.rs` — the template for `gallery-demo/src/main.rs`. Lines ~375–680 show the full Axum + AppShell + ActionRouter wiring. Gallery-demo is this minus auth, minus DB, minus Listmonk, minus seeds.
- `backend/crates/marionette/src/builders/standard.rs` — the ~700-line target of the per-component split. Lines 1–100 are the `Button` + `TextInput` + `Select`/`SelectOption` block; lines 100–200 cover `Checkbox` through `SurfaceMount`; lines 200–320 cover the form family; lines 320–490 cover `TableColumn` + `DataTable`; lines 537–590 cover overlays + feedback primitives.
- `backend/crates/marionette/src/builders/app_shell.rs` — hand-written 379-line builder. Template for where `AppShell::gallery_demo()` lives (same file, at the bottom). Read `AppShell::new().sidebar(...).header(...).main(...).build_with_children()` at lines 52–170 to design the hand-picked gallery AppShell demo.
- `backend/crates/marionette/src/gallery.rs` (or `gallery/mod.rs`) — the Phase 16-landed `DemoEntry` struct + `DEMOS` slice + `registered_demos()` memoization. Phase 17 consumes `registered_demos()` in exactly one place (the AppShell nav builder inside `gallery-demo/src/main.rs`) and nowhere else.
- `backend/crates/gallery-smoke/src/lib.rs` + `backend/crates/gallery-smoke/tests/registry_roundtrip.rs` — the toy-demo + roundtrip-test pattern. Phase 17's built-in demos are 18 more instances of this exact shape (minus the `tests/registry_roundtrip.rs` — that stays in gallery-smoke).
- `backend/crates/crm-demo/src/main.rs` lines 180–335 (the `handle_navigate` function) — existing AppShell construction pattern with sidebar + header + footer + content mount + modal mount + toast mount. Gallery-demo's AppShell construction mirrors this, substituting CRM-specific nav items with the auto-discovered `registered_demos()` iteration.

### Codebase intel

- `.planning/codebase/STRUCTURE.md` — `backend/crates/` layout. Will need minor update for the per-component-file refactor (D-B3) + the new gallery-demo member.
- `.planning/codebase/CONVENTIONS.md` — edition 2024, `#![warn(clippy::pedantic)]` in library crates, kebab-case crate names, snake_case modules, PascalCase builders. All applies to new gallery-demo code.
- `.planning/codebase/TESTING.md` — if the test shape for gallery-demo differs from crm-demo (unit vs integration), update accordingly. Phase 17's testing is light: compile + `cargo run` smoke path + success-criterion #5 "clicking every nav entry produces a screen, not an error surface" is UAT-level (Chrome MCP).
- `.planning/codebase/STACK.md` — Axum/Tokio/SeaORM for backend. Gallery-demo drops SeaORM; keeps Axum + Tokio.

### User preferences (global memory)

- `feedback_pre_deployment_no_backcompat.md` — no back-compat shims; fix root causes. **APPLIES to the per-component refactor**: import paths change cleanly; no deprecation aliases kept around "just in case".
- `feedback_options_need_reasoning.md` — every option comes with pros/cons/rationale; framework recipes preferred. **APPLIED throughout this discussion** (four gray areas × ~4 questions each, all with reasoned options).
- `feedback_no_handrolling_ui.md` — adopt framework recipes over hand-rolled UI. **APPLIES for the Home page and demo layouts**: use existing `Container` / `Grid` / `Heading` / `NavItem` primitives; don't invent new patterns.
- `feedback_use_chrome_for_uat.md` — Chrome-MCP for UAT, not a walkthrough handed to the user. **APPLIES**: Phase 17's success criterion #5 (every nav entry produces a screen, not an error surface) is verified by Chrome MCP navigation, not by sending the user a checklist.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`backend/crates/crm-demo/src/main.rs`** — the template for `gallery-demo/src/main.rs`. The CRM's `main.rs` covers AppShell construction, ActionRouter wiring, Axum `.route()` + `ServeDir` fallback, AppState initialization. Gallery-demo is this minus auth/DB/Listmonk, plus the `registered_demos()` nav iteration.
- **`backend/crates/marionette/src/builders/app_shell.rs`** — hand-written AppShell builder with slot-based composition (`sidebar`, `header`, `footer`, `main`, `popups`, `toasts`). Gallery uses `AppShell::new().sidebar(...).header(...).main(SurfaceMount::new("content")).popups(SurfaceMount::new("modal")).toasts(SurfaceMount::new("toasts")).build_with_children()`, same shape as CRM.
- **`backend/crates/marionette/src/builders/standard.rs`** — the builders being refactored. Their *content* doesn't change; only file placement + the added `gallery_demo()` sibling fn on in-scope ones.
- **`backend/crates/marionette/src/gallery.rs`** — Phase 16's `registered_demos()` iteration API. Gallery-demo calls this exactly once to build the nav.
- **`backend/crates/gallery-smoke/src/lib.rs`** — Phase 16's toy demo. Template for the sweep: `#[gallery_demo(key = "...", name = "...")] pub fn gallery_demo() -> Node { <builder calls> }`.
- **`marionette::router::ActionRouter` + `marionette::router::box_handler`** — existing action dispatch. gallery-demo uses these exactly the way CRM does.
- **`marionette::ws::{ws_handler, AppState}`** — WebSocket handler + app state type. Gallery-demo's AppState is `AppState { router, db, login_form, listmonk }` minus the DB/listmonk concerns. Planner may need to lightly refactor `AppState` if it hard-requires a DB, or use `Option<Arc<DatabaseConnection>>` already present.
- **`marionette::builders::Node`** type alias (`= (String, Component)`) — return type of `gallery_demo()` fns; already used by Phase 16's toy demo.
- **`frontend/build/`** — prebuilt SvelteKit assets served by the Rust binary via `ServeDir`. Gallery-demo reuses this directory with no changes.

### Established Patterns

- **Per-crate `#![warn(clippy::pedantic)]`** — gallery-demo's `main.rs` opens with this (same as crm-demo). New builder sub-modules inherit from `marionette/src/lib.rs`'s lint config.
- **Pure-fn builder composition** — every `gallery_demo()` fn is a linear sequence of `Builder::new(...).method(...).build()` calls returning a `(NodeId, Component)` tuple or `(NodeId, Vec<Descendants>)` tuple for composites. No loops, no if-branches, no state — verified by the macro's signature check.
- **Sub-surface semantics** — `main` (content), `modal` (popups), `toasts` — are distinct top-level surfaces. Gallery-demo's handlers target the appropriate surface per patch (e.g., `gallery-show` targets `content`; `gallery-demo/modal-open` targets `modal`; `gallery-demo/toast-fire` targets `toasts`).
- **Feature-gated demo symbols** — the `#[gallery_demo]` macro emits `#[cfg(feature = "gallery")]` on both fn body AND linkme static. Default `cargo build -p marionette` compiles zero demo code. Gallery-demo enables the feature via its Cargo.toml dep line.
- **Action namespace convention** — CRM uses `snake_case` for domain actions (`contact_list`) and `kebab-case` for framework actions (`fetch-rows`). Gallery-demo stays in the `gallery-demo/` namespace with `kebab-case` action names for consistency with framework conventions.

### Integration Points

- **`backend/Cargo.toml`** — append `"crates/gallery-demo"` to `[workspace] members`.
- **`backend/crates/marionette/src/builders/mod.rs`** — replace the current `pub mod standard;` declaration with per-component `pub mod` declarations + `pub use` re-exports for API preservation.
- **`backend/crates/marionette/GALLERY-DEMOS.md`** — new file.
- **`Makefile`** — new `gallery-dev` target running `cargo run -p gallery-demo` (plus whatever frontend rebuild or watch hook is used for `dev`).
- **`backend/crates/gallery-demo/Cargo.toml`** — new file; minimal dependency set.
- **`backend/crates/gallery-demo/src/main.rs`** — new file; full Axum + ActionRouter + AppShell + registered_demos() iteration + handler registration.
- **Surface targets**: `content`, `modal`, `toasts` — these are the three sub-surfaces the gallery patches against (inherited from AppShell's standard slot set).

</code_context>

<specifics>
## Specific Ideas

- **"The per-component file refactor is a deliberate scope expansion."** The user explicitly chose it over "same file, right below the builder" because the current `standard.rs` is already ~700 lines and adding ~18 demo fns would push it past 1100. The refactor is one-shot and mechanical; every demo is next to its own builder from here forward. Downstream maintenance wins outweigh the one-shot cost.

- **"AppShell's demo is hand-designed, not auto-nested."** Explicit user callout: the AppShell's sidebar/header/main content has too many reasonable combinations to pick from nested demos. The demo ships a curated "this is how you'd really build it" shell — specific nav entries pointing to a couple of other demos, a specific header, a specific main-content snippet. Other composites (Form, FieldSet, ConfirmDialog body) still follow DEMO-02's nested-call rule.

- **"Demos are decorative + minimal behavior, not passive artwork."** The user picked "canonical action names with no-op handlers" over "decorative, no actions". Demos fire real actions that route to a real namespace (`gallery-demo/*`), bindings update real state under `/demo/{key}/...`, and Modal/ConfirmDialog trigger-open behavior is real. The cost is small; the demo feels alive.

- **"Trigger button + closed modal, not statically open."** Explicit user pick. Modal demo opens in response to a click, not on nav-visit. Same for ConfirmDialog. This required stepping back the "single noop action covers everything" rule from Area 3 — gallery-demo registers a small purpose-built set (`modal-open`, `modal-close`, `confirm-open`, `confirm-accept`, `confirm-reject`, `toast-fire`) alongside `noop`.

- **"Flat alphabetical nav, grouping deferred."** User picked this over grouping-via-DemoEntry-extension and grouping-via-hand-curated-list. At ~25 total v1.2 demos, flat is readable; grouping would force Phase 16's `DemoEntry` shape open. If Phase 18/19 shows the need, a non-breaking `group: Option<&'static str>` field addition is a small follow-up.

- **"Curated Home page, not auto-redirect."** The gallery's first impression is intentional — a welcome Heading, explanatory Text, and a Grid of tiles derived from the registry. Zero chance of "click link → one gray button" being the first thing a visitor sees.

- **"Single `gallery-show` action with key payload routes all nav clicks."** Auto-extensible — new demos need zero handler changes. The handler is a ~20-line match over key-seeded state + a `registered_demos().find()` call + a Render message emission.

- **"Bind paths follow `/demo/{key}/...` convention everywhere."** Predictable, self-documenting, trivially mappable to seed routines per demo. Documented in GALLERY-DEMOS.md as part of the demo contract.

- **"Minimal per-demo seeds — Phase 18 takes the scale."** DataTable gets 5–10 rows (Phase 18 takes it to 500+, Phase 19 to 10 000+). Form gets 2–3 bound fields. Toast starts with one baseline entry. The "feels alive" bar is cheap; CAT-level data is Phase 18's job.

- **"gallery-demo is the 6th workspace crate, not the 5th."** REQUIREMENTS.md §CRATE-01 says "5th" based on a 2026-04-21 count. Phase 16's `gallery-smoke` took the 5th slot. Phase 17 plans should either update REQUIREMENTS.md wording or accept gallery-smoke as a test-fixture crate not counted in the product ordinal. Planner picks resolution.

- **"GALLERY-DEMOS.md is the author-facing contract, not just scaffolding documentation."** It covers (1) the pure-fn contract and why, (2) the skip list + rationale per component, (3) the `/demo/{key}/...` bind-path convention, (4) the `gallery-demo/*` action namespace + noop pattern, (5) the composite-nesting rule + AppShell exception, (6) the coverage matrix. Future component additions use this as the authoring guide.

- **"Chrome MCP for UAT."** Success criterion #5 ("every nav entry produces a screen, not an error surface") is verified by Chrome MCP navigation through every demo route — not by handing the user a checklist. Phase 17's UAT exercises the full registered_demos() → nav → click → render pipeline for all ~19 in-scope demos.

</specifics>

<deferred>
## Deferred Ideas

- **Grouping metadata on `DemoEntry`** — a `group: Option<&'static str>` field addition that the macro parses from `#[gallery_demo(group = "...")]`. Not needed for Phase 17's flat nav at ~25 entries. Re-open if Phase 18/19 UX shows the flat list becomes unwieldy (non-breaking addition — existing demos keep `group = None`).

- **`GALLERY-LINT`** (CI lint enforcing every built-in has a `#[gallery_demo]`) — v1.3+ per REQUIREMENTS.md. Phase 17 delivers coverage by sweep + documented skip list; a future CI lint hardens the convention.

- **Deep-link URL hash handling** — `#<demo-key>` on page load auto-fires `gallery-show` for that demo. Cheap on top of D-C3 but not in the success criteria; Claude's discretion if time permits.

- **Sharing the synthetic-data generator across Phase 17, 18, 19** — Phase 17 ships a tiny local helper for 5–10 DataTable rows; Phase 18 CAT-03 needs 500+; Phase 19 EXER-03 needs 10 000+. A shared `gallery-demo/src/fixtures.rs` generator is a natural v1.2 extraction; not Phase 17's job.

- **`ActionRouter` fallback / wildcard handler capability** — rejected for Phase 17 (scope creep, silent-failure risk in production apps). Every demo action stays explicitly registered. A future phase may add a debug-mode-only fallback for tooling purposes.

- **Framework-level composition machinery for composite demos** — rejected per REQUIREMENTS.md §Out of Scope. Composites are plain nested `gallery_demo()` calls. No "demo-combinator" abstraction.

- **Auto-generated screenshots / documentation from demos** — v1.3+ per REQUIREMENTS.md (`GALLERY-DEMOS-EXPORT`). Phase 17 ships the running gallery; shipping it as a published documentation artifact is a separate concern.

- **Theme editor** — `THEME-01`, Phase 20. Phase 17 makes no provision for runtime token editing; tokens are CSS-token-static. Phase 20 uses `document.documentElement.style.setProperty()` to override them live.

- **Live reactivity of the gallery on new demo addition during `cargo run`** — impossible by design (Rust compile-time registration). "New demos automatically appear" means "after rebuild". Not a gap.

- **Third-party (non-marionette) crates registering demos** — out of scope per REQUIREMENTS.md. Phase 17 only adds demos from `marionette`'s own builders + (transitively) the `gallery-smoke` toy + Phases 18/19's catalog/exerciser screens. A separate third-party demo crate would be a v1.3+ feature.

- **`SideNav` / `Container` / structural-piece standalone demos** — explicitly skipped per D-B2 with rationale. If a future phase adds a "component-in-isolation" mode (e.g., for screenshot automation), these may warrant individual demos then.

- **Noop handler's toast message richness** (e.g., showing full payload JSON, source demo key, timestamp) — Claude's discretion; default to a short one-line message that names the source demo.

- **Reviewed Todos (not folded)** — none. STATE.md §Pending Todos was empty.

</deferred>

---

*Phase: 17-gallery-crate-skeleton-colocated-built-in-demos*
*Context gathered: 2026-04-22*
