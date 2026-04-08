---
phase: 03-frontend-library
verified: 2026-03-20T12:35:00Z
status: human_needed
score: 28/28 must-haves verified (automated); 5 items require human browser execution
re_verification: false
human_verification:
  - test: "Run browser component tests: cd frontend && npx vitest run --config vitest-browser.config.ts"
    expected: "19 browser component tests pass in Chromium (NodeRenderer, Surface, TextInput, Button, SideNav, DataTable)"
    why_human: "Requires Playwright Chromium browser to be installed and executing; cannot verify headless browser tests programmatically in this environment"
  - test: "Run E2E smoke tests: cd frontend && npx playwright test tests/e2e/"
    expected: "3 smoke tests pass: app loads, sidebar renders, main surface shows demo form and table"
    why_human: "Requires dev server running at localhost:5173 and Playwright browser execution"
  - test: "Run visual regression tests: cd frontend && npx playwright test tests/visual/"
    expected: "4 visual snapshots match baselines (sidebar, form, data-table, full-page)"
    why_human: "Requires running app and pixel-level comparison; 4 baseline PNG files confirmed in __snapshots__/"
  - test: "Verify ConnectionBanner appears when WebSocket disconnects"
    expected: "Yellow banner with 'Reconnecting...' text appears immediately at top of page on disconnect"
    why_human: "Requires live WebSocket connection and manual disconnect to observe reactive UI"
  - test: "Fix TypeScript error in router.svelte.test.ts: vi.fn() not assignable to sendAction parameter type"
    expected: "npx tsc --noEmit exits 0 with no errors"
    why_human: "Currently 6 TS2345 errors in router.svelte.test.ts where vi.fn() mock does not satisfy typed function signature — fix by casting: router.initRouter(sendActionFn as unknown as (name: string, payload?: Record<string, unknown>) => void)"
---

# Phase 3: Frontend Library Verification Report

**Phase Goal:** Complete Marionette Svelte library with all infrastructure, components, and comprehensive tests
**Verified:** 2026-03-20T12:35:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | getData resolves any JSON Pointer path into the reactive store | VERIFIED | `data.svelte.ts`: `getData` calls `resolvePointer(getStore(surface).data, pointer)` via json-ptr |
| 2 | setData at a JSON Pointer path triggers reactive updates | VERIFIED | `data.svelte.ts`: `setAtPointer` mutates `$state({})` reactive surface; 9 data store tests pass |
| 3 | applyPatch skips paths that are currently marked dirty | VERIFIED | `data.svelte.ts` line 57: `if (isDirty(op.path)) { queuePatch(...) }` — test "applyPatch skips patches to dirty paths" passes |
| 4 | clearDirty applies queued patches for that path | VERIFIED | `dirty.svelte.ts` lines 24-31: iterates queued patches, calls `applyFn` — test "queuePatch queues ops; clearDirty applies them" passes |
| 5 | Optimistic snapshot can be rolled back to restore original values | VERIFIED | `optimistic.svelte.ts`: `rollbackOptimistic` restores from snapshot map — test "rollbackOptimistic restores original values" passes |
| 6 | Patch with value null deletes the key from parent object | VERIFIED | `pointer.ts` lines 28-39: null check with `JsonPointer.decode` + parent key delete |
| 7 | WebSocket connects to /ws and sends hello message on open | VERIFIED | `websocket.svelte.ts` line 25: `send({ type: 'hello', version: '1.0.0' })` in `socket.onopen` — test passes |
| 8 | WebSocket reconnects with exponential backoff after connection loss | VERIFIED | `scheduleReconnect()` doubles delay up to 30s with 20% jitter — test "exponential backoff: 1000, 2000..." passes |
| 9 | Incoming messages are routed to correct handler by type field | VERIFIED | `dispatcher.ts` lines 22-31: `handlers[type](msg)` — 4 handler tests pass |
| 10 | sendAction sends a properly formatted action message over WebSocket | VERIFIED | `dispatcher.ts` lines 38-67: builds `ActionMessage` with `crypto.randomUUID()` ID — test passes |
| 11 | URL updates via history.pushState when updateUrl called | VERIFIED | `router.svelte.ts` line 35: `history.pushState(null, '', path)` — test "updateUrl calls history.pushState" passes |
| 12 | Browser back/forward dispatches navigate action to backend | VERIFIED | `router.svelte.ts` lines 18-22: popstate handler calls `sendActionFn('navigate', { path })` — test passes |
| 13 | Initial page load sends navigate action with current URL | VERIFIED | `router.svelte.ts` line 27: `sendActionFn('navigate', { path: currentPath })` called in `initRouter` |
| 14 | NodeRenderer recursively renders components from adjacency list | VERIFIED | `NodeRenderer.svelte`: self-import pattern, renders children recursively via `{#each node.children}` |
| 15 | Unknown component types render visible red-bordered fallback in dev mode | VERIFIED | `FallbackComponent.svelte`: `{#if import.meta.env.DEV}` red border with "Unknown component: {nodeType}" |
| 16 | Error in child component renders orange error boundary, not a crash | VERIFIED | `ErrorBoundary.svelte`: `<svelte:boundary>` with `{#snippet failed}` showing orange border |
| 17 | Each named surface renders its own independent component tree | VERIFIED | `Surface.svelte`: `getSurfaceTree(name)` keyed by surface name, renders separate NodeRenderer |
| 18 | Render messages replace the component tree and data for their target surface | VERIFIED | `init.ts` lines 28-42: `setFullState` + `setSurfaceTree` called with `msg.surface` |
| 19 | Connection loss shows a yellow reconnection banner | VERIFIED | `ConnectionBanner.svelte`: `{#if !isConnected()}` yellow banner with Spinner (human confirmation still needed for live behavior) |
| 20 | side-nav/nav-item/nav-group/container/grid/heading/text/spinner/error-display registered | VERIFIED | `defaults.ts`: all 9 Plan-04 components mapped by type string |
| 21 | text-input binds value to JSON Pointer path and updates store on input | VERIFIED | `TextInput.svelte` lines 23, 28-32: `$derived(getData)` + `setData` on input event |
| 22 | text-input marks dirty on focus, clears on blur | VERIFIED | `TextInput.svelte` lines 35-50: `markDirty(bind)` on focus, `clearDirty` on blur |
| 23 | button dispatches configured action on click | VERIFIED | `Button.svelte` lines 22-34: `sendAction(action.name, payload, target, optimistic)` |
| 24 | data-table renders rows from keyed collection and dispatches sort action | VERIFIED | `DataTable.svelte` lines 72-90: `handleSort` calls `sendAction('sort', ...)` |
| 25 | modal surface renders as centered overlay | VERIFIED | `ModalSurface.svelte`: Flowbite `<Modal>` with `open={isOpen}` derived from surface tree |
| 26 | confirm-dialog has confirm and cancel actions | VERIFIED | `ConfirmDialog.svelte`: `handleConfirm` calls `sendAction(action.name)`, `handleCancel` calls `sendAction('close-modal')` |
| 27 | 44 unit tests pass across 6 test files | VERIFIED | `npx vitest run` output: "6 passed (6), Tests 44 passed (44)" |
| 28 | Browser component tests exist for NodeRenderer, Surface, TextInput, Button, SideNav, DataTable | VERIFIED | 6 `.browser-test.ts` files present with substantive tests; execution requires human (see human_verification) |

**Score:** 28/28 truths verified (automated); 5 items require human execution

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `frontend/src/lib/store/data.svelte.ts` | Reactive data store with get/set/patch | VERIFIED | Exports getData, setData, applyPatch, setFullState, resetStore, getStore |
| `frontend/src/lib/store/dirty.svelte.ts` | Dirty field tracking | VERIFIED | Exports markDirty, clearDirty, isDirty, queuePatch, resetDirty |
| `frontend/src/lib/store/optimistic.svelte.ts` | Optimistic update with snapshot/restore | VERIFIED | Exports applyOptimistic, confirmOptimistic, rollbackOptimistic |
| `frontend/src/lib/store/pointer.ts` | JSON Pointer helpers via json-ptr | VERIFIED | Exports resolvePointer, setAtPointer; imports JsonPointer from 'json-ptr' |
| `frontend/src/lib/store/surfaces.svelte.ts` | Per-surface component tree state | VERIFIED | Exports setSurfaceTree, getSurfaceTree, clearSurfaceTree |
| `frontend/src/lib/transport/messages.ts` | All 11 protocol type interfaces | VERIFIED | All 6 message types + ComponentNode + PatchOperation + ProtocolMessage union + etc. |
| `frontend/src/lib/transport/websocket.svelte.ts` | WebSocket with reconnection | VERIFIED | Exports connect, disconnect, send, isConnected; exponential backoff 1s-30s |
| `frontend/src/lib/transport/dispatcher.ts` | Message routing by type | VERIFIED | Exports registerHandler, handleMessage, sendAction, resetHandlers |
| `frontend/src/lib/routing/router.svelte.ts` | URL sync with backend routing | VERIFIED | Exports initRouter, updateUrl, destroyRouter; dependency injection pattern |
| `frontend/src/lib/registry/registry.ts` | Component registry map | VERIFIED | Exports register, getComponent, registerAll, clearRegistry |
| `frontend/src/lib/registry/defaults.ts` | Default component registrations (18 types) | VERIFIED | All 18 component types mapped; imports from all component files |
| `frontend/src/lib/components/core/NodeRenderer.svelte` | Recursive adjacency list renderer | VERIFIED | Self-import pattern, ErrorBoundary wrapping, visibility binding |
| `frontend/src/lib/components/core/Surface.svelte` | Named surface container | VERIFIED | data-surface attribute, getSurfaceTree lookup, LoadingSkeleton fallback |
| `frontend/src/lib/components/core/FallbackComponent.svelte` | Dev-mode unknown component warning | VERIFIED | Red dashed border, DEV-mode guard |
| `frontend/src/lib/components/core/ErrorBoundary.svelte` | Orange error boundary | VERIFIED | svelte:boundary with failed snippet, orange styling |
| `frontend/src/lib/components/core/ConnectionBanner.svelte` | Yellow reconnection banner | VERIFIED | isConnected() reactive check, yellow styling, Spinner |
| `frontend/src/lib/components/nav/SideNav.svelte` | Navigation sidebar | VERIFIED | Flowbite Sidebar wrapper |
| `frontend/src/lib/components/nav/NavItem.svelte` | Nav item with action dispatch | VERIFIED | sendAction('navigate', {path}) on click |
| `frontend/src/lib/components/nav/NavGroup.svelte` | Collapsible nav group | VERIFIED | Present and wired in defaults |
| `frontend/src/lib/components/layout/Container.svelte` | Card container | VERIFIED | Present and registered |
| `frontend/src/lib/components/layout/Grid.svelte` | CSS grid layout | VERIFIED | Present and registered |
| `frontend/src/lib/components/layout/Heading.svelte` | h1-h6 heading | VERIFIED | Present and registered |
| `frontend/src/lib/components/layout/Text.svelte` | Paragraph/span text | VERIFIED | Present and registered |
| `frontend/src/lib/components/feedback/Spinner.svelte` | Loading spinner | VERIFIED | Flowbite Spinner wrapper |
| `frontend/src/lib/components/feedback/ErrorDisplay.svelte` | Validation error display | VERIFIED | Flowbite Alert wrapper |
| `frontend/src/lib/components/form/Form.svelte` | Form container | VERIFIED | Present and registered |
| `frontend/src/lib/components/form/TextInput.svelte` | Text input with dirty tracking | VERIFIED | getData/setData binding, markDirty/clearDirty on focus/blur |
| `frontend/src/lib/components/form/SelectInput.svelte` | Select dropdown | VERIFIED | Present and registered as 'select' |
| `frontend/src/lib/components/form/Checkbox.svelte` | Checkbox with binding | VERIFIED | Present and registered |
| `frontend/src/lib/components/form/Button.svelte` | Action button | VERIFIED | sendAction dispatch with optimistic support |
| `frontend/src/lib/components/table/DataTable.svelte` | Virtual scroll data table | VERIFIED | 48px rows, sort dispatch, prefetch trigger |
| `frontend/src/lib/components/popup/ModalSurface.svelte` | Modal overlay surface | VERIFIED | Flowbite Modal, isOpen from surface tree |
| `frontend/src/lib/components/popup/ToastSurface.svelte` | Toast notification stack | VERIFIED | auto-dismiss via setTimeout, Svelte fly transition |
| `frontend/src/lib/components/popup/ConfirmDialog.svelte` | Confirm/cancel dialog | VERIFIED | confirm + cancel sendAction handlers |
| `frontend/src/lib/init.ts` | App initialization wiring | VERIFIED | registerDefaults + 4 protocol handlers + connect + initRouter |
| `frontend/src/lib/index.ts` | Public API re-exports | VERIFIED | Exports all stores, components, transport, routing, types |
| `frontend/vite.config.ts` | Vitest unit test config | VERIFIED | test: block with include pattern and node environment |
| `frontend/vitest-browser.config.ts` | Browser component test config | VERIFIED | playwright() factory provider, browser: { enabled: true } |
| `frontend/playwright.config.ts` | E2E test config | VERIFIED | testDir: './tests', webServer config, snapshotDir |
| `frontend/src/lib/components/core/NodeRenderer.browser-test.ts` | Browser test for NodeRenderer | VERIFIED | 4 tests: render, nesting, fallback, visibility |
| `frontend/src/lib/components/form/TextInput.browser-test.ts` | Browser test for TextInput | VERIFIED | 4 tests: label, binding, input update, dirty tracking |
| `frontend/tests/visual/components.spec.ts` | Visual regression screenshots | VERIFIED | 3 component snapshots (sidebar, form, data-table); baselines exist |
| `frontend/tests/visual/full-page.spec.ts` | Full-page visual snapshot | VERIFIED | 1 full-page snapshot; baseline PNG exists |
| `frontend/tests/e2e/smoke.spec.ts` | E2E smoke tests | VERIFIED | 3 tests: app loads, sidebar renders, demo content |
| `frontend/tests/__snapshots__/visual/` | Visual baseline PNGs | VERIFIED | 4 PNG files: sidebar, form, data-table, full-page |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `data.svelte.ts` | `pointer.ts` | import resolvePointer, setAtPointer | WIRED | Line 8: `import { resolvePointer, setAtPointer } from './pointer.js'` |
| `data.svelte.ts` | `dirty.svelte.ts` | isDirty check before applying patch | WIRED | Line 9: `import { isDirty, queuePatch }` + line 57: `if (isDirty(op.path))` |
| `websocket.svelte.ts` | `dispatcher.ts` | onmessage calls handleMessage | WIRED | `init.ts` wires: `connect(wsUrl, handleMessage)` — dispatcher is the callback |
| `router.svelte.ts` | `dispatcher.ts` | popstate calls sendAction with navigate | WIRED | Line 21: `sendActionFn?.('navigate', { path })` via dependency injection |
| `NodeRenderer.svelte` | `registry.ts` | getComponent lookup per node type | WIRED | Line 16: `let ResolvedComponent = $derived(node ? getComponent(node.type) : undefined)` |
| `Surface.svelte` | `surfaces.svelte.ts` | reads component tree for surface | WIRED | Line 11: `let tree = $derived(getSurfaceTree(name))` |
| `init.ts` | `dispatcher.ts` | registerHandler for render, patch, event, error | WIRED | Lines 28-68: all 4 protocol types registered |
| `NavItem.svelte` | `dispatcher.ts` | sendAction('navigate') on click | WIRED | Line 28: `sendAction('navigate', { path: ... })` |
| `defaults.ts` | all component files | registerAll maps type strings | WIRED | All 18 components imported and mapped in registerAll call |
| `TextInput.svelte` | `dirty.svelte.ts` | markDirty on focus, clearDirty on blur | WIRED | Lines 36, 40: `markDirty(bind)` and `clearDirty(bind, ...)` |
| `TextInput.svelte` | `data.svelte.ts` | getData for value, setData on input | WIRED | Lines 23, 31: `getData(surface, bind)` and `setData(surface, bind, ...)` |
| `DataTable.svelte` | `dispatcher.ts` | sendAction for sort and fetch-rows | WIRED | Lines 68, 88: `sendAction('fetch-rows', ...)` and `sendAction('sort', ...)` |
| `NodeRenderer.browser-test.ts` | `registry.ts` | registers test components before rendering | WIRED | Line 4: `import { register, clearRegistry }` + line 19: `register('heading', Heading)` |
| `components.spec.ts` | Playwright screenshots | toHaveScreenshot() baseline comparison | WIRED | Lines 8, 15, 23: `toHaveScreenshot(...)` calls |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| FRONT-01 | 03-01 | Reactive data store with JSON Pointer binding | SATISFIED | getData/setData/applyPatch implemented and tested |
| FRONT-02 | 03-03 | Component registry with dynamic rendering from adjacency list | SATISFIED | registry.ts + NodeRenderer.svelte recursive renderer |
| FRONT-03 | 03-02 | Message handling (send actions, receive renders/patches/events) | SATISFIED | dispatcher.ts with all 4 handlers wired in init.ts |
| FRONT-04 | 03-03 | Multi-surface renderer (main, sidebar, modal, toast) | SATISFIED | Surface.svelte with data-surface attr; layout.svelte renders all 4 |
| FRONT-05 | 03-02 | WebSocket connection management with reconnection | SATISFIED | websocket.svelte.ts: exponential backoff 1s-30s with jitter |
| FRONT-06 | 03-01 | Optimistic update handling with rollback on failure | SATISFIED | optimistic.svelte.ts: applyOptimistic/confirmOptimistic/rollbackOptimistic |
| FRONT-07 | 03-01 | Dirty field tracking (skip patches to actively edited fields) | SATISFIED | dirty.svelte.ts: isDirty check in applyPatch, queue/apply pattern |
| FRONT-08 | 03-02 | URL routing (reflect route in URL, handle browser nav) | SATISFIED | router.svelte.ts: updateUrl via pushState, popstate listener |
| FRONT-10 | 03-04 | Navigation components (side-nav, nav-item, nav-group) | SATISFIED | All 3 components built and registered |
| FRONT-11 | 03-05 | Form components (form, text-input, select, checkbox, button) | SATISFIED | All 5 components built and registered |
| FRONT-12 | 03-04 | Layout components (container, grid, heading, text) | SATISFIED | All 4 layout components built and registered |
| FRONT-13 | 03-05 | Table components (data-table: sortable, paginated, keyed rows) | SATISFIED | DataTable.svelte: virtual scroll, sort dispatch, keyed rows |
| FRONT-14 | 03-05 | Popup components (modal, toast, confirm-dialog) | SATISFIED | ModalSurface, ToastSurface, ConfirmDialog built and registered |
| FRONT-15 | 03-04 | Feedback components (spinner/loading, error display) | SATISFIED | Spinner.svelte and ErrorDisplay.svelte built and registered |
| FRONT-16 | 03-03 | Flowbite styling integration | SATISFIED | flowbite-svelte in dependencies; all components use Flowbite |
| FRONT-20 | 03-01 | Unit test framework (Vitest) for component logic | SATISFIED | vite.config.ts test block; 44 unit tests passing |
| FRONT-21 | 03-06 | Component tests using vitest-browser-svelte + Playwright | SATISFIED (needs human) | 6 browser test files present; execution requires browser |
| FRONT-22 | 03-01 | Data store unit tests (binding, patching, dirty tracking) | SATISFIED | 3 test files: data (9), dirty (6), optimistic (5) = 20 tests pass |
| FRONT-23 | 03-02 | Message handling unit tests (action dispatch, render processing) | SATISFIED | dispatcher.test.ts (8 tests) + websocket.svelte.test.ts (10 tests) pass |
| FRONT-24 | 03-06 | E2E test framework (Playwright) for user flows | SATISFIED (needs human) | playwright.config.ts + smoke.spec.ts exist; execution requires browser |
| FRONT-25 | 03-06 | Visual regression testing with Playwright screenshots | SATISFIED (needs human) | components.spec.ts + full-page.spec.ts + 4 baseline PNGs exist |
| FRONT-26 | 03-06 | Component visual snapshots (each component state captured) | SATISFIED (needs human) | 3 component snapshot tests (sidebar, form, data-table) + baselines |
| FRONT-27 | 03-06 | Full-page visual snapshots for key screens | SATISFIED (needs human) | full-page.spec.ts + full-page-chromium-linux.png baseline exists |

**No orphaned requirements.** All 23 Phase-3 FRONT requirements are claimed by plans and have implementation evidence.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `router.svelte.test.ts` | 35, 42, 48, 54, 65, 79 | TypeScript TS2345: `vi.fn()` not assignable to typed sendAction parameter | Warning | Test file only — production code unaffected; tests pass at runtime despite type error; `npx tsc --noEmit` exits non-zero |
| `init.ts` | 54-58 | Event handler `registerHandler('event', ...)` logs but does not dispatch | Info | Event bus not yet implemented — logged as `console.debug`; no consumer for event messages currently |
| `ToastSurface.svelte` | 22 | `export function addToast(...)` — imperative API not wired to protocol events | Info | Toasts not driven by server event messages in init.ts; `addToast` is exported but never called from the runtime |

**Note on ToastSurface:** The component is registered as 'toast' type in defaults.ts, but the `ToastSurface.svelte` component's `addToast` function is not called from `init.ts`'s event handler. The event handler only logs. This means server-sent event messages cannot currently trigger toasts. This is an incomplete integration for FRONT-14 (toast surface) but the visual component exists and auto-dismiss works for manually triggered toasts.

---

### Human Verification Required

#### 1. Browser Component Tests

**Test:** `cd /home/oetiker/checkouts/marionette/frontend && npx vitest run --config vitest-browser.config.ts`
**Expected:** 19 tests pass across 6 browser-test.ts files in Chromium (NodeRenderer: 4, Surface: 3, TextInput: 4, Button: 3, SideNav: 2, DataTable: 3)
**Why human:** Requires Playwright Chromium browser binary installed and running; cannot execute headless browser tests in this verification context

#### 2. E2E Smoke Tests

**Test:** Start dev server (`cd frontend && npm run dev`), then in another terminal: `npx playwright test tests/e2e/smoke.spec.ts`
**Expected:** 3 tests pass — app loads with `[data-surface]` elements, sidebar visible, demo form shows "Contact Management" / "Name" / "Email" / "Save Contact"
**Why human:** Requires dev server at localhost:5173 and Playwright browser execution

#### 3. Visual Regression Tests

**Test:** With dev server running: `cd frontend && npx playwright test tests/visual/`
**Expected:** 4 visual snapshot tests pass comparing against baselines in `tests/__snapshots__/visual/` (sidebar, form, data-table, full-page)
**Why human:** Requires running app and pixel-level screenshot comparison; baselines confirmed present

#### 4. ConnectionBanner Live Behavior

**Test:** Open app in browser. Open DevTools Network tab. Block the WebSocket connection or stop the dev server.
**Expected:** Yellow banner "Connection lost. Reconnecting..." appears immediately at the top of the page
**Why human:** Requires live browser observation of reactive state change

#### 5. TypeScript Error Remediation

**Test:** `cd frontend && npx tsc --noEmit`
**Expected:** Exits 0 with no errors
**Current:** 6 TS2345 errors in `router.svelte.test.ts` — `vi.fn()` is not assignable to the typed `sendAction` parameter. Fix: cast the mock: `router.initRouter(sendActionFn as unknown as (name: string, payload?: Record<string, unknown>) => void)` in each test's `initRouter` call.
**Why human:** A developer decision is needed on whether to fix the mock cast or relax the `initRouter` parameter type

---

## Gaps Summary

No blocking gaps exist. All 23 Phase-3 FRONT requirements have working implementations verified in code. The 44 unit tests pass.

Three items merit attention before proceeding to Phase 4:

1. **TypeScript test error (Warning):** `router.svelte.test.ts` has 6 TS2345 errors where `vi.fn()` mocks are passed to a strictly-typed parameter. Tests pass at runtime, but `tsc --noEmit` exits non-zero. Fix is a one-line cast per call site.

2. **Toast not wired to protocol events (Info):** The `event` handler in `init.ts` only logs. Server-sent event messages with `name='toast'` do not trigger `ToastSurface.addToast`. This is a partial implementation of the event bus described in FRONT-14. If the intent is backend-driven toasts, this needs wiring in Phase 4 or as a small follow-up.

3. **Browser/E2E/Visual test execution (Human needed):** The test infrastructure, test files, and visual baselines all exist and are substantive. Execution requires a human with a browser environment to confirm all 30 browser+E2E+visual tests pass.

---

_Verified: 2026-03-20T12:35:00Z_
_Verifier: Claude (gsd-verifier)_
