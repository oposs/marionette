# Phase 12 Deferred Items

Items discovered during execution that are out of scope for the current plan.

## From 12-01-scaffolding

### Pre-existing TypeScript errors in tests/helpers/schema-validator.ts

`npm run check` reports 3 errors about `fs`, `path`, `url` module resolution:
- `tests/helpers/schema-validator.ts:4` — Cannot find module 'fs'
- `tests/helpers/schema-validator.ts:5` — Cannot find module 'path'
- `tests/helpers/schema-validator.ts:6` — Cannot find module 'url'

These predate Phase 12 (present before the shadcn Sidebar install). Likely missing `@types/node` or the test helper tsconfig needs `"types": ["node"]`. Not caused by Plan 12-01 changes.

## From 12-02-protocol-crate

### Pre-existing clippy pedantic failures in crm-demo

**Discovered during:** Task 2 verification — `cargo clippy --workspace -- -D warnings`

**Scope:** Entirely in `backend/crates/crm-demo/` (not touched by Plan 12-02)

**Count:** 76 errors across ~20 distinct lint categories, including:
- `clippy::struct_field_names` (audit_log, company, contact, interaction, listmonk_cache, listmonk_sync, note, user — all models)
- `clippy::too_many_lines` (8 functions: 106, 109, 123, 159, 199, 200, 321, 388 lines)
- `clippy::map_unwrap_or` / `map_unwrap_or_else`
- `clippy::cast_possible_truncation` (i64 → i32)
- `clippy::implicit_clone`
- `clippy::doc_markdown` (WsSession, SeaORM, etc. not in backticks)
- `clippy::needless_borrows_for_generic_args`
- `clippy::manual_let_else`
- `clippy::collapsible_if`
- `clippy::manual_pattern_char_comparison`
- `clippy::useless_format`
- `clippy::very_complex_type`

**Root cause:** Toolchain drift — the pinned clippy version (1.93.0) introduced new lints that weren't in effect when this code was originally written. None of these are caused by Plan 12-02's changes.

**Verification:** `git stash && cargo clippy -p crm-demo -- -D warnings` (pre-Plan-12-02 state) reproduces all 76 errors.

**In-scope crates are clean:** `cargo clippy -p marionette-protocol -p marionette -- -D warnings` exits 0 after Plan 12-02.

**Recommended resolution:** Dedicated lint-cleanup plan in Phase 12 or early Phase 13 — mechanical fixes only, no behavior changes. Alternative: add targeted `#[allow(...)]` at crate root with a TODO for each category.

## From 12-06-frontend-shell-components

### Pre-existing popup browser-test failures (ConfirmDialog + ToastSurface)

**Discovered during:** Task 4 verification — `npx vitest --config vitest-browser.config.ts --run`.

**Scope:** 5 failing tests in `frontend/src/lib/components/popup/`:
- `ConfirmDialog.browser-test.ts` — 4/4 tests fail ("renders title and message", "renders confirm and cancel buttons", "dispatches action on confirm click", "dispatches close-modal on cancel click"). All throw Playwright locator errors — likely the dialog markup changed post-shadcn update and the selectors drifted.
- `ToastSurface.browser-test.ts` — 1/3 tests fail ("removes toast on dismiss click") with "strict mode violation: ... resolved to 2 elements" on `getByLabelText('Dismiss')`. Double-render or leaked state between tests; the `.first()` selector would fix it.

**Verification these are pre-existing:** `git stash && npx vitest --config vitest-browser.config.ts --run src/lib/components/popup/` on the pre-Plan-12-06 tree reproduces exactly the same 5 failures. Plan 12-06 does not touch `src/lib/components/popup/*`.

**In-scope tests are all green after Plan 12-06:**
- `SurfaceMount.browser-test.ts` — 2/2 passing
- `AppShell.browser-test.ts` — 3/3 passing
- `websocket.connection-status.test.ts` — 5/5 passing
- `websocket.svelte.test.ts` — 10/10 passing (unit tests)
- All 58 unit tests under `vitest --run` passing

**Recommended resolution:** Folded into Phase 13 (form screens) or a targeted popup-fix plan — selector updates only, no behavior changes.

## From 12-08-demo-and-e2e

### TextInput `input_type` -> `type` prop mismatch

**Discovered during:** Task 2 writing the node-patch-focus E2E test (the login form's password field had to be selected by grid wrapper rather than `input[type="password"]`).

**Scope:** `frontend/src/lib/components/form/TextInput.svelte` reads `props.type` for the HTML input `type` attribute, while the backend builder (`backend/crates/marionette/src/builders/standard.rs::TextInput`) has a `input_type: Option<String>` field that the `#[derive(ComponentBuilder)]` macro serializes under the key `"input_type"`. The two don't match, so **the login password field has never actually rendered as `type="password"` since the AppShell migration** — it has always been `type="text"` (or the browser default) exposing the password as plaintext in the DOM.

**Severity:** Low for the pre-deployment posture (no real users, no real credentials, admin password is a demo literal). Higher if the CRM ever ships — a plaintext-rendering password field is a UX bug at minimum and a soft information-disclosure surface in certain environments.

**Pre-existing:** Yes. The divergence predates Plan 12-08. The commit introducing the `input_type` builder field probably expected the frontend to read `props.input_type` (or the macro to convert snake_case to camelCase). Neither happened.

**Fix options:**
1. Change `TextInput.svelte` to read `props.input_type ?? props.type ?? 'text'` (quick, preserves backward compat).
2. Change the macro to emit camelCase keys (`inputType`), then update `TextInput.svelte` to read `props.inputType`. Bigger scope — affects every builder with snake_case optional fields.
3. Rename the backend builder field to `type` (requires `r#type` or a rename attribute since `type` is a reserved word).

**Recommended resolution:** Option 1 as a one-line fix in Phase 13 or as a targeted `fix()` commit. Until then, Plan 12-08's E2E test locates the password input via its grid wrapper.

### Stale `integration.spec.ts` / `protocol-conformance.spec.ts` assertions

**Discovered during:** Task 2 / Task 3 baseline check.

**Scope:**
- `frontend/tests/e2e/integration.spec.ts` asserts `getByText('Welcome to Marionette')` and `getByText('Click Me')` — these strings come from the pre-CRM demo app and no longer appear after the Plan 12-07 CRM integration landed. Those tests fail against the current crm-demo backend.
- `frontend/tests/e2e/protocol-conformance.spec.ts` (original contents) asserts the same strings in its `patch message` case.

**Pre-existing:** Yes. These tests were written against an older demo backend and never updated when the CRM replaced it. Plan 12-08's new tests don't rely on them.

**Recommended resolution:** Plan 12-08's Task 3 extends `protocol-conformance.spec.ts` to cover the new node-op shapes. The old "Welcome to Marionette" assertion cases should be removed or rewritten against the CRM landing screen in a follow-up cleanup commit.

