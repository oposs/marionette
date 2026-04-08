---
phase: 06-crm-auth-foundation
plan: 01
subsystem: auth
tags: [bcrypt, session-cookie, sea-orm, axum-extra, sqlite, websocket-auth]

# Dependency graph
requires:
  - phase: 04-backend-toolkit
    provides: ActionRouter, AppState, WsSession, session entity, builders
  - phase: 05-integration
    provides: E2E wiring, crm-demo binary, integration tests
provides:
  - User and audit_log SeaORM entities with SQLite migrations
  - HTTP login/logout endpoints with bcrypt credential verification
  - Session cookie auth on WebSocket upgrade
  - Login form SDUI render for unauthenticated sessions
  - Default admin seeding on first startup
affects: [06-crm-auth-foundation, 07-crm-contacts]

# Tech tracking
tech-stack:
  added: [bcrypt 0.19, chrono 0.4, axum-extra 0.12, time 0.3]
  patterns: [HTTP-cookie-based session auth, spawn_blocking for bcrypt, login_form on AppState]

key-files:
  created:
    - backend/crates/crm-demo/src/entities/user.rs
    - backend/crates/crm-demo/src/entities/audit_log.rs
    - backend/crates/crm-demo/src/entities/mod.rs
    - backend/crates/crm-demo/src/migration/mod.rs
    - backend/crates/crm-demo/src/migration/m20260323_000001_create_user.rs
    - backend/crates/crm-demo/src/migration/m20260323_000002_create_audit_log.rs
    - backend/crates/crm-demo/src/seed.rs
    - backend/crates/crm-demo/src/handlers/auth.rs
    - backend/crates/crm-demo/src/handlers/mod.rs
  modified:
    - backend/Cargo.toml
    - backend/crates/crm-demo/Cargo.toml
    - backend/crates/crm-demo/src/main.rs
    - backend/crates/marionette/Cargo.toml
    - backend/crates/marionette/src/ws.rs
    - backend/crates/marionette/src/session.rs

key-decisions:
  - "AppState gains optional login_form field for generic unauthenticated session rendering"
  - "CRM-demo has its own Migrator separate from marionette's for app-specific tables"
  - "time crate added as workspace dependency for cookie max_age Duration type"

patterns-established:
  - "HTTP POST login -> session cookie -> WS reconnect for authentication flow"
  - "AppState.login_form: Option<ProtocolMessage> for app-specific login rendering"
  - "crm-demo modules: entities/, handlers/, migration/, seed.rs"

requirements-completed: [CRM-13]

# Metrics
duration: 7min
completed: 2026-03-23
---

# Phase 06 Plan 01: CRM Auth Foundation Summary

**Bcrypt login with HTTP-only session cookies, WebSocket cookie auth on upgrade, and SDUI login form for unauthenticated sessions**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-23T08:15:38Z
- **Completed:** 2026-03-23T08:22:38Z
- **Tasks:** 2
- **Files modified:** 16

## Accomplishments
- User and audit_log entities with SQLite migrations following TOOLING.md conventions
- HTTP login/logout endpoints with bcrypt verification and session cookie management
- WebSocket handler reads session cookie on upgrade and populates WsSession auth fields
- Unauthenticated WebSocket connections receive SDUI login form render
- Default admin account seeded on first startup (env-configurable credentials)
- Real SQLite database replaces MockDatabase in crm-demo

## Task Commits

Each task was committed atomically:

1. **Task 1: Database entities, migrations, seed, and login endpoint** - `42e18c1` (feat)
2. **Task 2: WebSocket session auth from cookie and login form render** - `92e18ff` (feat)

## Files Created/Modified
- `backend/crates/crm-demo/src/entities/user.rs` - User SeaORM entity (user table)
- `backend/crates/crm-demo/src/entities/audit_log.rs` - AuditLog SeaORM entity
- `backend/crates/crm-demo/src/migration/m20260323_000001_create_user.rs` - User table migration
- `backend/crates/crm-demo/src/migration/m20260323_000002_create_audit_log.rs` - Audit log table migration
- `backend/crates/crm-demo/src/seed.rs` - Default admin seeding with bcrypt
- `backend/crates/crm-demo/src/handlers/auth.rs` - Login/logout HTTP handlers
- `backend/crates/crm-demo/src/main.rs` - Real SQLite, migrations, routes, login form
- `backend/crates/marionette/src/ws.rs` - Cookie extraction, session DB lookup, login form send
- `backend/crates/marionette/src/session.rs` - WsSession::with_auth constructor

## Decisions Made
- AppState gains `login_form: Option<ProtocolMessage>` field -- keeps ws.rs generic while allowing app-specific login rendering
- CRM-demo has its own Migrator (separate from marionette's) because user/audit_log are app-specific tables
- Added `time` crate as workspace dependency for `cookie::time::Duration` type compatibility with axum-extra CookieBuilder
- navigate and demo_click actions now require `AuthRequirement::Authenticated`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `time` crate for cookie max_age Duration**
- **Found during:** Task 1 (login endpoint)
- **Issue:** `cookie::time::Duration` not accessible through axum-extra re-exports
- **Fix:** Added `time = "0.3"` as workspace dependency
- **Files modified:** backend/Cargo.toml, backend/crates/crm-demo/Cargo.toml
- **Verification:** `cargo check -p crm-demo` succeeds
- **Committed in:** 42e18c1 (Task 1 commit)

**2. [Rule 3 - Blocking] Updated integration tests for new AppState field**
- **Found during:** Task 2 (WebSocket auth)
- **Issue:** Adding `login_form` to AppState broke existing integration tests
- **Fix:** Added `login_form: None` to AppState constructors in test files
- **Files modified:** backend/crates/crm-demo/tests/integration_test.rs, backend/crates/marionette/tests/ws_integration.rs
- **Verification:** `cargo test --workspace` passes (all 15 tests)
- **Committed in:** 92e18ff (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Auth foundation complete: login, sessions, cookie auth all wired
- Ready for Plan 02 (user management CRUD screens) and Plan 03 (audit trail)
- Frontend WebSocket reconnection after login will use existing reconnect logic

---
*Phase: 06-crm-auth-foundation*
*Completed: 2026-03-23*
