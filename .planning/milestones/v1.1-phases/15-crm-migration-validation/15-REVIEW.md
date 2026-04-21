---
phase: 15-crm-migration-validation
reviewed: 2026-04-18T10:00:00Z
depth: standard
files_reviewed: 33
files_reviewed_list:
  - backend/crates/marionette/src/builders/standard.rs
  - backend/crates/marionette/src/validation.rs
  - backend/crates/marionette/src/lib.rs
  - backend/crates/crm-demo/src/entities/contact.rs
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/crm-demo/src/handlers/company.rs
  - backend/crates/crm-demo/src/handlers/user.rs
  - backend/crates/crm-demo/src/handlers/interaction.rs
  - backend/crates/crm-demo/src/handlers/note.rs
  - backend/crates/crm-demo/src/handlers/listmonk.rs
  - backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs
  - backend/crates/crm-demo/src/migration/mod.rs
  - backend/crates/crm-demo/src/seed.rs
  - backend/crates/crm-demo/tests/contact_persistence.rs
  - frontend/src/lib/init.ts
  - frontend/src/lib/components/form/Form.svelte
  - frontend/src/lib/components/form/Form.browser-test.ts
  - frontend/tests/helpers/schema-validator.ts
  - frontend/tests/e2e/ci-guards.spec.ts
  - frontend/tests/e2e/company-edit.spec.ts
  - frontend/tests/e2e/user-edit.spec.ts
  - frontend/tests/e2e/interaction-edit.spec.ts
  - frontend/tests/uat/company-edit-uat.spec.ts
  - frontend/tests/uat/user-edit-uat.spec.ts
  - frontend/tests/uat/interaction-edit-uat.spec.ts
  - frontend/tests/uat/contact-tag-add-uat.spec.ts
  - frontend/tests/uat/contact-note-add-uat.spec.ts
  - frontend/tests/visual/form.spec.ts
  - frontend/package.json
  - spec/PROTOCOL.md
  - CONCEPT.md
  - TOOLING.md
  - .planning/codebase/STACK.md
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 15: Code Review Report

**Reviewed:** 2026-04-18T10:00:00Z
**Depth:** standard
**Files Reviewed:** 33
**Status:** issues_found

## Summary

Phase 15 delivers the CRM form migration: a SeaORM migration adding `contact_country`, `contact_notes`, and `contact_opt_in`; a new `form_shell()` helper and `validation_error_patch()` function in the marionette crate; and rewrites of the company, user, interaction, and note handlers to the canonical Phase 14 FieldSet + form_shell + per-field validation pattern. Frontend changes gate test hooks behind `import.meta.env.DEV` and fix the Form submit payload to pass the actual store subtree instead of `{}`.

Overall quality is high: the security model is well-considered (bind-path injection warning documented, server-side role validation on RadioGroup values, DEV-gate on test hooks), all new handlers have inline unit tests, and the E2E / UAT spec coverage is thorough. Three warnings and four informational items are noted below; none are blockers.

---

## Warnings

### WR-01: `contact_persistence.rs` integration test not found — file listed in scope is missing

**File:** `backend/crates/crm-demo/tests/contact_persistence.rs`
**Issue:** This file was listed as a deliverable (Plan 15-01) and appears in the review scope, but does not exist on disk. If the integration test confirming the three new columns persist correctly was intentionally deferred, that is a gap in the Plan 15-01 acceptance criteria ("integration test verifying persistence of country/notes/opt_in"). Without it there is no automated proof that the migration + entity + save handler wiring round-trips the new fields.
**Fix:** Either create the test:
```rust
// backend/crates/crm-demo/tests/contact_persistence.rs
#[tokio::test]
async fn contact_country_notes_opt_in_persist() {
    let db = marionette::test_db().await;
    // ... insert contact with country="CH", notes="hello", opt_in=true
    // ... find_by_id and assert all three columns match
}
```
or record the deferral explicitly in `deferred-items.md` so the gap is tracked.

---

### WR-02: Edit-mode contact form pre-populates `country`, `notes`, and `opt_in` as empty/false regardless of stored values

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:454-466`
**Issue:** When rendering the edit form for an existing contact, the handler hardcodes `"country": ""`, `"notes": ""`, and `"optIn": false` rather than reading the three new columns that were added in migration 11 and now exist on `contact::Model`. If a user saved a contact with country="CH", notes="some text", and opt_in=true, then re-opened the edit form, those values would be silently overwritten with the defaults on the next save — a data-loss regression hidden by the stale comment "The contact entity does not yet have these columns (Phase 15 will add them)".
```rust
// current (wrong after migration 11 lands)
"country": "",
"notes": "",
"optIn": false
```
**Fix:** Read from the entity model that is already fetched on line 443:
```rust
"country": found.contact_country.as_deref().unwrap_or(""),
"notes": found.contact_notes.as_deref().unwrap_or(""),
"optIn": found.contact_opt_in
```
The comment block on line 449 was written in Phase 14 when the columns did not yet exist; it must be removed.

---

### WR-03: `form_shell` double-inserts heading, back_button, and form_child into the nodes map

**File:** `backend/crates/marionette/src/builders/standard.rs:649-661`
**Issue:** `build_with_children()` already emits the three child tuples as part of `container_nodes`. The function then iterates those with `nodes.insert(id, c)` (line 649-651) and afterwards explicitly re-inserts the same three tuples again (lines 656-658). Each re-insertion silently overwrites the previous identical value, so the functional result is correct. However, the comment on line 652-655 acknowledges this is redundant and frames it as an intentional guard — but if the macro's emission order ever changes such that `build_with_children` stops emitting the children tuples, the explicit inserts would be the only safety net. The current code therefore contains a structural inconsistency: either the explicit inserts are the authoritative path (in which case iterating `container_nodes` for the children is redundant), or `container_nodes` is authoritative (in which case the explicit inserts are needless clones). The unnecessary `clone()` calls on lines 640, 641, 642 and 656, 657, 658 (six clones of protocol `Component` values) are the concrete cost.
**Fix:** Remove the three explicit re-inserts (lines 656-658) and the `.clone()` calls on the container children argument, relying entirely on `build_with_children`'s output. If the authoritative contract is that `build_with_children` always includes child tuples, document that contract in the macro rather than silently guarding against it here:
```rust
// Remove lines 656-658:
// nodes.insert(heading.0, heading.1);
// nodes.insert(back_button.0, back_button.1);
// nodes.insert(form_child.0, form_child.1);
// And remove the .clone() calls since the values are moved into build_with_children.
```

---

## Info

### IN-01: Stale comment in `handle_contact_form` references Phase 14 pre-migration state

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:449-453`
**Issue:** Comment block reads "The contact entity does not yet have these columns (Phase 15 will add them), so they seed to empty/false". Phase 15 has now added the columns. This stale comment will mislead future readers into thinking the empty defaults are intentional rather than a bug (see WR-02).
**Fix:** Remove or replace with: "Phase 15 migration 11 added these columns; read from `found.*` — see the lines below."

---

### IN-02: `caller_id` falls back to `0` on unauthenticated sessions — silent audit corruption

**File:** `backend/crates/crm-demo/src/handlers/company.rs:543-548`, `user.rs:538-543`, `contact.rs:1127-1131`, `interaction.rs:255-259`
**Issue:** Every save handler contains the pattern:
```rust
let caller_id: i32 = session
    .user_id
    .as_ref()
    .and_then(|id| id.parse().ok())
    .unwrap_or(0);
```
`user_id 0` does not correspond to any real user in the database, so audit records created by an unauthenticated (or session-less) caller are silently attributed to a phantom user. The session extractor should already reject unauthenticated requests upstream, but the silent fallback here is a defense-in-depth gap: if the auth middleware ever allows a request through without a valid session, audit records are written with a dangling `user_id=0` FK that could confuse log analysis.
**Fix:** Return an error rather than silently defaulting:
```rust
let caller_id: i32 = session
    .user_id
    .as_ref()
    .and_then(|id| id.parse().ok())
    .ok_or_else(|| ActionError::Internal("Missing or invalid session user_id".into()))?;
```

---

### IN-03: `test_get_history_fetches_when_cache_stale` races on `LISTMONK_CLIENT` global

**File:** `backend/crates/crm-demo/src/handlers/listmonk.rs:711-788`
**Issue:** The test sets `LISTMONK_CLIENT` via `OnceLock::set` on line 744. `OnceLock` only allows a single initialization; if any other test in the same binary has already called `LISTMONK_CLIENT.set(...)`, the `set` call silently returns `Err` (the `let _ =` discards it). The test will then use whatever client was set first, potentially pointing at the wrong mock server URL. If the mock server from that earlier test has already been dropped, the HTTP call will fail non-deterministically.
**Fix:** Use `OnceLock::get_or_init` with a conditional path, or redesign `get_cached_or_fetch_history` to accept an optional `client` parameter in tests. Alternatively, document in the test that it only exercises the cache-miss path when no previous test has initialised the global — and assert `LISTMONK_CLIENT.get().is_none()` at the top of the test to fail fast rather than silently.

---

### IN-04: UAT specs hardcode `http://localhost:5173/` — breaks CI without dev server

**File:** `frontend/tests/uat/company-edit-uat.spec.ts:36`, `user-edit-uat.spec.ts`, `interaction-edit-uat.spec.ts`, `contact-tag-add-uat.spec.ts`, `contact-note-add-uat.spec.ts`
**Issue:** The UAT specs call `page.goto('http://localhost:5173/')` with an absolute URL rather than using the base URL configured in the Playwright config. The E2E specs in `tests/e2e/` correctly use `page.goto('/')` (relative). The UAT specs will silently try port 5173 even when a CI runner sets a different `baseURL` in the Playwright config (e.g., for ephemeral port allocation).
**Fix:** Replace the hardcoded URL with `page.goto('/')` in all UAT files:
```typescript
// Before
await page.goto('http://localhost:5173/');
// After
await page.goto('/');
```

---

_Reviewed: 2026-04-18T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
