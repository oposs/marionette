# Codebase Concerns

**Analysis Date:** 2026-04-08

## Tech Debt

**Duplicate authentication path (WS login vs HTTP login):**
- Issue: Login is implemented twice — once as a WebSocket action in `backend/crates/crm-demo/src/main.rs` (`handle_login_action`) and once as an HTTP POST endpoint in `backend/crates/crm-demo/src/handlers/auth.rs` (`handle_login`). The WS path embeds `_auth_user_id` / `_auth_role` in the render payload as a side-channel so `ws.rs` can mutate the `WsSession` in-place. This is a fragile convention; any new session field must be added in both places.
- Files: `backend/crates/crm-demo/src/main.rs:33-119`, `backend/crates/crm-demo/src/handlers/auth.rs`
- Impact: Behaviour diverges silently if one path is updated without the other; the embedded `_auth_user_id` key leaks implementation detail into every first render payload.
- Fix approach: Consolidate to a single login handler; use a dedicated auth-update channel or message type rather than smuggling data through `render.data`.

**Global static for Listmonk client:**
- Issue: `LISTMONK_CLIENT` is a `OnceLock<Arc<ListmonkClient>>` module-level static in `backend/crates/crm-demo/src/handlers/listmonk.rs`. This makes the dependency invisible to callers, untestable without environment setup, and means the handler module cannot be used in multiple apps with different configurations.
- Files: `backend/crates/crm-demo/src/handlers/listmonk.rs:12-22`
- Impact: Cannot unit-test Listmonk handlers without a real or mocked Listmonk server; testability is blocked.
- Fix approach: Thread the client through `AppState.listmonk` (already stored there but not consumed by handlers) and remove the global static. Handlers should receive the client via `HandlerContext` or `AppState`.

**Hardcoded database path:**
- Issue: `main.rs` connects to `"sqlite://crm.db?mode=rwc"` — a literal relative path with no configuration option.
- Files: `backend/crates/crm-demo/src/main.rs:251`
- Impact: Forces running from the `backend/` directory; cannot change the path without a code change; CI and production deployments are tightly coupled to the working directory.
- Fix approach: Read `DATABASE_URL` from environment with a documented default.

**`fetch-rows` ignores active filters:**
- Issue: The virtual-scroll incremental load action (`fetch-rows`) in `handle_fetch_rows` always fetches from the full unfiltered contact set using `order_by_asc(ContactName)`. The initial `contact_list` render applies filter conditions, but the subsequent paginated requests do not, so scrolling down after a search will load out-of-filter rows.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:408-510`
- Impact: DataTable shows wrong rows when a filter is active and the user scrolls past the first 100 results.
- Fix approach: `fetch-rows` payload should include the same filter parameters as `contact_list`; the handler should re-apply the same `Condition` built in `render_contact_list`.

**Contact email has no UNIQUE constraint:**
- Issue: The `contact` table DDL does not include a `UNIQUE` constraint on `contact_email`, unlike `user.user_email` which is `UNIQUE`. The save handler validates `contains('@')` but not uniqueness.
- Files: `backend/crates/crm-demo/src/migration/m20260323_000004_create_contact.rs`, `backend/crates/crm-demo/src/handlers/contact.rs:985-1008`
- Impact: Duplicate email addresses can be inserted, which will cause `find_subscriber_by_email` in the Listmonk sync path to silently update the wrong subscriber or create duplicates.
- Fix approach: Add a `UNIQUE` constraint on `contact_email` in a new migration; add a uniqueness check in the save handler with a user-visible error message.

**`now_sqlite()` duplicated three times:**
- Issue: A helper formatting the current UTC time as `"YYYY-MM-DD HH:MM:SS"` appears verbatim in `contact.rs` (line 17), `listmonk.rs` (sync success, lines 72-83), and `listmonk.rs` (sync error, lines 113-124).
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:17-28`, `backend/crates/crm-demo/src/handlers/listmonk.rs:72-83,113-124`
- Impact: If the format needs to change, three sites must be updated in sync.
- Fix approach: Extract to a shared `crate::util::now_sqlite()` function.

**`handle_contact_form` is 500+ lines:**
- Issue: The function builds the create form, edit form, tags section, notes section, interactions section, Listmonk sync section, and mailing history — all in one body with conditional branches. It is the largest handler at ~500 lines.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:512-983`
- Impact: Difficult to read, test, and extend; any new section appended to the edit view increases its length further.
- Fix approach: Extract `build_tags_section`, `build_notes_section`, `build_interactions_section`, `build_sync_section` as separate private async functions each returning `Vec<(String, Component)>`.

**DataTable column sort sends action but backend ignores it:**
- Issue: `DataTable.svelte` calls `sendAction('sort', { column, direction })` when a sortable column header is clicked, but no `sort` action is registered in the backend router and the contact list handler has a fixed `order_by_asc(ContactName)`. The sort state in the UI is purely cosmetic.
- Files: `frontend/src/lib/components/table/DataTable.svelte:100-118`, `backend/crates/crm-demo/src/main.rs:305-440`
- Impact: Clicking a column header shows a sort arrow UI but data order does not change. Users expecting functional sort receive misleading feedback.
- Fix approach: Either register a `sort` action handler that re-runs the query with the requested order, or remove the `sortable` props from columns until server-side sort is implemented.

**PatchMessage lacks surface field:**
- Issue: The `patch` protocol message has no `surface` field; the frontend handler in `init.ts` hardcodes `applyPatch('main', ...)`. This prevents sending patches to surfaces other than `main` (e.g., `sidebar`, `modal`).
- Files: `frontend/src/lib/init.ts:45-53`, `backend/crates/marionette-protocol/src/messages.rs`
- Impact: Any multi-surface patch scenario (sidebar updates, modal state changes) cannot be implemented without extending the protocol.
- Fix approach: Add an optional `surface` field to `PatchMessage`; default to `"main"` if absent for backward compatibility.

---

## Known Bugs

**Virtual scroll fetch-rows does not carry filters:**
- Symptoms: After filtering contacts (search/tag/date), scrolling past the first 100 visible rows fetches unfiltered contacts from the database and replaces placeholder rows with wrong data.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:408-510`, `frontend/src/lib/components/table/DataTable.svelte:73-98`
- Trigger: Apply any filter → scroll to row > 100.
- Workaround: None; the filter must be re-entered after scroll.

**Mailing history table appears/disappears based on data presence:**
- Symptoms: The `history-table` DataTable node is only emitted in the render payload when `has_history` is true. If a cache miss returns empty data initially, the table node is never sent, so `Refresh History` will populate `mailingHistory` data but no table node exists to render it.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:904-933`
- Trigger: Contact has been synced to Listmonk but cache is cold; first open shows "No mailing history" text; user clicks Refresh; history data arrives but table node is absent.
- Workaround: Reload the contact form (navigate away and back) to get the table node included.

---

## Security Considerations

**Session cookie missing `Secure` flag:**
- Risk: The session cookie is set with `http_only(true)` and `same_site(SameSite::Lax)` but no `.secure(true)` flag. Over a non-HTTPS connection (e.g., development proxy) the cookie is transmitted in the clear.
- Files: `backend/crates/crm-demo/src/handlers/auth.rs:87-92`
- Current mitigation: `SameSite::Lax` provides partial CSRF protection.
- Recommendations: Set `.secure(true)` when behind TLS; add a configuration flag (`COOKIE_SECURE=true`) so development without TLS still works.

**No rate limiting on login endpoints:**
- Risk: Both the HTTP `/api/login` endpoint and the WebSocket `login` action accept unlimited attempts. Password brute-force is unconstrained.
- Files: `backend/crates/crm-demo/src/handlers/auth.rs:31-95`, `backend/crates/crm-demo/src/main.rs:33-119`
- Current mitigation: None.
- Recommendations: Add a `tower_governor` or equivalent rate-limiting layer on `/api/login`; implement per-IP failure counting for the WebSocket login action.

**No CSRF protection on HTTP API endpoints:**
- Risk: The `/api/login` and `/api/logout` endpoints accept `POST` from any origin. No CORS policy is configured in the Axum router, meaning any origin can trigger state-changing requests.
- Files: `backend/crates/crm-demo/src/main.rs:453-465`
- Current mitigation: `SameSite::Lax` cookie reduces impact for non-navigation `POST` from cross-origin forms.
- Recommendations: Add a `tower_http::cors::CorsLayer` with an explicit `allow_origin` list; consider double-submit cookie or custom header CSRF token for the HTTP API.

**Expired sessions are never purged from the database:**
- Risk: The `session` table grows unboundedly. Every login creates a row; logout deletes it if called, but WS-only sessions (never call `/api/logout`) and expired cookies accumulate indefinitely.
- Files: `backend/crates/crm-demo/src/handlers/auth.rs:106-112`, `backend/crates/marionette/src/ws.rs:65-99`
- Current mitigation: Expiry is checked on WS connect; expired sessions are rejected but not deleted.
- Recommendations: Add a background task (or an on-connect cleanup query) that deletes sessions where `session_expires < now()`.

**Listmonk credentials stored in environment variables without validation:**
- Risk: `LISTMONK_USER` and `LISTMONK_PASSWORD` are passed directly as Basic Auth credentials. If the environment is misconfigured (e.g., empty string), the client silently uses an empty credential and may succeed against an improperly secured Listmonk instance.
- Files: `backend/crates/crm-demo/src/listmonk.rs:29-38`
- Current mitigation: `from_env()` returns `None` if any variable is absent (uses `ok()?`), but does not check for empty strings.
- Recommendations: Validate that credentials are non-empty before constructing the client.

---

## Performance Bottlenecks

**`handle_listmonk_sync_all` runs synchronously in the request loop:**
- Problem: Syncing all contacts iterates one-by-one over every contact, making a round-trip HTTP call to Listmonk per contact (find + create/update + set_lists). With 500 contacts this is hundreds of sequential HTTP calls blocking the WebSocket handler task.
- Files: `backend/crates/crm-demo/src/handlers/listmonk.rs:198-266`
- Cause: No concurrency, no streaming progress, no timeout.
- Improvement path: Use `futures::stream::iter(...).map(...).buffer_unordered(N)` for concurrent syncs; send intermediate patch messages to show progress; run as a background task and notify on completion.

**Contact edit form fires 2–3 separate tag full-table scans:**
- Problem: `handle_contact_form` calls `tag::Entity::find().all()` twice (once for the tags section, once again inside the same branch for the tag_name_map), plus a `contact_tag::Entity::find()` per contact. These are all table-scans on every form open.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:638-643`
- Cause: No shared query results; data is fetched in separate blocks rather than being reused.
- Improvement path: Load the tag table once per handler invocation and reuse the result.

**Contact list always loads all tags:**
- Problem: `render_contact_list` calls `tag::Entity::find().all()` to build a name map, even when no tag filter is active. For large tag tables this is a full scan on every list render.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:213-218`
- Cause: The all-tags load is inside the `if !contact_ids.is_empty()` block but is unconditional within it.
- Improvement path: Use a `WHERE tag_id IN (...)` query keyed to the actual contact_tag rows returned rather than loading all tags.

---

## Fragile Areas

**WsSession auth state set by parsing render payload fields:**
- Files: `backend/crates/marionette/src/ws.rs:246-268`
- Why fragile: Auth state on a live WebSocket connection is updated by looking for `_auth_user_id` and `_auth_role` in `render.data`. Any rename of these hidden keys, or a login handler that returns multiple `Render` messages before the auth-carrying one, will silently leave the session unauthenticated.
- Safe modification: When changing the login handler, verify the embedded key names match what `ws.rs` reads on lines 257-261; consider replacing with a dedicated `ProtocolMessage::Authenticated` variant.
- Test coverage: No test covers the WS auth-update path directly.

**Component tree uses flat HashMap with string IDs:**
- Files: `frontend/src/lib/store/surfaces.svelte.ts`, `backend/crates/marionette-protocol/src/component.rs`
- Why fragile: Components reference children by string ID. Duplicate IDs silently overwrite nodes in the HashMap. IDs are generated with a UUID suffix for unnamed components but callers can set explicit IDs (e.g., `"contact-form-root"`) that may collide across views.
- Safe modification: Always use globally unique IDs; avoid reusing IDs like `"tags-heading"` or `"note-form"` across different screens that could theoretically coexist.
- Test coverage: No test verifies ID uniqueness across a full render response.

**Dirty path queue not cleared on surface reset:**
- Files: `frontend/src/lib/store/dirty.svelte.ts`, `frontend/src/lib/store/data.svelte.ts`
- Why fragile: When a new `render` message arrives and `setFullState` replaces the surface data, the `dirtyPaths` set and `pendingPatches` map are not cleared. A path marked dirty before a navigation will silently suppress incoming patches on the new screen if its key happens to match.
- Safe modification: Call `resetDirty()` from the `render` message handler in `init.ts` when resetting surface state.
- Test coverage: No test covers the interaction between `setFullState` and stale dirty paths.

**Optimistic update surface is always empty string:**
- Files: `frontend/src/lib/transport/dispatcher.ts:63`, `frontend/src/lib/store/optimistic.svelte.ts`
- Why fragile: `applyOptimistic(id, '', optimistic.patch)` passes an empty string as the surface name, so the snapshot/restore operates on `getData('', path)` rather than the actual surface. If the optimistic path does not exist on the `''` surface, the rollback silently no-ops instead of restoring the real value.
- Safe modification: `sendAction` should accept a `surface` parameter and pass it through; or the optimistic patch type should include a surface field.
- Test coverage: Optimistic rollback is unit-tested in `optimistic.svelte.test.ts` but always uses `''` surface, masking the bug.

---

## Scaling Limits

**SQLite as the only supported database:**
- Current capacity: Single-writer SQLite; suitable for small teams (<20 concurrent users).
- Limit: Write contention under concurrent WebSocket sessions; no connection pooling for writes (SeaORM's `sqlx` SQLite driver serialises writes by default).
- Scaling path: SeaORM supports PostgreSQL; migrating requires schema review and replacing SQLite-specific datetime strings with proper `TIMESTAMPTZ` columns.

**WebSocket mpsc channel buffer fixed at 32 messages:**
- Current capacity: 32 queued messages per connected client.
- Limit: A slow client or burst of large render payloads will block the `tx.send()` call in `handle_text_message`, stalling the reader loop for that session.
- Files: `backend/crates/marionette/src/ws.rs:61`
- Scaling path: Use an unbounded channel or a larger buffer; add a per-session message counter to detect slow consumers and close their connection.

---

## Dependencies at Risk

**`flowbite-svelte-icons` (frontend):**
- Risk: Heavy icon library imported only for `ChevronUpOutline` and `ChevronDownOutline` in `DataTable.svelte`. Adds significant bundle weight for two icons.
- Impact: Increased frontend bundle size; any upstream breaking change forces an upgrade.
- Migration plan: Inline the two SVGs directly in `DataTable.svelte` and remove the dependency.

---

## Missing Critical Features

**No input sanitization on free-text fields:**
- Problem: Contact name, email, notes, and tag names are stored and rendered as plain strings. The frontend renders them via `String(rowData[col.key] ?? '')` in the DataTable and as `Text` components, so XSS through the SDUI layer is not possible — but if data is ever exported to HTML email templates or displayed outside the SDUI renderer the raw strings are unescaped.
- Blocks: Safe export/reporting features.

**No confirmation dialog for destructive WebSocket actions:**
- Problem: `contact_delete` and `user_delete` are triggered by a `click` action with no frontend confirmation step. The `ConfirmDialog` component exists in the registry but is not wired to any delete action.
- Blocks: Accidental deletion cannot be undone (no soft-delete exists).

**Event bus not implemented:**
- Problem: The `event` protocol message type handler in `init.ts` logs to `console.debug` and does nothing else. The comment says "Event bus will be implemented in a later plan."
- Files: `frontend/src/lib/init.ts:55-58`
- Blocks: Any cross-component communication that relies on server-pushed events (e.g., real-time collaboration, async job completion notifications).

---

## Test Coverage Gaps

**`handle_login_action` WS path has no unit test:**
- What's not tested: The WS login path that embeds `_auth_user_id` in render data; session state mutation in `ws.rs` on login success.
- Files: `backend/crates/crm-demo/src/main.rs:33-119`, `backend/crates/marionette/src/ws.rs:246-268`
- Risk: Auth regressions are invisible until manual testing.
- Priority: High

**`fetch-rows` filter gap is not tested:**
- What's not tested: That scrolling after a filter returns only filtered rows.
- Files: `backend/crates/crm-demo/src/handlers/contact.rs:408-510`
- Risk: The known filter-loss bug could regress silently.
- Priority: High

**Frontend optimistic rollback uses wrong surface:**
- What's not tested: That `rollbackOptimistic` restores values on the correct (non-empty) surface.
- Files: `frontend/src/lib/store/optimistic.svelte.ts`, `frontend/src/lib/transport/dispatcher.ts:63`
- Risk: Rollback silently no-ops; user sees stale optimistic data after a server error.
- Priority: Medium

**`listmonk_sync_all` concurrency and timeout are not tested:**
- What's not tested: Behaviour under Listmonk API errors partway through bulk sync; timeout on slow API.
- Files: `backend/crates/crm-demo/src/handlers/listmonk.rs:198-266`
- Risk: A single failing contact could block the sync of all subsequent contacts.
- Priority: Medium

---

*Concerns audit: 2026-04-08*
