---
phase: 12-protocol-node-patching-appshell
reviewed: 2026-04-10T19:16:41Z
depth: standard
files_reviewed: 48
files_reviewed_list:
  - backend/crates/crm-demo/src/handlers/audit.rs
  - backend/crates/crm-demo/src/handlers/company.rs
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/crm-demo/src/handlers/interaction.rs
  - backend/crates/crm-demo/src/handlers/user.rs
  - backend/crates/crm-demo/src/main.rs
  - backend/crates/crm-demo/tests/integration_test.rs
  - backend/crates/marionette-protocol/src/data.rs
  - backend/crates/marionette-protocol/src/messages.rs
  - backend/crates/marionette/src/builders/app_shell.rs
  - backend/crates/marionette/src/builders/mod.rs
  - backend/crates/marionette/src/builders/standard.rs
  - backend/crates/marionette/src/router.rs
  - backend/crates/marionette/src/ws.rs
  - backend/crates/marionette/tests/ws_integration.rs
  - frontend/src/lib/components/core/Surface.svelte
  - frontend/src/lib/components/core/SurfaceMount.browser-test.ts
  - frontend/src/lib/components/core/SurfaceMount.svelte
  - frontend/src/lib/components/form/SelectInput.svelte
  - frontend/src/lib/components/nav/SideNav.svelte
  - frontend/src/lib/components/shell/AppShell.browser-test.ts
  - frontend/src/lib/components/shell/AppShell.svelte
  - frontend/src/lib/index.ts
  - frontend/src/lib/init.ts
  - frontend/src/lib/registry/defaults.ts
  - frontend/src/lib/store/data.svelte.test.ts
  - frontend/src/lib/store/data.svelte.ts
  - frontend/src/lib/store/dirty.svelte.test.ts
  - frontend/src/lib/store/dirty.svelte.ts
  - frontend/src/lib/store/optimistic.svelte.test.ts
  - frontend/src/lib/store/optimistic.svelte.ts
  - frontend/src/lib/store/sidebar.svelte.ts
  - frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts
  - frontend/src/lib/store/surfaces.svelte.test.ts
  - frontend/src/lib/store/surfaces.svelte.ts
  - frontend/src/lib/transport/dispatcher.test.ts
  - frontend/src/lib/transport/messages.ts
  - frontend/src/lib/transport/websocket.connection-status.test.ts
  - frontend/src/lib/transport/websocket.svelte.test.ts
  - frontend/src/lib/transport/websocket.svelte.ts
  - frontend/src/lib/utils.ts
  - frontend/src/routes/+layout.svelte
  - frontend/tests/e2e/integration.spec.ts
  - frontend/tests/e2e/node-patch-focus.spec.ts
  - frontend/tests/e2e/protocol-conformance.spec.ts
  - frontend/tests/e2e/shell-nav.spec.ts
  - spec/schemas/data.yaml
  - spec/schemas/message.yaml
findings:
  critical: 0
  warning: 3
  info: 7
  total: 10
status: issues_found
---

# Phase 12: Code Review Report

**Reviewed:** 2026-04-10T19:16:41Z
**Depth:** standard
**Files Reviewed:** 48
**Status:** issues_found

## Summary

Phase 12's protocol node-patching + AppShell surface work is in good shape. The
protocol crate (`marionette-protocol`), the hand-written AppShell builder, the
frontend surface tree mutators, and the end-to-end node-patch demo are
well-tested: `surfaces.svelte.ts` has a dedicated focus-preservation browser
test, the protocol enum has round-trip tests for every variant including an
unknown-discriminator negative case, and the E2E protocol-conformance spec
validates real wire frames against the updated schemas.

Three warnings stand out:

1. **Auth side-channel leak into main surface data (WR-01).** `handle_login_action`
   embeds `_auth_user_id` and `_auth_role` into the first Render message's
   `data` so that `ws.rs` can extract them; `ws.rs` reads them but never strips
   them, so those fields end up persisted in the frontend's `main` surface data
   store where any component can read them via JSON pointer lookup. Not a
   credentials leak (no secrets), but an unintended internal-state exposure.
2. **Silent failure on last-login update (WR-02).** The `let _ =
   active_user.update(&*ctx.db).await;` pattern discards a database error
   without logging, so a failed last-login write is invisible to operators.
3. **Fragile hard-coded child-insertion index (WR-03).** `handle_contact_country_change`
   pins `insert_index = 6` with a comment about the current `contact-form`
   children order. Any reordering of the form (e.g., adding a new field before
   Save) silently mis-positions the country-specific child without any compile
   or test error.

The remaining seven findings are info-level hygiene items: dead code, unused
destructured props, N+1 queries inside loops that the plan already
acknowledges as "demo scale", and duplicated nav-active-patch helpers across
five handler files.

No critical security issues were found. The `__mrnSendAction` E2E test hook
exposed on `window` is explicitly documented in `init.ts` as safe (anything an
attacker can do through it they can also do by crafting a WebSocket
ActionMessage directly), so it is not flagged.

## Warnings

### WR-01: Auth side channel leaks `_auth_user_id` / `_auth_role` into main surface data

**File:** `backend/crates/crm-demo/src/main.rs:108-117`, `backend/crates/marionette/src/ws.rs:247-271`

**Issue:** `handle_login_action` embeds internal auth bookkeeping into the first
Render message's `data` so that `ws.rs` can pick it up and update `WsSession`:

```rust
// main.rs:108-117
for msg in &mut messages {
    if let ProtocolMessage::Render(render) = msg {
        if let Some(data) = render.data.as_object_mut() {
            data.insert("_auth_user_id".into(), serde_json::json!(user_id));
            data.insert("_auth_role".into(), serde_json::json!(user_role));
        }
        break;
    }
}
```

`ws.rs` reads `_auth_user_id` / `_auth_role` out of `render.data` to seed the
session roles, **but never removes them**:

```rust
// ws.rs:254-267
for r in &responses {
    if let ProtocolMessage::Render(render) = r {
        if let Some(user_id) = render.data.get("_auth_user_id")...
```

The first Render in the login response targets `surface: "main"` (the shell),
so these fields land inside the frontend's `main` data store after
`setFullState('main', msg.data)` and remain readable via
`getData('main', '/_auth_user_id')` from any component. This is a soft
information exposure — the values are not secret (the user ID and role are
already encoded in the session cookie), but the leak is unintentional and
surface data is specifically the portion of state meant to be user-facing and
data-bound. A future handler that does `data-bind="/_auth_*"` or an audit dump
of surface state will expose these internals.

**Fix:** Strip the fields in `ws.rs` after consuming them, so the data that
reaches the frontend no longer carries the side channel:

```rust
// ws.rs, inside the login-response loop — use get_mut + as_object_mut and
// remove the fields after reading. Use &mut responses instead of &responses:
for r in &mut responses {
    if let ProtocolMessage::Render(render) = r {
        if let Some(obj) = render.data.as_object_mut() {
            if let Some(user_id) = obj.remove("_auth_user_id").and_then(|v| v.as_i64()) {
                session.user_id = Some(user_id.to_string());
            }
            if let Some(role) = obj.remove("_auth_role").and_then(|v| v.as_str().map(str::to_owned)) {
                session.roles = vec![role];
            }
        }
        break;
    }
}
```

A cleaner alternative (no side channel) is to have `handle_login_action` return
an out-of-band signal via a new `ActionResult` variant or a session-update
message type, but the above strip-on-consume is a one-line fix and preserves
the current architecture.

### WR-02: Silent error on last-login update discards diagnostic information

**File:** `backend/crates/crm-demo/src/main.rs:93`

**Issue:** After successfully verifying the password, the handler updates
`user_last_login` with `let _ = active_user.update(&*ctx.db).await;`. A failure
here (connection pool exhausted, transient lock, schema drift) is swallowed
entirely — no log, no metric, no user-visible error. Because the rest of the
login flow continues regardless, an operator debugging "why does last_login
never update" has no signal at all.

**Fix:** Log at `warn!` level on failure — this preserves the "last-login
update is best-effort" intent while making failures diagnosable:

```rust
if let Err(e) = active_user.update(&*ctx.db).await {
    tracing::warn!(user_id, error = %e, "failed to update last_login (best-effort)");
}
```

### WR-03: Hard-coded `insert_index: usize = 6` fragile to form restructuring

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:1409-1413`

**Issue:** `handle_contact_country_change` inserts the country-specific field
(Canton / State / Bundesland) into the `contact-form` children array at a
hard-coded index:

```rust
// Index 6 places it after the country select (contact-form children order:
// name, email, phone, title, company, country, save, cancel — country is at
// index 5 and the new field slots in at index 6).
let insert_index: usize = 6;
```

`handle_contact_form` (same file, lines 594-606) defines the actual children
order via `Form::new().children(vec![...])`. If a future edit adds any field
before Save (e.g., a "department" field at the end of the contacts section, or
moves `country_select` earlier), the constant stays 6 but now points into the
wrong slot — the country-specific field appears between, say, Title and
Company with no compile error and no runtime error, only a visibly-wrong form
layout that E2E tests may or may not catch depending on their selectors. The
node-patch-focus E2E test locates fields by label text, so it would still pass
even with a mis-positioned Canton child.

**Fix:** Derive the insertion index at runtime from the current children array
so the constant never drifts. One option is to have the country-change handler
emit a `SetChildren` op that rebuilds the children list in full, with the
country field spliced in at a well-known anchor (after `contact-form-country`):

```rust
// Pseudocode — replace the hard-coded index with an anchor lookup.
// Assumes the handler can inspect the form's current children, which
// requires either passing them in the action payload or looking them up
// through a helper. Alternative: use SetChildren to rebuild the list
// from a canonical ordering function shared with handle_contact_form.
const COUNTRY_ANCHOR: &str = "contact-form-country";
// ... then find the anchor position and insert after it.
```

A lighter fix is to at least add a compile-time assertion by co-locating
the constant with a `#[cfg(test)] mod tests` that reconstructs the form and
asserts the anchor is at index 5, so a restructure breaks the test rather than
the runtime layout.

## Info

### IN-01: Dead code — `frontend/src/lib/store/sidebar.svelte.ts` is not imported anywhere

**File:** `frontend/src/lib/store/sidebar.svelte.ts:1-4`

**Issue:** The file exports `isSidebarOpen`, `toggleSidebar`, and `closeSidebar`
backed by a module-local `$state(false)`, but a grep of `frontend/src` shows no
importers. The shadcn Sidebar primitive (`AppShell.svelte`) manages its own
open/close state through `Sidebar.Provider`, so this module is dead weight.
Notably it is also not re-exported from `frontend/src/lib/index.ts`.

**Fix:** Delete the file (it is currently untracked in git per `git status`),
or re-export + use it if there is a planned caller.

### IN-02: `SelectInput.svelte` destructures `children` and `action` props that are never consumed

**File:** `frontend/src/lib/components/form/SelectInput.svelte:11-22`

**Issue:** The `children` snippet is destructured from `$props()` but the
template never renders `{@render children?.()}` — the shadcn Select owns the
children layout. Leaving the unused binding is a style-lint miss that would
also trip `svelte-check` in strict mode.

**Fix:** Drop `children` from the props destructure, or prefix with underscore
(`children: _children`) to match the convention already used elsewhere in this
file for `bind`/`_bind` (note: this file uses `bind` and `action` directly in
the handler, so only `children` is truly unused).

### IN-03: Five copies of `nav_active_patch` — obvious duplication

**Files:**
- `backend/crates/crm-demo/src/handlers/audit.rs:223-239`
- `backend/crates/crm-demo/src/handlers/company.rs:393-409`
- `backend/crates/crm-demo/src/handlers/contact.rs:956-972`
- `backend/crates/crm-demo/src/handlers/interaction.rs:164-180`
- `backend/crates/crm-demo/src/handlers/user.rs:310-326`

**Issue:** The `nav_active_patch(active_slug: &str) -> ProtocolMessage` helper
is literally identical in all five handler modules — same slug list, same op
building, same docstring. Any new nav item (e.g., the plan's mention of adding
a "Settings" slug) must be added to all five copies or one screen will render
with a stale active indicator.

**Fix:** Move the helper to a shared location, e.g.,
`crm-demo/src/handlers/mod.rs` or a new `crm-demo/src/nav.rs`, and import from
each handler.

### IN-04: N+1 query inside loop — `company.rs::render_company_list` computes contact counts per row

**File:** `backend/crates/crm-demo/src/handlers/company.rs:112-131`

**Issue:** For each company in the list, a separate `contact::Entity::find().count()`
query runs. At demo scale (≤20 companies) this is harmless, but the pattern is
worth flagging because it will bite the moment the seed data or a real dataset
grows. The same file later loads per-note authors via `find_by_id` inside a
loop at line 338-345. The similar N+1 in `contact.rs` (notes author lookup,
`interaction` users batch-load) is at least partially batched, so company.rs
sticks out as inconsistent.

**Fix:** Precompute a single `SELECT company_id, COUNT(*) FROM contact GROUP
BY company_id` query and look up counts from the resulting `HashMap<i32, i64>`.
Same shape for the notes author batch-load.

### IN-05: `contact.rs` tag-filter fallback sentinel uses negative ID instead of a no-op result

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:152-154, 165-166`

**Issue:** When tag filtering matches zero tags, the code appends
`contact::Column::ContactId.eq(-1)` to force no rows. This works because IDs
are always positive, but it's an implicit magic value that would break if the
schema ever uses signed IDs or sentinel rows. A clearer pattern is to return
an empty `Vec` early, skipping the main query entirely.

**Fix:**

```rust
if matching_tag_ids.is_empty() || tagged_ids.is_empty() {
    // Short-circuit: no tags match, no contacts can match — return empty list.
    return Ok(vec![
        ProtocolMessage::Render(RenderMessage { /* empty list render */ }),
        nav_active_patch("contacts"),
    ]);
}
```

Or at minimum extract the sentinel to a named `const NO_MATCH_SENTINEL_ID: i32 = -1;`.

### IN-06: `setFullState` clear-and-assign pattern could use spread but is correct for reactivity

**File:** `frontend/src/lib/store/data.svelte.ts:56-64`

**Issue:** This is deliberately verbose for Svelte 5 `$state` reactivity — the
comment explains that reassignment would break reactivity. Flagging only so
that a future refactor doesn't "simplify" it into `store.data = data` and
break the reactive proxy. Consider pulling this into a unit test that asserts
the reference identity is preserved after `setFullState`.

**Fix:** Add a test case:

```ts
it('setFullState preserves store.data reference', () => {
    const store = getStore('main');
    const ref = store.data;
    setFullState('main', { x: 1 });
    expect(getStore('main').data).toBe(ref);
});
```

### IN-07: `handle_contact_country_change` builds a redundant `String` allocation

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:1376-1382`

**Issue:** `country` is built as `.to_string()` from an `&str` and then
immediately used via `country.as_str()` in the downstream `match`. The
allocation is unnecessary — the whole flow could work on the borrowed `&str`
since the payload is cloned earlier.

**Fix:**

```rust
let country: &str = payload
    .get("contactForm")
    .and_then(|v| v.get("country"))
    .and_then(|v| v.as_str())
    .or_else(|| payload.get("country").and_then(|v| v.as_str()))
    .unwrap_or("");
// ... then use `country` directly in the match. Note: the `Set` op
// currently stores `json!(country)` which needs a cloned String, so
// materialize it once at that single use site.
```

Minor; the current code is correct.

---

_Reviewed: 2026-04-10T19:16:41Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
