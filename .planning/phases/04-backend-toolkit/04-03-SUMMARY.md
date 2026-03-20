---
phase: 04-backend-toolkit
plan: 03
subsystem: api
tags: [action-router, extractors, authorization, sea-orm, async-dispatch]

# Dependency graph
requires:
  - phase: 04-backend-toolkit/02
    provides: "Protocol message types, AuthRequirement enum, action/requires macros"
provides:
  - "ActionRouter with name-based dispatch and BoxedHandler registration"
  - "Typed extractors: Payload<T>, Db, Session with FromHandlerContext trait"
  - "Authorization checking (None/Authenticated/Role)"
  - "ActionError enum with conversion to ErrorMessage"
  - "HandlerContext struct for passing request context to handlers"
affects: [04-backend-toolkit/04, 04-backend-toolkit/05, crm-demo]

# Tech tracking
tech-stack:
  added: [sea-orm mock feature for testing]
  patterns: [action-router-dispatch, typed-extractor, arc-database-connection, box-handler-wrapper]

key-files:
  created:
    - backend/crates/marionette/src/router.rs
    - backend/crates/marionette/src/extractors.rs
    - backend/crates/marionette/src/error.rs
    - backend/crates/marionette/src/auth.rs
  modified:
    - backend/crates/marionette/src/lib.rs
    - backend/crates/marionette/Cargo.toml

key-decisions:
  - "Arc<DatabaseConnection> wrapper for cheap cloning across extractors (DatabaseConnection not Clone)"
  - "box_handler helper function instead of macro for wrapping async fns"
  - "Auth stub created in Task 1 for compilation; tests added in Task 2"

patterns-established:
  - "ActionRouter builder pattern: ActionRouter::new().action(name, handler, auth)"
  - "FromHandlerContext trait for typed extraction from HandlerContext"
  - "BoxedHandler type alias for async handler functions"
  - "ActionError -> Vec<ProtocolMessage> conversion for error responses"

requirements-completed: [BACK-01, BACK-04, BACK-07, BACK-12]

# Metrics
duration: 6min
completed: 2026-03-20
---

# Phase 4 Plan 3: Action Routing Summary

**ActionRouter with name-based dispatch, typed extractors (Payload/Db/Session), and auth checking (None/Authenticated/Role)**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-20T15:11:16Z
- **Completed:** 2026-03-20T15:17:51Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- ActionRouter dispatches incoming actions by name to registered BoxedHandler functions
- Typed extractors (Payload<T>, Db, Session) provide ergonomic handler parameter access via FromHandlerContext trait
- Authorization checking enforces None/Authenticated/Role requirements before handler execution
- ActionError enum converts cleanly to ErrorMessage protocol responses
- 17 tests covering auth logic, router dispatch, and error conversion (all passing, clippy clean)

## Task Commits

Each task was committed atomically:

1. **Task 1: ActionRouter, error types, and typed extractors** - `a48bdfc` (feat)
2. **Task 2: Auth checking logic and router/auth tests** - `78c674e` (test)

## Files Created/Modified
- `backend/crates/marionette/src/router.rs` - ActionRouter with dispatch, BoxedHandler, box_handler helper
- `backend/crates/marionette/src/extractors.rs` - Payload<T>, Db, Session, HandlerContext, FromHandlerContext trait
- `backend/crates/marionette/src/error.rs` - ActionError enum, ActionResult type, protocol message conversion
- `backend/crates/marionette/src/auth.rs` - check_auth function for authorization requirements
- `backend/crates/marionette/src/lib.rs` - Module declarations and re-exports
- `backend/crates/marionette/Cargo.toml` - Added sea-orm, futures deps; mock feature for tests

## Decisions Made
- Used Arc<DatabaseConnection> instead of bare DatabaseConnection because sea-orm's DatabaseConnection does not implement Clone
- Created box_handler as a plain function rather than a macro for simpler ergonomics
- Auth module was created in Task 1 (needed for router compilation) with tests added in Task 2

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] DatabaseConnection not Clone**
- **Found during:** Task 2 (test compilation)
- **Issue:** sea_orm::DatabaseConnection does not implement Clone, causing derive(Clone) on Db to fail
- **Fix:** Wrapped in Arc<DatabaseConnection> for both Db extractor and HandlerContext
- **Files modified:** backend/crates/marionette/src/extractors.rs
- **Verification:** cargo check and cargo test pass
- **Committed in:** 78c674e (Task 2 commit)

**2. [Rule 1 - Bug] Clippy option_as_ref_cloned lint**
- **Found during:** Task 2 (clippy verification)
- **Issue:** .as_ref().cloned() on Option should be .clone() per clippy pedantic
- **Fix:** Simplified to .clone()
- **Files modified:** backend/crates/marionette/src/extractors.rs
- **Verification:** cargo clippy -- -D warnings passes clean
- **Committed in:** 78c674e (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes necessary for compilation and lint compliance. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ActionRouter ready for WebSocket session integration (Plan 04)
- Typed extractors ready for use in crm-demo handler functions (Plan 05)
- Auth checking integrates with #[requires] macro output from Plan 02

---
*Phase: 04-backend-toolkit*
*Completed: 2026-03-20*
