# Phase 4: Backend Toolkit - Context

**Gathered:** 2026-03-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the complete Marionette Rust backend toolkit: protocol message types (hand-written, validated against spec), derive macros for typed component builders and action handler registration, Axum-integrated WebSocket session management, action routing with typed extractors, SeaORM entity patterns for SQLite persistence, and permission/authorization utilities. The toolkit lives across three crates (marionette-protocol, marionette-macros, marionette) and is consumed by the crm-demo crate.

</domain>

<decisions>
## Implementation Decisions

### Component builder ergonomics
- Builder pattern (fluent method chain), NOT proc macro DSL
- Full IDE support, clear compiler errors, normal Rust control flow for conditionals and loops
- All component types get typed builders via `#[derive(ComponentBuilder)]` — standard components ship pre-built in the marionette crate, projects derive their own for custom types using the same macro
- No generic props map — all props are typed struct fields, the derive macro generates builder methods
- Both `.child()` (chaining, conditionals) and `.children(vec![...])` (bulk static lists) for nesting — both append to the same internal Vec

### Action routing
- Name-based dispatch: handler functions registered by action name, similar to Axum's router
- Axum-style typed extractors for handler parameters (payload, DB, session)
- `#[action(name = "save-contact")]` derive macro on handler functions generates:
  - Action name constant (`pub const SAVE_CONTACT: &str = "save-contact"`)
  - Auto-registration in the router
- Component builders reference action name constants: `Button::new("Save").action(Action::submit(actions::SAVE_CONTACT))`
- Single source of truth for action names — no string duplication between component builders and router

### Database & persistence
- SQLite everywhere — dev and production. Zero setup, single file, embedded
- SeaORM for ORM with migrations
- Handlers ARE the business logic layer — no separate entity-to-SDUI mapping framework
- Handlers query DB, apply business logic (joins, filtering, permissions, computed fields), return SDUI messages
- The toolkit provides ergonomic builders and extractors; the logic inside handlers is application-specific

### Authorization
- Two-layer approach:
  - **Coarse-grained (declarative):** `#[requires(authenticated)]` or `#[requires(role = "admin")]` attributes on handler functions. Framework enforces before handler runs. Can't forget the basics.
  - **Fine-grained (manual):** Row-level and field-level permission checks inside handlers (e.g., "owner or admin" checks). Business logic that depends on the data.
- Handlers without `#[requires]` are public (e.g., navigate)

### Claude's Discretion
- Exact WebSocket session management implementation (ping/pong, session state)
- SeaORM migration file structure and naming
- Error response format for unauthorized actions
- How the `#[derive(ComponentBuilder)]` macro generates the builder methods internally
- Integration test harness design (test database setup/teardown)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol specification (source of truth for types)
- `spec/PROTOCOL.md` — Authoritative protocol manual: message types, transport, data binding, error handling
- `spec/openapi.yaml` — OpenAPI 3.1 entry point
- `spec/schemas/message.yaml` — All 6 message type schemas (Rust types must match these)
- `spec/schemas/component.yaml` — Component structure (builders must produce this shape)
- `spec/schemas/data.yaml` — PatchOperation, KeyedCollection, ValidationError
- `spec/schemas/common.yaml` — Surface, JsonPointer, MessageId

### Project definition
- `TOOLING.md` — Tech stack (axum, sea-orm, utoipa for API docs), SQL conventions (table naming, field naming, JSON constraints)
- `.planning/REQUIREMENTS.md` — BACK-01 through BACK-15 requirements

### Prior phases
- `.planning/phases/01-project-infrastructure/01-CONTEXT.md` — Cargo workspace structure, crate layout
- `.planning/phases/02-protocol-specification/02-CONTEXT.md` — WebSocket-only transport, message envelope, hand-written Rust types
- `.planning/phases/03-frontend-library/03-CONTEXT.md` — Frontend component registry, message types consumed by frontend

### Existing code
- `backend/Cargo.toml` — Workspace root with shared dependencies
- `backend/crates/marionette-protocol/Cargo.toml` — Protocol types crate (serde, serde_json)
- `backend/crates/marionette-macros/Cargo.toml` — Proc macro crate
- `backend/crates/marionette/Cargo.toml` — Main library crate (axum, tower-http, tracing)
- `backend/crates/crm-demo/Cargo.toml` — Binary crate consuming the toolkit

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `backend/Cargo.toml` workspace already has serde, serde_json, tokio, axum, tower-http, tracing as shared dependencies
- `backend/crates/marionette-protocol/src/lib.rs` — stub with clippy pedantic, ready for protocol types
- `backend/crates/marionette-macros/src/lib.rs` — stub, ready for proc macros
- `backend/crates/marionette/src/lib.rs` — stub, ready for framework code
- `backend/crates/crm-demo/src/main.rs` — has tokio::main and tracing init

### Established Patterns
- Edition 2024 with resolver 3
- `#![warn(clippy::pedantic)]` and `#![allow(clippy::module_name_repetitions)]` on all crates
- Workspace dependency inheritance (`edition.workspace = true`, `serde.workspace = true`)

### Integration Points
- `spec/schemas/*.yaml` — Rust types must serialize to match these schemas
- `frontend/src/lib/transport/messages.ts` — TypeScript interfaces that must align with Rust types
- Vite proxy: `/ws` → `ws://localhost:3001` — WebSocket endpoint the frontend connects to
- `make dev` starts both frontend and backend concurrently

</code_context>

<specifics>
## Specific Ideas

- The derive macro approach (`#[derive(ComponentBuilder)]`, `#[action]`) is central to the developer experience — it eliminates string duplication, provides compile-time safety, and keeps the API ergonomic
- SQLite everywhere keeps deployment simple — single binary + single file database, no infrastructure dependencies
- Handlers as the business logic layer (not a mapping framework) mirrors CallBackery's approach — the framework provides ergonomics, the developer writes the logic
- Two-layer auth (declarative attributes + manual checks) prevents security oversights while supporting real-world permission complexity

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-backend-toolkit*
*Context gathered: 2026-03-20*
