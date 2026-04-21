---
phase: 12
plan: 06
subsystem: frontend
tags: [frontend, svelte, shell, shadcn, appshell, surface-mount, connection-status]
wave: 3
status: complete
requirements: [SHELL-01, SHELL-02, SHELL-03, SHELL-04]
dependency_graph:
  requires:
    - "Plan 12-01 (shadcn Sidebar scaffolding)"
    - "Plan 12-04 (fine-grained surface store mutation + applyPatch dispatcher)"
    - "Plan 12-05 (AppShell backend builder with sidebarNodeId/headerNodeId/... slot keys)"
  provides:
    - "Working AppShell.svelte composing shadcn Sidebar.Provider/Root/Content/Inset/Trigger with six NodeRenderer-mounted slots (sidebar/header/footer/main/popups/toasts)"
    - "SurfaceMount.svelte: recursive <Surface name={props.name}/> redirection component"
    - "Both components registered as 'app-shell' and 'surface-mount' in registry/defaults.ts"
    - "Top-level routes/+layout.svelte collapsed to single <Surface name='main'/>"
    - "websocket.svelte.ts publishes connection state into /system/connectionStatus via applyPatch on every open/close/disconnect transition (D-B6 footer indicator wiring)"
    - "ConnectionBanner.svelte + browser test retired; lib/index.ts no longer exports it; Surface.svelte layoutClasses map simplified to main-only"
  affects:
    - "routes/+layout.svelte (collapsed)"
    - "Surface.svelte (layout classes simplified)"
    - "lib/index.ts (ConnectionBanner export removed)"
    - "Phase 12-07 (CRM integration can now construct AppShell via backend builder and render into a working shell)"
tech_stack:
  added: []
  patterns:
    - "Standard SDUI component contract with underscore-prefixed unused params (bind:_bind, action:_action, surface:_surface) + eslint-disable-line @typescript-eslint/no-unused-vars — same shape scaffold introduced in Plan 12-01"
    - "shadcn Sidebar composition as namespace-imported primitive: import * as Sidebar from $lib/components/ui/sidebar; <Sidebar.Provider>/<Sidebar.Root>/<Sidebar.Content>/<Sidebar.Inset>/<Sidebar.Trigger>"
    - "Browser test viewport forcing via page.viewport(1280, 800) in beforeEach — required because shadcn Sidebar auto-switches to the closed-by-default Sheet.Root portal path under its 768px mobile breakpoint, making slot content invisible in the default playwright viewport (414×896)"
    - "applyPatch-mocked unit test pattern for transport-layer side effects: vi.mock('$lib/store/data.svelte', ...) + vi.resetModules() per test to get fresh module-local state"
key_files:
  created:
    - path: "frontend/src/lib/transport/websocket.connection-status.test.ts"
      purpose: "5 unit tests proving publishConnectionStatus is called with the exact tagged Set op shape on open/close/disconnect + a full lifecycle test + an error-swallowing test"
  modified:
    - path: "frontend/src/lib/components/core/SurfaceMount.svelte"
      change: "scaffold replaced with real implementation — renders <Surface name={props.name}/> when props.name is truthy"
    - path: "frontend/src/lib/components/core/SurfaceMount.browser-test.ts"
      change: "scaffold test.todo replaced with 2 real browser tests (sub-surface mounts, LoadingSkeleton shown when tree absent)"
    - path: "frontend/src/lib/components/shell/AppShell.svelte"
      change: "scaffold replaced with real implementation — shadcn Sidebar composition wraps Inset flex column, six slot IDs resolved via $derived + NodeRenderer mounts"
    - path: "frontend/src/lib/components/shell/AppShell.browser-test.ts"
      change: "scaffold test.todo replaced with 3 real browser tests (all slots render, missing slots don't crash, Sidebar.Trigger present). Viewport forced to 1280×800 via page.viewport() in beforeEach."
    - path: "frontend/src/lib/registry/defaults.ts"
      change: "registers 'surface-mount': SurfaceMount and 'app-shell': AppShell (20 total component types now)"
    - path: "frontend/src/routes/+layout.svelte"
      change: "collapsed from 7 DOM nodes (<ConnectionBanner/> + flex wrapper + 4 <Surface/> mounts) down to 1 <Surface name='main'/> + {@render children()}. D-B9."
    - path: "frontend/src/lib/index.ts"
      change: "removed ConnectionBanner re-export (the file is deleted)"
    - path: "frontend/src/lib/components/core/Surface.svelte"
      change: "layoutClasses map simplified — sidebar/modal/toast entries removed (those surfaces now mount via SurfaceMount inside AppShell, where the shell handles framing). Only 'main' remains."
    - path: "frontend/src/lib/transport/websocket.svelte.ts"
      change: "added publishConnectionStatus() helper + 3 call sites (onopen → 'connected', onclose → 'reconnecting' | 'offline' depending on currentUrl, disconnect → 'offline'). Uses applyPatch('main', [{op:'set', path:'/system/connectionStatus', value: ...}]) matching Plan 12-04's tagged Set op shape."
    - path: ".planning/phases/12-protocol-node-patching-appshell/deferred-items.md"
      change: "logged 5 pre-existing popup browser-test failures (ConfirmDialog 4/4 + ToastSurface 1/3) — reproduced on the pre-Plan-12-06 tree, out of scope per SCOPE BOUNDARY rule"
  deleted:
    - path: "frontend/src/lib/components/core/ConnectionBanner.svelte"
      reason: "D-B6 retirement — reactive connection-state display migrated to transport layer + /system/connectionStatus data path + AppShell footer binding"
    - path: "frontend/src/lib/components/core/ConnectionBanner.browser-test.ts"
      reason: "Test follows its component"
decisions:
  - "Used namespace import `import * as Sidebar from '$lib/components/ui/sidebar'` (matches the ui/sidebar/index.ts named-export style verified in-file). Sub-component names used: Sidebar.Provider, Sidebar.Root, Sidebar.Content, Sidebar.Inset, Sidebar.Trigger. The index.ts also exports the long-form aliases (SidebarProvider etc.) but the short `Sidebar.*` form is cleaner for composition."
  - "AppShell puts popups and toasts NodeRenderers INSIDE Sidebar.Inset rather than as siblings of Sidebar.Provider. Placement doesn't matter visually because Dialog/Toast are portaled to document.body, but nesting them inside Inset keeps the DOM tree clean and satisfies the RESEARCH Example 3 note about 'either location works'."
  - "SurfaceMount's bind/action/surface props are accepted (contract-compatible) but destructured with an `_` prefix + a single eslint-disable-line `@typescript-eslint/no-unused-vars` comment to avoid the svelte-check `state_referenced_locally` warnings that the `void bind; void action; void surface;` pattern triggers (same fix as the Plan 12-01 scaffold). Only `props.name` is consumed."
  - "AppShell browser test forces a desktop-sized viewport (1280×800) via page.viewport() in beforeEach. This is required: the default playwright test viewport is 414×896 (mobile), which flips shadcn's useSidebar().isMobile to true, which routes the sidebar content into a closed-by-default Sheet.Root (Dialog) — making the slot content absent from the DOM and invisible to baseElement.textContent assertions. The 1280×800 value matches a typical desktop and is well above the 768px md breakpoint."
  - "publishConnectionStatus() is a try/catch-wrapped helper that swallows applyPatch errors with console.debug — unit tests without a store initialization would otherwise throw when mutating /system/connectionStatus. In production the first Render seeds the data path and subsequent publishes apply cleanly."
  - "disconnect() explicitly publishes 'offline' at the end even though socket.close() → onclose also publishes it (via the `currentUrl ? 'reconnecting' : 'offline'` branch, which resolves to 'offline' because disconnect() clears currentUrl first). The double-emit is harmless (same value) and ensures 'offline' is published even if the socket was already null."
  - "Used `vitest/browser` (not the deprecated `@vitest/browser/context`) for the page.viewport() import — future-proof against the next vitest major."
  - "Chose to add a NEW test file (websocket.connection-status.test.ts) rather than extending the existing websocket.svelte.test.ts because the two files have different mock boundaries: the existing file mocks WebSocket only, whereas the connection-status tests also need to mock applyPatch. Mixing the two in one file would have required either double-mocking or reshuffling the existing tests."
metrics:
  tasks_completed: 4
  tasks_total: 4
  commits: 4
  files_created: 1  # websocket.connection-status.test.ts
  files_modified: 9  # SurfaceMount.svelte + test, AppShell.svelte + test, defaults.ts, +layout.svelte, index.ts, Surface.svelte, websocket.svelte.ts, deferred-items.md (minus the double-count on defaults.ts which was modified twice)
  files_deleted: 2  # ConnectionBanner.svelte + browser test
  duration_minutes: ~35
  completed_date: "2026-04-10"
---

# Phase 12 Plan 06: Frontend Shell Components Summary

**One-liner:** Real SurfaceMount + AppShell Svelte implementations wired through the existing registry and NodeRenderer pipeline, top-level layout collapsed to a single Surface, ConnectionBanner retired in favor of a transport-layer publishConnectionStatus() pushing into `/system/connectionStatus` for the AppShell footer indicator.

## What Shipped

### Task 1 — SurfaceMount.svelte + registry + browser tests (commit 8b5264d)

- Replaced the Plan 12-01 scaffold with a real implementation: `<Surface name={props.name}/>` when `props.name` is truthy, nothing otherwise. Contract-compatible (accepts bind/action/surface) but only consumes `props.name`.
- 2 browser tests replacing the 2 `test.todo`: sub-surface content renders after `setSurfaceTree` + `setFullState`; missing tree shows the `[data-surface]` element (LoadingSkeleton path).
- Registered as `'surface-mount'` in `registry/defaults.ts`.
- `npm run check` green (baseline 3 deferred errors only); SurfaceMount browser test 2/2 passing.

### Task 2 — AppShell.svelte with shadcn Sidebar composition + browser tests (commit 2c4aad9)

- Replaced the Plan 12-01 scaffold with a real shadcn Sidebar composition:
  ```
  <Sidebar.Provider>
    <Sidebar.Root collapsible="offcanvas">
      <Sidebar.Content> NodeRenderer[sidebarNodeId] </Sidebar.Content>
    </Sidebar.Root>
    <Sidebar.Inset>
      <div flex-col>
        <header> <Sidebar.Trigger/> NodeRenderer[headerNodeId] </header>
        <main>   NodeRenderer[mainNodeId] </main>
        <footer> NodeRenderer[footerNodeId] </footer>
        NodeRenderer[popupsNodeId]
        NodeRenderer[toastsNodeId]
      </div>
    </Sidebar.Inset>
  </Sidebar.Provider>
  ```
  Slot IDs resolved via `$derived(props.{sidebar,header,footer,main,popups,toasts}NodeId)`. Nodes looked up from `getSurfaceTree(surface).nodes`. Missing IDs gracefully skipped with `{#if slotId && nodes[slotId]}`.
- 3 browser tests: all slots render (sidebar asserted via `data-sidebar="sidebar"` query — see deviation 1), missing slots don't crash, Sidebar.Trigger present (`button[data-sidebar="trigger"]`).
- Registered as `'app-shell'` in `registry/defaults.ts`.
- `npm run check` green; AppShell browser test 3/3 passing.

### Task 3 — websocket.svelte.ts publishes connection state into /system/connectionStatus (commit 466f694)

- Added top-level import `import { applyPatch } from '$lib/store/data.svelte'`.
- Added private `publishConnectionStatus(state)` helper that calls `applyPatch('main', [{op:'set', path:'/system/connectionStatus', value: state}])` with a try/catch wrapper.
- 3 invocation sites:
  1. `socket.onopen` → `publishConnectionStatus('connected')`
  2. `socket.onclose` → `publishConnectionStatus(currentUrl ? 'reconnecting' : 'offline')`
  3. `disconnect()` → `publishConnectionStatus('offline')` (after `currentUrl = null`)
- New test file `websocket.connection-status.test.ts` with 5 tests:
  1. `onopen` publishes `connected`
  2. `onclose` (currentUrl still set) publishes `reconnecting`
  3. `disconnect()` publishes `offline` (and NO non-offline emissions)
  4. Full lifecycle open→close→reconnect→open emits `['connected', 'reconnecting', 'connected']`
  5. publishConnectionStatus swallows applyPatch errors (store not ready)
- Uses `vi.mock('$lib/store/data.svelte', ...)` with a fresh `applyPatchMock` and `vi.resetModules()` per test.
- grep count: `publishConnectionStatus(` appears 4× in websocket.svelte.ts (definition + 3 call sites). ✓
- `npm run check` green; `vitest --run websocket` → 15/15 (10 existing + 5 new).

### Task 4 — Collapse +layout.svelte and retire ConnectionBanner (commit df316d2)

**Pre-edit safety check (W-03):** Ran the three greps required by the plan. Each returned only the `+layout.svelte` line about to be rewritten, zero other files. Safe to delete.

- `routes/+layout.svelte` collapsed from 7 DOM nodes (ConnectionBanner + flex wrapper + 4 Surface mounts) to `<Surface name="main"/>` + `{@render children()}`. Exactly one Surface mount at top level.
- Deleted `ConnectionBanner.svelte` and `ConnectionBanner.browser-test.ts`. `lib/index.ts` no longer re-exports `ConnectionBanner` (line removed). `isConnected()` export stays — it's still used by `src/routes/+page.svelte:64`.
- `Surface.svelte` `layoutClasses` map simplified: `sidebar:` / `modal:` / `toast:` entries removed, only `main:` remains. The `modal:` and `toast:` entries were always empty strings, and `sidebar:` is now unreachable because those surfaces are only mounted via SurfaceMount inside AppShell (which does not use a top-level layout class).
- Scrubbed historical ConnectionBanner references from Task 3's new files' doc comments (rephrased to cite D-B6 only) so `grep -rn 'ConnectionBanner' frontend/src/` returns exit 1 (zero matches).
- `npm run check` → baseline 3 deferred errors only. `npm run build` → green (4.46s client + 10.11s server + static adapter wrote build/). `vitest --run` → 58/58 unit tests pass. `AppShell.browser-test` + `SurfaceMount.browser-test` → 5/5 pass.

## Shadcn Sidebar Sub-Component Names (Output Requirement)

The plan's `<output>` section asks for the exact Sidebar sub-component names used. From `frontend/src/lib/components/ui/sidebar/index.ts`, the named exports include both the short form (Provider, Root, Content, Inset, Trigger) and the long form (SidebarProvider, SidebarRoot, ...). Plan 12-06 uses the **short form** via namespace import:

```typescript
import * as Sidebar from '$lib/components/ui/sidebar';
// Used in markup:
<Sidebar.Provider> <Sidebar.Root collapsible="offcanvas">
  <Sidebar.Content> ... </Sidebar.Content>
</Sidebar.Root> <Sidebar.Inset> ... </Sidebar.Inset> </Sidebar.Provider>
// And within <header>:
<Sidebar.Trigger />
```

## Svelte MCP Server

The svelte MCP server was **not available** in this executor's tool manifest (no `mcp__svelte__*` tools exposed). The autofixer step from Task 2's `<action>` step 7 was therefore skipped, but the equivalent guard (`npm run check` reporting 0 new warnings and 0 new errors) was enforced after every file edit. All svelte-check warnings on Plan 12-06 files were resolved before commit — final state shows only the baseline 3 pre-existing errors in `tests/helpers/schema-validator.ts` (logged in deferred-items.md since 12-01).

## Files Deleted

- `frontend/src/lib/components/core/ConnectionBanner.svelte` (1)
- `frontend/src/lib/components/core/ConnectionBanner.browser-test.ts` (1)

**Total: 2 files deleted.** No empty directories left behind — `frontend/src/lib/components/core/` still contains Surface.svelte, NodeRenderer.svelte, SurfaceMount.svelte, LoadingSkeleton.svelte, ErrorBoundary.svelte, FallbackComponent.svelte, so the dir stays.

## `npm run build` Outcome

Succeeded **on first try** after Task 4's edits. No adjustments needed. Build output confirms:
- Static adapter wrote to `build/`
- Client bundle: largest chunk 172.27 kB (gzip 49.59 kB)
- Server bundle: index.js 125.75 kB
- sidebar-menu-button.js chunk (23.68 kB) confirms the shadcn Sidebar block is bundled into the layout

## Verification Results

- `cd frontend && npm run check` → 3 errors, 0 warnings (all 3 are the baseline `tests/helpers/schema-validator.ts` Node-type errors already in `deferred-items.md` from Plan 12-01).
- `cd frontend && npm run build` → success, 14.57s total, adapter-static wrote build/.
- `cd frontend && npx vitest --run` → 58/58 unit tests pass (includes the 5 new websocket.connection-status tests + the 10 existing websocket.svelte tests).
- `cd frontend && npx vitest --config vitest-browser.config.ts --run AppShell.browser-test SurfaceMount.browser-test` → 5/5 pass.
- `cd frontend && npx vitest --config vitest-browser.config.ts --run` → 73/78 pass (5 failures in `src/lib/components/popup/` are pre-existing — documented in deferred-items.md).
- `grep -rn 'ConnectionBanner' frontend/src/` → zero hits (exit 1).
- `grep -c 'Surface name=' frontend/src/routes/+layout.svelte` → 1.
- `grep -q "'app-shell': AppShell" frontend/src/lib/registry/defaults.ts` → OK.
- `grep -q "'surface-mount': SurfaceMount" frontend/src/lib/registry/defaults.ts` → OK.
- `grep -q 'publishConnectionStatus' frontend/src/lib/transport/websocket.svelte.ts` → OK (4 occurrences: 1 definition + 3 call sites).
- `grep -q "'/system/connectionStatus'" frontend/src/lib/transport/websocket.svelte.ts` → OK.
- W-03 pre-edit proof greps (`<Surface name="sidebar"`, `<Surface name="modal"`, `<Surface name="toast"` in frontend/src) → zero matches after Task 4.

## Deviations from Plan

### 1. [Rule 1 — Bug] AppShell browser test viewport

**Found during:** Task 2 verification — initial `AppShell.browser-test.ts` run.

**Issue:** Two of three tests failed. The sidebar slot content was missing from `baseElement.textContent` even though header/main/footer slots rendered fine. Investigation via a debug `console.log(window.innerWidth)` revealed the default playwright test viewport is **414×896** (mobile iPhone Plus-like), which is below the shadcn Sidebar 768px mobile breakpoint. This flips `useSidebar().isMobile` to true, which routes the sidebar content through `Sheet.Root` (a Dialog portal) that is closed by default — so the slot content is absent from the DOM entirely, not just visually hidden.

**Fix:** Added `await page.viewport(1280, 800)` in `beforeEach`, imported from `vitest/browser` (the non-deprecated path). This forces the Sidebar into its desktop path and the test assertions against `sidebarInner.textContent` pass. The SidebarContent assertion also switched from `baseElement.textContent` to `querySelector('[data-slot="sidebar-inner"], [data-sidebar="sidebar"]')` to be explicit about WHERE in the DOM the sidebar lives (the fixed-position container has `hidden md:block` responsive classes).

**Files modified:** `frontend/src/lib/components/shell/AppShell.browser-test.ts`

**Commit:** `2c4aad9`

### 2. [Rule 3 — Blocking] Run `npm install` and `npx svelte-kit sync` before `npm run check`

**Found during:** Task 1 first `npm run check` attempt.

**Issue:** The worktree was fresh — `frontend/node_modules` did not exist and `.svelte-kit/tsconfig.json` had not been generated. svelte-check failed with "Cannot read file '.svelte-kit/tsconfig.json'".

**Fix:** Ran `cd frontend && npm install` (which installed 290+ packages) followed by `npx svelte-kit sync` to generate the sveltekit type bridge. After that, `npm run check` ran normally with only the 3 pre-existing deferred errors.

**Files modified:** None (pre-install side effects only — `frontend/node_modules/` and `frontend/.svelte-kit/` are gitignored).

**Commit:** n/a (pre-Task-1 environment setup).

### 3. [Rule 1 — Warning] Underscore-prefix unused props instead of `void` to silence `state_referenced_locally`

**Found during:** Task 1 first `npm run check` after initial SurfaceMount.svelte edit.

**Issue:** My initial implementation used the `void bind; void action; void surface;` pattern to mark unused props as intentionally unused. svelte-check reported three warnings of the form `This reference only captures the initial value of 'bind'. Did you mean to reference it inside a closure instead?` (`state_referenced_locally`). This is Svelte 5's reactivity rule: `$props()` destructured values are reactive, and reading them at the top of `<script>` captures only the initial value — Svelte flags this even when the read is a `void` discard.

**Fix:** Switched to `let { props = {}, bind: _bind, action: _action, surface: _surface } = $props()` with a single `// eslint-disable-next-line @typescript-eslint/no-unused-vars` comment above the destructure. This is the same pattern Plan 12-01's scaffold settled on (see 12-01 Deviation 3). Warnings disappear, type-safety preserved.

**Files modified:** `frontend/src/lib/components/core/SurfaceMount.svelte`

**Commit:** `8b5264d`

### 4. [Out of scope — logged] Pre-existing popup browser-test failures

**Found during:** Task 4 verification — `npx vitest --config vitest-browser.config.ts --run`.

**Issue:** 5 browser tests fail in `frontend/src/lib/components/popup/`:
- `ConfirmDialog.browser-test.ts` (4/4 tests) — Playwright locator errors suggesting the dialog markup changed and selectors drifted.
- `ToastSurface.browser-test.ts` (1/3 tests) — "strict mode violation: resolved to 2 elements" on `getByLabelText('Dismiss')`, suggesting leaked state between tests.

**Resolution:** `git stash && npx vitest --config vitest-browser.config.ts --run src/lib/components/popup/` on the pre-Plan-12-06 tree reproduces exactly the same 5 failures. Plan 12-06 does not touch `src/lib/components/popup/*`. Per the SCOPE BOUNDARY rule, these are **out of scope** and logged to `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` for a future popup-fix plan.

**Files modified:** `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md`

**Commit:** `df316d2` (as part of Task 4).

No Rule 2 (missing critical functionality), no Rule 4 (architectural change), no authentication gates, no checkpoints.

## Threat Flags

None. Plan 12-06 introduces no new security surface — the threat register in the plan (T-12-13 XSS via heading text, T-12-14 recursion stack overflow, T-12-15 sidebar trigger UI-local state) all have their dispositions honored by the implementation:

- **T-12-13 (XSS via heading text) — accept:** AppShell and SurfaceMount contain zero `{@html ...}` uses. All slot content flows through NodeRenderer → leaf component props where Svelte's default escaping applies. Verified by grep: `grep -n '@html' frontend/src/lib/components/shell/AppShell.svelte frontend/src/lib/components/core/SurfaceMount.svelte` → zero hits.
- **T-12-14 (recursive surface-mount stack overflow) — mitigate:** SurfaceMount → Surface → NodeRenderer → SurfaceMount (cycle) cannot stack-overflow because each Surface has its own independent tree; cycling surfaces A → B → A produces two separate Surfaces each rendering the other's tree, never re-triggering the SAME Surface instance's render. Documented in RESEARCH Pattern 2.
- **T-12-15 (Sidebar.Trigger UI-local state) — accept:** `Sidebar.Trigger.onclick` only calls `sidebar.toggle()` which mutates local SidebarState instance state. No action dispatch, no server round trip, no data tampering surface. SHELL-05 (persistence) is explicitly deferred to v2.

## Known Stubs

None. All code paths are wired end-to-end:
- AppShell renders all six slots via NodeRenderer (missing slots are gracefully skipped, not placeholder-filled).
- SurfaceMount renders the named sub-surface directly through the existing Surface machinery.
- publishConnectionStatus is called at all three lifecycle transitions (no TODO stubs).
- routes/+layout.svelte is in its final minimal form — nothing pending.

## Commits

- `8b5264d` — feat(12-06): implement SurfaceMount + register + 2 browser tests
- `2c4aad9` — feat(12-06): implement AppShell.svelte with shadcn Sidebar composition
- `466f694` — feat(12-06): publish ws connection state to /system/connectionStatus (D-B6)
- `df316d2` — refactor(12-06): collapse +layout.svelte and retire ConnectionBanner (D-B9)

## Self-Check: PASSED

### File existence
- `frontend/src/lib/components/core/SurfaceMount.svelte` FOUND
- `frontend/src/lib/components/core/SurfaceMount.browser-test.ts` FOUND
- `frontend/src/lib/components/shell/AppShell.svelte` FOUND
- `frontend/src/lib/components/shell/AppShell.browser-test.ts` FOUND
- `frontend/src/lib/registry/defaults.ts` FOUND (contains 'surface-mount' and 'app-shell')
- `frontend/src/routes/+layout.svelte` FOUND (contains exactly `<Surface name="main"/>`)
- `frontend/src/lib/index.ts` FOUND (does NOT contain ConnectionBanner)
- `frontend/src/lib/components/core/Surface.svelte` FOUND (layoutClasses has only `main:`)
- `frontend/src/lib/transport/websocket.svelte.ts` FOUND (contains publishConnectionStatus)
- `frontend/src/lib/transport/websocket.connection-status.test.ts` FOUND
- `frontend/src/lib/components/core/ConnectionBanner.svelte` DELETED ✓
- `frontend/src/lib/components/core/ConnectionBanner.browser-test.ts` DELETED ✓

### Commit existence
- `8b5264d` FOUND in git log
- `2c4aad9` FOUND in git log
- `466f694` FOUND in git log
- `df316d2` FOUND in git log

### Verification gates
- `npm run check` PASSED (baseline 3 deferred errors only)
- `npm run build` PASSED (14.57s, adapter-static wrote build/)
- `vitest --run` PASSED (58/58)
- `vitest --config vitest-browser.config.ts --run AppShell.browser-test` PASSED (3/3)
- `vitest --config vitest-browser.config.ts --run SurfaceMount.browser-test` PASSED (2/2)
- `vitest --config vitest-browser.config.ts --run websocket.connection-status` PASSED (5/5)
- `grep -rn 'ConnectionBanner' frontend/src/` → zero hits (exit 1)

---
*Phase: 12-protocol-node-patching-appshell*
*Completed: 2026-04-10*
