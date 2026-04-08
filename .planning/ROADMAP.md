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

- [ ] **Phase 10: Foundation** - Install shadcn-svelte, rewrite CSS theming, remove all Flowbite dependencies
- [ ] **Phase 11: Leaf Component Migration** - Re-implement all existing SDUI components with shadcn-svelte primitives and lucide icons
- [ ] **Phase 12: AppShell** - Responsive sidebar shell with header/footer, CSS variable theming, and backend builder
- [ ] **Phase 13: DataTable Enhancements** - Server-driven filter bar, infinite scroll, and column visibility
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
**Plans**: TBD
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
**Plans**: TBD
**UI hint**: yes

### Phase 12: AppShell
**Goal**: Applications get a professional responsive shell with collapsible sidebar, header, and footer out of the box
**Depends on**: Phase 11
**Requirements**: SHELL-01, SHELL-02, SHELL-03, SHELL-04
**Success Criteria** (what must be TRUE):
  1. AppShell renders a collapsible sidebar on desktop and a sheet overlay on mobile using shadcn Sidebar composable
  2. Header area displays app title and user menu; footer area displays status and version info
  3. Shell styling uses CSS variable theming via `--sidebar-*` tokens for consistent appearance
  4. Backend AppShell builder exists following the FormScreen/TableScreen pattern with slot-based child references
  5. The CRM app renders inside the AppShell with working navigation between screens
**Plans**: TBD
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
**Plans**: TBD
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
| 10. Foundation | v1.1 | 0/0 | Not started | - |
| 11. Leaf Component Migration | v1.1 | 0/0 | Not started | - |
| 12. AppShell | v1.1 | 0/0 | Not started | - |
| 13. DataTable Enhancements | v1.1 | 0/0 | Not started | - |
| 14. FormScreen Enhancements | v1.1 | 0/0 | Not started | - |
| 15. CRM Migration & Validation | v1.1 | 0/0 | Not started | - |

---
*Created: 2026-01-24*
*Updated: 2026-04-08 -- v1.1 roadmap added*
