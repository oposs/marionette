# OpenSDUI + Marionette

## What This Is

OpenSDUI is an open protocol specification for server-driven UI. Marionette is its reference implementation: a Svelte 5 + shadcn-svelte frontend library paired with a Rust + Axum backend toolkit. The backend controls what the frontend renders using three primitives — components, data, and messages. A demo CRM validates the protocol end-to-end with authentication, CRUD, search/filtering, and Listmonk integration; a companion gallery app (v1.2) serves as the design-iteration harness and frontend-capability exerciser.

## Core Value

The protocol must be clean, well-specified, and demonstrate that server-driven UI can be done right — enabling rapid business app development where backend developers control UI without requiring frontend expertise.

## Current Milestone: v1.2 Gallery Demo App + Auto-Discoverable Component Demos

**Goal:** Ship a dedicated gallery app that serves as both a visual-iteration harness and an SDUI-frontend exerciser, backed by a first-class auto-discoverable demo mechanism colocated with every marionette built-in component — so design iteration stops being blocked by the opinionated CRM surface and so new components automatically surface in the gallery.

**Target features:**
- `#[gallery_demo]` proc macro in `marionette-macros` with `inventory`/`linkme` distributed-slice registration
- `gallery` cargo feature gate on the `marionette` crate (default OFF) — production consumers do not compile demo code
- New `gallery-demo` workspace crate — thin backend (no auth, no DB, in-memory state only), AppShell-based nav
- Colocated `gallery_demo() -> Node` siblings for all existing built-in component builders (~20 components), pure-fn contract
- Catalog screens (Buttons, Forms, DataTable, Feedback, Typography & tokens) with composite demos via nested fn calls
- Exerciser screens: Nested AppShell, rapid node-patching, pathological scale (stress-tests protocol + frontend)
- Live CSS-token editor screen (scope-flexible; see seed `gallery-live-token-editor`)

## Current State

**Shipped:**
- v1.0 MVP (2026-04-08)
- v1.1 shadcn-svelte + High-Level Components (2026-04-18)

**What's built:**
- OpenAPI 3.1 protocol specification — now 1.1.0 with protocol node-patch operations (Phase 12)
- Svelte 5 frontend library on shadcn-svelte primitives + lucide icons; AppShell, enhanced DataTable (filter bar, virtualization, column visibility), enhanced forms (Field anatomy, FieldSet/FieldSeparator, Textarea/RadioGroup/Switch)
- Rust backend toolkit: derive macros, action routing, WebSocket sessions, SeaORM persistence; surface-scoped patches; hand-written AppShell builder
- CRM demo fully migrated to the new stack (Phase 15) — Flowbite is gone; CI guards prevent regressions

**Tech stack:** Rust (Axum, SeaORM, tokio), Svelte 5 (shadcn-svelte, Vite), SQLite for the CRM demo only, ~53k LOC across 273 files

## Requirements

### Validated

- ✓ OpenAPI 3.1 specification defining components, data, messages — v1.0
- ✓ Protocol manual explaining concepts, patterns, and rationale — v1.0
- ✓ Renders component adjacency lists from server — v1.0
- ✓ Handles data binding via JSON Pointers — v1.0
- ✓ Handles message passing to/from server — v1.0
- ✓ Axum integration for serving SDUI responses — v1.0
- ✓ SeaORM patterns for data persistence — v1.0
- ✓ Rust macros for ergonomic component construction — v1.0
- ✓ Multi-user authentication with roles — v1.0
- ✓ Companies management (CRUD) — v1.0
- ✓ Contacts management (CRUD, belong to companies) — v1.0
- ✓ Interactions/activity tracking per contact — v1.0
- ✓ Listmonk integration: sync contacts to lists — v1.0
- ✓ Listmonk integration: view mailing history per contact — v1.0
- ✓ shadcn-svelte is the sole component framework (Flowbite fully removed, CI-guarded) — v1.1 Phase 10
- ✓ All SDUI leaf components rebuilt on shadcn-svelte + lucide icons — v1.1 Phase 11
- ✓ Protocol 1.1.0 with node-patch operations (set-node / delete-node / set-children) — v1.1 Phase 12
- ✓ AppShell as a first-class SDUI component with responsive sidebar, mobile sheet, surface-scoped patches — v1.1 Phase 12
- ✓ DataTable with server-driven filter bar, virtualized infinite scroll, column visibility — v1.1 Phase 13
- ✓ Form Field anatomy (label/description/error) + FieldSet + FieldSeparator + Textarea + RadioGroup + Switch — v1.1 Phase 14
- ✓ CRM demo fully migrated to the new stack end-to-end — v1.1 Phase 15

### Active

*Canonical list lives in `.planning/REQUIREMENTS.md` (populated by the requirements step of `/gsd-new-milestone`). Summary by category:*

- [ ] **Framework hooks** — `#[gallery_demo]` proc macro, `inventory`/`linkme` registration backbone, `gallery` cargo feature gate
- [ ] **Gallery crate skeleton** — new `backend/crates/gallery-demo/` workspace member, thin backend, AppShell-based nav from auto-discovered registry
- [ ] **Colocated built-in demos** — pure-fn `gallery_demo() -> Node` siblings for every existing built-in component
- [ ] **Catalog screens** — Buttons, Forms, DataTable, Feedback, Typography & tokens
- [ ] **Exerciser screens** — Nested AppShell, rapid node-patching, pathological scale
- [ ] **Live CSS-token editor** — scope-flexible, see seed `gallery-live-token-editor`

### Out of Scope

- Complex workflow engine — simple navigation and forms only for v1
- Offline support — always-connected assumption
- Mobile-native implementations — web-first, Svelte only for v1
- Custom component authoring by end users — fixed component library

## Context

**Prior Art:**
- CallBackery (Perl-based SDUI) — learned from its inconsistencies and ad-hoc evolution
- A2UI's adjacency list pattern — adopted for component tree representation
- Existing SDUI solutions (Lona, Server-Driven UI frameworks) — mostly proprietary or incomplete

**Design Principles:**
- LLM-friendliness: protocol should be easy for AI to understand and generate
- Stateless protocol: no session state embedded in the protocol itself
- "Puppet master / smart puppet" mental model: backend orchestrates, frontend renders intelligently
- Component types as open strings (not enums) for extensibility
- No capability negotiation: frontend library is bundled, knows all components

**Purpose:**
- Showcase company's ability to deliver first-rate code
- Fill the gap in open source SDUI solutions
- Enable internal use for future projects

## Constraints

- **Tech Stack (Backend)**: Rust, Axum, SeaORM, utoipa, tokio — per TOOLING.md
- **Tech Stack (Frontend)**: Svelte 5, shadcn-svelte, Vite — per TOOLING.md
- **Build System**: Makefile-based builds with standard targets
- **Testing**: Unit (Vitest), Component (Playwright component), E2E (Playwright)
- **CI/CD**: GitHub Actions workflows
- **Quality Bar**: Showcase-quality — clean architecture, comprehensive tests, documentation that explains thinking

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Three primitives: Components, Data, Messages | Minimal surface area, covers all SDUI needs | ✓ Good |
| Components as flat adjacency list | Simpler than nested trees, easier to diff and patch, inspired by A2UI | ✓ Good |
| Data binding via JSON Pointers | Standard, well-understood, enables precise targeting | ✓ Good |
| Stateless protocol | Simplifies debugging, enables horizontal scaling, session state lives in backend | ✓ Good |
| Component types as open strings | Extensibility without protocol changes, frontend validates | ✓ Good |
| No capability negotiation | Frontend library bundled with app, always knows full component set | ✓ Good |
| Rust macros for component construction | Ergonomic DX, avoids verbose JSON/YAML in backend code | ✓ Good |
| serde tagged enum for protocol messages | Clean serialization with type discriminator | ✓ Good |
| mpsc channel for WebSocket reader/writer | Avoids Arc<Mutex> complexity on sender | ✓ Good |
| Type-erased AppState extension | Avoids leaking app types into library crate | ✓ Good |
| OnceLock for external service clients | Simple global access from handlers | ⚠️ Revisit — consider DI pattern for testability |
| SQLite for demo CRM | Simplest persistence for demo, zero config | ✓ Good |
| shadcn-svelte over Flowbite | Accessible primitives + theming model, drives CSS-token discipline | ✓ Good — v1.1 |
| TanStack Table Core in server-driven mode | Canonical shadcn data-table recipe; client row models disabled via manualSorting | ✓ Good — v1.1 Phase 13 |
| Protocol 1.1.0 node-patch operations | Closes CONCEPT.md's "patch by node ID" promise; enables focus-preservation across sibling mutation | ✓ Good — v1.1 Phase 12 |
| Gallery app as second demo alongside CRM | CRM is too opinionated as a design-iteration surface; gallery serves visual iteration + SDUI-frontend exerciser | → v1.2 |
| Auto-discoverable demos via `#[gallery_demo]` + inventory/linkme | Long-haul investment — eliminates registry drift as new components land; gated behind `gallery` cargo feature so production consumers stay lean | → v1.2 Phase A |
| Pure `fn() -> Node` demo contract | Keeps demo code in the `marionette` crate harmless; composite demos are nested fn calls; stateful fixtures live in the gallery binary, not the framework crate | → v1.2 |

---
## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-21 — v1.1 milestone closed; v1.2 (gallery-demo + auto-discoverable component demos) opened*
