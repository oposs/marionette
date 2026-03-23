# Phase 6: CRM Auth & Foundation - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement user authentication, user management, role-based access control, and audit trail for the CRM demo. Users log in with username/password, stay logged in across sessions, and access features based on their role (admin or user). Admins can create, edit, and delete user accounts and assign roles. The system records who changed what and when. Contact/company CRUD is Phase 7 — this phase builds the auth foundation they depend on.

</domain>

<decisions>
## Implementation Decisions

### Login & sessions
- Username/password authentication with bcrypt-hashed passwords
- Session token stored as HTTP-only cookie — survives page reloads
- WebSocket connection reads session cookie on upgrade to identify the user
- Login screen is the first render — backend sends login form if no valid session
- Successful login redirects to main app view via navigate action

### User management
- Admin-only CRUD screens for user accounts using the SDUI pattern (backend renders forms/tables via protocol)
- User table with columns: name, email, role, last login
- User form with fields: name, email, password (create/change), role select
- Default admin account seeded on first startup (configurable credentials via env vars)
- Two roles: `admin` and `user` — simple enum, no complex permission matrix

### Audit trail
- Record-level audit logging: who changed what record and when
- Stored in an `audit_log` table: `audit_log_id`, `audit_log_user`, `audit_log_table`, `audit_log_record_id`, `audit_log_action` (create/update/delete), `audit_log_timestamp`
- Field-level changes stored as JSON diff in `audit_log_changes` column
- Queryable via admin screen — filter by user, table, date range
- Automatically recorded by a middleware/helper — not manually called per handler

### Claude's Discretion
- Exact bcrypt cost factor
- Session token format and expiry duration
- Cookie name and attributes (SameSite, Secure flags)
- Audit log query screen layout and pagination
- How to handle concurrent sessions (allow multiple or single-session)
- Password strength requirements for the demo
- Whether to show audit trail inline on records or as a separate admin screen

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend toolkit (auth infrastructure)
- `backend/crates/marionette/src/auth.rs` — check_auth function, AuthRequirement enum
- `backend/crates/marionette/src/session.rs` — WsSession struct with auth state
- `backend/crates/marionette/src/ws.rs` — WebSocket handler, AppState struct
- `backend/crates/marionette/src/router.rs` — ActionRouter with #[requires] support
- `backend/crates/marionette/src/db.rs` — SeaORM patterns, init_db, test_db

### Backend macros
- `backend/crates/marionette-macros/src/requires.rs` — #[requires] attribute macro
- `backend/crates/marionette-macros/src/action.rs` — #[action] handler macro

### Component builders
- `backend/crates/marionette/src/builders/standard.rs` — Standard component builders (Form, TextInput, DataTable, etc.)

### Protocol
- `spec/PROTOCOL.md` — Message types, surfaces, data binding
- `spec/schemas/message.yaml` — Message schemas

### Conventions
- `TOOLING.md` — SQL conventions (singular table names, prefixed fields, JSON validation)

### Integration
- `backend/crates/crm-demo/src/main.rs` — Current Axum server wiring

### Prior phases
- `.planning/phases/04-backend-toolkit/04-CONTEXT.md` — Two-layer auth, SQLite, SeaORM, action routing
- `.planning/phases/05-integration/05-CONTEXT.md` — E2E wiring, demo handlers

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `auth.rs` already has `check_auth(requirement, session)` for None/Authenticated/Role checks
- `#[requires(authenticated)]` and `#[requires(role = "admin")]` macros already work
- `WsSession` has `user_id`, `username`, `role` fields for auth state
- Standard builders: `Form`, `TextInput`, `Select`, `Button`, `DataTable` — all ready for user management screens
- `init_db` + SeaORM migrations pattern established
- `handle_navigate` + `handle_demo_click` in crm-demo show the handler pattern

### Established Patterns
- Action handlers return `Result<Vec<ProtocolMessage>, ActionError>`
- Component builders use `.child()` chaining with typed props
- SeaORM entities follow TOOLING.md SQL conventions

### Integration Points
- `crm-demo/src/main.rs` — add user/auth routes and handlers here
- `AppState` — extend with auth-related state if needed
- WebSocket upgrade — read session cookie to populate WsSession auth fields
- Login form — first render on unauthenticated WebSocket connect

</code_context>

<specifics>
## Specific Ideas

- Login form is rendered via the protocol like any other SDUI screen — backend detects unauthenticated session and sends a login form render
- The audit trail should be automatic — a middleware or helper that wraps entity mutations, not manual logging in every handler
- Default admin seeded on first run ensures the system is usable immediately

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-crm-auth-foundation*
*Context gathered: 2026-03-23*
