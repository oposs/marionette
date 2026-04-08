# Architecture

**Analysis Date:** 2026-04-08

## Pattern Overview

**Overall:** Server-Driven UI (SDUI) over WebSocket with an adjacency-list component model

This is the **OpenSDUI protocol** reference implementation called **Marionette**. The backend is the "puppet master" — it decides what UI to render, what data to display, and what business logic to execute. The frontend is a "smart puppet" — it knows how to render component types, bind data reactively, and dispatch user actions, but does not contain business logic.

**Key Characteristics:**
- All UI decisions live in the Rust backend; frontend is a generic renderer
- Communication is exclusively over a WebSocket (`/ws`) using JSON messages with a `type` discriminator
- Component trees are flat adjacency lists (node ID → `Component` struct), not nested trees
- Data binding uses RFC 6901 JSON Pointer paths (`/user/name`, `/users/u-123/email`)
- The backend is stateless per-message; session state is held by the frontend WebSocket connection and a DB-persisted session cookie
- Errors are protocol messages (`type: "error"`), not HTTP error codes

## Layers

**Protocol (`marionette-protocol`):**
- Purpose: Shared data types for messages, components, and data structures — no behavior
- Location: `backend/crates/marionette-protocol/src/`
- Contains: `ProtocolMessage` enum, `Component` struct, `PatchOperation`, `ValidationError`, `AuthRequirement`, type aliases
- Depends on: `serde`, `serde_json` only
- Used by: both `marionette` (toolkit) and `crm-demo` (application)

**Toolkit (`marionette`):**
- Purpose: Reusable backend framework — WebSocket lifecycle, action routing, session, auth, DB utilities
- Location: `backend/crates/marionette/src/`
- Contains: `ws_handler`, `AppState`, `ActionRouter`, `HandlerContext`, `Session`, `auth::check_auth`, builder types
- Depends on: `marionette-protocol`, `axum`, `sea-orm`, `tokio`
- Used by: application crates (currently `crm-demo`)

**Proc-Macros (`marionette-macros`):**
- Purpose: Compile-time code generation for component builders and action metadata
- Location: `backend/crates/marionette-macros/src/`
- Contains: `#[derive(ComponentBuilder)]`, `#[action]`, `#[requires]` macros
- Depends on: `syn`, `darling`, `proc-macro2`
- Used by: `marionette` (standard builders), application crates

**Application (`crm-demo`):**
- Purpose: CRM reference app — action handlers, domain entities, migrations, business logic
- Location: `backend/crates/crm-demo/src/`
- Contains: `main.rs` (server startup + action router wiring), `handlers/`, `entities/`, `migration/`, `seed/`, `listmonk/`, `audit/`
- Depends on: `marionette`, `marionette-protocol`, `sea-orm`, `bcrypt`, `axum`
- Used by: deployed as the runnable binary

**Frontend Library (`marionette` npm package):**
- Purpose: Svelte 5 SDUI runtime — WebSocket transport, reactive stores, component registry, protocol message handling
- Location: `frontend/src/lib/`
- Contains: `init.ts` (entry point), `transport/`, `store/`, `registry/`, `routing/`, `components/`
- Depends on: SvelteKit, Flowbite-Svelte, Svelte 5 runes
- Used by: the SvelteKit host app in `frontend/src/routes/`

**Frontend App (SvelteKit host):**
- Purpose: Thin shell that mounts the Marionette surfaces and initializes the runtime
- Location: `frontend/src/routes/`
- Contains: `+layout.svelte` (surface mount points), `+page.svelte`
- Depends on: `$lib` (the Marionette library)

## Data Flow

**Outbound (Backend → Frontend):**

1. WebSocket connection arrives at `GET /ws` → `ws_handler` in `backend/crates/marionette/src/ws.rs`
2. Session cookie `marionette_session` is looked up in the DB; `WsSession` is populated
3. Server sends `Hello { version }` then either the login form (unauthenticated) or nothing (authenticated session resumes via a `navigate` action)
4. After `Hello`, the frontend router calls `sendAction('navigate', { path })` which triggers a render
5. Backend responds with one or more `ProtocolMessage::Render` (one per surface) containing a flat `nodes` map and `data` JSON
6. Frontend `init.ts` routes `render` messages → `setFullState(surface, data)` + `setSurfaceTree(surface, root, nodes)`
7. Svelte reactivity re-renders `<Surface name="main" />` which calls `NodeRenderer` recursively

**Inbound (Frontend → Backend):**

1. User interacts with a component → component calls `sendAction(name, payload, source)`
2. `dispatcher.ts` generates a UUID correlation ID, optionally applies an optimistic patch, sends `ActionMessage` over WebSocket
3. `ws.rs` `read_loop` receives the text frame, parses as `ProtocolMessage`, dispatches to `ActionRouter::dispatch`
4. `ActionRouter` looks up the handler by `action.name`, runs `check_auth`, calls the handler with `HandlerContext { action, db, session }`
5. Handler returns `ActionResult` = `Ok(Vec<ProtocolMessage>)` or `Err(ActionError)`
6. Errors convert to `ProtocolMessage::Error` via `From<ActionError>`; responses are sent back through the mpsc channel

**Patch Flow:**

- Backend can send incremental `PatchMessage { patch: [{ path, value }] }` to update specific data paths without re-rendering
- Frontend `applyPatch('main', ops)` skips paths marked dirty (field actively being edited) and queues them instead
- Optimistic updates: frontend applies patch locally immediately, rolls back on `Error` response with matching correlation ID

**State Management:**
- Per-surface reactive data store in `frontend/src/lib/store/data.svelte.ts` using Svelte 5 `$state` rune
- Component tree per surface in `frontend/src/lib/store/surfaces.svelte.ts`
- Dirty field tracking in `frontend/src/lib/store/dirty.svelte.ts` (prevents server patches clobbering active edits)
- Optimistic update registry in `frontend/src/lib/store/optimistic.svelte.ts`
- Sidebar open/close state in `frontend/src/lib/store/sidebar.svelte.ts`
- Toast notification queue in `frontend/src/lib/store/toasts.svelte.ts`

## Key Abstractions

**`ProtocolMessage` (Rust enum / TypeScript union):**
- Purpose: The wire format discriminated union covering all message types
- Rust: `backend/crates/marionette-protocol/src/messages.rs`
- TypeScript: `frontend/src/lib/transport/messages.ts`
- Pattern: `#[serde(tag = "type", rename_all = "lowercase")]` for JSON serialization; `{ type: 'render' | 'patch' | 'action' | 'event' | 'error' | 'hello' }`

**`Component` (adjacency list node):**
- Purpose: Represents one node in the UI tree — type string, props, children IDs, bind path, action, visibility
- Files: `backend/crates/marionette-protocol/src/component.rs`, `frontend/src/lib/transport/messages.ts` (`ComponentNode`)
- Pattern: `type` is an open string (e.g., `"button"`, `"data-table"`); frontend registry maps strings to Svelte components

**`ActionRouter` (backend):**
- Purpose: Routes `action.name` strings to handler functions with auth enforcement
- File: `backend/crates/marionette/src/router.rs`
- Pattern: Builder pattern `.action(name, box_handler(fn), AuthRequirement::Authenticated)` — registered at startup in `main.rs`

**`HandlerContext` (backend):**
- Purpose: Dependency bag passed to every action handler
- File: `backend/crates/marionette/src/extractors.rs`
- Fields: `action: ActionMessage`, `db: Arc<DatabaseConnection>`, `session: Session`

**Component Registry (frontend):**
- Purpose: Maps component type strings to Svelte component constructors
- Files: `frontend/src/lib/registry/registry.ts`, `frontend/src/lib/registry/defaults.ts`
- Pattern: `register('button', Button)` / `getComponent('button')` — called by `NodeRenderer.svelte`

**`Surface` (frontend):**
- Purpose: Named render target (`"main"`, `"sidebar"`, `"modal"`, `"toast"`) — mounts the root `NodeRenderer` for that surface
- File: `frontend/src/lib/components/core/Surface.svelte`
- Pattern: `<Surface name="main" />` in the layout; driven by `getSurfaceTree(name)` from the store

**Standard Component Builders (backend):**
- Purpose: Fluent Rust API for constructing protocol `Component` values without raw JSON
- File: `backend/crates/marionette/src/builders/standard.rs`
- Pattern: `Button::new("Save").variant("primary").action(ComponentAction::submit("save")).build()` → `(id, Component)`; `#[derive(ComponentBuilder)]` generates the API

## Entry Points

**Backend server:**
- Location: `backend/crates/crm-demo/src/main.rs`
- Triggers: `cargo run` in `backend/`; listens on `0.0.0.0:3001`
- Responsibilities: DB connect + migrate, seed demo data, wire `ActionRouter`, create `AppState`, register HTTP routes (`/ws`, `/api/health`, `/api/login`, `/api/logout`), serve static frontend build as SPA fallback

**WebSocket upgrade:**
- Location: `backend/crates/marionette/src/ws.rs` — `ws_handler`
- Triggers: HTTP `GET /ws` upgrade from any client
- Responsibilities: extract session cookie, authenticate `WsSession`, spawn write loop, send `Hello`, send login form if unauthenticated, run read loop

**Frontend runtime:**
- Location: `frontend/src/lib/init.ts` — `initMarionette()`
- Triggers: called from `frontend/src/routes/+page.svelte` on mount
- Responsibilities: register default components, register protocol message handlers, connect WebSocket, initialize URL router which sends the first `navigate` action

**SvelteKit layout:**
- Location: `frontend/src/routes/+layout.svelte`
- Triggers: SvelteKit page load
- Responsibilities: render the two primary surfaces (`sidebar`, `main`) and overlay surfaces (`ModalSurface`, `ToastSurface`), mobile hamburger menu

## Error Handling

**Strategy:** Errors are protocol messages — `ProtocolMessage::Error { errors: Vec<ValidationError> }` — never HTTP status codes over the WebSocket.

**Patterns:**
- `ActionError` enum (`NotFound`, `Unauthorized`, `BadPayload`, `Internal`) implements `From<ActionError> for Vec<ProtocolMessage>` — handlers return `ActionResult` and errors auto-convert to error messages
- Frontend `init.ts` error handler stores errors at `/_errors` in the main surface data store AND shows toasts via `addToast('error', message)`
- Optimistic rollback: if an `Error` message has a correlation ID, `rollbackOptimistic(id)` reverts the local optimistic patch
- `ErrorBoundary.svelte` wraps each `NodeRenderer` to prevent a single bad component from crashing the surface

## Cross-Cutting Concerns

**Logging:** `tracing` crate; `tracing_subscriber::fmt::init()` at startup; handlers use `debug!`, `info!`, `warn!`, `error!` macros; session ID included in all log entries

**Validation:** Backend validates in handlers using `ActionError::BadPayload`; errors sent as `ProtocolMessage::Error`; client-side validation is a component prop concern (`required: true`, `pattern: "..."`)

**Authentication:** Session cookie (`marionette_session`) checked on WebSocket upgrade; inline auth state update on `login` action success (auth info embedded in render response data under `_auth_user_id` / `_auth_role` keys); `ActionRouter` enforces `AuthRequirement` before calling any handler

**Database:** SeaORM with SQLite (`crm.db`); entities in `backend/crates/crm-demo/src/entities/`; migrations via `sea-orm-migration` in `backend/crates/crm-demo/src/migration/`

---

*Architecture analysis: 2026-04-08*
