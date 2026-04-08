---
phase: 09-crm-listmonk
plan: 02
subsystem: crm
tags: [listmonk, sync, subscriber, newsletter, wiremock]

# Dependency graph
requires:
  - phase: 09-crm-listmonk
    provides: ListmonkClient HTTP wrapper, listmonk_sync entity, AppState.listmonk field
provides:
  - Single contact sync handler (handle_listmonk_sync)
  - Bulk sync handler (handle_listmonk_sync_all)
  - Sync status badges on contact list and detail views
  - Tag-to-list mapping during sync
  - Blocklist-on-delete for Listmonk subscribers
  - Email-change propagation to Listmonk
affects: [09-crm-listmonk]

# Tech tracking
tech-stack:
  added: [wiremock]
  patterns: [OnceLock static for cross-handler service access, best-effort external API calls]

key-files:
  created:
    - backend/crates/crm-demo/src/handlers/listmonk.rs
  modified:
    - backend/crates/crm-demo/src/handlers/contact.rs
    - backend/crates/crm-demo/src/handlers/mod.rs
    - backend/crates/crm-demo/src/main.rs
    - backend/crates/crm-demo/src/listmonk.rs
    - backend/crates/crm-demo/Cargo.toml
    - backend/Cargo.toml
    - backend/crates/crm-demo/tests/integration_test.rs

key-decisions:
  - "OnceLock static for ListmonkClient access from handlers (avoids modifying marionette library crate)"
  - "Best-effort blocklist on contact delete (warn on failure, never fail the delete)"
  - "Best-effort email change propagation (warn on failure, never fail the save)"
  - "pub(crate) fields on ListmonkClient for wiremock test construction"

patterns-established:
  - "OnceLock pattern: global static for optional service clients accessed from handlers"
  - "Best-effort external calls: tracing::warn on failure, never propagate errors to user-facing operations"

requirements-completed: [CRM-15]

# Metrics
duration: 6min
completed: 2026-03-23
---

# Phase 9 Plan 02: Contact-to-Listmonk Sync Summary

**Contact sync handlers with tag-to-list mapping, sync status badges, blocklist-on-delete, and email-change propagation using wiremock tests**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-23T11:43:22Z
- **Completed:** 2026-03-23T11:49:27Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Single and bulk contact-to-Listmonk sync with tag-to-list mapping
- Sync status badges (Synced/Error/Not synced) on contact list and detail views
- Blocklist subscriber in Listmonk on contact delete (best-effort)
- Email change propagation to Listmonk subscriber on contact save
- Unit tests with wiremock mock HTTP server for create, update, and error scenarios

## Task Commits

Each task was committed atomically:

1. **Task 1: Listmonk sync handlers with email-change propagation and tests** - `5c2fd43` (feat)
2. **Task 2: Add sync button and status badges to contact views** - `602d65d` (feat)

## Files Created/Modified
- `backend/crates/crm-demo/src/handlers/listmonk.rs` - Sync handlers (single + bulk), sync_one_contact core function, OnceLock client access, wiremock tests
- `backend/crates/crm-demo/src/handlers/contact.rs` - Sync status badges in list, sync button/status in detail, blocklist-on-delete, email-change propagation
- `backend/crates/crm-demo/src/handlers/mod.rs` - Registered listmonk handler module
- `backend/crates/crm-demo/src/main.rs` - Registered listmonk_sync and listmonk_sync_all actions, OnceLock initialization
- `backend/crates/crm-demo/src/listmonk.rs` - Made struct fields pub(crate) for test access
- `backend/crates/crm-demo/Cargo.toml` - Added wiremock dev-dependency
- `backend/Cargo.toml` - Added wiremock to workspace dependencies
- `backend/crates/crm-demo/tests/integration_test.rs` - Added listmonk field to AppState construction

## Decisions Made
- Used OnceLock static for ListmonkClient access from handlers -- simplest approach that avoids modifying the marionette library crate
- Made ListmonkClient fields pub(crate) to allow wiremock test construction without an explicit test constructor
- Best-effort pattern for all external Listmonk calls during contact operations (delete, save) -- warn on failure, never propagate

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed integration test missing listmonk field**
- **Found during:** Task 1
- **Issue:** Existing integration_test.rs constructs AppState without the new listmonk field (added in Plan 01)
- **Fix:** Added `listmonk: None` to the AppState construction in integration_test.rs
- **Files modified:** backend/crates/crm-demo/tests/integration_test.rs
- **Verification:** All integration tests pass
- **Committed in:** 5c2fd43 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Auto-fix necessary for test compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Sync handlers are registered and functional
- Plan 03 (subscriber detail panel) can now build on the sync status data and subscriber IDs stored in listmonk_sync table
- All acceptance criteria verified

---
*Phase: 09-crm-listmonk*
*Completed: 2026-03-23*
