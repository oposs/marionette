---
phase: 09-crm-listmonk
plan: 03
subsystem: crm
tags: [listmonk, mailing-history, cache, subscriber-export]

# Dependency graph
requires:
  - phase: 09-crm-listmonk
    provides: ListmonkClient.get_subscriber_export, listmonk_sync entity, listmonk_cache entity, OnceLock client pattern
provides:
  - Mailing history fetch with 15-minute cache (get_cached_or_fetch_history)
  - Force-refresh handler (handle_listmonk_history_refresh)
  - Mailing History section on contact detail view with DataTable
  - Empty state messaging for unsynced contacts
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [SQLite datetime parsing for cache age comparison, upsert cache pattern (delete + insert)]

key-files:
  created: []
  modified:
    - backend/crates/crm-demo/src/handlers/listmonk.rs
    - backend/crates/crm-demo/src/handlers/contact.rs
    - backend/crates/crm-demo/src/main.rs

key-decisions:
  - "15-minute cache duration for mailing history (configurable via CACHE_DURATION_SECS constant)"
  - "Combined timeline merging campaign_views and link_clicks sorted by date descending"
  - "parse_sqlite_datetime_to_unix helper for cache age comparison using time crate"

patterns-established:
  - "Cache age check: parse SQLite datetime to unix timestamp, compare with now"
  - "History timeline: merge multiple API response arrays into single sorted timeline"

requirements-completed: [CRM-16]

# Metrics
duration: 3min
completed: 2026-03-23
---

# Phase 9 Plan 03: Mailing History Display Summary

**Per-contact mailing history from Listmonk subscriber export with 15-minute local cache, DataTable display, and on-demand refresh**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-23T11:52:00Z
- **Completed:** 2026-03-23T11:55:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Mailing history fetched from Listmonk subscriber export endpoint with campaign views and link clicks
- 15-minute local cache in listmonk_cache table with automatic staleness detection
- Contact detail view shows Mailing History section with DataTable (campaign, date, status)
- Refresh button forces cache invalidation and re-fetch from Listmonk API
- Empty state message when contact is not synced or has no mailing history
- 4 new unit tests validating cache freshness, staleness, empty states

## Task Commits

Each task was committed atomically:

1. **Task 1: Mailing history fetch, cache, and refresh handler with tests** - `5fe66fe` (test, RED) + `9381563` (feat, GREEN)
2. **Task 2: Display mailing history section on contact detail** - `0492d35` (feat)

_Note: Task 1 used TDD with separate RED and GREEN commits._

## Files Created/Modified
- `backend/crates/crm-demo/src/handlers/listmonk.rs` - Added get_cached_or_fetch_history (15-min cache), handle_listmonk_history_refresh, parse_sqlite_datetime_to_unix helper, 4 unit tests
- `backend/crates/crm-demo/src/handlers/contact.rs` - Added Mailing History section with DataTable, refresh button, empty state, mailingHistory data binding
- `backend/crates/crm-demo/src/main.rs` - Registered listmonk_history_refresh action

## Decisions Made
- Used 15-minute cache duration (CACHE_DURATION_SECS constant) as suggested in context
- Combined campaign_views (status: "sent") and link_clicks (status: "clicked") into a single timeline sorted by date descending
- Custom parse_sqlite_datetime_to_unix helper using time crate for cache age comparison (avoids adding chrono dependency)
- Mailing History section placed after Listmonk Sync section for natural flow: sync first, then view history

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- This is the final plan in the final phase -- the CRM Listmonk integration is complete
- All contact sync, status display, and mailing history features are functional
- All acceptance criteria verified

---
*Phase: 09-crm-listmonk*
*Completed: 2026-03-23*
