# Requirements: Marionette v1.2

**Defined:** 2026-04-21
**Core Value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI
**Milestone goal:** Ship a dedicated gallery app that serves as visual-iteration harness and SDUI-frontend exerciser, backed by a first-class auto-discoverable demo mechanism colocated with every marionette built-in.

Prior requirements (v1.0, v1.1) are archived in `.planning/milestones/v1.0-REQUIREMENTS.md` and `.planning/milestones/v1.1-REQUIREMENTS.md` respectively. Their validated items live in the `## Validated` section of `PROJECT.md`.

## v1.2 Requirements

### Framework hooks (the auto-discovery spine)

- [ ] **FRAME-01**: `#[gallery_demo]` attribute proc macro exists in `marionette-macros` and applies to a `pub fn name() -> Node` item. Applying it registers the function in a distributed slice at compile time with a stable key derived from the component type name. Misapplication (wrong signature, wrong visibility) produces a clear compiler error that names the violated rule.
- [ ] **FRAME-02**: `marionette` crate exposes a registry-iteration API (e.g. `pub fn registered_demos() -> impl Iterator<Item = DemoEntry>` returning component-type key + `fn() -> Node`) backed by `inventory` or `linkme`. The choice is recorded in a Key Decisions entry with rationale. Iteration order is stable.
- [ ] **FRAME-03**: `gallery` cargo feature on the `marionette` crate gates all demo-related code. Default build (`cargo build -p marionette`) compiles zero demo symbols and zero registry entries — verified by a test that inspects the built artifact size or binary contents. The `gallery-demo` binary enables the feature explicitly.

### Gallery crate skeleton

- [ ] **CRATE-01**: New `backend/crates/gallery-demo/` workspace member exists as the 6th Cargo workspace entry (the 5th slot is occupied by `gallery-smoke`, a permanent test-fixture crate landed in Phase 16), with thin backend scaffolding: no auth, no database, no migrations — in-memory `Arc<RwLock<_>>` state only. `cargo run -p gallery-demo` starts the app on its own port against the shared frontend.
- [ ] **CRATE-02**: Gallery `main.rs` builds its AppShell navigation by iterating the auto-discovered demo registry — no hand-maintained menu list. Adding a new `#[gallery_demo]` fn anywhere in the workspace causes the gallery to automatically surface it on next build.

### Built-in demos (colocated, pure-fn)

- [ ] **DEMO-01**: Every existing built-in component in `backend/crates/marionette/src/builders/` ships a sibling `pub fn gallery_demo() -> Node` annotated with `#[gallery_demo]`, covering all currently-registered SDUI components (Button, TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch, DataTable, AppShell, NavItem, Sidebar pieces, ModalSurface, ConfirmDialog, ToastSurface, FieldSet, FieldSeparator, Container, Heading, plus any others in the current registry).
- [ ] **DEMO-02**: Demo contract is enforced by convention (and documented in a short `GALLERY-DEMOS.md` under `backend/crates/marionette/` or equivalent): pure `fn() -> Node`, no external state, no I/O, no fixtures. Composite demos are built by calling other `gallery_demo()` functions directly — `FormScreen::gallery_demo()` invokes `TextInput::gallery_demo()`, `SelectInput::gallery_demo()`, etc.

### Catalog screens (clean showcases)

- [ ] **CAT-01**: Gallery includes a Buttons & Actions screen showing every Button variant × size × state (default / destructive / outline / ghost / link × sm/md/lg × normal/disabled/loading/icon-only).
- [ ] **CAT-02**: Gallery includes a Forms screen rendering every input type (text / select / checkbox / switch / radio / textarea) across every visual state (normal / disabled / error / focused / with-description), grouped with `FieldSet` and `FieldSeparator`, including a live validation patch-demo that exercises error-clearing on correction.
- [ ] **CAT-03**: Gallery includes a DataTable screen with filter bar, virtualized infinite scroll, column visibility toggle, and per-`ColumnKind` rendering, seeded with enough synthetic rows to exercise virtualization (≥500 rows).
- [ ] **CAT-04**: Gallery includes a Feedback screen showing toast dispatch, confirm dialog flow, modal surface, and empty / loading / error placeholder states side-by-side.
- [ ] **CAT-05**: Gallery includes a Typography & tokens screen rendering the text scale, the lucide-svelte icon catalog (searchable or grid), and OKLCH swatches for every semantic token in `app.css`.

### Exerciser screens (frontend robustness)

- [ ] **EXER-01**: Gallery includes a Nested AppShell screen where an outer AppShell hosts an inner AppShell in its content slot. Demonstrates whether shadcn `SidebarProvider` context, mobile-sheet behaviour, keyboard shortcut handling, and `--sidebar-*` CSS tokens compose under nesting — and captures any gaps as deferred items.
- [ ] **EXER-02**: Gallery includes a Rapid Patching screen that fires node patches at a configurable interval (default ~500ms) while a text input retains focus. Verifies PATCH-02's focus-preservation invariant under sustained mutation pressure.
- [ ] **EXER-03**: Gallery includes a Pathological Scale screen combining a DataTable with ≥10 000 synthetic rows and a FormScreen with ≥80 synthetic fields on a single page — captures performance baselines and surfaces scaling issues in the frontend surface store, virtualizer, and SurfaceMount patch application.

### Theme tools (force multiplier, scope-flexible)

- [ ] **THEME-01**: Gallery includes a Live Token Editor screen offering controls (color picker / slider / numeric input) for core shadcn theme variables (`--primary`, `--background`, `--foreground`, `--radius`, `--sidebar-*`, etc.). Changes apply via `document.documentElement.style.setProperty()` and re-render all gallery screens in place. An export affordance emits the current token set as a pasteable `@theme`/`:root` block. Scope note: may be scoped down to a subset of tokens or deferred to a follow-up milestone if the owning phase runs long (see seed `gallery-live-token-editor`).

## v1.3+ Requirements

Deferred to a future milestone — tracked for context, not in v1.2 scope.

- **GALLERY-LINT**: CI lint enforcing "every built-in component must have a `#[gallery_demo]`" as a hard rule rather than convention (may emerge from DEMO-01's manual sweep experience)
- **GALLERY-DEMOS-EXPORT**: Publish `marionette-gallery-demo` as a documentation artifact library users can run out of the box (`cargo run -p gallery-demo` already works locally; this is about surfacing it as a shipped story)
- **THEME-EXPORT-WRITE**: Theme editor writes exported tokens directly back to `frontend/src/app.css` on user confirmation, not just clipboard paste

## Out of Scope

| Feature | Reason |
|---------|--------|
| Auth / login in the gallery | The gallery is a design harness; auth would slow boot and reintroduce CRM-style friction |
| Database-backed persistence in the gallery | Stateful fixtures belong in handlers, not durable storage; restart-is-reset is a feature |
| Framework-level composition machinery for composite demos | Composite demos are plain nested fn calls — adding a "demo-combinator" abstraction is premature |
| Auto-generated documentation from demos (e.g. screenshots exported) | Useful, but a v1.3+ concern; v1.2 is about the harness, not the publishing pipeline |
| Demos for third-party components outside the `marionette` crate | Scope-limit: v1.2 ships demos for built-ins only |
| Backend-driven theme tokens (server pushes theme to client) | Theme lives client-side; server-driven theming is a different product direction |

## Traceability

Populated by `/gsd-plan-phase` runs as phases are defined.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FRAME-01 | Phase 16 | Pending |
| FRAME-02 | Phase 16 | Pending |
| FRAME-03 | Phase 16 | Pending |
| CRATE-01 | Phase 17 | Pending |
| CRATE-02 | Phase 17 | Pending |
| DEMO-01  | Phase 17 | Pending |
| DEMO-02  | Phase 17 | Pending |
| CAT-01   | Phase 18 | Pending |
| CAT-02   | Phase 18 | Pending |
| CAT-03   | Phase 18 | Pending |
| CAT-04   | Phase 18 | Pending |
| CAT-05   | Phase 18 | Pending |
| EXER-01  | Phase 19 | Pending |
| EXER-02  | Phase 19 | Pending |
| EXER-03  | Phase 19 | Pending |
| THEME-01 | Phase 20 | Pending |

**Coverage:**
- v1.2 requirements: 16 total
- Mapped to phases: 16 (Phases 16–20)
- Unmapped: 0

---
*Requirements defined: 2026-04-21 via /gsd-new-milestone*
*Traceability populated: 2026-04-21 via /gsd-roadmap (v1.2 Phases 16–20)*
