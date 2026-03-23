---
phase: 09-crm-listmonk
plan: 01
subsystem: api
tags: [reqwest, listmonk, sea-orm, http-client, newsletter]

# Dependency graph
requires:
  - phase: 08-crm-features
    provides: contact entities, tag system, interaction tables
provides:
  - ListmonkClient struct with all Listmonk API methods
  - listmonk_sync and listmonk_cache SeaORM entities and migrations
  - AppState extension field for external service clients
  - reqwest as workspace runtime dependency
affects: [09-02-PLAN, 09-03-PLAN]

# Tech tracking
tech-stack:
  added: [reqwest (runtime)]
  patterns: [type-erased AppState extension via Arc<dyn Any>, optional external service client]

key-files:
  created:
    - backend/crates/crm-demo/src/listmonk.rs
    - backend/crates/crm-demo/src/entities/listmonk_sync.rs
    - backend/crates/crm-demo/src/entities/listmonk_cache.rs
    - backend/crates/crm-demo/src/migration/m20260323_000009_create_listmonk_sync.rs
    - backend/crates/crm-demo/src/migration/m20260323_000010_create_listmonk_cache.rs
  modified:
    - backend/Cargo.toml
    - backend/crates/crm-demo/Cargo.toml
    - backend/crates/crm-demo/src/entities/mod.rs
    - backend/crates/crm-demo/src/migration/mod.rs
    - backend/crates/marionette/src/ws.rs
    - backend/crates/crm-demo/src/main.rs

key-decisions:
  - "Type-erased extension field (Arc<dyn Any + Send + Sync>) on AppState to avoid leaking CRM-specific types into marionette library crate"
  - "reqwest added to workspace dependencies with json feature for .json() request builder"

patterns-established:
  - "Optional external service client: from_env() returns None when env vars missing, validate_connection() with timeout on startup"
  - "Type-erased AppState extension: store Arc<ConcreteType> as Arc<dyn Any>, downcast in handlers"

requirements-completed: [CRM-15, CRM-16]

# Metrics
duration: 2min
completed: 2026-03-23
---

# Phase 9 Plan 01: Listmonk Foundation Summary

**ListmonkClient HTTP wrapper with reqwest, listmonk_sync/cache entities and migrations, type-erased AppState extension field**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-23T11:38:23Z
- **Completed:** 2026-03-23T11:40:30Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- ListmonkClient with all API methods: subscriber CRUD, list management, blocklist, export
- listmonk_sync table for tracking sync status per contact with error capture
- listmonk_cache table for storing mailing history JSON blobs
- AppState extended with type-erased extension field for external service clients
- CRM starts gracefully when Listmonk is not configured or unreachable

## Task Commits

Each task was committed atomically:

1. **Task 1: ListmonkClient, entities, and migrations** - `3627b46` (feat)
2. **Task 2: Extend AppState with listmonk field and wire startup** - `2698a5a` (feat)

## Files Created/Modified
- `backend/crates/crm-demo/src/listmonk.rs` - ListmonkClient with all API methods
- `backend/crates/crm-demo/src/entities/listmonk_sync.rs` - SeaORM entity for sync status
- `backend/crates/crm-demo/src/entities/listmonk_cache.rs` - SeaORM entity for mailing history cache
- `backend/crates/crm-demo/src/migration/m20260323_000009_create_listmonk_sync.rs` - Sync table migration
- `backend/crates/crm-demo/src/migration/m20260323_000010_create_listmonk_cache.rs` - Cache table migration
- `backend/Cargo.toml` - Added reqwest to workspace dependencies
- `backend/crates/crm-demo/Cargo.toml` - Moved reqwest from dev to runtime dependency
- `backend/crates/crm-demo/src/entities/mod.rs` - Registered listmonk entities
- `backend/crates/crm-demo/src/migration/mod.rs` - Registered listmonk migrations
- `backend/crates/marionette/src/ws.rs` - Added listmonk extension field to AppState
- `backend/crates/crm-demo/src/main.rs` - Wired ListmonkClient initialization at startup

## Decisions Made
- Used type-erased extension field (`Option<Arc<dyn Any + Send + Sync>>`) on AppState to avoid leaking CRM-specific types into the marionette library crate
- Added reqwest to workspace with `json` feature (in addition to `rustls-tls`) for `.json()` request builder support

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required. Listmonk integration is optional and the CRM starts without it.

## Next Phase Readiness
- ListmonkClient is available on AppState for handler use
- listmonk_sync and listmonk_cache tables will be created on next migration run
- Ready for sync handler implementation (09-02) and mailing history display (09-03)

---
*Phase: 09-crm-listmonk*
*Completed: 2026-03-23*
