---
phase: 05-integration
plan: 01
subsystem: integration
tags: [axum, tower-http, websocket, static-serving, spa-fallback, demo]

# Dependency graph
requires:
  - phase: 03-frontend-library
    provides: Built SvelteKit app with WebSocket transport and component rendering
  - phase: 04-backend-toolkit
    provides: ActionRouter, ws_handler, AppState, component builders, session management
provides:
  - Axum server serving SvelteKit static files with SPA fallback
  - WebSocket endpoint at /ws dispatching actions via ActionRouter
  - Demo navigate handler returning render message with component tree
  - Demo click handler returning patch message
  - Health endpoint at /api/health
  - Graceful client hello message handling in ws.rs
  - Integration tests verifying full round-trip and SPA fallback
affects: [05-02, 06-crm-features, e2e-testing]

# Tech tracking
tech-stack:
  added: [reqwest (dev-dependency for HTTP tests)]
  patterns: [ServeDir with SPA fallback, generic JSON type-checking before action dispatch]

key-files:
  created:
    - backend/crates/crm-demo/tests/integration_test.rs
  modified:
    - backend/crates/crm-demo/src/main.rs
    - backend/crates/crm-demo/Cargo.toml
    - backend/crates/marionette/src/ws.rs
    - backend/crates/marionette/tests/ws_integration.rs
    - backend/Cargo.toml
    - Makefile

key-decisions:
  - "Generic JSON parsing before action dispatch to handle hello/action/unknown message types"
  - "MockDatabase for crm-demo since no real DB needed for protocol demo"
  - "SPA fallback test checks for frontend/build existence and skips gracefully if missing"

patterns-established:
  - "Message type routing: parse as serde_json::Value, check type field, then parse as specific message type"
  - "Integration test server: bind to port 0, spawn, construct URLs from local_addr"

requirements-completed: [INTEG-01, INTEG-02]

# Metrics
duration: 5min
completed: 2026-03-23
---

# Phase 5 Plan 01: Integration Summary

**Axum server serving SvelteKit static files with SPA fallback, WebSocket dispatch with hello handling, and demo navigate/click round-trip**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-23T06:34:02Z
- **Completed:** 2026-03-23T06:39:02Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- crm-demo binary serves built SvelteKit app at / with SPA fallback for deep routes
- WebSocket at /ws sends server hello and gracefully handles client hello without errors
- Navigate action returns render message with heading, text, button components
- Demo click action returns patch message updating /message data
- Health endpoint at /api/health for liveness checking
- 5 integration tests covering hello exchange, navigate, click, health, and SPA fallback

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix ws.rs hello handling and wire crm-demo Axum server** - `ff36acd` (feat)
2. **Task 2: Backend integration test for demo round-trip including SPA fallback** - `a546472` (test)

## Files Created/Modified
- `backend/crates/crm-demo/src/main.rs` - Complete Axum server with static serving, WS, health, and demo handlers
- `backend/crates/crm-demo/Cargo.toml` - Added marionette-protocol, serde_json, sea-orm deps; dev-deps for tests
- `backend/crates/crm-demo/tests/integration_test.rs` - 5 integration tests for full round-trip
- `backend/crates/marionette/src/ws.rs` - Generic JSON parsing to route hello vs action messages
- `backend/crates/marionette/tests/ws_integration.rs` - Fixed test messages to include "type": "action"
- `backend/Cargo.toml` - Added "mock" feature to sea-orm workspace dependency
- `Makefile` - Build frontend before backend; added e2e target

## Decisions Made
- Generic JSON parsing before action dispatch: parse as serde_json::Value, check "type" field, then dispatch. This avoids errors when frontend sends hello messages.
- MockDatabase for crm-demo: no real DB needed for protocol demo, avoids SQLite setup complexity.
- SPA fallback test gracefully skips if frontend/build is not present, with clear message to run `make build`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed existing WS integration tests to include "type": "action"**
- **Found during:** Task 1 (ws.rs hello handling)
- **Issue:** Existing ws_integration.rs tests sent action messages without "type": "action" field. After adding message type routing, these messages were rejected as "unexpected message type: "
- **Fix:** Added `"type": "action"` to all action messages in ws_integration.rs tests
- **Files modified:** backend/crates/marionette/tests/ws_integration.rs
- **Verification:** All 5 existing WS integration tests pass
- **Committed in:** ff36acd (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Necessary fix for correctness. The test messages were not conforming to the protocol spec (missing type discriminator).

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- crm-demo binary is fully functional: serves frontend, handles WebSocket protocol, dispatches actions
- Ready for Plan 02 (E2E Playwright tests) to validate full browser-to-backend round-trip
- Ready for Phase 6+ to add real CRM business logic handlers

---
*Phase: 05-integration*
*Completed: 2026-03-23*
