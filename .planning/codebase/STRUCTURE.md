# Codebase Structure

**Analysis Date:** 2026-04-08

## Directory Layout

```
marionette/                             # Monorepo root
├── backend/                            # Rust Cargo workspace
│   ├── Cargo.toml                      # Workspace manifest (4 members)
│   ├── rustfmt.toml                    # Rust formatting config
│   ├── crm.db                          # SQLite database (runtime artifact)
│   └── crates/
│       ├── marionette-protocol/        # Pure protocol types (no behavior)
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── common.rs           # Type aliases, AuthRequirement
│       │       ├── component.rs        # Component, ComponentAction structs
│       │       ├── data.rs             # PatchOperation, ValidationError
│       │       └── messages.rs         # ProtocolMessage enum (all message types)
│       ├── marionette-macros/          # Procedural macros
│       │   └── src/
│       │       ├── lib.rs              # ComponentBuilder, #[action], #[requires]
│       │       ├── action.rs
│       │       ├── component_builder.rs
│       │       └── requires.rs
│       ├── marionette/                 # Reusable toolkit crate
│       │   ├── src/
│       │   │   ├── lib.rs              # Public re-exports
│       │   │   ├── auth.rs             # check_auth() enforcing AuthRequirement
│       │   │   ├── builders/
│       │   │   │   ├── mod.rs
│       │   │   │   ├── node.rs         # Low-level node builder
│       │   │   │   └── standard.rs     # 18 typed component builders
│       │   │   ├── db.rs               # init_db(), session entity, test_db()
│       │   │   ├── error.rs            # ActionError enum, ActionResult type
│       │   │   ├── extractors.rs       # HandlerContext, Session, Payload<T>, Db
│       │   │   ├── migration/          # Session table migration
│       │   │   ├── router.rs           # ActionRouter, box_handler
│       │   │   ├── session.rs          # WsSession (in-memory per connection)
│       │   │   └── ws.rs               # ws_handler, AppState, session loop
│       │   └── tests/                  # Integration tests for toolkit
│       └── crm-demo/                   # CRM application (runnable binary)
│           ├── src/
│           │   ├── main.rs             # Server entry point, router wiring
│           │   ├── audit.rs            # Audit log utilities
│           │   ├── listmonk.rs         # Listmonk API client
│           │   ├── seed.rs             # Demo data seeding
│           │   ├── entities/           # SeaORM entity models
│           │   │   ├── mod.rs
│           │   │   ├── audit_log.rs
│           │   │   ├── company.rs
│           │   │   ├── contact.rs
│           │   │   ├── contact_tag.rs
│           │   │   ├── interaction.rs
│           │   │   ├── listmonk_cache.rs
│           │   │   ├── listmonk_sync.rs
│           │   │   ├── note.rs
│           │   │   ├── tag.rs
│           │   │   └── user.rs
│           │   ├── handlers/           # Action handler functions
│           │   │   ├── mod.rs
│           │   │   ├── audit.rs        # handle_audit_list
│           │   │   ├── auth.rs         # REST /api/login, /api/logout
│           │   │   ├── company.rs      # handle_company_*
│           │   │   ├── contact.rs      # handle_contact_* (largest handler)
│           │   │   ├── interaction.rs  # handle_interaction_*
│           │   │   ├── listmonk.rs     # handle_listmonk_*
│           │   │   ├── note.rs         # handle_note_save
│           │   │   └── user.rs         # handle_user_* (admin only)
│           │   └── migration/          # DB schema migrations
│           └── tests/                  # CRM integration tests
├── frontend/                           # SvelteKit application
│   ├── package.json
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── src/
│   │   ├── app.css                     # Global Tailwind + Flowbite styles
│   │   ├── lib/                        # Marionette library (exported as $lib)
│   │   │   ├── index.ts                # Public API barrel export
│   │   │   ├── init.ts                 # initMarionette() / destroyMarionette()
│   │   │   ├── components/
│   │   │   │   ├── core/
│   │   │   │   │   ├── Surface.svelte          # Named surface mount point
│   │   │   │   │   ├── NodeRenderer.svelte     # Recursive component renderer
│   │   │   │   │   ├── ConnectionBanner.svelte # WS connection status
│   │   │   │   │   ├── ErrorBoundary.svelte    # Per-node error isolation
│   │   │   │   │   ├── FallbackComponent.svelte
│   │   │   │   │   └── LoadingSkeleton.svelte
│   │   │   │   ├── feedback/
│   │   │   │   │   ├── Spinner.svelte
│   │   │   │   │   └── ErrorDisplay.svelte
│   │   │   │   ├── form/
│   │   │   │   │   ├── Button.svelte
│   │   │   │   │   ├── Checkbox.svelte
│   │   │   │   │   ├── Form.svelte
│   │   │   │   │   ├── SelectInput.svelte
│   │   │   │   │   └── TextInput.svelte
│   │   │   │   ├── layout/
│   │   │   │   │   ├── Container.svelte
│   │   │   │   │   ├── Grid.svelte
│   │   │   │   │   ├── Heading.svelte
│   │   │   │   │   └── Text.svelte
│   │   │   │   ├── nav/
│   │   │   │   │   ├── NavGroup.svelte
│   │   │   │   │   ├── NavItem.svelte
│   │   │   │   │   └── SideNav.svelte
│   │   │   │   ├── popup/
│   │   │   │   │   ├── ConfirmDialog.svelte
│   │   │   │   │   ├── ModalSurface.svelte
│   │   │   │   │   └── ToastSurface.svelte
│   │   │   │   ├── screen/
│   │   │   │   │   ├── FormScreen.svelte       # Composite form-page pattern
│   │   │   │   │   └── TableScreen.svelte      # Composite table-page pattern
│   │   │   │   └── table/
│   │   │   │       └── DataTable.svelte
│   │   │   ├── registry/
│   │   │   │   ├── registry.ts                 # register(), getComponent()
│   │   │   │   └── defaults.ts                 # registerDefaults() mapping type→component
│   │   │   ├── routing/
│   │   │   │   └── router.svelte.ts            # URL sync + initial navigate action
│   │   │   ├── store/
│   │   │   │   ├── data.svelte.ts              # Per-surface reactive data (JSON Pointer)
│   │   │   │   ├── dirty.svelte.ts             # Dirty field tracking
│   │   │   │   ├── optimistic.svelte.ts        # Optimistic update registry
│   │   │   │   ├── pointer.ts                  # RFC 6901 JSON Pointer utilities
│   │   │   │   ├── sidebar.svelte.ts           # Sidebar open/close state
│   │   │   │   ├── surfaces.svelte.ts          # Per-surface component tree
│   │   │   │   └── toasts.svelte.ts            # Toast notification queue
│   │   │   └── transport/
│   │   │       ├── dispatcher.ts               # sendAction(), handleMessage(), handler registry
│   │   │       ├── messages.ts                 # TypeScript types mirroring the protocol
│   │   │       └── websocket.svelte.ts         # WebSocket with exponential backoff reconnect
│   │   ├── routes/
│   │   │   ├── +layout.svelte          # Surface mount points, mobile nav
│   │   │   └── +page.svelte            # Calls initMarionette() on mount
│   │   └── static/                     # Static assets
│   ├── build/                          # SvelteKit build output (served by Rust)
│   └── node_modules/
├── spec/                               # OpenAPI 3.1 protocol specification
│   ├── openapi.yaml
│   └── schemas/                        # component.yaml, data.yaml, message.yaml
├── CONCEPT.md                          # OpenSDUI protocol concept document
├── TOOLING.md                          # Development tooling notes
├── Makefile                            # Top-level build commands
└── mise.toml                           # Runtime version manager config
```

## Directory Purposes

**`backend/crates/marionette-protocol/src/`:**
- Purpose: Shared data structures — no runtime behavior, no I/O
- Contains: Rust structs/enums for all wire protocol types with `serde` derives
- Key files: `messages.rs` (the `ProtocolMessage` enum), `component.rs`, `data.rs`, `common.rs`

**`backend/crates/marionette/src/`:**
- Purpose: Reusable toolkit — the "framework" layer that applications import
- Contains: WebSocket session loop, action router, auth, extractors, DB helpers, component builders
- Key files: `ws.rs` (connection lifecycle), `router.rs` (action dispatch), `builders/standard.rs` (component API)

**`backend/crates/marionette-macros/src/`:**
- Purpose: Proc-macro crate for compile-time code generation
- Contains: `ComponentBuilder` derive macro, `#[action]` and `#[requires]` attribute macros
- Key files: `component_builder.rs`, `action.rs`, `requires.rs`

**`backend/crates/crm-demo/src/`:**
- Purpose: CRM application — the reference implementation of a Marionette-based app
- Contains: action handlers, SeaORM entities, migrations, seed data, Listmonk integration
- Key files: `main.rs` (wires everything), `handlers/contact.rs` (most complex handler)

**`frontend/src/lib/`:**
- Purpose: The Marionette frontend library — importable by any SvelteKit app
- Contains: all protocol handling, stores, component registry, built-in components
- Key files: `init.ts` (runtime bootstrap), `transport/dispatcher.ts`, `store/data.svelte.ts`, `components/core/NodeRenderer.svelte`

**`frontend/src/routes/`:**
- Purpose: Thin SvelteKit host shell — mounts surfaces and starts the runtime
- Contains: Only `+layout.svelte` and `+page.svelte`; all real logic is in `$lib`

**`spec/`:**
- Purpose: Machine-readable OpenAPI 3.1 specification of the OpenSDUI protocol
- Contains: `openapi.yaml` and YAML schemas for component, data, message structures
- Generated: No — hand-authored
- Committed: Yes

**`frontend/build/`:**
- Purpose: SvelteKit static build output — served by the Rust backend as the SPA
- Generated: Yes (`pnpm build`)
- Committed: Yes (so the Rust binary can serve it without a separate frontend build step)

## Key File Locations

**Entry Points:**
- `backend/crates/crm-demo/src/main.rs`: Rust `main()` — server startup, action router wiring
- `frontend/src/routes/+page.svelte`: SvelteKit entry — calls `initMarionette()`
- `frontend/src/lib/init.ts`: `initMarionette()` — wires all frontend subsystems together

**Configuration:**
- `backend/Cargo.toml`: Workspace-level dependency versions
- `frontend/package.json`: NPM dependencies
- `mise.toml`: Rust + Node version pinning
- `Makefile`: Top-level `make dev`, `make build`, etc.

**Protocol Types:**
- `backend/crates/marionette-protocol/src/messages.rs`: Rust protocol message types
- `frontend/src/lib/transport/messages.ts`: TypeScript mirror of the same types
- `spec/openapi.yaml`: Authoritative OpenAPI specification

**Core Runtime:**
- `backend/crates/marionette/src/ws.rs`: WebSocket session loop
- `backend/crates/marionette/src/router.rs`: `ActionRouter` — action name → handler dispatch
- `frontend/src/lib/components/core/NodeRenderer.svelte`: Recursive component renderer
- `frontend/src/lib/store/data.svelte.ts`: Reactive per-surface data store

**Component Catalog:**
- `backend/crates/marionette/src/builders/standard.rs`: All 18 Rust component builders
- `frontend/src/lib/registry/defaults.ts`: Component type string → Svelte component mappings

**Domain Handlers (CRM):**
- `backend/crates/crm-demo/src/handlers/contact.rs`: Contact CRUD + table + tag operations
- `backend/crates/crm-demo/src/handlers/user.rs`: User management (admin only)
- `backend/crates/crm-demo/src/handlers/company.rs`: Company CRUD
- `backend/crates/crm-demo/src/handlers/listmonk.rs`: Listmonk mailing list sync

**Database:**
- `backend/crates/crm-demo/src/entities/`: SeaORM entity files (one per table)
- `backend/crates/crm-demo/src/migration/`: Schema migration files

## Naming Conventions

**Files (Rust):**
- Snake case: `action_router.rs`, `contact.rs`, `standard.rs`
- Module files use `mod.rs` for subdirectory modules

**Files (TypeScript/Svelte):**
- Svelte components: `PascalCase.svelte` (e.g., `NodeRenderer.svelte`, `DataTable.svelte`)
- TypeScript modules: `camelCase.ts` or `camelCase.svelte.ts` for Svelte rune modules (e.g., `data.svelte.ts`, `dispatcher.ts`)
- Test files: `name.test.ts` or `name.browser-test.ts`

**Directories:**
- Rust crates: `kebab-case` (e.g., `marionette-protocol`, `crm-demo`)
- Frontend: `kebab-case` directories, matching the component family (e.g., `components/form/`, `components/nav/`)

**Rust identifiers:**
- Handler functions: `handle_<entity>_<action>` (e.g., `handle_contact_list`, `handle_user_save`)
- Builder structs: `PascalCase` matching the component type (e.g., `Button`, `TextInput`, `SideNav`)
- Action name strings: `snake_case` for multi-word (e.g., `"contact_save"`, `"fetch-rows"`)

**Protocol strings (component types, action names, surface names):**
- Component type strings: `kebab-case` (e.g., `"text-input"`, `"data-table"`, `"side-nav"`)
- Action names: typically `snake_case` for domain actions (e.g., `"contact_list"`) and `kebab-case` for framework actions (e.g., `"fetch-rows"`)
- Surface names: lowercase single words (`"main"`, `"sidebar"`, `"modal"`, `"toast"`)

## Where to Add New Code

**New action handler (CRM feature):**
- Implementation: `backend/crates/crm-demo/src/handlers/<entity>.rs`
- Register: add `.action("name", box_handler(handle_fn), AuthRequirement::...)` in `backend/crates/crm-demo/src/main.rs`
- Tests: `backend/crates/crm-demo/tests/`

**New domain entity:**
- SeaORM model: `backend/crates/crm-demo/src/entities/<entity>.rs`
- Migration: `backend/crates/crm-demo/src/migration/` (new `m_<timestamp>_<description>.rs` file)
- Register in: `backend/crates/crm-demo/src/migration/mod.rs`

**New component type (backend builder):**
- Add struct with `#[derive(ComponentBuilder)]` to `backend/crates/marionette/src/builders/standard.rs`
- Re-export from `backend/crates/marionette/src/builders/mod.rs` if needed

**New component type (frontend):**
- Create: `frontend/src/lib/components/<family>/<ComponentName>.svelte`
- Register: add to `registerAll(...)` call in `frontend/src/lib/registry/defaults.ts`

**New frontend store:**
- Create: `frontend/src/lib/store/<name>.svelte.ts` (use `$state` rune for reactivity)
- Export: add public exports to `frontend/src/lib/index.ts`

**New protocol message type:**
- Add variant to `ProtocolMessage` enum in `backend/crates/marionette-protocol/src/messages.rs`
- Mirror the TypeScript type in `frontend/src/lib/transport/messages.ts`
- Register handler in `frontend/src/lib/init.ts`

**Shared utilities:**
- Frontend helpers: `frontend/src/lib/store/pointer.ts` (data path utilities) or new file in `frontend/src/lib/`
- Backend helpers: `backend/crates/marionette/src/` (if reusable across apps) or `backend/crates/crm-demo/src/` (if CRM-specific)

## Special Directories

**`frontend/build/`:**
- Purpose: Pre-built SvelteKit static output committed to the repo
- Generated: Yes (`pnpm build` in `frontend/`)
- Committed: Yes — the Rust server serves it directly with `ServeDir::new("../frontend/build")`

**`frontend/.svelte-kit/`:**
- Purpose: SvelteKit internal build artifacts and generated types
- Generated: Yes
- Committed: No (`.gitignore`)

**`backend/target/`:**
- Purpose: Rust build artifacts
- Generated: Yes
- Committed: No

**`backend/.planning/debug/`:**
- Purpose: Developer debug session notes
- Generated: No (hand-written)
- Committed: Yes

**`.planning/`:**
- Purpose: GSD planning documents — phases, codebase analysis, verification notes
- Generated: Partially (by GSD tooling)
- Committed: Yes

---

*Structure analysis: 2026-04-08*
