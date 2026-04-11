# Roadmap: OpenSDUI + Marionette

## Milestones

- ✅ **v1.0 MVP** — Phases 1-9 (shipped 2026-04-08)
- 🚧 **v1.1 shadcn-svelte + High-Level Components** — Phases 10-15 (in progress)

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

### 🚧 v1.1 shadcn-svelte + High-Level Components (In Progress)

**Milestone Goal:** Replace Flowbite with shadcn-svelte and add high-level organisational components (AppShell, enhanced DataTable, enhanced FormScreen) so apps get professional screens out of the box.

- [x] **Phase 10: Foundation** - Install shadcn-svelte, rewrite CSS theming, remove all Flowbite dependencies (completed 2026-04-09)
- [x] **Phase 11: Leaf Component Migration** - Re-implement all existing SDUI components with shadcn-svelte primitives and lucide icons (completed 2026-04-09)
- [x] **Phase 12: Protocol Node Patching + AppShell** - Extend the protocol with incremental component-tree patches, then build the responsive AppShell on top (completed 2026-04-10)
- [x] **Phase 13: DataTable Enhancements** - Server-driven filter bar, infinite scroll, and column visibility (completed 2026-04-11)
- [ ] **Phase 14: FormScreen Enhancements** - Consistent field styling and grouped card sections
- [ ] **Phase 15: CRM Migration & Validation** - Migrate all CRM screens and validate zero Flowbite residue

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
**Plans**: TBD
**UI hint**: yes

### Phase 15: CRM Migration & Validation
**Goal**: The CRM demo runs entirely on the new component stack, proving the migration is complete and everything works end-to-end
**Depends on**: Phase 13, Phase 14
**Requirements**: COMP-03
**Success Criteria** (what must be TRUE):
  1. All CRM screens (login, companies, contacts, interactions, audit log) render and function correctly
  2. Zero Flowbite references remain anywhere in the codebase (grep confirms clean break)
  3. CRM navigation, CRUD operations, search/filtering, and Listmonk integration all work as before
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases 13 and 14 can execute in parallel after Phase 12. Phase 15 requires both to be complete.

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
| 10. Foundation | v1.1 | 3/3 | Complete    | 2026-04-09 |
| 11. Leaf Component Migration | v1.1 | 5/5 | Complete    | 2026-04-10 |
| 12. Protocol Node Patching + AppShell | v1.1 | 8/8 | Complete    | 2026-04-10 |
| 13. DataTable Enhancements | v1.1 | 7/7 | Complete   | 2026-04-11 |
| 14. FormScreen Enhancements | v1.1 | 0/0 | Not started | - |
| 15. CRM Migration & Validation | v1.1 | 0/0 | Not started | - |

---
*Created: 2026-01-24*
*Updated: 2026-04-09 -- Phase 11 plans created*
