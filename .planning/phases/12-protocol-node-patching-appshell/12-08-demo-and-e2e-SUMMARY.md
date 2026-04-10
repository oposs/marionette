---
phase: 12-protocol-node-patching-appshell
plan: 08
subsystem: fullstack
tags: [crm, demo, e2e, playwright, focus-preservation, toast-lifecycle, protocol-conformance]

requires:
  - phase: 12-02
    provides: "tagged PatchOperation enum + PatchMessage.surface (Rust)"
  - phase: 12-03
    provides: "spec/schemas node-op oneOf + PatchMessage.surface + PROTOCOL.md 1.1.0"
  - phase: 12-04
    provides: "frontend fine-grained surface mutators + D-A6 focus preservation + applyPatch dispatcher"
  - phase: 12-05
    provides: "AppShell + SurfaceMount builders"
  - phase: 12-06
    provides: "AppShell.svelte + SurfaceMount.svelte + single-Surface layout"
  - phase: 12-07
    provides: "handle_navigate AppShell wiring + handler retargeting to content sub-surface + nav_active_patch helpers"
provides:
  - "contact_country_change action + handler emitting mixed content-surface PatchMessage + toasts-surface PatchMessage (5 node-op variants exercised end-to-end)"
  - "dismiss_toast action + handler closing the D-B15 toast lifecycle via delete-node on toasts sub-surface"
  - "Country Select field on contact form (`contact-form-country`) wired to contact_country_change via ComponentAction::change"
  - "handle_navigate now seeds the toasts sub-surface with an empty toasts-root Container (third Render after shell + content)"
  - "SelectInput.svelte dispatches sendAction on value change (Rule 2 missing critical functionality)"
  - "Client websocket hello bumped to 1.1.0 (Rule 1 drift fix)"
  - "__mrnSendAction test hook on window for E2E tests that need to trigger actions without moving keyboard focus"
  - "node-patch-focus.spec.ts — 3 E2E tests: D-A6 focus preservation + CH→US swap + D-B15 toast lifecycle"
  - "shell-nav.spec.ts — 2 E2E tests: AppShell landmarks + content sub-surface nav swap; mobile Sidebar.Trigger"
  - "protocol-conformance.spec.ts — 4 E2E tests including new node-tree-ops patch validation against the updated schemas"
  - "integration.spec.ts rewritten against the current crm-demo backend (no more Welcome to Marionette / Click Me stale assertions)"
affects:
  - "Phase 13 (form screens) — will inherit the SelectInput change-dispatch pattern + __mrnSendAction test hook"
  - "Phase 15 (CRM cleanup) — will inherit the country-swap demo as the canonical reference for node-patch driven UX"

tech-stack:
  added: []
  patterns:
    - "Two-message response pattern for node-patch demos: one PatchMessage per surface (content + toasts), each carrying its own atomic op batch"
    - "ComponentAction::change wired to frontend SelectInput sendAction — the missing sibling of Button's click dispatch"
    - "window.__mrnSendAction E2E hook for focus-isolated tests: the only way to prove D-A6 is to trigger the server-side patch WITHOUT clicking a focusable UI element (which would naturally steal focus)"
    - "Grid-wrapper locator pattern for inputs without label/input association: `page.locator('div.grid:has(label:has-text(\"Name\"))').getByRole('textbox')`"

key-files:
  created:
    - ".planning/phases/12-protocol-node-patching-appshell/12-08-demo-and-e2e-SUMMARY.md"
  modified:
    - "backend/crates/crm-demo/src/handlers/contact.rs — added country Select to handle_contact_form; added handle_contact_country_change (content + toasts patches); added handle_dismiss_toast; added contactForm.country to initial form data"
    - "backend/crates/crm-demo/src/main.rs — registered contact_country_change and dismiss_toast actions; seeded toasts sub-surface with empty toasts-root Container in handle_navigate (third Render)"
    - "frontend/src/lib/components/form/SelectInput.svelte — dispatches sendAction on change when action.type === 'change'"
    - "frontend/src/lib/init.ts — exposes window.__mrnSendAction as E2E test hook"
    - "frontend/src/lib/transport/websocket.svelte.ts — client hello version bumped from 1.0.0 to 1.1.0"
    - "frontend/src/lib/transport/websocket.svelte.test.ts — assertion updated to 1.1.0"
    - "frontend/tests/e2e/node-patch-focus.spec.ts — 3 real E2E tests replacing the 3-line scaffold"
    - "frontend/tests/e2e/shell-nav.spec.ts — 2 real E2E tests replacing the 3-line scaffold"
    - "frontend/tests/e2e/protocol-conformance.spec.ts — hello test filters on direction, new node-tree-ops test case added, version check bumped to 1.1.0"
    - "frontend/tests/e2e/integration.spec.ts — rewritten against current crm-demo backend"
    - ".planning/phases/12-protocol-node-patching-appshell/deferred-items.md — logged TextInput input_type bug + stale integration.spec.ts note"

key-decisions:
  - "Toast node is a Button, not a Heading — the Heading SDUI component ignores its action field on the frontend, so a clickable dismissable Heading would never fire dismiss_toast. Using Button respects the existing component contract and requires no shell-mount changes."
  - "D-A6 focus preservation is proved via the __mrnSendAction window hook, not via UI-driven Select click. Clicking the shadcn Select trigger naturally moves keyboard focus to the trigger button — that's native browser behavior unrelated to node patching. The only way to isolate the test to the patch application step is to dispatch the action without moving focus, which means calling sendAction directly. The window hook is a narrow, intentional test-only surface and is safe in production: any JS attacker already has full WebSocket access."
  - "SelectInput was not dispatching change actions before this plan (Rule 2 — missing critical functionality). Without this fix the entire Plan 08 demo would be a dead wire. Added sendAction to handleValueChange mirroring Button's pattern: full surface data spread with action.payload overrides."
  - "Country field ids are canonical and fixed (`contact-ch-canton`, `contact-us-state`, `contact-de-bundesland`) so the idempotent teardown loop in handle_contact_country_change can unconditionally remove all three candidates before inserting the new one. Deleting a non-existent node is a no-op in the frontend store (D-A8 GC tolerance)."
  - "Toasts sub-surface is seeded in handle_navigate via a third Render (after shell + content) rather than lazily in the first insert-child op. Lazy initialization would have required every patch handler to check-and-seed the sub-surface, leaking shell-mount concerns into every screen handler."
  - "integration.spec.ts stale assertions rewritten rather than deleted — the file still covers the WS connect, navigate flow, login flow, health endpoint, and SPA fallback, all against the current crm-demo. Deleting it would have lost coverage; rewriting preserves intent."

requirements-completed: [PATCH-01, PATCH-02, PATCH-03, SHELL-01, SHELL-02, SHELL-04]

duration: ~70 min
completed: 2026-04-10
---

# Phase 12 Plan 08: Demo + E2E Summary

**Goal-backward gate for Phase 12 closed: the country-select node-patch demo proves PatchOperation end-to-end with focus preservation and toast lifecycle, and 15/15 Playwright E2E tests validate the full protocol + AppShell wiring against the real crm-demo backend.**

## Goal-backward gate

This plan is Phase 12's final gate. It closes the phase's 8 success criteria and all 7 must-haves in one atomic delivery:

| Phase 12 Success Criterion | Closed by |
|---|---|
| 1. PatchMessage carries data + tree ops atomically | Plans 02/03/04 delivered the shape; Plan 08's `protocol-conformance.spec.ts:patch message with node tree ops` captures a live wire frame with all 5 node-op variants and validates it against `spec/schemas/message.yaml` |
| 2. Focus preservation works | Plan 04 proved it at the store level with a browser test; Plan 08's `node-patch-focus.spec.ts:country-select change preserves focus on Name` proves it end-to-end against the real backend via the `__mrnSendAction` hook |
| 3. Version 1.1.0 + CONCEPT.md reconciled | Plan 02 bumped ws.rs; Plan 03 bumped spec/PROTOCOL.md + CONCEPT.md; Plan 08 asserts `helloFrame.data.version === '1.1.0'` on the live wire; and bumped the client's own hello frame to match (Rule 1 drift fix) |
| 4. AppShell collapsible sidebar desktop + mobile sheet | Plan 06 shipped the component; Plan 08's `shell-nav.spec.ts:Sidebar.Trigger is present in the header` shrinks the viewport and asserts the trigger is visible |
| 5. Header title + user menu; footer status + version | Plan 07 wired `handle_navigate`; Plan 08 asserts the "Marionette v1.1 · Protocol 1.1.0" literal and "© 2026 Marionette" legal text and "Marionette CRM" header title are all visible |
| 6. CSS variable theming via --sidebar-* tokens | Plan 01 + Plan 06 delivered; Plan 08's shell-nav test passively validates styled rendering (the sidebar offcanvas sheet in mobile mode requires the tokens to be wired for the trigger to find the right primitive) |
| 7. AppShell is a normal SDUI component | Plan 05 + Plan 06 + Plan 07 delivered; Plan 08 does not touch this — the test suite runs against the normal component pipeline, implicitly proving no special protocol powers are needed |
| 8. CRM runs inside AppShell with nav + one node-mutation demo | Plan 07 delivered nav; Plan 08 delivers the node-mutation demo (country-select field swap) and proves it end-to-end with 3 tests in node-patch-focus.spec.ts (focus preservation, field swap, toast lifecycle) |

| Phase 12 Must-Have | Closed by |
|---|---|
| PatchMessage tagged enum + surface field exist end-to-end | `grep -q 'enum PatchOperation' backend/crates/marionette-protocol/src/data.rs` + `protocol-conformance.spec.ts` live-wire validation |
| Frontend applies node patches reactively; focused inputs retain focus | `node-patch-focus.spec.ts` D-A6 canonical proof: Name input retains focus + cursor + value across a sibling patch batch |
| HelloMessage reports protocol version 1.1.0 | `expect(helloFrame.data.version).toBe('1.1.0')` in protocol-conformance + integration specs |
| CONCEPT.md's "easy to patch by node ID" claim matches the implemented protocol | Plan 03 updated CONCEPT.md; Plan 08's demo is the worked example proving the claim |
| AppShell renders with collapsible sidebar, header, footer, main via shadcn | `shell-nav.spec.ts` asserts the landmarks + footer literal + header title, and the mobile Sidebar.Trigger |
| AppShell is a normal SDUI component | Nothing in Plan 08 treats it specially — the tests go through the same NavItem → sendAction → handle_navigate → Render pipeline as any other screen |
| CRM app runs inside AppShell with working nav between screens | `shell-nav.spec.ts` clicks Contacts → Companies and verifies the content sub-surface swaps while the shell persists |
| Contact form country-select triggers node-patch flow swapping siblings with preserved focus | `node-patch-focus.spec.ts` 3 tests — focus preservation + CH→US swap + D-B15 toast lifecycle |
| Protocol conformance E2E validates live wire messages against schemas | `protocol-conformance.spec.ts:patch message with node tree ops` |
| Toast lifecycle D-B15 demonstrated end-to-end | `node-patch-focus.spec.ts:D-B15 toast lifecycle` asserts insert-child then delete-node round-trip on the `toasts` sub-surface |

## Performance

- **Duration:** ~70 min wall-clock (3 tasks + iterative E2E debugging)
- **Completed:** 2026-04-10
- **Tasks:** 3 (all autonomous; no checkpoints)
- **Files modified:** 11

## Accomplishments

### Task 1 — Backend country-change demo + SelectInput change dispatch

- Added `handle_contact_country_change` to `backend/crates/crm-demo/src/handlers/contact.rs`. The handler emits two atomic `PatchMessage`s per invocation:
  - **Content surface**: `Set /contactForm/country` → `RemoveChild + DeleteNode` (idempotent cleanup of `contact-ch-canton`, `contact-us-state`, `contact-de-bundesland`) → `SetNode + InsertChild` (Canton Select for CH, State TextInput for US, Bundesland TextInput for DE). Index 6 places the new field after the country Select in the contact-form children order.
  - **Toasts surface**: `RemoveChild + DeleteNode` (idempotent cleanup of `toast-country-change`) → `SetNode + InsertChild` of a dismissable Button toast whose label is "Country set to {Switzerland|United States|Germany|none}". The toast's click action is `dismiss_toast`.
- Added `handle_dismiss_toast` — emits `RemoveChild + DeleteNode` on the toasts sub-surface to close the D-B15 lifecycle.
- Added the Country `Select` field to `handle_contact_form` with `id("contact-form-country")`, `bind("/contactForm/country")`, and `action(ComponentAction::change("contact_country_change"))`. Four options: `""`/`Select...`, `CH`/`Switzerland`, `US`/`United States`, `DE`/`Germany`. The `contactForm.country` field was added to the initial form data for both create and edit modes so the bind resolves cleanly on first mount.
- Registered both new actions in `main.rs` `action_router` chain under `AuthRequirement::Authenticated`.
- Extended `handle_navigate` to emit a third `Render` message seeding the `toasts` sub-surface with an empty `toasts-root` Container. This is the minimum sub-surface initialization needed before any `InsertChild` op can reference `toasts-root` as a parent. Order: shell → content → toasts.
- **Rule 2 deviation — SelectInput change dispatch**: `frontend/src/lib/components/form/SelectInput.svelte` did not call `sendAction` on value change. Without this dispatch, `contact_country_change` would never fire and the entire Plan 08 demo would be a dead wire. Added `sendAction` inside `handleValueChange` mirroring Button's pattern: full surface data spread with `action.payload` overrides. The change is gated on `action?.type === 'change' && action.name` so pre-Plan-08 SelectInput usages (which have no action) are unaffected.

### Task 2 — node-patch-focus E2E (3 tests, all passing)

- **Canonical D-A6 focus-preservation proof**: focus Name field, type "Hello", set cursor to index 3, dispatch `contact_country_change` via `window.__mrnSendAction` (the new test hook — see below), wait for the Canton field to appear via SetNode + InsertChild, assert the Name input is still `document.activeElement` with `selectionStart === 3` and `value === 'Hello'`. The test explicitly avoids clicking the shadcn Select trigger in the UI because doing so naturally moves keyboard focus to the trigger button — native browser behavior unrelated to node patching and irrelevant to D-A6's contract.
- **CH → US swap test**: programmatically swap Country from Switzerland to United States, assert Canton disappears (`toHaveCount(0)`) and State appears. Proves the RemoveChild + DeleteNode teardown loop is correct and the different-country-same-index insertion works.
- **D-B15 toast lifecycle test**: trigger a country change, assert the "Country set to Switzerland" toast Button is visible (insert-child + set-node on `toasts` surface), click it, assert it disappears (delete-node + remove-child on `toasts` surface). This is the only test in the phase that exercises node patching on a sub-surface other than `content`.
- **`__mrnSendAction` test hook**: `frontend/src/lib/init.ts` now exposes `window.__mrnSendAction` referring to the dispatcher's `sendAction`. This is a narrow, intentional test-only surface — any JS attacker with code execution already has full WebSocket access, so no new privilege is introduced.

### Task 3 — shell-nav + protocol-conformance E2E + integration.spec.ts rewrite + Rule 1 fix

- **shell-nav.spec.ts**:
  1. Login → AppShell renders → assert `<header>` + `<footer>` landmarks, "Marionette CRM" header title, "Marionette v1.1 · Protocol 1.1.0" footer version literal, "© 2026 Marionette" legal text, and the 5 NavItem buttons (Home, Contacts, Companies, Users, Audit Log). Click Companies → assert "Company Management" content + shell persistence. Click back to Contacts → assert "Contact Management" content.
  2. Narrow viewport to 375×700 → assert `button[data-sidebar="trigger"]` is visible. This is the shadcn Sidebar's mobile sheet trigger.
- **protocol-conformance.spec.ts**:
  - Hello test now filters on `direction === 'received'` to avoid picking up the client's outgoing hello frame, and asserts `version === '1.1.0'`.
  - New test `patch message with node tree ops conforms to schema` drives the country-change flow via `__mrnSendAction`, polls for a content-surface patch frame containing at least one non-`set` op, validates it against `spec/schemas/message.yaml#/PatchMessage`, then sanity-checks that across all captured patch frames the 5 node-op variants plus `set` all appear at least once. Also validates a toasts-surface PatchMessage.
  - Existing render / action / hello / (old patch) tests retained; the old patch test was replaced in place.
- **integration.spec.ts rewritten**: the original file asserted on `Welcome to Marionette` and `Click Me` strings from a pre-CRM demo that no longer exists, and hard-coded hello version `1.0.0`. The file now asserts against the current crm-demo: server hello version 1.1.0, login form on main surface, post-login content sub-surface Render with Contact Management. The existing health + SPA fallback tests are preserved unchanged.
- **Rule 1 drift fix**: `frontend/src/lib/transport/websocket.svelte.ts` sent `version: '1.0.0'` in its outgoing hello frame, drifting from the server's `1.1.0` value set in Plan 02. Bumped to `1.1.0` and updated the one unit test (`websocket.svelte.test.ts`) that asserted the old value.

## Task Commits

1. **Task 1 — Country-select demo + dismiss_toast handler + SelectInput change dispatch** — `edb1f88` (feat)
2. **Task 2 — node-patch-focus E2E + __mrnSendAction test hook** — `0d15675` (test)
3. **Task 3 — shell-nav + protocol-conformance E2E + integration.spec.ts rewrite + hello version bump** — `be50c4c` (test)

## Files Created/Modified

### Backend (Task 1)
- `backend/crates/crm-demo/src/handlers/contact.rs` — `+~220 lines`. Country Select added to `handle_contact_form`; `contactForm.country` added to initial form data (create + edit); `handle_contact_country_change` and `handle_dismiss_toast` appended at the end of the file.
- `backend/crates/crm-demo/src/main.rs` — registered `contact_country_change` and `dismiss_toast` actions; seeded toasts sub-surface with empty `toasts-root` Container in `handle_navigate` (third Render).

### Frontend (Tasks 1-3)
- `frontend/src/lib/components/form/SelectInput.svelte` — dispatches `sendAction` on value change when `action?.type === 'change'`; imports `getAllData` + `sendAction`.
- `frontend/src/lib/init.ts` — exposes `window.__mrnSendAction` test hook after `connect(wsUrl, handleMessage)`.
- `frontend/src/lib/transport/websocket.svelte.ts` — client hello version bumped to `1.1.0`.
- `frontend/src/lib/transport/websocket.svelte.test.ts` — assertion updated to `1.1.0`.

### E2E tests (Tasks 2-3)
- `frontend/tests/e2e/node-patch-focus.spec.ts` — 3 tests replacing the 3-line scaffold.
- `frontend/tests/e2e/shell-nav.spec.ts` — 2 tests replacing the 3-line scaffold.
- `frontend/tests/e2e/protocol-conformance.spec.ts` — hello test updated to filter received direction + version 1.1.0 check; new node-tree-ops patch test added.
- `frontend/tests/e2e/integration.spec.ts` — rewritten against the current crm-demo.

### Planning (housekeeping)
- `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` — logged TextInput `input_type` bug + stale integration.spec.ts note.

## Country-specific demo field IDs

Fixed, canonical IDs for the three country-specific fields the demo cycles through:

| Country code | Field id | Label | Bind path |
|---|---|---|---|
| `CH` | `contact-ch-canton` | Canton (Select) | `/contactForm/canton` |
| `US` | `contact-us-state` | State (TextInput) | `/contactForm/usState` |
| `DE` | `contact-de-bundesland` | Bundesland (TextInput) | `/contactForm/bundesland` |

The teardown loop in `handle_contact_country_change` unconditionally emits `RemoveChild + DeleteNode` for all three IDs on every call, then inserts the new one. Deleting a non-existent node is a no-op in the frontend store (D-A8 GC tolerance).

## Test totals

### Before this plan
- Backend `cargo test --workspace`: 81 passed
- Frontend unit `vitest --run`: 58 passed
- Frontend browser `vitest --config vitest-browser.config.ts --run`: 73 passed, 5 pre-existing popup failures
- Frontend E2E `npx playwright test --config playwright.e2e.config.ts`: mostly green but 3 stale `integration.spec.ts` failures (Welcome to Marionette / Click Me / 1.0.0 hello) and 6 unfilled scaffolds in the 3 Plan 08 target spec files

### After this plan
- Backend `cargo test --workspace`: **81 passed, 0 failed** (no new tests, no regressions)
- Frontend unit `vitest --run`: **58 passed, 0 failed**
- Frontend browser `vitest --config vitest-browser.config.ts --run`: **73 passed, 5 pre-existing popup failures** (ConfirmDialog 4 + ToastSurface 1, documented in deferred-items.md, unchanged from Plan 12-06 baseline)
- Frontend E2E `npx playwright test --config playwright.e2e.config.ts`: **15 passed, 0 failed** — full suite green

E2E test breakdown:
| Spec | Tests | Status |
|---|---|---|
| `integration.spec.ts` | 5 | all passing (rewritten) |
| `node-patch-focus.spec.ts` | 3 | all passing (new) |
| `shell-nav.spec.ts` | 2 | all passing (new) |
| `protocol-conformance.spec.ts` | 4 | all passing (3 retained + 1 rewritten) |
| `smoke.spec.ts` | 1 | passing (unchanged) |

## ajv / schema-validator configuration

**No changes required.** `frontend/tests/helpers/schema-validator.ts` already handles the new node-op oneOf in `spec/schemas/data.yaml` — Plan 12-03 wrote it to merge all `SCHEMA_FILES` into a single `$defs` map and rewrite cross-file refs, which transparently covers the new `PatchOperationSetNode` / `PatchOperationInsertChild` / etc. definitions without any discriminator-specific plumbing. The validator's `strict: false` + `allErrors: true` ajv options accept `oneOf` with an OpenAPI `discriminator: { propertyName, mapping }` block as a normal `oneOf` for validation purposes.

Plan 08 Task 3 uses `validator.validatePatch(patchMsg.data)` and `validator.validateHello(helloFrame.data)` directly — no inline schema compilation, no ajv changes needed.

## Rule 1/2 deviations auto-applied

**1. [Rule 2 — Missing critical functionality] SelectInput did not dispatch change actions**
- **Found during:** Task 1, while verifying the demo wire.
- **Issue:** `handleValueChange` in `SelectInput.svelte` only called `setData` locally; it never invoked `sendAction`, so the `change` action attached by the backend builder was a dead wire. This was a pre-existing gap in the form-component suite — all other interactive components (Button, TextInput) dispatch actions, SelectInput was the exception.
- **Fix:** Added a `sendAction` call inside `handleValueChange` gated on `action?.type === 'change' && action.name`, mirroring Button's pattern: full surface data spread with `action.payload` overrides.
- **Files modified:** `frontend/src/lib/components/form/SelectInput.svelte`
- **Committed in:** `edb1f88`

**2. [Rule 1 — Bug] Client hello drift from server version**
- **Found during:** Task 3, while bumping the E2E hello version assertion.
- **Issue:** `websocket.svelte.ts` sent `version: '1.0.0'` in its outgoing hello frame while the server's `ws.rs` returned `1.1.0` after Plan 12-02. Client-server protocol versions must match; this is a Rule 1 bug.
- **Fix:** Bumped client hello to `1.1.0` and updated the one unit test.
- **Files modified:** `frontend/src/lib/transport/websocket.svelte.ts`, `frontend/src/lib/transport/websocket.svelte.test.ts`
- **Committed in:** `be50c4c`

**3. [Rule 2 — Missing critical functionality] integration.spec.ts stale pre-CRM assertions**
- **Found during:** Task 3, running the full E2E suite after adding the new specs.
- **Issue:** The file asserted `getByText('Welcome to Marionette')`, `getByText('Click Me')`, and `version === '1.0.0'` — all of which came from the pre-CRM demo app that no longer exists. These tests had been silently failing the full E2E suite for some time.
- **Fix:** Rewrote the 3 failing tests against the current crm-demo backend. The health + SPA-fallback tests were preserved unchanged.
- **Files modified:** `frontend/tests/e2e/integration.spec.ts`
- **Committed in:** `be50c4c`

## Decisions Made

See `key-decisions` in the frontmatter. The two notable judgment calls were:

1. **Toast node is a Button, not a Heading** — the plan instructed to use a Heading for the toast, but the frontend Heading component ignores `action`, so clicking it would never fire `dismiss_toast`. Using Button respects the existing component contract without touching the AppShell mount tree.
2. **D-A6 focus proof uses `__mrnSendAction`, not UI clicks** — see the detailed explanation in the accomplishments section. The alternative (clicking the Select trigger) would fail on native browser semantics, not on a D-A6 regression.

## Verification

### `"1.0.0"` hunt in backend crates
```bash
$ grep -rn '"1.0.0"' backend/crates/ --include='*.rs'
(no output — all 1.0.0 literals removed; client hello in websocket.svelte.ts also bumped)
```

### `surface: "main"` in CRM handlers — expected false positives only
```bash
$ grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/handlers/
backend/crates/crm-demo/src/handlers/audit.rs:236:        surface: "main".into(),
backend/crates/crm-demo/src/handlers/contact.rs:969:        surface: "main".into(),
backend/crates/crm-demo/src/handlers/interaction.rs:177:        surface: "main".into(),
backend/crates/crm-demo/src/handlers/company.rs:406:        surface: "main".into(),
backend/crates/crm-demo/src/handlers/user.rs:323:        surface: "main".into(),
```
All five hits are inside the `nav_active_patch` helpers (one per handler file), which correctly target the `main` surface to update `/nav/active/*` paths per D-B13. No Render message targets `main` for screen content after Plan 07.

### Clippy in-scope crates
```bash
$ cd backend && cargo clippy -p marionette-protocol -p marionette -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.47s
(no output — clean)
```

### Clippy crm-demo — baseline unchanged
```bash
$ cd backend && cargo clippy -p crm-demo 2>&1 | grep -c '^warning:'
77
```
77 warnings, matching the pre-Plan-08 baseline logged in `deferred-items.md` from Plan 12-02. Plan 08 introduced no new clippy warnings; one initial `doc_markdown` warning on my `handle_dismiss_toast` doc comment was fixed in place before committing Task 1.

## Deferred Items

Logged in `deferred-items.md`:

1. **TextInput `input_type` → `type` prop mismatch** — pre-existing bug discovered while writing Task 2's E2E test. The login password field has never rendered as `type="password"` since the backend builder serializes `input_type` while the frontend reads `props.type`. Plan 08 works around it by locating the password input via its grid wrapper. Recommended resolution: one-line fix in Phase 13 (change `TextInput.svelte` to read `props.input_type ?? props.type ?? 'text'`).
2. **Stale `integration.spec.ts` / `protocol-conformance.spec.ts` assertions** — logged in deferred-items.md for reference; the actual fixes landed in Plan 08 Task 3 (`be50c4c`).

## Known Stubs

None. All code paths are wired end-to-end:
- Country Select has 4 real options plus the placeholder, all backed by the backend handler.
- `contact_country_change` emits real PatchMessages targeting two real sub-surfaces with real ops.
- `dismiss_toast` emits a real delete-node patch.
- E2E tests drive real WebSocket sessions against the real backend; no mocked transport.

The one intentional deferral from Plan 12-07 (header user name showing `User: {integer_id}` rather than a display name) remains, tracked for Phase 15.

## Threat Flags

None. No new security-relevant surface beyond what the plan's `<threat_model>` already covered:
- T-12-19 (fabricated country value) — mitigated by the `match country.as_str()` pattern; unknown values emit no InsertChild ops, producing a no-op.
- T-12-20 (rapid country changes) — accepted per plan; shadcn Select debounces changes naturally, and each patch is ≤6 ops.
- T-12-21 (test logs leak PII) — accepted per plan; only seeded admin credentials touch the E2E path, no real user data.

The new `__mrnSendAction` test hook is not a threat surface increase: any JS attacker with code execution already has direct WebSocket access via the `WebSocket` constructor, so the hook adds no new privilege.

## Next Phase Readiness

Phase 12 is **complete**. All 8 success criteria and 7 must-haves are closed. Wave 5 of Phase 12 ships.

- **Phase 13 (form screens)** can start. Inheritable patterns from Plan 08:
  - `ComponentAction::change` + SelectInput dispatch is the canonical pattern for select-driven node patches.
  - The `__mrnSendAction` E2E hook is the canonical pattern for focus-isolated E2E tests.
  - The grid-wrapper input locator is the canonical pattern for locating bits-ui-backed inputs without label/input association.
  - The `contactForm.country` → `handle_contact_country_change` → two-PatchMessage-response pattern is the reference implementation for any future node-patch-driven form UX.
- **Phase 15 (CRM cleanup)** can start. Still-open items:
  - TextInput `input_type` bug (deferred-items.md).
  - 77 crm-demo clippy pedantic warnings (deferred-items.md, Plan 12-02 baseline).
  - Header user name display (Plan 12-07 known stub).
  - 5 popup browser-test failures (deferred-items.md, Plan 12-06 baseline).

No blockers for downstream plans.

## Self-Check: PASSED

Files verified on disk:
- `backend/crates/crm-demo/src/handlers/contact.rs` — FOUND (modified, `handle_contact_country_change` + `handle_dismiss_toast` present)
- `backend/crates/crm-demo/src/main.rs` — FOUND (modified, both actions registered, toasts sub-surface seeded)
- `frontend/src/lib/components/form/SelectInput.svelte` — FOUND (modified, sendAction present)
- `frontend/src/lib/init.ts` — FOUND (modified, `__mrnSendAction` hook present)
- `frontend/src/lib/transport/websocket.svelte.ts` — FOUND (modified, `1.1.0`)
- `frontend/src/lib/transport/websocket.svelte.test.ts` — FOUND (modified, `1.1.0`)
- `frontend/tests/e2e/node-patch-focus.spec.ts` — FOUND (3 tests)
- `frontend/tests/e2e/shell-nav.spec.ts` — FOUND (2 tests)
- `frontend/tests/e2e/protocol-conformance.spec.ts` — FOUND (4 tests)
- `frontend/tests/e2e/integration.spec.ts` — FOUND (rewritten)
- `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` — FOUND (appended)
- `.planning/phases/12-protocol-node-patching-appshell/12-08-demo-and-e2e-SUMMARY.md` — FOUND (this file)

Commits verified in git log:
- `edb1f88` — FOUND (Task 1 feat)
- `0d15675` — FOUND (Task 2 test)
- `be50c4c` — FOUND (Task 3 test)

Acceptance criteria spot-check:
- `grep -q 'id("contact-form-country")' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'handle_contact_country_change' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'handle_dismiss_toast' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q '"contact_country_change"' backend/crates/crm-demo/src/main.rs` — OK
- `grep -q '"dismiss_toast"' backend/crates/crm-demo/src/main.rs` — OK
- `grep -q 'PatchOperation::InsertChild' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'PatchOperation::SetNode' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'PatchOperation::RemoveChild' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'PatchOperation::DeleteNode' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'surface: "toasts"' backend/crates/crm-demo/src/handlers/contact.rs` — OK
- `grep -q 'toasts-root' backend/crates/crm-demo/src/main.rs` — OK
- `grep -q 'surface: "toasts"' backend/crates/crm-demo/src/main.rs` — OK
- `grep -q 'test.skip' frontend/tests/e2e/node-patch-focus.spec.ts` — returns no match (scaffold replaced)
- `grep -q 'test.skip' frontend/tests/e2e/shell-nav.spec.ts` — returns no match (scaffold replaced)
- `grep -q 'selectionStart' frontend/tests/e2e/node-patch-focus.spec.ts` — OK
- `grep -q "Country set to Switzerland" frontend/tests/e2e/node-patch-focus.spec.ts` — OK
- `grep -q "toHaveCount" frontend/tests/e2e/node-patch-focus.spec.ts` — OK (the toast dismiss assertion)
- `grep -q 'Marionette v1.1 · Protocol 1.1.0' frontend/tests/e2e/shell-nav.spec.ts` — OK
- `grep -q 'data-sidebar="trigger"' frontend/tests/e2e/shell-nav.spec.ts` — OK
- `grep -n '"1.0.0"' frontend/tests/e2e/protocol-conformance.spec.ts` — returns zero lines
- `grep -rn '"1.0.0"' backend/crates/ --include='*.rs'` — returns zero lines
- `cargo build -p crm-demo` — exits 0
- `cargo test --workspace` — 81 passed
- `npx playwright test --config playwright.e2e.config.ts` — 15 passed

Phase closing one-liner for PROGRESS.md:
> **Phase 12 complete: protocol node-patching landed on version 1.1.0, AppShell shipped as a first-class SDUI component with working CRM navigation, and the country-select demo proves node-level tree mutation end-to-end with focus preservation and the D-B15 toast lifecycle.**

---
*Phase: 12-protocol-node-patching-appshell*
*Completed: 2026-04-10*
