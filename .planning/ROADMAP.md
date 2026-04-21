# Roadmap: OpenSDUI + Marionette

## Milestones

- ✅ **v1.0 MVP** — Phases 1-9 (shipped 2026-04-08)
- ✅ **v1.1 shadcn-svelte + High-Level Components** — Phases 10-15 (shipped 2026-04-18)
- 🚧 **v1.2 Gallery Demo App + Auto-Discoverable Component Demos** — Phases 16-20 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-9) — SHIPPED 2026-04-08</summary>

- [x] Phase 1: Project Infrastructure (3/3 plans) — Makefile, CI/CD, project structure
- [x] Phase 2: Protocol Specification (3/3 plans) — OpenAPI 3.1, protocol manual, examples
- [x] Phase 3: Frontend Library (6/6 plans) — Svelte 5 components, data store, WebSocket, tests
- [x] Phase 4: Backend Toolkit (5/5 plans) — Rust macros, action routing, WebSocket, SeaORM
- [x] Phase 5: Integration (2/2 plans) — Axum serves Svelte, E2E protocol validation
- [x] Phase 6: CRM Auth & Foundation (3/3 plans) — Login, roles, audit trail
- [x] Phase 7: CRM Core (3/3 plans) — Companies, contacts CRUD
- [x] Phase 8: CRM Features (4/4 plans) — Notes, tags, search, interactions
- [x] Phase 9: CRM Listmonk (3/3 plans) — Subscriber sync, mailing history

Full archive: [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md)

</details>

<details>
<summary>✅ v1.1 shadcn-svelte + High-Level Components (Phases 10-15) — SHIPPED 2026-04-18</summary>

- [x] **Phase 10: Foundation** — Install shadcn-svelte, rewrite CSS theming, remove all Flowbite dependencies (completed 2026-04-09)
- [x] **Phase 11: Leaf Component Migration** — Re-implement all existing SDUI components with shadcn-svelte primitives and lucide icons (completed 2026-04-10)
- [x] **Phase 12: Protocol Node Patching + AppShell** — Extend the protocol with incremental component-tree patches, then build the responsive AppShell on top (completed 2026-04-10)
- [x] **Phase 13: DataTable Enhancements** — Server-driven filter bar, infinite scroll, and column visibility (completed 2026-04-11)
- [x] **Phase 14: FormScreen Enhancements** — Consistent field styling and grouped card sections (completed 2026-04-18)
- [x] **Phase 15: CRM Migration & Validation** — Migrate all CRM screens and validate zero Flowbite residue (completed 2026-04-18)

</details>

### 🚧 v1.2 Gallery Demo App + Auto-Discoverable Component Demos (In Progress)

**Milestone Goal:** Ship a dedicated gallery app that serves as both a visual-iteration harness and an SDUI-frontend exerciser, backed by a first-class auto-discoverable demo mechanism colocated with every marionette built-in — so design iteration stops being blocked by the opinionated CRM surface and new components automatically surface in the gallery.

- [ ] **Phase 16: Framework Hooks** — `#[gallery_demo]` proc macro + `inventory`/`linkme` registration backbone + `gallery` cargo feature gate on the `marionette` crate
- [ ] **Phase 17: Gallery Crate Skeleton + Colocated Built-in Demos** — new `gallery-demo` workspace member with auto-discovered AppShell nav; `gallery_demo()` siblings for every existing built-in component
- [ ] **Phase 18: Catalog Screens** — Buttons & Actions, Forms, DataTable, Feedback, Typography & tokens
- [ ] **Phase 19: Exerciser Screens** — Nested AppShell, Rapid Patching, Pathological Scale
- [ ] **Phase 20: Live Token Editor** — CSS-token editor screen with live apply + exportable `@theme` block (scope-flexible)

## Phase Details

### Phase 10: Foundation
**Goal**: The frontend builds and renders with shadcn-svelte as the sole component framework -- Flowbite is completely gone
**Depends on**: Phase 9 (v1.0 complete)
**Requirements**: FOUND-01, FOUND-02, FOUND-03
**Success Criteria** (what must be TRUE):
  1. Running `npx shadcn-svelte@latest init` artifacts exist: components.json, utils.ts, and cn() helper are available
  2. app.css uses OKLCH semantic color tokens and shadcn theme system with no Flowbite plugin references
  3. Zero Flowbite packages remain in package.json and zero Flowbite imports exist in any source file
  4. The frontend compiles and the dev server starts without errors
**Plans**: 3 plans
Plans:
- [x] 10-01-PLAN.md — shadcn-svelte init + OKLCH CSS theme + Surface.svelte semantic tokens
- [x] 10-02-PLAN.md — Stub all 17 Flowbite components + remove Flowbite packages
- [x] 10-03-PLAN.md — Visual verification checkpoint
**UI hint**: yes

### Phase 11: Leaf Component Migration
**Goal**: Every existing SDUI component renders using shadcn-svelte primitives and lucide icons instead of Flowbite
**Depends on**: Phase 10
**Requirements**: COMP-01, COMP-02
**Success Criteria** (what must be TRUE):
  1. All SDUI components (Button, TextInput, Select, Checkbox, etc.) render correctly using shadcn-svelte primitives
  2. All icons render using lucide-svelte with no flowbite-svelte-icons imports anywhere
  3. Existing component tests pass with the new implementations (or are updated to match new markup)
  4. The demo page renders all component types without errors
**Plans**: 5 plans
Plans:
- [x] 11-01-PLAN.md — Install shadcn-svelte primitives + icon registry
- [x] 11-02-PLAN.md — Migrate form components (Button, TextInput, SelectInput, Checkbox, Form) + tests
- [x] 11-03-PLAN.md — Migrate popup/table components (ModalSurface, ConfirmDialog, ToastSurface, DataTable) + tests
- [x] 11-04-PLAN.md — Migrate layout/nav/core/feedback components + tests
- [x] 11-05-PLAN.md — Migrate screen components (FormScreen, TableScreen) + tests + visual verification
**UI hint**: yes

### Phase 12: Protocol Node Patching + AppShell
**Goal**: The OpenSDUI protocol supports incremental component-tree mutation (closing the gap between CONCEPT.md's "patch by node ID" promise and the implemented data-only PatchMessage), and applications get a professional responsive shell built as a normal SDUI component on top of that capability
**Depends on**: Phase 11
**Requirements**: PATCH-01, PATCH-02, PATCH-03, SHELL-01, SHELL-02, SHELL-03, SHELL-04
**Success Criteria** (what must be TRUE):
  1. `PatchMessage` carries both data operations and component-tree operations (set-node, delete-node, set-children) in a single atomic batch, with one applied-in-declared-order semantics documented in `spec/PROTOCOL.md` and schema-defined in `spec/schemas/data.yaml` + `spec/schemas/message.yaml`
  2. Frontend surface store applies node patches reactively without remounting unrelated nodes: a focused text-input retains focus and cursor position across arbitrary node patches to sibling nodes (focus-preservation test proves this)
  3. Protocol version reported by `HelloMessage` bumps to `"1.1.0"` and `CONCEPT.md` is updated so its "easy to patch — update one node by ID" claim matches the actual protocol
  4. AppShell renders a collapsible sidebar on desktop and a sheet overlay on mobile using shadcn Sidebar composable
  5. Header area displays app title and user menu; footer area displays status and version info
  6. Shell styling uses CSS variable theming via `--sidebar-*` tokens for consistent appearance
  7. AppShell is a normal first-class SDUI component: registered in `frontend/src/lib/registry/defaults.ts`, hand-written backend builder in `backend/crates/marionette/src/builders/` following the same recipe any other high-level structural component would use, with slot children addressed by name in props referencing top-level adjacency-list nodes (no special protocol superpowers)
  8. The CRM app renders inside the AppShell with working navigation between screens, and at least one interactive flow demonstrates node-level mutation end-to-end (e.g., a select change that swaps a field in place without clobbering sibling focus)
**Plans**: 8 plans
Plans:
- [x] 12-01-scaffolding-PLAN.md — Install shadcn sidebar/toast, rename --sidebar tokens, create Wave 1+ scaffold files
- [x] 12-02-protocol-crate-PLAN.md — Rewrite PatchOperation as tagged enum + add PatchMessage.surface + bump HelloMessage to 1.1.0
- [x] 12-03-protocol-spec-schemas-PLAN.md — Update spec/schemas/data.yaml oneOf + message.yaml surface + PROTOCOL.md + CONCEPT.md
- [x] 12-04-frontend-store-PLAN.md — TS PatchOperation union + init.ts fix + fine-grained surfaces store + focus-preservation test
- [x] 12-05-backend-builders-PLAN.md — SurfaceMount derived builder + hand-written AppShell builder with 6 slot methods
- [x] 12-06-frontend-shell-components-PLAN.md — AppShell.svelte + SurfaceMount.svelte + registry + +layout.svelte collapse + ConnectionBanner retirement
- [x] 12-07-crm-integration-PLAN.md — handle_navigate builds AppShell + migrate handlers to surface "content" + interactive verification checkpoint
- [x] 12-08-demo-and-e2e-PLAN.md — Country-select demo on contact form + node-patch-focus E2E + shell-nav E2E + protocol-conformance schema validation
**UI hint**: yes

### Phase 13: DataTable Enhancements
**Goal**: DataTable supports server-driven filtering, infinite scroll for large datasets, and user-controlled column visibility
**Depends on**: Phase 12
**Requirements**: TABLE-01, TABLE-02, TABLE-03
**Success Criteria** (what must be TRUE):
  1. DataTable displays a filter bar with text input and dropdowns that dispatch filter actions to the server
  2. Scrolling past the visible data triggers progressive server-side loading via IntersectionObserver sentinel
  3. User can toggle column visibility through a column visibility control
  4. Sorting and filtering reset the scroll position and fetched data ranges
**Plans**: 7 plans
Plans:
- [x] 13-01-scaffolding-PLAN.md — Install @tanstack deps + shadcn CLI data-table/dropdown-menu + svelte-virtual smoke test + sendAction returns id + onIntersect action + seed bump to 120 contacts
- [x] 13-02-backend-builder-PLAN.md — Extend Rust DataTable struct with Filter/ColumnKind enums, hand-written .filter() helper, hidden_default/total_rows/row_id_key fields, inline tests
- [x] 13-03-fetch-rows-handler-PLAN.md — Generic backend fetch_rows handler with source dispatch table, limit cap, per-source auth, action-id echo, integration tests
- [x] 13-04-datatable-actions-component-PLAN.md — DataTableActions.svelte DropdownMenu component + XSS-safe browser test
- [x] 13-05-datatable-rewrite-PLAN.md — Rewrite DataTable.svelte to recipe shape (filter bar, virtualizer, sentinel, column visibility, per-kind cells, stale discard) + rewritten browser tests + retire TableScreen + CI guard
- [x] 13-06-crm-list-handler-migration-PLAN.md — Migrate 4 CRM list handlers (audit/contact/company/user) to new DataTable shape with inline filters, total_rows, source, ColumnKind::Actions + add source field to backend DataTable struct + update spec/PROTOCOL.md example
- [x] 13-07-e2e-and-textinput-fix-PLAN.md — TextInput input_type bug fix + datatable-filter E2E + datatable-infinite-scroll E2E + protocol-conformance extension + human-verify checkpoint for column visibility non-persistence
**UI hint**: yes

### Phase 14: FormScreen Enhancements
**Goal**: Forms display professional field layouts with consistent label/description/error styling and visual grouping
**Depends on**: Phase 12
**Requirements**: FORM-01, FORM-02
**Success Criteria** (what must be TRUE):
  1. Form fields display label, description, and error message in a consistent layout using shadcn Field components
  2. Related fields can be grouped in card sections with headings and visual separators
  3. Field styling works correctly for all input types (text, select, checkbox, textarea)
**Plans**: 8 plans
Plans:
- [x] 14-01-PLAN.md — Install shadcn primitives + scaffold RED browser-tests + fix NodeRenderer unmount race (D-E2)
- [x] 14-02-PLAN.md — Rewrite TextInput with internal Field.Field wrap + Form.svelte Field.Group tweak + backend description/full_width + D-E1 regression test
- [x] 14-03-PLAN.md — Rewrite SelectInput with internal Field.Field wrap + preserve Phase 12 country-select change-action + backend description/full_width/placeholder/disabled
- [x] 14-04-PLAN.md — Rewrite Checkbox with internal Field.Field horizontal wrap + backend description/full_width
- [x] 14-05-PLAN.md — Add Textarea SDUI component + registry + backend builder (D-E3)
- [x] 14-06-PLAN.md — Add RadioGroup + Switch SDUI components + registry + backend builders + RadioOption (D-E4)
- [x] 14-07-PLAN.md — Add FieldSet + FieldSeparator SDUI components (responsive grid + cols override) + backend builders (D-C1 / D-C2 / D-C3 / D-C4)
- [x] 14-08-PLAN.md — Delete FormScreen orphan + migrate contact.rs edit form + spec/PROTOCOL.md + E2E + visual rebaseline + Chrome-MCP UAT
**UI hint**: yes

### Phase 15: CRM Migration & Validation
**Goal**: The CRM demo runs entirely on the new component stack, proving the migration is complete and everything works end-to-end
**Depends on**: Phase 13, Phase 14
**Requirements**: COMP-03
**Success Criteria** (what must be TRUE):
  1. All CRM screens (login, companies, contacts, interactions, audit log) render and function correctly
  2. Zero Flowbite references remain anywhere in the codebase (grep confirms clean break)
  3. CRM navigation, CRUD operations, search/filtering, and Listmonk integration all work as before
**Plans**: 7 plans
Plans:
- [x] 15-01-PLAN.md — Contact schema extension (country/notes/opt_in) + persistence + integration test
- [x] 15-02-PLAN.md — form_shell() + validation_error_patch() helpers with unit tests
- [x] 15-03-PLAN.md — Handler sweep A: company + user edit forms (FieldSet, RadioGroup, validation rewiring)
- [x] 15-04-PLAN.md — Handler sweep B: interaction + contact inline/refactor + note/tag save validation
- [x] 15-05-PLAN.md — Scope-closure bundle (D-G1 dev-gate, D-G2 Form.svelte payload, D-G3 Button builder, D-G4 node: prefix)
- [x] 15-06-PLAN.md — Flowbite CI guard + doc brand-voice sweep + PROTOCOL.md validation surgery
- [x] 15-07-PLAN.md — E2E specs + visual rebaseline + Chrome-MCP/Playwright UAT per screen + phase-gate
**UI hint**: yes

### Phase 16: Framework Hooks
**Goal**: The auto-discovery spine is in place — `#[gallery_demo]` proc macro, a stable registry-iteration API backed by `inventory` or `linkme`, and a `gallery` cargo feature gate that keeps production builds of `marionette` free of demo code
**Depends on**: Phase 15 (v1.1 complete)
**Requirements**: FRAME-01, FRAME-02, FRAME-03
**Success Criteria** (what must be TRUE):
  1. Applying `#[gallery_demo]` to a `pub fn name() -> Node` registers the function in a compile-time distributed slice keyed by component-type name; misuse (wrong signature, wrong visibility) produces a clear compiler error that names the violated rule
  2. `marionette::registered_demos()` (or equivalent) returns a stable-ordered iterator of `DemoEntry { key, fn_ptr }` values backed by `inventory` or `linkme` — registration-library choice logged in PROJECT.md Key Decisions with rationale
  3. Default `cargo build -p marionette` compiles zero demo symbols and zero registry entries (verified by a test inspecting artifact contents or symbol table); enabling the `gallery` feature brings them back
  4. A smoke test in the workspace registers a toy demo fn via `#[gallery_demo]`, enables the `gallery` feature, iterates the registry, and asserts the toy key is present in stable order
**Plans**: 4 plans
Plans:
- [x] 16-01-PLAN.md — marionette::gallery module: DemoEntry, linkme-backed DEMOS slice, registered_demos() API + gallery cargo feature
- [x] 16-02-PLAN.md — #[gallery_demo] attribute proc macro in marionette-macros (darling attr parsing + syn signature validation)
- [ ] 16-03-PLAN.md — gallery-smoke crate with toy demo + trybuild error-message fixtures + FRAME-03 symbol-table test
- [ ] 16-04-PLAN.md — docs closure: PROJECT.md Key Decisions row for linkme + STATE.md blocker close + Phase 17 hand-off
**UI hint**: no

### Phase 17: Gallery Crate Skeleton + Colocated Built-in Demos
**Goal**: The `gallery-demo` crate exists as the 5th workspace member with a thin in-memory backend, and every existing built-in component in `marionette/src/builders/` ships a pure-fn `gallery_demo() -> Node` sibling — so adding a new `#[gallery_demo]` anywhere in the workspace automatically surfaces it in the gallery's AppShell nav on next build
**Depends on**: Phase 16
**Requirements**: CRATE-01, CRATE-02, DEMO-01, DEMO-02
**Success Criteria** (what must be TRUE):
  1. `cargo run -p gallery-demo` starts the app on its own port against the shared frontend with no auth, no database, no migrations — only `Arc<RwLock<_>>` in-memory state
  2. The gallery's AppShell navigation is built at runtime by iterating the auto-discovered demo registry (no hand-maintained menu list); adding a new `#[gallery_demo]` and rebuilding causes the new entry to appear in nav without touching the gallery binary
  3. Every currently-registered built-in component (Button, TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch, DataTable, AppShell, NavItem, Sidebar pieces, ModalSurface, ConfirmDialog, ToastSurface, FieldSet, FieldSeparator, Container, Heading, plus any others) has a `pub fn gallery_demo() -> Node` sibling annotated with `#[gallery_demo]`
  4. `GALLERY-DEMOS.md` (under `backend/crates/marionette/` or equivalent) documents the pure-fn contract: no external state, no I/O, no fixtures; composite demos are nested `gallery_demo()` calls; stateful fixtures live in gallery handlers
  5. Each built-in demo renders without panicking when visited in the running gallery; clicking every nav entry produces a screen, not an error surface
**Plans**: TBD
**UI hint**: yes

### Phase 18: Catalog Screens
**Goal**: Five clean catalog screens compose the built-in demos into curated showcases — Buttons & Actions, Forms, DataTable, Feedback, and Typography & tokens — each demonstrating the full visual surface for its component family
**Depends on**: Phase 17
**Requirements**: CAT-01, CAT-02, CAT-03, CAT-04, CAT-05
**Success Criteria** (what must be TRUE):
  1. Buttons & Actions screen renders every Button variant × size × state combination (default / destructive / outline / ghost / link × sm/md/lg × normal/disabled/loading/icon-only) visible on one page
  2. Forms screen renders every input type (text / select / checkbox / switch / radio / textarea) across normal / disabled / error / focused / with-description states, grouped with `FieldSet` and `FieldSeparator`, and includes a live validation patch-demo where correcting an invalid field clears its error via node patch
  3. DataTable screen shows the filter bar, virtualized infinite scroll, column visibility toggle, and per-`ColumnKind` rendering seeded with ≥500 synthetic rows so virtualization actually engages
  4. Feedback screen shows toast dispatch, confirm dialog flow, modal surface, and empty / loading / error placeholder states side-by-side and individually triggerable
  5. Typography & tokens screen renders the full text scale, the lucide-svelte icon catalog (searchable or in a grid), and OKLCH swatches for every semantic token defined in `app.css`
**Plans**: TBD
**UI hint**: yes

### Phase 19: Exerciser Screens
**Goal**: Three frontend robustness exercisers — Nested AppShell, Rapid Patching, Pathological Scale — surface capability edges that a clean business app never hits, and capture any gaps as deferred items
**Depends on**: Phase 17
**Requirements**: EXER-01, EXER-02, EXER-03
**Success Criteria** (what must be TRUE):
  1. Nested AppShell screen renders an outer AppShell hosting an inner AppShell in its content slot; observations about `SidebarProvider` context behaviour, mobile-sheet composition, keyboard shortcut scoping, and `--sidebar-*` token inheritance are captured in the phase report (gaps deferred to v1.3 seeds rather than forcing fixes in v1.2)
  2. Rapid Patching screen fires node patches at a configurable interval (default ≈500 ms) while a text input retains focus; PATCH-02's focus-preservation invariant holds for ≥60 seconds of sustained mutation pressure without losing focus or cursor position
  3. Pathological Scale screen mounts a single page containing a DataTable seeded with ≥10 000 synthetic rows and a FormScreen with ≥80 synthetic fields; the page renders without freezing the browser, virtualization keeps scroll responsive, and observed performance baselines (time-to-first-paint, scroll FPS) are recorded in the phase report
  4. Every exerciser screen is reachable from the auto-discovered gallery nav and executes without console errors under normal use
**Plans**: TBD
**UI hint**: yes

### Phase 20: Live Token Editor
**Goal**: The gallery includes a Live Token Editor screen that lets a designer tweak core shadcn theme tokens in real time and export the result as a pasteable `@theme`/`:root` block — the single highest-leverage force multiplier for look-and-feel iteration (scope-flexible per seed `gallery-live-token-editor`)
**Depends on**: Phase 17 (catalog screens not strictly required, but landing after Phase 18 means the editor has stable screens to iterate against)
**Requirements**: THEME-01
**Success Criteria** (what must be TRUE):
  1. Live Token Editor screen offers controls (color picker / slider / numeric input) for core shadcn theme variables — at minimum `--primary`, `--background`, `--foreground`, `--radius`, and the `--sidebar-*` family
  2. Changing a control applies the value to `document.documentElement` via `style.setProperty()` and all currently-mounted gallery screens re-render with the new token in place (no backend round-trip required)
  3. An export affordance emits the current token set as a pasteable `@theme`/`:root` block copyable to clipboard or visible in a read-only textarea
  4. If token coverage is scoped down during execution, the reduced scope is documented in the phase report and the deferred tokens are recorded as a seed for v1.3
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**

- **v1.2 dependency chain:** Phase 16 (framework rails) must land before Phase 17 (demo colocation needs the proc macro + registry). Phase 17 must land before Phases 18, 19, and 20 (all three consume the demo registry and/or the running gallery binary).
- **Parallelization after Phase 17:** Phases 18 (Catalog) and 19 (Exerciser) are independent screen sets and can execute in parallel — they touch disjoint gallery handlers, share no state, and their success criteria do not overlap. Phase 20 (Live Token Editor) can also start immediately after Phase 17, but is best scheduled after Phase 18 so there is a stable catalog surface for the designer to iterate against visually. If v1.2 budget tightens, Phase 20 is the natural deferral target (see seed `gallery-live-token-editor`).
- **v1.0 / v1.1 historical order:** Phases 13 and 14 executed in parallel after Phase 12; Phase 15 required both.

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Project Infrastructure | v1.0 | 3/3 | Complete | 2026-01-24 |
| 2. Protocol Specification | v1.0 | 3/3 | Complete | 2026-03-18 |
| 3. Frontend Library | v1.0 | 6/6 | Complete | 2026-03-20 |
| 4. Backend Toolkit | v1.0 | 5/5 | Complete | 2026-03-20 |
| 5. Integration | v1.0 | 2/2 | Complete | 2026-03-21 |
| 6. CRM Auth & Foundation | v1.0 | 3/3 | Complete | 2026-03-22 |
| 7. CRM Core | v1.0 | 3/3 | Complete | 2026-03-22 |
| 8. CRM Features | v1.0 | 4/4 | Complete | 2026-03-23 |
| 9. CRM Listmonk | v1.0 | 3/3 | Complete | 2026-03-23 |
| 10. Foundation | v1.1 | 3/3 | Complete | 2026-04-09 |
| 11. Leaf Component Migration | v1.1 | 5/5 | Complete | 2026-04-10 |
| 12. Protocol Node Patching + AppShell | v1.1 | 8/8 | Complete | 2026-04-10 |
| 13. DataTable Enhancements | v1.1 | 7/7 | Complete | 2026-04-11 |
| 14. FormScreen Enhancements | v1.1 | 8/8 | Complete | 2026-04-18 |
| 15. CRM Migration & Validation | v1.1 | 7/7 | Complete | 2026-04-18 |
| 16. Framework Hooks | v1.2 | 0/? | Pending | — |
| 17. Gallery Crate Skeleton + Colocated Built-in Demos | v1.2 | 0/? | Pending | — |
| 18. Catalog Screens | v1.2 | 0/? | Pending | — |
| 19. Exerciser Screens | v1.2 | 0/? | Pending | — |
| 20. Live Token Editor | v1.2 | 0/? | Pending | — |

---
*Created: 2026-01-24*
*Updated: 2026-04-21 — v1.2 roadmap appended (Phases 16–20)*
