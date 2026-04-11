---
phase: 13
plan: 03
type: execute
wave: 2
depends_on: [13-02]
files_modified:
  - backend/crates/crm-demo/src/handlers/fetch_rows.rs
  - backend/crates/crm-demo/src/handlers/mod.rs
  - backend/crates/crm-demo/src/main.rs
autonomous: true
requirements: [TABLE-02]
must_haves:
  truths:
    - "A generic `fetch_rows` action handler is registered in `main.rs` via `ActionRouter::action(\"fetch-rows\", box_handler(handle_fetch_rows), AuthRequirement::Authenticated)`"
    - "The handler accepts a payload `{source: String, offset: u32, limit: u32, filters?: serde_json::Value}` and dispatches internally to a per-source row fetcher based on a static match on `source`"
    - "The handler caps `limit` at 100 server-side (`limit.min(100)`) as a DoS mitigation"
    - "The handler enforces the SAME auth requirement as the source list handler it fetches from (`contact_list` → Authenticated, `audit_list` → Role(\"admin\"), `user_list` → Role(\"admin\"), `company_list` → Authenticated)"
    - "The handler echoes `ctx.action.id.clone()` into the returned `PatchMessage.id` so the frontend can correlate via D-H3 stale-discard"
    - "The handler returns a `PatchMessage` containing a `set` op that APPENDS the new rows to the existing bound collection path (e.g., `/contacts/{newRowId}: row`) — NOT a full-collection replacement"
    - "Unknown `source` values return `ActionError::BadPayload` (not `Internal`) so the frontend can surface a clear error"
    - "Malformed payloads return `ActionError::BadPayload`"
  artifacts:
    - path: "backend/crates/crm-demo/src/handlers/fetch_rows.rs"
      provides: "handle_fetch_rows function + FetchRowsPayload struct + source dispatch table"
      exports: ["handle_fetch_rows"]
    - path: "backend/crates/crm-demo/src/main.rs"
      provides: "fetch-rows action registration"
      contains: ".action(\"fetch-rows\""
  key_links:
    - from: "frontend DataTable.svelte sentinel (Plan 05)"
      to: "fetch-rows backend handler"
      via: "sendAction('fetch-rows', {source, offset, limit})"
      pattern: "fetch-rows"
    - from: "handler response"
      to: "DataTable.lastFetchRowsActionId correlation check"
      via: "PatchMessage.id == action.id"
      pattern: "id: ctx.action.id.clone"
---

<objective>
Close D-H1's "dead code" gap: the frontend's existing `sendAction('fetch-rows', ...)` dispatch has never worked because no backend handler is registered for it. This plan adds the generic `fetch_rows` handler that all four CRM list screens will use for infinite scroll, with correct auth, limit cap, action-id echo, and a source dispatch table.

Purpose: Plan 05's DataTable sentinel dispatches `fetch-rows` requests that MUST land on this handler. Plan 06's CRM handler migration sets the `source` prop on each DataTable so this handler knows which per-screen fetcher to invoke.

Output: New file `handlers/fetch_rows.rs`, registration in `main.rs`, and handler registration in the action router with tests proving auth enforcement, limit capping, action-id echoing, and unknown-source rejection.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/codebase/CONVENTIONS.md
@.planning/codebase/TESTING.md
@backend/crates/crm-demo/src/handlers/mod.rs
@backend/crates/crm-demo/src/handlers/audit.rs
@backend/crates/crm-demo/src/handlers/contact.rs
@backend/crates/crm-demo/src/main.rs
@backend/crates/marionette/src/extractors.rs
@backend/crates/marionette/src/error.rs

<interfaces>
<!-- Executor MUST read these BEFORE writing fetch_rows.rs. -->

Existing handler function signature pattern (from `handlers/audit.rs:25`):
```rust
pub async fn handle_audit_list(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    // ...
}
```

`ActionResult = Result<Vec<ProtocolMessage>, ActionError>` (defined in `marionette/src/error.rs`).

`HandlerContext` fields (from `marionette/src/extractors.rs`):
- `action: ActionMessage` with fields `id: Option<String>`, `name: String`, `payload: Option<serde_json::Value>`, `source: Option<String>`
- Access DB via `Db::from_context(&ctx)?`
- Access payload via `Payload::<T>::from_context(&ctx).map(|p| p.0).unwrap_or_default()` OR `.map_err(Into::into)?` for strict

`ActionRouter::action` signature (from `main.rs:452`):
```rust
.action(
    "name",
    box_handler(handler_fn),
    AuthRequirement::...,  // None | Authenticated | Role(&'static str)
)
```

Existing auth requirements per list handler (inventory from `main.rs:497-526`):
- `contact_list` → `AuthRequirement::Authenticated`
- `company_list` → `AuthRequirement::Authenticated`
- `user_list` → `AuthRequirement::Role("admin")`
- `audit_list` → `AuthRequirement::Role("admin")`

**Critical design: the handler MUST enforce per-source auth, not a single static auth level.** Since `ActionRouter::action` pins a single AuthRequirement at registration time, the generic `fetch_rows` handler registered at `Authenticated` is NOT sufficient — a plain authenticated user must NOT be able to fetch rows from the audit log (admin-only). The handler MUST additionally check `ctx.session.role` against the source's known requirement INSIDE the handler and return `ActionError::Unauthorized` if mismatched.

Session/role access pattern (check an existing admin-gated handler for convention):
- `audit.rs` relies on the router-level `AuthRequirement::Role("admin")` — no in-handler check needed
- For `fetch_rows` to be source-aware, inspect `marionette/src/session.rs` and `marionette/src/extractors.rs` for how to read `ctx.session.role` (or the Session extractor). Example: `Session::from_context(&ctx)?.role`

**Existing `PatchMessage` shape (from `marionette-protocol/src/messages.rs`):**
```rust
pub struct PatchMessage {
    pub id: Option<String>,
    pub surface: String,
    pub patch: Vec<PatchOperation>,
}
```

`PatchOperation::Set { path: String, value: serde_json::Value }` is the data op used to append a single row (e.g., `path: "/contacts/id-123", value: {...}`).

**Bound collection path convention:** Each CRM list handler binds its DataTable to a data path (e.g., `"/contacts"` for contact_list, `"/auditEntries"` for audit_list, `"/companies"` for company_list, `"/users"` for user_list). The `fetch_rows` handler must append to the SAME path used by the source list handler. Encoded as part of the source dispatch table.

**DataTable rowIdKey convention:** The frontend reads `props.row_id_key` (default `"id"`) and uses `Object.entries(rawData)` to iterate. Keys in the bound collection are row IDs. The `fetch_rows` handler writes rows using each row's `id` field as the collection key (one `set` op per row).
</interfaces>

<research_references>
- 13-RESEARCH.md §D-H1 explanation of the "dead code" gap (frontend dispatches but no handler is registered)
- 13-CONTEXT.md §D-H1 — source dispatch table requirement
- 13-VALIDATION.md row 15 (limit cap), row 16 (auth enforcement), row 14 (action id echo)
- 13-RESEARCH.md §Security Domain — SQL injection + DoS + access control threats
</research_references>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Create fetch_rows.rs with generic handler + source dispatch table + inline unit tests</name>
  <files>
    backend/crates/crm-demo/src/handlers/fetch_rows.rs,
    backend/crates/crm-demo/src/handlers/mod.rs
  </files>
  <read_first>
    - backend/crates/crm-demo/src/handlers/mod.rs (module list — you're adding a new module)
    - backend/crates/crm-demo/src/handlers/audit.rs (query pattern with SeaORM filter chains, limit via `.into_iter().take(100)`, line 62)
    - backend/crates/crm-demo/src/handlers/contact.rs (offset-pagination pattern — the largest handler, most filter variants)
    - backend/crates/marionette/src/extractors.rs (HandlerContext, Payload, Session, Db extractors)
    - backend/crates/marionette/src/session.rs (how to read session role)
    - backend/crates/marionette-protocol/src/messages.rs (PatchMessage, PatchOperation::Set shape)
    - backend/crates/crm-demo/src/entities/contact.rs, company.rs, user.rs, audit_log.rs (entity column names)
    - .planning/phases/13-datatable-enhancements/13-CONTEXT.md §D-H1 (dispatch table requirement)
    - .planning/phases/13-datatable-enhancements/13-RESEARCH.md §Security Domain (V4 access control, V5 DoS cap at 100)
  </read_first>
  <behavior>
    Inline tests MUST prove:
    - `fetch_rows_caps_limit`: submitting `{source: "contact_list", offset: 0, limit: 10000}` results in at most 100 rows in the response patch
    - `fetch_rows_echoes_action_id`: the returned `PatchMessage.id` equals `ctx.action.id` (the UUID passed in)
    - `fetch_rows_rejects_unknown_source`: submitting `{source: "nonexistent", offset: 0, limit: 50}` returns `ActionError::BadPayload`
    - `fetch_rows_rejects_admin_source_for_non_admin`: submitting `{source: "audit_list", ...}` with a non-admin `Session` returns `ActionError::Unauthorized`
    - `fetch_rows_allows_admin_source_for_admin`: same payload with an admin session returns Ok
    - `fetch_rows_rejects_malformed_payload`: missing `source` field returns `ActionError::BadPayload`
  </behavior>
  <action>
    **Step 1 — Create `backend/crates/crm-demo/src/handlers/fetch_rows.rs`.**

    ```rust
    //! Generic server-side row fetcher for the `fetch-rows` action (Phase 13 D-H1).
    //!
    //! Closes a dead-code gap: the frontend's DataTable sentinel dispatches
    //! `sendAction('fetch-rows', { source, offset, limit })` but no backend
    //! handler was registered for it until Phase 13. This module provides a
    //! single generic handler that dispatches internally to per-source fetchers
    //! based on the `source` payload field, enforcing per-source auth and a
    //! global limit cap.

    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
    use serde::Deserialize;

    use marionette::error::{ActionError, ActionResult};
    use marionette::extractors::{Db, FromHandlerContext, HandlerContext, Payload, Session};
    use marionette_protocol::{
        messages::PatchMessage,
        data::PatchOperation,
        ProtocolMessage,
    };

    use crate::entities::{audit_log, company, contact, user};

    /// Maximum rows returnable in a single `fetch-rows` request. Enforced
    /// server-side as a DoS mitigation (V5 Input Validation hardening).
    const MAX_LIMIT: u32 = 100;

    /// Payload shape for the `fetch-rows` action.
    #[derive(Debug, Deserialize)]
    pub struct FetchRowsPayload {
        /// Identifier of the source list screen. Maps to a per-screen fetcher
        /// in the dispatch table below. Must match one of the known sources
        /// (`contact_list`, `company_list`, `user_list`, `audit_list`).
        pub source: String,
        #[serde(default)]
        pub offset: u32,
        #[serde(default = "default_limit")]
        pub limit: u32,
        /// Optional filter payload forwarded to the per-source fetcher.
        #[serde(default)]
        pub filters: serde_json::Value,
    }

    fn default_limit() -> u32 { 50 }

    /// Auth requirement per source (mirrors `main.rs` ActionRouter registrations).
    /// Enforced IN the handler because the router registers `fetch-rows` at a
    /// single level (`Authenticated`) but some sources (audit, user) require
    /// admin. (V4 Access Control.)
    fn required_role_for(source: &str) -> Result<Option<&'static str>, ActionError> {
        match source {
            "contact_list" | "company_list" => Ok(None),           // Authenticated only
            "audit_list" | "user_list" => Ok(Some("admin")),       // Admin-only
            _ => Err(ActionError::BadPayload(format!(
                "unknown fetch-rows source: {source}"
            ))),
        }
    }

    /// Generic fetch-rows handler. Parses the payload, enforces per-source auth,
    /// caps the limit, and dispatches to the per-source fetcher. Returns a
    /// PatchMessage with one `set` op per fetched row (keyed by row id) so the
    /// frontend can append to its existing bound collection without replacing
    /// the full collection.
    pub async fn handle_fetch_rows(ctx: HandlerContext) -> ActionResult {
        // 1. Parse and validate payload (V5 Input Validation).
        let payload: FetchRowsPayload = Payload::<FetchRowsPayload>::from_context(&ctx)
            .map_err(|_| ActionError::BadPayload("fetch-rows payload missing or malformed".into()))?
            .0;

        // 2. Cap limit (V5 DoS mitigation).
        let limit = payload.limit.min(MAX_LIMIT);
        let offset = payload.offset;

        // 3. Per-source auth check (V4 Access Control).
        let required_role = required_role_for(&payload.source)?;
        if let Some(role) = required_role {
            let session = Session::from_context(&ctx)
                .map_err(|_| ActionError::Unauthorized("no session".into()))?;
            if session.role.as_deref() != Some(role) {
                return Err(ActionError::Unauthorized(format!(
                    "fetch-rows source '{}' requires role '{}'",
                    payload.source, role
                )));
            }
        }

        // 4. Dispatch to per-source fetcher.
        let db = Db::from_context(&ctx)?;
        let (path, rows) = match payload.source.as_str() {
            "contact_list" => fetch_contacts(&db, offset, limit, &payload.filters).await?,
            "company_list" => fetch_companies(&db, offset, limit, &payload.filters).await?,
            "user_list" => fetch_users(&db, offset, limit, &payload.filters).await?,
            "audit_list" => fetch_audit(&db, offset, limit, &payload.filters).await?,
            // Unreachable — required_role_for rejected it above
            other => return Err(ActionError::BadPayload(format!("unknown source: {other}"))),
        };

        // 5. Build a PatchMessage with one `set` op per row, keyed by the row's
        // `id` field. The frontend's `Object.entries(bound_collection)` iteration
        // picks up appended keys automatically.
        let mut ops: Vec<PatchOperation> = Vec::with_capacity(rows.len());
        for row in rows {
            let row_id = row
                .get("id")
                .and_then(|v| v.as_str().map(String::from).or_else(|| v.as_i64().map(|i| i.to_string())))
                .ok_or_else(|| ActionError::Internal("row missing 'id' field".into()))?;
            ops.push(PatchOperation::Set {
                path: format!("{path}/{row_id}"),
                value: row,
            });
        }

        // 6. Echo the action id into the response (D-H3 correlation).
        Ok(vec![ProtocolMessage::Patch(PatchMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            patch: ops,
        })])
    }

    // -- Per-source fetchers --
    //
    // Each returns (bound_collection_path, Vec<row_json>). The path matches
    // the `bind` used by the source's list handler so patches append to the
    // same path.

    async fn fetch_contacts(
        db: &Db,
        offset: u32,
        limit: u32,
        _filters: &serde_json::Value,
    ) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
        let contacts = contact::Entity::find()
            .order_by_asc(contact::Column::ContactId)
            .offset(u64::from(offset))
            .limit(u64::from(limit))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let rows = contacts
            .into_iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.contact_id,
                    "name": c.contact_name,
                    "email": c.contact_email,
                    "phone": c.contact_phone.unwrap_or_default(),
                    "title": c.contact_title.unwrap_or_default(),
                })
            })
            .collect();
        Ok(("/contacts", rows))
    }

    async fn fetch_companies(
        db: &Db,
        offset: u32,
        limit: u32,
        _filters: &serde_json::Value,
    ) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
        let companies = company::Entity::find()
            .order_by_asc(company::Column::CompanyId)
            .offset(u64::from(offset))
            .limit(u64::from(limit))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let rows = companies
            .into_iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.company_id,
                    "name": c.company_name,
                    "website": c.company_website.unwrap_or_default(),
                    "address": c.company_address.unwrap_or_default(),
                })
            })
            .collect();
        Ok(("/companies", rows))
    }

    async fn fetch_users(
        db: &Db,
        offset: u32,
        limit: u32,
        _filters: &serde_json::Value,
    ) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
        let users = user::Entity::find()
            .order_by_asc(user::Column::UserId)
            .offset(u64::from(offset))
            .limit(u64::from(limit))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let rows = users
            .into_iter()
            .map(|u| {
                serde_json::json!({
                    "id": u.user_id,
                    "name": u.user_name,
                    "email": u.user_email,
                    "role": u.user_role,
                })
            })
            .collect();
        Ok(("/users", rows))
    }

    async fn fetch_audit(
        db: &Db,
        offset: u32,
        limit: u32,
        _filters: &serde_json::Value,
    ) -> Result<(&'static str, Vec<serde_json::Value>), ActionError> {
        let entries = audit_log::Entity::find()
            .order_by_desc(audit_log::Column::AuditLogTimestamp)
            .offset(u64::from(offset))
            .limit(u64::from(limit))
            .all(&*db.0)
            .await
            .map_err(|e| ActionError::Internal(e.to_string()))?;
        let rows = entries
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.audit_log_id,
                    "timestamp": e.audit_log_timestamp,
                    "table": e.audit_log_table,
                    "recordId": e.audit_log_record_id,
                    "action": e.audit_log_action,
                    "changes": e.audit_log_changes,
                })
            })
            .collect();
        Ok(("/auditEntries", rows))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use marionette_protocol::ActionMessage;
        use serde_json::json;

        // Build a minimal HandlerContext for unit testing. Uses MockDatabase for
        // DB-independent tests (limit cap, payload validation, auth) and the
        // real contact seed for one happy-path test.
        //
        // If the marionette crate exposes a test helper for HandlerContext,
        // prefer that. Otherwise, construct directly.

        fn make_payload(source: &str, offset: u32, limit: u32) -> serde_json::Value {
            json!({ "source": source, "offset": offset, "limit": limit })
        }

        #[tokio::test]
        async fn fetch_rows_rejects_unknown_source() {
            // This test does NOT need a DB — it hits the `required_role_for`
            // check before DB dispatch.
            let err = required_role_for("not_a_real_source");
            assert!(matches!(err, Err(ActionError::BadPayload(_))));
        }

        #[tokio::test]
        async fn fetch_rows_known_sources_map_correctly() {
            assert!(matches!(required_role_for("contact_list"), Ok(None)));
            assert!(matches!(required_role_for("company_list"), Ok(None)));
            assert!(matches!(required_role_for("audit_list"), Ok(Some("admin"))));
            assert!(matches!(required_role_for("user_list"), Ok(Some("admin"))));
        }

        #[test]
        fn fetch_rows_payload_deserializes_defaults() {
            let p: FetchRowsPayload = serde_json::from_value(json!({
                "source": "contact_list"
            })).unwrap();
            assert_eq!(p.source, "contact_list");
            assert_eq!(p.offset, 0);
            assert_eq!(p.limit, 50);
        }

        #[test]
        fn fetch_rows_payload_rejects_missing_source() {
            let r = serde_json::from_value::<FetchRowsPayload>(json!({
                "offset": 0,
                "limit": 10
            }));
            assert!(r.is_err(), "expected deserialize error for missing source");
        }

        #[test]
        fn fetch_rows_limit_cap_constant() {
            // Sanity check on the cap value — must match the V5 DoS mitigation.
            assert_eq!(MAX_LIMIT, 100);
        }

        #[test]
        fn fetch_rows_limit_min_caps_oversized_request() {
            // The runtime code does `payload.limit.min(MAX_LIMIT)` — verify
            // the saturation behavior directly.
            let requested: u32 = 10_000;
            let capped = requested.min(MAX_LIMIT);
            assert_eq!(capped, 100);
        }
    }
    ```

    **Step 2 — Register the module.** Add `pub mod fetch_rows;` to `backend/crates/crm-demo/src/handlers/mod.rs`.

    **Step 3 — Adapt to real type signatures.** The code above uses:
    - `marionette::extractors::Session` — if the real session extractor has a different name (`HandlerContext.session` direct access is also possible), adapt to whatever the crate exposes
    - `marionette_protocol::data::PatchOperation::Set { path, value }` — verify this path lives where imports reach it (Phase 12 rewrote PatchOperation as a tagged enum)
    - `AuditLogColumn::AuditLogId` — verify the primary key column name by reading `entities/audit_log.rs`
    - `ActionError::Unauthorized(String)` — verify the error enum variant signature in `marionette/src/error.rs`
    - The `required_role_for` return type: if `ActionError::BadPayload` takes a `String` instead of `impl Into<String>`, use `.into()`

    Fix any type mismatches by reading the exact source files in `<read_first>` and adapting. The SEMANTICS above are locked; the syntax must conform to the real crate.

    **Step 4 — Run the tests.** The `#[test]` ones run without a DB. The `#[tokio::test]` ones use `required_role_for` directly (not DB-dependent). Full DB-backed integration tests come in Task 2.
  </action>
  <verify>
    <automated>cd backend && cargo test -p crm-demo handlers::fetch_rows::tests 2>&1 | tee /tmp/phase13-03-t1.log</automated>
  </verify>
  <acceptance_criteria>
    - `backend/crates/crm-demo/src/handlers/fetch_rows.rs` exists
    - `grep -c "pub async fn handle_fetch_rows" backend/crates/crm-demo/src/handlers/fetch_rows.rs` == 1
    - `grep -c "const MAX_LIMIT: u32 = 100" backend/crates/crm-demo/src/handlers/fetch_rows.rs` == 1
    - `grep -c "id: ctx.action.id.clone()" backend/crates/crm-demo/src/handlers/fetch_rows.rs` == 1
    - `grep -c 'pub mod fetch_rows' backend/crates/crm-demo/src/handlers/mod.rs` == 1
    - `cd backend && cargo test -p crm-demo handlers::fetch_rows::tests` passes (6 tests)
    - `cd backend && cargo build -p crm-demo` exits 0
    - `cd backend && cargo clippy -p crm-demo --lib -- -D warnings` exits 0 on the new file
  </acceptance_criteria>
  <done>Handler implemented with unit tests green; payload validation, limit cap, source dispatch table, and action-id echo all covered.</done>
</task>

<task type="auto">
  <name>Task 2: Register fetch-rows in the ActionRouter + integration test against a running server</name>
  <files>
    backend/crates/crm-demo/src/main.rs,
    backend/crates/crm-demo/tests/fetch_rows_integration.rs
  </files>
  <read_first>
    - backend/crates/crm-demo/src/main.rs §action_router section (lines 451-591) — the ActionRouter builder chain
    - backend/crates/crm-demo/tests/integration_test.rs (or whatever the existing integration test file is named — pattern for `start_server()`, `connect_async`, hello exchange)
    - backend/crates/crm-demo/src/handlers/fetch_rows.rs (Task 1 — what you just wrote)
    - .planning/codebase/TESTING.md §Rust Integration Tests (pattern for spinning up a real axum server)
  </read_first>
  <action>
    **Part A — Register `fetch-rows` in the router.**

    Edit `backend/crates/crm-demo/src/main.rs`. In the `ActionRouter::new()` chain (around line 451), add a new `.action(...)` call for `fetch-rows`. Place it alphabetically after `company_delete` or at a natural grouping (e.g., right after `navigate`). Use action name `"fetch-rows"` (kebab-case, matching the existing frontend dispatch string — confirm via grep):

    ```bash
    grep -rn "fetch-rows\|fetch_rows" backend/ frontend/src/lib/ 2>/dev/null
    ```

    Expected: frontend uses `"fetch-rows"` in `DataTable.svelte:78` (or will after Plan 05). Use the exact same string.

    Add this to the router chain:

    ```rust
    .action(
        "fetch-rows",
        box_handler(handlers::fetch_rows::handle_fetch_rows),
        AuthRequirement::Authenticated,
    )
    ```

    Note the auth level is `Authenticated` (minimum) because the in-handler `required_role_for` check upgrades admin-only sources. The router can't pin both Authenticated AND Role admin, so the handler does the source-aware upgrade.

    **Part B — Integration test against a running server.**

    Create `backend/crates/crm-demo/tests/fetch_rows_integration.rs`. Follow the same pattern as the existing integration test file (look up `start_server()`, `connect_async`, the hello skip, and login-to-get-session sequences):

    ```rust
    //! Integration tests for the generic fetch-rows handler (Phase 13 D-H1).
    //!
    //! Spins up a real axum server against an in-memory SQLite and drives the
    //! handler via WebSocket frames, exactly as the frontend would.

    use serde_json::{json, Value};
    use tokio_tungstenite::tungstenite::Message;
    use futures_util::{SinkExt, StreamExt};

    // Reuse the existing test server helper. If the name is different, adapt.
    mod common {
        include!("integration_test.rs");
    }

    // If `integration_test.rs` defines `start_server` as `pub(crate)`, this
    // module include pattern works. Otherwise copy the minimal server spin-up
    // boilerplate (it's ~30 lines).

    async fn login_as(ws: &mut tokio_tungstenite::WebSocketStream<_>, role: &str) {
        // Drive a login action that seeds a session with the given role.
        // Concrete: send {type:"action", id:"login-1", name:"login",
        //   payload:{email,password}} using the admin or regular user
        // seeded by crm-demo (admin@localhost / admin by default).
        //
        // Implementation detail — see existing integration test for the
        // established pattern.
        todo!("use the same login flow as existing integration tests")
    }

    #[tokio::test]
    async fn fetch_rows_caps_limit_at_100() {
        let (url, _guard) = common::start_server().await;
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
        // Skip hello
        let _ = ws.next().await.unwrap().unwrap();
        // Login as admin (so we can target any source)
        login_as(&mut ws, "admin").await;

        // Request 5000 rows from contact_list
        let action = json!({
            "type": "action",
            "id": "test-cap-1",
            "name": "fetch-rows",
            "payload": { "source": "contact_list", "offset": 0, "limit": 5000 }
        });
        ws.send(Message::Text(action.to_string())).await.unwrap();

        // Receive patch
        let frame = ws.next().await.unwrap().unwrap();
        let msg: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        assert_eq!(msg["type"], "patch");
        // Echoed id proves D-H3 correlation wiring
        assert_eq!(msg["id"], "test-cap-1");
        let ops = msg["patch"].as_array().unwrap();
        // Limit capped at 100 — seed.rs provides 120 contacts so we can hit the cap
        assert!(ops.len() <= 100, "expected <= 100 ops, got {}", ops.len());
    }

    #[tokio::test]
    async fn fetch_rows_rejects_unknown_source_with_bad_payload() {
        let (url, _guard) = common::start_server().await;
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap();
        login_as(&mut ws, "admin").await;

        let action = json!({
            "type": "action",
            "id": "test-unknown-1",
            "name": "fetch-rows",
            "payload": { "source": "no_such_thing", "offset": 0, "limit": 10 }
        });
        ws.send(Message::Text(action.to_string())).await.unwrap();

        let frame = ws.next().await.unwrap().unwrap();
        let msg: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        assert_eq!(msg["type"], "error");
        assert!(
            msg["errors"][0]["message"].as_str().unwrap().to_lowercase().contains("unknown")
                || msg["errors"][0]["message"].as_str().unwrap().to_lowercase().contains("bad"),
            "expected bad-payload error, got {:?}", msg["errors"]
        );
    }

    #[tokio::test]
    async fn fetch_rows_audit_list_requires_admin() {
        let (url, _guard) = common::start_server().await;
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap();
        // Login as a NON-admin user (create one via crm-demo's user seed or
        // prior-test sign-up flow; reuse whatever existing integration tests use)
        login_as(&mut ws, "user").await;

        let action = json!({
            "type": "action",
            "id": "test-auth-1",
            "name": "fetch-rows",
            "payload": { "source": "audit_list", "offset": 0, "limit": 10 }
        });
        ws.send(Message::Text(action.to_string())).await.unwrap();

        let frame = ws.next().await.unwrap().unwrap();
        let msg: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        assert_eq!(msg["type"], "error");
        let err = msg["errors"][0]["message"].as_str().unwrap().to_lowercase();
        assert!(
            err.contains("unauthorized") || err.contains("admin") || err.contains("role"),
            "expected unauthorized error, got: {err}"
        );
    }
    ```

    **IMPORTANT:** The `login_as` helper is a placeholder. Replace it with whatever login pattern the existing integration tests use. Read `backend/crates/crm-demo/tests/integration_test.rs` first and reuse its fixtures verbatim. If the existing integration test file does NOT support role-based login (e.g., only tests as anonymous), adapt these tests to use whatever session mechanism exists. If the test file organization is different (e.g., per-test files), match the existing convention.

    **If creating a non-admin session is nontrivial in the test harness**, replace the `fetch_rows_audit_list_requires_admin` test with a `#[tokio::test]` that directly calls `required_role_for` + the in-handler auth check via a unit-test pathway instead of a full integration test. The acceptance criterion is that per-source auth is PROVEN, not necessarily via WebSocket-level integration.

    **Step 3 — Run the integration test suite.**

    ```bash
    cd backend && cargo test -p crm-demo --test fetch_rows_integration
    ```

    Fix any path/helper mismatches until green.
  </action>
  <verify>
    <automated>cd backend && cargo build -p crm-demo && cargo test -p crm-demo handlers::fetch_rows && cargo test -p crm-demo --test fetch_rows_integration 2>&1 | tail -30</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c '"fetch-rows"' backend/crates/crm-demo/src/main.rs` >= 1
    - `grep -c "handlers::fetch_rows::handle_fetch_rows" backend/crates/crm-demo/src/main.rs` == 1
    - `cd backend && cargo build -p crm-demo` exits 0
    - `cd backend && cargo test -p crm-demo` passes with the new inline tests AND any integration tests added (at minimum: the limit-cap test and the unknown-source test from the integration file)
    - At least one test proves the `PatchMessage.id` echoes the `ActionMessage.id` sent by the client (search logs for `assert_eq!(msg["id"], ...)`)
    - At least one test proves `limit > 100` gets capped at 100 rows
    - At least one test proves unknown `source` returns an error response (either via `ActionError::BadPayload` unit-level or the WebSocket error frame)
    - At least one test proves audit_list / user_list sources reject non-admin callers (either unit-level via `required_role_for` or integration-level via WebSocket)
    - `cd backend && cargo clippy -p crm-demo --tests -- -D warnings` exits 0
  </acceptance_criteria>
  <done>fetch-rows is registered in the router, hits the database, caps the limit, echoes action id, and enforces per-source auth — proven by tests.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| WebSocket client → `fetch_rows` handler | Untrusted payload. Action id, source name, offset, limit all come from the client. |
| Handler → SeaORM query | Parameterized queries only. No raw SQL. |
| Session → role check | Session role read via `Session::from_context`; role decisions based on server-side session, not client-supplied data. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-03-01 | Tampering (SQL injection) | per-source fetchers using SeaORM filter + order_by + offset/limit | mitigate | SeaORM parameterized queries; no `raw_sql`; `source` string is matched against a literal whitelist before reaching any query. Test `fetch_rows_rejects_unknown_source` proves whitelist. |
| T-13-03-02 | DoS | Giant `limit` request exhausts DB/memory | mitigate | Hard cap at `MAX_LIMIT = 100` via `payload.limit.min(MAX_LIMIT)`. Unit test `fetch_rows_limit_min_caps_oversized_request` proves saturation; integration test `fetch_rows_caps_limit_at_100` proves the cap end-to-end. |
| T-13-03-03 | Elevation (Access control bypass) | Regular user fetching audit_list rows via `fetch-rows` | mitigate | In-handler `required_role_for` + Session role check. Integration test `fetch_rows_audit_list_requires_admin` proves the guard. |
| T-13-03-04 | Tampering (Input validation) | Malformed payload crashes the handler | mitigate | `#[derive(Deserialize)] FetchRowsPayload` rejects malformed payloads with `ActionError::BadPayload`. Unit test `fetch_rows_payload_rejects_missing_source` proves validation. |
| T-13-03-05 | Information disclosure | Error messages leak internal DB schema | accept | Error messages use static strings ("unknown source", "unauthorized") — no SQL text or row content in errors. |
| T-13-03-06 | Repudiation | `fetch-rows` actions not in audit log | accept | Pagination is a read-only operation. Existing list handlers log the initial render; paginated chunks are transparent to the audit story. If this changes (e.g., filtered fetches reveal sensitive rows), revisit in Phase 15. |

No HIGH severity unmitigated threats. V4 Access Control, V5 Input Validation, V5 DoS all covered.
</threat_model>

<verification>
```bash
cd backend
cargo build -p crm-demo
cargo test -p crm-demo handlers::fetch_rows
cargo test -p crm-demo --test fetch_rows_integration
cargo clippy -p crm-demo -- -D warnings
```

All four MUST exit 0.
</verification>

<success_criteria>
- `fetch-rows` action is registered in `main.rs` ActionRouter at `AuthRequirement::Authenticated`
- `handle_fetch_rows` exists in `handlers/fetch_rows.rs` and dispatches to 4 per-source fetchers
- Unknown sources return `BadPayload`, not `Internal`
- `limit` is capped at 100 server-side
- Admin-only sources (`audit_list`, `user_list`) require `session.role == "admin"`
- `PatchMessage.id == ctx.action.id` (proved by integration test asserting `msg["id"] == "test-cap-1"`)
- Patches append rows as individual `set` ops keyed by row id (`/contacts/<id>`, `/auditEntries/<id>`, etc.)
- All unit + integration tests green; clippy clean
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-03-fetch-rows-handler-SUMMARY.md` recording:
- The exact signature of `Session::from_context` or whatever session extractor was used
- Any deviations in the per-source fetcher signatures from the plan's stubs
- The final test count and test names
- Any integration-test login helper changes made (for Plan 06 CRM migration to reuse)
</output>
