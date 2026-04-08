# Phase 6: CRM Auth & Foundation - Research

**Researched:** 2026-03-23
**Domain:** Authentication, RBAC, audit logging in Rust/Axum SDUI architecture
**Confidence:** HIGH

## Summary

Phase 6 implements user authentication (login/sessions), user management (admin CRUD), role-based access control (admin/user), and an automatic audit trail for the CRM demo. The existing marionette toolkit already provides the core building blocks: `check_auth`, `#[requires]` macros, `WsSession` with auth fields, component builders (Form, TextInput, Select, DataTable, Button), SeaORM migrations, and the action router with auth requirement dispatch. The main work is wiring these together: adding a `user` table, a login action handler, cookie-based session persistence, admin screens via SDUI protocol, and an audit logging middleware.

The architecture is WebSocket-first. The session cookie is set via a standard HTTP login endpoint and read during the WebSocket upgrade. Once authenticated, the `WsSession` is populated with user_id/roles and all subsequent action dispatches go through the existing `check_auth` pipeline. User management and audit trail screens are rendered as SDUI -- backend sends forms/tables via the protocol, exactly like the existing demo.

**Primary recommendation:** Use bcrypt 0.19 for password hashing, extend the existing session table with cookie-based tokens, add `user` and `audit_log` tables via SeaORM migrations, and implement all screens as SDUI action handlers using the established builder/router pattern.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Username/password authentication with bcrypt-hashed passwords
- Session token stored as HTTP-only cookie -- survives page reloads
- WebSocket connection reads session cookie on upgrade to identify the user
- Login screen is the first render -- backend sends login form if no valid session
- Successful login redirects to main app view via navigate action
- Admin-only CRUD screens for user accounts using the SDUI pattern (backend renders forms/tables via protocol)
- User table with columns: name, email, role, last login
- User form with fields: name, email, password (create/change), role select
- Default admin account seeded on first startup (configurable credentials via env vars)
- Two roles: `admin` and `user` -- simple enum, no complex permission matrix
- Record-level audit logging: who changed what record and when
- Stored in an `audit_log` table: `audit_log_id`, `audit_log_user`, `audit_log_table`, `audit_log_record_id`, `audit_log_action` (create/update/delete), `audit_log_timestamp`
- Field-level changes stored as JSON diff in `audit_log_changes` column
- Queryable via admin screen -- filter by user, table, date range
- Automatically recorded by a middleware/helper -- not manually called per handler

### Claude's Discretion
- Exact bcrypt cost factor
- Session token format and expiry duration
- Cookie name and attributes (SameSite, Secure flags)
- Audit log query screen layout and pagination
- How to handle concurrent sessions (allow multiple or single-session)
- Password strength requirements for the demo
- Whether to show audit trail inline on records or as a separate admin screen

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CRM-12 | Admin can manage users and assign roles | User entity + admin CRUD handlers + SDUI screens (Form/DataTable builders) + `#[requires(role = "admin")]` |
| CRM-13 | User can log in and access features based on role | Login action handler + bcrypt verification + session cookie + WsSession auth population + check_auth pipeline |
| CRM-14 | System records audit trail (who changed what when) | audit_log table + automatic middleware/helper that wraps entity mutations |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bcrypt | 0.19.0 | Password hashing | De facto Rust bcrypt crate, simple API (hash/verify), well-maintained |
| uuid | 1.x (already in workspace) | Session token generation | Already used for session IDs, UUID v4 tokens are sufficient |
| chrono | 0.4.44 | Timestamp handling for session expiry and audit log | Standard Rust datetime library, serde support |
| sea-orm | 1.1 (already in workspace) | User/audit_log entities and queries | Already the project ORM |
| sea-orm-migration | 1.1 (already in workspace) | Database migrations | Already established pattern |
| axum-extra | 0.12.5 | Cookie extraction in HTTP handlers | Official axum companion crate, provides `CookieJar` extractor |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tower-cookies | 0.11.0 | Alternative cookie middleware | Only if axum-extra CookieJar is insufficient (unlikely) |
| async-trait | 0.1.89 | Already used by sea-orm-migration | No new dep needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| bcrypt | argon2 | Argon2 is stronger but bcrypt is explicitly chosen by user decision |
| axum-extra cookies | tower-cookies | tower-cookies adds middleware layer; axum-extra is simpler extractor |
| chrono | time | chrono is more widely used, better serde integration |

**Installation (workspace Cargo.toml additions):**
```toml
bcrypt = "0.19"
chrono = { version = "0.4", features = ["serde"] }
axum-extra = { version = "0.12", features = ["cookie"] }
```

## Architecture Patterns

### Recommended Project Structure
```
backend/crates/crm-demo/src/
├── main.rs                    # Axum server wiring, router setup
├── handlers/
│   ├── mod.rs
│   ├── auth.rs                # login, logout actions
│   ├── user.rs                # user CRUD actions (admin)
│   └── audit.rs               # audit log query action (admin)
├── entities/
│   ├── mod.rs
│   ├── user.rs                # SeaORM user entity
│   └── audit_log.rs           # SeaORM audit_log entity
├── middleware/
│   ├── mod.rs
│   └── audit.rs               # Audit trail helper
└── seed.rs                    # Default admin seeding
```

### Pattern 1: Login Flow (HTTP POST + Cookie + WebSocket)
**What:** Login is a standard HTTP POST (not WebSocket) that sets the session cookie, then the client reconnects WebSocket with the cookie.
**When to use:** Always for the login flow -- WebSocket can't set cookies.

**Flow:**
1. Frontend loads, WebSocket connects, no cookie -> backend sends login form render
2. Login form submits via HTTP POST to `/api/login` (NOT via WebSocket action)
3. Backend validates credentials, creates session row, sets HTTP-only cookie
4. Frontend receives redirect, reconnects WebSocket -- this time cookie is present
5. WebSocket upgrade handler reads cookie, looks up session, populates WsSession auth fields
6. Backend sends main app render (authenticated view)

**Alternative considered:** Login via WebSocket action message. Problem: WebSocket messages cannot set HTTP-only cookies. The cookie must be set via an HTTP response header. So the login POST must be a regular HTTP endpoint.

### Pattern 2: Session Cookie on WebSocket Upgrade
**What:** Extract session cookie during the HTTP upgrade request to populate WsSession
**When to use:** Every WebSocket connection

```rust
// In ws_handler, extract cookies before upgrade
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    let session_token = jar.get("session").map(|c| c.value().to_owned());
    ws.on_upgrade(move |socket| handle_session(socket, state, session_token))
}
```

Then in `handle_session`, look up the session token in the database to populate WsSession fields.

### Pattern 3: SDUI Admin Screens (User Management)
**What:** Backend renders user list/form via protocol components, exactly like the demo
**When to use:** All user management screens

```rust
// User list handler using existing builders
async fn handle_user_list(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let users = user::Entity::find().all(&*db.0).await
        .map_err(|e| ActionError::Internal(e.to_string()))?;

    let columns = vec![
        TableColumn { key: "name".into(), label: "Name".into(), sortable: Some(true) },
        TableColumn { key: "email".into(), label: "Email".into(), sortable: Some(true) },
        TableColumn { key: "role".into(), label: "Role".into(), sortable: Some(true) },
        TableColumn { key: "lastLogin".into(), label: "Last Login".into(), sortable: Some(true) },
    ];
    let table = DataTable::new(columns).id("user-table").bind("/users").build();
    // ... build container with heading, table, "New User" button
    // Return RenderMessage with user data
}
```

### Pattern 4: Automatic Audit Trail
**What:** A helper function that wraps entity mutations and automatically logs changes
**When to use:** Every create/update/delete operation on auditable tables

```rust
/// Record an audit log entry. Called by handler helpers, not manually per handler.
pub async fn audit_log(
    db: &DatabaseConnection,
    user_id: i32,
    table: &str,
    record_id: i32,
    action: &str,      // "create", "update", "delete"
    changes: serde_json::Value, // JSON diff of changed fields
) -> Result<(), sea_orm::DbErr> {
    let entry = audit_log::ActiveModel {
        audit_log_user: Set(user_id),
        audit_log_table: Set(table.to_owned()),
        audit_log_record_id: Set(record_id),
        audit_log_action: Set(action.to_owned()),
        audit_log_changes: Set(changes.to_string()),
        ..Default::default()
    };
    entry.insert(db).await?;
    Ok(())
}
```

For computing JSON diffs on update: compare the old model (fetched before update) with the new values. Store as `{"field": {"old": X, "new": Y}}`.

### Anti-Patterns to Avoid
- **Login via WebSocket action:** Cannot set HTTP-only cookies. Must use HTTP POST endpoint.
- **Manual audit logging in every handler:** Use a shared helper function. The CONTEXT.md explicitly says "not manually called per handler" -- provide a wrapper that handlers call for mutations.
- **Storing passwords in plain text or with weak hashing:** Always bcrypt with cost >= 10.
- **Session tokens as sequential integers:** Use UUID v4 for unpredictable tokens.
- **Blocking bcrypt in async context:** bcrypt is CPU-intensive. Use `tokio::task::spawn_blocking` for hash/verify operations.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Password hashing | Custom hash function | bcrypt crate | Timing attacks, salt handling, cost factor tuning |
| Session tokens | Random string generation | uuid::Uuid::new_v4() | Cryptographic randomness guaranteed |
| Cookie handling | Manual Set-Cookie headers | axum-extra CookieJar | Proper escaping, SameSite, Secure, HttpOnly attributes |
| JSON diff computation | Custom diff algorithm | Simple old/new comparison | Only need field-level diff, not deep nested diff |
| Timestamp formatting | Manual string formatting | chrono DateTime | Timezone handling, ISO 8601 compliance |

**Key insight:** The existing marionette toolkit already handles auth checking, action routing with auth requirements, component building, and database patterns. This phase is primarily about adding new entities and handlers that use existing infrastructure.

## Common Pitfalls

### Pitfall 1: Blocking Async Runtime with bcrypt
**What goes wrong:** bcrypt hash/verify is CPU-intensive (intentionally slow). Running it on the tokio async runtime blocks other tasks.
**Why it happens:** Calling `bcrypt::hash()` directly in an async function.
**How to avoid:** Always wrap in `tokio::task::spawn_blocking`:
```rust
let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password, cost))
    .await
    .map_err(|e| ActionError::Internal(e.to_string()))?
    .map_err(|e| ActionError::Internal(e.to_string()))?;
```
**Warning signs:** Login becomes slow under concurrent connections.

### Pitfall 2: Cookie Not Sent on WebSocket Upgrade
**What goes wrong:** Session cookie exists but WebSocket doesn't receive it.
**Why it happens:** Cookie domain/path mismatch, or SameSite=Strict blocking cross-origin.
**How to avoid:** Set cookie with `Path=/`, `SameSite=Lax` (not Strict), same domain as WebSocket endpoint.
**Warning signs:** WebSocket always connects as anonymous despite successful login.

### Pitfall 3: Session Expiry Race Condition
**What goes wrong:** Session expires mid-WebSocket-connection, actions fail unexpectedly.
**Why it happens:** Session checked only at WebSocket upgrade, not per-action.
**How to avoid:** For the demo, use long-lived sessions (24h+). Optionally re-check session validity periodically, but not needed for MVP.
**Warning signs:** Authenticated users suddenly get "Unauthorized" errors after time passes.

### Pitfall 4: Forgetting to Seed Default Admin
**What goes wrong:** Fresh database has no users, nobody can log in.
**Why it happens:** Seed logic not wired into startup, or migration doesn't include seed.
**How to avoid:** Run seed check in `main()` after `init_db()`, before server starts. Check if user table is empty, insert default admin if so.
**Warning signs:** First-time users see login screen but have no credentials.

### Pitfall 5: Audit Log on Failed Operations
**What goes wrong:** Audit log records an action that actually failed (e.g., validation error).
**Why it happens:** Audit logged before the actual DB operation completes.
**How to avoid:** Log audit AFTER the mutation succeeds, not before.
**Warning signs:** Audit trail shows changes that don't match actual database state.

### Pitfall 6: WsSession Immutability After Login
**What goes wrong:** After login via HTTP POST and cookie set, the existing WebSocket session is still anonymous.
**Why it happens:** The WsSession was created at connection time and is not updated when the user logs in via HTTP.
**How to avoid:** After HTTP login, the frontend must reconnect the WebSocket. The new connection will read the cookie and create an authenticated WsSession.
**Warning signs:** User logs in but WebSocket actions still fail auth checks.

## Code Examples

### Database Schema (SQLite migrations)

```sql
-- user table
CREATE TABLE user (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL UNIQUE,
    user_password TEXT NOT NULL,  -- bcrypt hash
    user_role TEXT NOT NULL DEFAULT 'user' CHECK (user_role IN ('admin', 'user')),
    user_last_login TEXT,  -- ISO 8601 timestamp, nullable
    user_created TEXT NOT NULL DEFAULT (datetime('now'))
);

-- audit_log table
CREATE TABLE audit_log (
    audit_log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    audit_log_user INTEGER NOT NULL,
    audit_log_table TEXT NOT NULL,
    audit_log_record_id INTEGER NOT NULL,
    audit_log_action TEXT NOT NULL CHECK (audit_log_action IN ('create', 'update', 'delete')),
    audit_log_changes TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(audit_log_changes)),
    audit_log_timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (audit_log_user) REFERENCES user(user_id)
);
```

### SeaORM Entity Pattern (user)

```rust
pub mod user {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub user_id: i32,
        pub user_name: String,
        #[sea_orm(unique)]
        pub user_email: String,
        pub user_password: String,
        pub user_role: String,
        pub user_last_login: Option<String>,
        pub user_created: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
```

### HTTP Login Endpoint

```rust
use axum::{extract::State, Json, response::IntoResponse};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn handle_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), StatusCode> {
    // 1. Look up user by email
    let user = user::Entity::find()
        .filter(user::Column::UserEmail.eq(&req.username))
        .one(&*state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Verify password (spawn_blocking for CPU-bound bcrypt)
    let hash = user.user_password.clone();
    let password = req.password.clone();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 3. Create session token
    let token = uuid::Uuid::new_v4().to_string();
    // ... insert session row in DB ...

    // 4. Set HTTP-only cookie
    let cookie = Cookie::build(("session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(24))
        .build();

    Ok((jar.add(cookie), Json(serde_json::json!({"ok": true}))))
}
```

### Audit Helper

```rust
use sea_orm::ActiveValue::Set;

pub async fn record_audit(
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
    table: &str,
    record_id: i32,
    action: &str,
    changes: serde_json::Value,
) -> Result<(), ActionError> {
    let entry = audit_log::ActiveModel {
        audit_log_user: Set(user_id),
        audit_log_table: Set(table.to_owned()),
        audit_log_record_id: Set(record_id),
        audit_log_action: Set(action.to_owned()),
        audit_log_changes: Set(changes.to_string()),
        ..Default::default()
    };
    audit_log::Entity::insert(entry)
        .exec(db).await
        .map_err(|e| ActionError::Internal(e.to_string()))?;
    Ok(())
}
```

## Discretion Recommendations

| Decision Area | Recommendation | Rationale |
|---------------|---------------|-----------|
| bcrypt cost factor | 10 | Fast enough for demo, still secure. Default is 12 which is fine too |
| Session token format | UUID v4 | Already used for session IDs, unpredictable, no new dependency |
| Session expiry | 24 hours | Reasonable for demo, avoids annoying re-logins during testing |
| Cookie name | `marionette_session` | Namespaced, descriptive |
| Cookie attributes | HttpOnly, SameSite=Lax, Path=/ | Standard secure defaults. Omit Secure flag for localhost dev |
| Concurrent sessions | Allow multiple | Simpler to implement, better UX for demo |
| Password requirements | Minimum 8 characters | Demo-appropriate, not worth complex validation |
| Audit trail display | Separate admin screen | Cleaner UI, easier to query/filter, inline can wait for later |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual cookie parsing | axum-extra CookieJar extractor | axum 0.7+ | Type-safe, automatic parsing |
| bcrypt crate 0.15 | bcrypt 0.19 | 2024 | API stable, no breaking changes |
| Raw SQL sessions | SeaORM entity-based sessions | Project convention | Consistent with existing session entity pattern |

## Open Questions

1. **Login form rendering mechanism**
   - What we know: Backend sends login form when no valid session. Frontend renders it via protocol.
   - What's unclear: Does the login form submit via HTTP POST (must, for cookie) or does the form action trigger a special "login" handler?
   - Recommendation: Login form rendered via SDUI but form submission is intercepted by frontend to POST to `/api/login`. After success, frontend reconnects WebSocket. This requires a small frontend-side convention for login forms.

2. **Frontend WebSocket reconnection after login**
   - What we know: After HTTP POST login sets cookie, WebSocket must reconnect to pick up the session.
   - What's unclear: Does frontend auto-reconnect, or does the login response trigger reconnection?
   - Recommendation: Login HTTP response returns JSON with `{"ok": true}`. Frontend JS detects this and triggers WebSocket close + reconnect. The existing reconnection logic in FRONT-05 should handle this.

3. **Session table location**
   - What we know: Session entity already exists in `marionette/src/db.rs`. It has `session_user` as `Option<i32>`.
   - What's unclear: Should CRM-specific user/audit entities go in `crm-demo` or `marionette`?
   - Recommendation: User and audit_log entities belong in `crm-demo` (app-specific). Session entity stays in `marionette` (toolkit). Migrations should be split: session migration stays in marionette, user/audit_log migrations go in crm-demo.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | workspace Cargo.toml |
| Quick run command | `cargo test -p crm-demo` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRM-12 | Admin can create/edit/delete users via SDUI | integration | `cargo test -p crm-demo --test user_management` | No -- Wave 0 |
| CRM-12 | Non-admin cannot access user management | unit | `cargo test -p crm-demo user::test_admin_only` | No -- Wave 0 |
| CRM-13 | Login with valid credentials succeeds | integration | `cargo test -p crm-demo --test auth_flow` | No -- Wave 0 |
| CRM-13 | Login with invalid credentials fails | integration | `cargo test -p crm-demo --test auth_flow` | No -- Wave 0 |
| CRM-13 | WebSocket reads session cookie on upgrade | integration | `cargo test -p crm-demo --test ws_session` | No -- Wave 0 |
| CRM-13 | Role-based access check works | unit | `cargo test -p marionette auth::tests` | Yes (existing) |
| CRM-14 | Audit log records create/update/delete | unit | `cargo test -p crm-demo audit::tests` | No -- Wave 0 |
| CRM-14 | Audit log query returns filtered results | integration | `cargo test -p crm-demo --test audit_query` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p crm-demo`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crm-demo/tests/auth_flow.rs` -- covers CRM-13 login/logout
- [ ] `crm-demo/tests/user_management.rs` -- covers CRM-12 admin CRUD
- [ ] `crm-demo/tests/ws_session.rs` -- covers CRM-13 cookie-based session
- [ ] `crm-demo/src/handlers/` module structure -- handler organization
- [ ] `crm-demo/src/entities/` module structure -- entity organization
- [ ] Migration files for `user` and `audit_log` tables

## Sources

### Primary (HIGH confidence)
- Existing codebase: `auth.rs`, `session.rs`, `ws.rs`, `db.rs`, `router.rs`, `extractors.rs`, `standard.rs` -- all read and analyzed
- `06-CONTEXT.md` -- locked decisions and canonical references
- `TOOLING.md` -- SQL conventions (singular tables, prefixed fields, JSON validation)
- cargo registry: bcrypt 0.19.0, chrono 0.4.44, axum-extra 0.12.5

### Secondary (MEDIUM confidence)
- axum-extra cookie extraction pattern -- based on axum 0.8 + axum-extra 0.12 compatibility (verified versions match)
- bcrypt spawn_blocking pattern -- standard Rust async best practice for CPU-bound work

### Tertiary (LOW confidence)
- None -- all findings verified against codebase or crate registry

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates verified against registry, existing patterns established
- Architecture: HIGH -- builds directly on existing marionette toolkit patterns
- Pitfalls: HIGH -- well-known Rust async + auth patterns

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable domain, no fast-moving dependencies)
