# Phase 5: Integration - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the Marionette frontend library and backend toolkit together into a working end-to-end system. Axum serves the built SvelteKit app as static files, WebSocket connects and establishes the protocol handshake, a complete action→render→patch round-trip works, and protocol messages are validated against the OpenAPI schemas. This phase produces a working demo that proves the stack, not CRM business logic (that's Phase 6+).

</domain>

<decisions>
## Implementation Decisions

### Static file serving
- Axum serves the built SvelteKit app (`frontend/build/`) as static files using tower-http ServeDir
- SPA fallback: all non-file routes serve `index.html` (adapter-static with `fallback: 'index.html'` already configured in Phase 1)
- `make build` produces both `frontend/build/` and the Rust binary

### WebSocket integration
- Single WebSocket at `/ws` — frontend connects on page load via the transport module (Phase 3)
- Backend ws_handler (Phase 4) upgrades the connection, sends hello, dispatches actions via ActionRouter
- Vite proxy handles `/ws` in dev mode; production serves everything from Axum directly

### Demo content for E2E
- Minimal but realistic: a "hello" screen that backend renders via the protocol
- Backend registers a `navigate` action handler that returns a render message with a few components (heading, text, button)
- Button click sends an action, backend responds with a patch — proving the full round-trip
- No database required for the demo — pure in-memory protocol exercise

### Protocol conformance
- Validate WebSocket messages against the OpenAPI schemas at test time
- Use the bundled `spec/openapi.yaml` as the schema source
- Playwright E2E tests capture WebSocket frames and validate structure

### Claude's Discretion
- Exact demo screen content and component tree
- How to capture and validate WebSocket frames in Playwright tests
- Whether to add a `/api/health` REST endpoint for basic liveness checking
- Build script orchestration details in Makefile
- Error handling for missing `frontend/build/` directory

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol specification
- `spec/PROTOCOL.md` — Authoritative protocol manual (message types, transport, handshake)
- `spec/openapi.yaml` — OpenAPI 3.1 schemas for conformance validation
- `spec/schemas/message.yaml` — Message type schemas

### Frontend (what connects)
- `frontend/src/lib/transport/websocket.svelte.ts` — WebSocket client with reconnection
- `frontend/src/lib/transport/dispatcher.ts` — Message routing by type
- `frontend/src/lib/init.ts` — App initialization wiring
- `frontend/src/lib/store/surfaces.svelte.ts` — Per-surface state management

### Backend (what serves)
- `backend/crates/marionette/src/ws.rs` — WebSocket handler, AppState
- `backend/crates/marionette/src/router.rs` — ActionRouter
- `backend/crates/crm-demo/src/main.rs` — Binary entry point

### Infrastructure
- `Makefile` — Build targets (dev, build, test)
- `frontend/svelte.config.js` — adapter-static with SPA fallback
- `frontend/vite.config.ts` — Vite proxy configuration

### Prior phase contexts
- `.planning/phases/01-project-infrastructure/01-CONTEXT.md` — Dev server, Vite proxy, Makefile targets
- `.planning/phases/02-protocol-specification/02-CONTEXT.md` — WebSocket-only, message types
- `.planning/phases/03-frontend-library/03-CONTEXT.md` — Data store, component registry, surfaces
- `.planning/phases/04-backend-toolkit/04-CONTEXT.md` — Action routing, WebSocket sessions, builders

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Frontend WebSocket transport connects to `/ws` and handles hello handshake automatically
- Frontend init.ts wires dispatcher handlers for render/patch/event/error messages
- Backend ws_handler dispatches actions via ActionRouter and sends responses
- Backend AppState holds ActionRouter and DatabaseConnection
- crm-demo main.rs has tokio::main and tracing init — ready to add Axum router

### Established Patterns
- Frontend: Svelte 5 runes, $state stores, typed message interfaces
- Backend: Axum handlers, typed extractors, serde tagged unions
- Testing: Vitest (frontend unit), vitest-browser-svelte (components), Playwright (E2E), cargo test (backend)

### Integration Points
- `frontend/build/` → Axum ServeDir (production static serving)
- `/ws` → ws_handler (WebSocket upgrade)
- Frontend init() → WebSocket connect → hello → navigate action → render response
- Makefile `make dev` starts both servers concurrently with Vite proxy

</code_context>

<specifics>
## Specific Ideas

- The integration demo should be minimal — just enough to prove the full round-trip works
- No database needed for the demo — keep it pure protocol exercise
- Playwright E2E tests should capture actual WebSocket frames to validate conformance

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-integration*
*Context gathered: 2026-03-23*
