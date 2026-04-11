---
phase: 13-datatable-enhancements
plan: 04
subsystem: ui
tags: [datatable, dropdown-menu, shadcn-svelte, bits-ui, xss-mitigation, actions-cell]

# Dependency graph
requires:
  - phase: 13-datatable-enhancements
    plan: 01
    provides: shadcn-svelte dropdown-menu primitive at $lib/components/ui/dropdown-menu, sendAction correlation-id return
provides:
  - "DataTableActions.svelte: minimal per-row actions DropdownMenu component — consumes items: { label, action }[] and dispatches via sendAction(name, payload, target)"
  - "DataTableActions.browser-test.ts: 5 vitest-browser-svelte tests proving render, click dispatch, name-fallback, and XSS escape (V5 security)"
  - "Exported ActionItem type on the module script for downstream consumers"
affects:
  - 13-05-datatable-rewrite

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "bits-ui child snippet pattern: <Trigger>{#snippet child({ props })}<Button {...props} .../>{/snippet}</Trigger> — unwraps the trigger element so we can supply our own shadcn Button with its own icon and aria-label"
    - "DropdownMenu.Item onSelect={() => handler(item)} — bits-ui MenuItemProps fires the same event on click and keyboard Enter/Space activation, avoiding the need to wire both onclick and onkeydown"
    - "XSS-safe dynamic label rendering via Svelte text interpolation {item.label} — the ONLY mitigation in this component; every other render path for user-supplied strings in this file must pass through a {#each} / text expression, never {@html}"
    - "Module-script type export: exporting the ActionItem interface from a <script lang=\"ts\" module> block so callers in other .svelte/.ts files can import the type without instantiating the component"

key-files:
  created:
    - "frontend/src/lib/components/table/DataTableActions.svelte"
    - "frontend/src/lib/components/table/DataTableActions.browser-test.ts"
  modified: []

key-decisions:
  - "Trigger child snippet signature uses { props } destructuring — matches bits-ui WithChild<{}>.child: Snippet<[{ props: Record<string, unknown> }]> exactly. Verified against node_modules/bits-ui/dist/internal/types.d.ts lines 22-29 rather than guessing. Plan 13-05 should reuse this exact pattern for the 'Columns' trigger."
  - "Item event is onSelect (NOT onclick) — bits-ui MenuItemProps exposes onSelect?: (event: Event) => void (types.d.ts line 64). onSelect fires on both mouse click and keyboard activation, giving free keyboard-accessibility. Verified the shadcn-svelte dropdown-menu-item.svelte wrapper passes restProps straight through to bits-ui, so onSelect works end-to-end."
  - "sendAction invocation shape matches the existing DataTable.svelte row-click dispatch (line 111: sendAction(action.name ?? 'select-row', { id: rowId }, action.target)). We emit sendAction(item.action.name ?? item.action.type, item.action.payload, item.action.target) to stay structurally identical — so when Plan 13-06 migrates the CRM list handlers, the action payloads land unchanged."
  - "ActionItem interface declared in a <script lang=\"ts\" module> block (vs a plain script) so TypeScript consumers importing { type ActionItem } don't pay any runtime cost and the type survives tree-shaking. Svelte 5 requires the module script for type-only exports."

requirements-completed: [TABLE-01]

# Metrics
duration: ~6min
completed: 2026-04-10
---

# Phase 13 Plan 04: DataTableActions Component Summary

**Minimal Svelte 5 DataTableActions component built on shadcn-svelte DropdownMenu primitives, with a 5-test browser suite proving render, click dispatch, name fallback, and XSS escape — the missing piece that resolves the latent `[object Object]` actions-column bug confirmed in 13-RESEARCH.md and readies Plan 13-05's DataTable rewrite for `column.kind: 'actions'` wiring.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-04-10T21:14:00Z
- **Completed:** 2026-04-10T21:17:30Z
- **Tasks:** 1 (TDD RED → GREEN)
- **Files created:** 2
- **Files modified:** 0

## Accomplishments

- `frontend/src/lib/components/table/DataTableActions.svelte` implemented as a 51-line Svelte 5 component that renders a shadcn-svelte `DropdownMenu.Root` with a ghost-icon `Button` trigger (lucide `EllipsisVertical`, `aria-label="Row actions"`) and one `DropdownMenu.Item` per element of an `items: ActionItem[]` prop. Empty arrays render the trigger alone.
- Click/keyboard activation of any item dispatches `sendAction(item.action.name ?? item.action.type, item.action.payload, item.action.target)` — exactly the shape the existing `DataTable.svelte` row-click uses and the shape Plan 13-06's migrated CRM handlers expect.
- `ActionItem` type exported from a `<script module>` block so downstream callers (Plan 13-05's DataTable rewrite) can import `{ type ActionItem } from '$lib/components/table/DataTableActions.svelte'` without instantiating the component.
- `frontend/src/lib/components/table/DataTableActions.browser-test.ts` covers 5 assertions:
  1. Trigger renders even when `items=[]`
  2. Items list rendered after trigger click (Edit / Delete)
  3. Click dispatches `sendAction('contact_delete', { contact_id: 7 }, 'modal')` with full payload + target
  4. Fallback to `action.type` when `action.name` is missing → `sendAction('custom_action', undefined, undefined)`
  5. **XSS proof:** label `<script>window.__pwned = true</script>` renders as literal text (verified via `getByText(evil, { exact: true })`), no active `<script>` descendant contains the payload, and `window.__pwned` remains `undefined` after render
- All 5 tests pass in 4.77s (vitest-browser-svelte + real Chromium via `@vitest/browser-playwright`).
- Full table directory suite (`npx vitest --config vitest-browser.config.ts --run src/lib/components/table/`) — 10/10 green across 3 test files. No regression in the existing `DataTable.browser-test.ts` or `SvelteVirtualSmoke.browser-test.ts`.

## Task Commits

1. **Task 1 RED: add failing browser test for DataTableActions** — `3d98d8d` (test)
2. **Task 1 GREEN: implement DataTableActions DropdownMenu component** — `4b11e90` (feat)

Both committed atomically with `--no-verify` per the parallel-worktree execution protocol (avoids pre-commit hook contention across sibling worktree agents in Wave 2).

## Files Created/Modified

### Created

- `frontend/src/lib/components/table/DataTableActions.svelte` — 51 lines. Module script exports `ActionItem` interface; instance script handles props + `handleSelect()`; template composes `DropdownMenu.Root > Trigger (child snippet → ghost icon Button) > Content align="end" > {#each items} DropdownMenu.Item onSelect`.
- `frontend/src/lib/components/table/DataTableActions.browser-test.ts` — 75 lines. Mocks `$lib/transport/dispatcher` with `vi.mock`, clears mocks in `beforeEach`, five `test()` cases.

### Modified

None.

## Decisions Made

- **Trigger child snippet destructures `{ props }`** — I validated this against `node_modules/bits-ui/dist/internal/types.d.ts` lines 22-29 which declares `child: Snippet<[{ props: Record<string, unknown> }]>`. The shadcn-svelte `dropdown-menu-trigger.svelte` wrapper spreads `restProps` directly into `DropdownMenuPrimitive.Trigger`, so the child snippet path is live. Plan 13-05 must use the same destructure for its `Columns` trigger.
- **Item event is `onSelect`, not `onclick`** — `bits-ui` `MenuItemProps` declares `onSelect?: (event: Event) => void` (bits-ui `menu/types.d.ts` line 64). `onSelect` fires on mouse click AND keyboard Enter/Space, so keyboard activation works for free. Using `onclick` would lose keyboard accessibility.
- **Dispatcher invocation mirrors `DataTable.svelte:111`** — The existing row-click dispatch is `sendAction(action.name ?? 'select-row', { id: rowId }, action.target)`. I chose the same arity/shape — `sendAction(name ?? type, payload, target)` — so when Plan 13-06 migrates the CRM list handlers, backend route dispatch stays structurally unchanged.
- **`ActionItem` declared in `<script lang="ts" module>`** — Svelte 5 requires the `module` block for type-only exports visible to TypeScript importers. This keeps the type reusable by `renderComponent(DataTableActions, { items })` call sites in Plan 13-05 without forcing them to redeclare the shape.
- **No Svelte MCP query issued** — the plan instructs querying `mcp__svelte__*` tools but those are not available in this parallel-executor agent's tool set (only Read/Write/Edit/Bash/Grep/Glob). I compensated by reading the real `bits-ui` type definitions directly from `node_modules/bits-ui/dist/bits/menu/types.d.ts` and `node_modules/bits-ui/dist/internal/types.d.ts`, which is the actual source of truth the MCP would consult. Every API claim in the component is grounded in the installed TypeScript declarations — see the `key-decisions` above for the exact lines referenced.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `.svelte-kit/tsconfig.json` missing on first vitest run**
- **Found during:** Task 1 RED test run (pre-implementation RED check)
- **Issue:** The worktree's `node_modules` and `.svelte-kit/` were both uninitialised. vitest-browser-svelte's Vite plugin failed to resolve `tsconfig.json`'s `extends: "./.svelte-kit/tsconfig.json"` reference and aborted the test run with a vite:esbuild internal error.
- **Fix:** Ran `npm install --prefer-offline --no-audit --no-fund` (~23s, 299 packages) followed by `npx svelte-kit sync` to generate `.svelte-kit/tsconfig.json`. Both are standard worktree bootstrapping steps, not code changes.
- **Files modified:** None under version control (`node_modules/` and `.svelte-kit/` are gitignored).
- **Verification:** RED test run confirmed the missing-component error (correct RED state), then GREEN test run after implementation returned 5/5 pass.
- **Committed in:** N/A — no tracked files changed.

**2. [Rule 3 - Informational] First vitest run flaked on Vite `optimizeDeps` hot-reload**
- **Found during:** Task 1 GREEN first run
- **Issue:** The first post-implementation run hit Vite's "unexpectedly reloaded a test" glitch because `@lucide/svelte/icons/ellipsis-vertical`, `bits-ui`, `tailwind-variants`, etc. were newly-discovered deps that triggered `optimizeDeps` mid-test. The run aborted with `Failed to fetch dynamically imported module: …/DataTableActions.svelte`.
- **Fix:** Re-ran the exact same command. Second run completed cleanly in 4.77s with 5/5 pass — the optimized deps were already cached. This is a known vitest-browser-svelte caveat documented by the warning itself. Did NOT add to `optimizeDeps.include` in `vitest-browser.config.ts` since the cache now holds the entries and a fix there would affect other plans' sibling worktrees.
- **Files modified:** None.
- **Verification:** Second run of the task-level test command + a subsequent full-directory run (`vitest .../table/`) both green.
- **Committed in:** N/A.

### Acceptance-criteria clarification (not a deviation)

The plan's acceptance criterion `grep -c "sendAction" frontend/src/lib/components/table/DataTableActions.svelte == 1` is literally unsatisfiable — any component that both imports and invokes `sendAction` will have at least 2 grep hits (`import { sendAction } from ...` + the actual call). The spirit of the criterion is "single dispatch path" (no duplicate dispatching logic), which my implementation satisfies with exactly one call site. For reference, the existing `DataTable.svelte` has 3 `sendAction` grep hits (1 import + 2 call sites for sort and row-click). Flagging for Plan 13-05 so its acceptance-criteria can be phrased as "at most one call site per dispatch purpose".

**Total deviations:** 2 Rule 3 auto-fixed (both environmental bootstrap, no code impact).
**Impact on plan:** Zero — both deviations were resolved by standard tooling invocations. Component implementation exactly matches the plan's target code block (same imports, same `handleSelect`, same template shape).

## Issues Encountered

- **Pre-existing stray error in `DataTable.browser-test.ts`** — when running the full `src/lib/components/table/` suite, a runtime stack trace surfaces from inside `DataTable.svelte:165` (sort-icon chevron rendering) during the "dispatches sort action on header click" test. All 10 tests still pass; the error is reported as a cosmetic warning by vitest, not a failure. Out of scope for this plan and pre-existing on the base commit `1ef724b`. Not logged to `deferred-items.md` because Plan 13-05 is about to rewrite `DataTable.svelte` entirely and will eliminate this file.

## User Setup Required

None — no external service configuration, no env vars, no auth gates.

## Next Plan Readiness

- **Plan 13-05 (DataTable rewrite):** Unblocked. Must import `DataTableActions` from `$lib/components/table/DataTableActions.svelte` and wire the `actions` cell kind via `renderComponent(DataTableActions, { items: rowData[col.key] })`. The `ActionItem` type is available for import via `import type { ActionItem } from '$lib/components/table/DataTableActions.svelte'`.
- **Plan 13-06 (CRM list handler migration):** Indirectly unblocked. The CRM handlers' existing row-action shape `{ label, action: { type, name, payload } }` already matches `ActionItem` exactly — the migration does not need to reshape row data for the actions column, only to ensure each column using actions is declared with `kind: 'actions'` in the column spec.

## Self-Check

Verifying claims before completion.

### Files
- `frontend/src/lib/components/table/DataTableActions.svelte` — FOUND
- `frontend/src/lib/components/table/DataTableActions.browser-test.ts` — FOUND
- `.planning/phases/13-datatable-enhancements/13-04-datatable-actions-component-SUMMARY.md` — FOUND

### Commits
- `3d98d8d` (test RED) — FOUND in `git log`
- `4b11e90` (feat GREEN) — FOUND in `git log`

### Verification Commands
- `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTableActions.browser-test.ts` → **5 passed** in 4.77s
- `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/` (full table dir) → **10 passed** across 3 test files
- `npm run check` (svelte-check) → 3 errors, all pre-existing in `tests/helpers/schema-validator.ts` (already logged to `deferred-items.md` by Plan 13-01); **0 new errors in files touched by this plan**
- `npx tsc --noEmit 2>&1 | grep DataTableActions` → **empty** (no tsc errors attributable to the new files)

### Acceptance Criteria
- `DataTableActions.svelte` exists: ✓
- `grep -c "{@html" DataTableActions.svelte` == 0: ✓ (actual: 0)
- `grep -c "sendAction" DataTableActions.svelte` == 1 (plan target): actual is 2 (1 import + 1 call) — documented above as a grep-semantics clarification, intent "single dispatch path" satisfied
- `grep -c "DropdownMenu" DataTableActions.svelte` >= 3: ✓ (actual: 9 — import + Root + Trigger open/close + Content open/close + Item open/close)
- `grep -c 'aria-label="Row actions"' DataTableActions.svelte` == 1: ✓
- Test file exists with 5 tests: ✓
- Browser test suite exits 0 with 5 passed: ✓
- tsc errors for this plan's files: 0 ✓
- XSS test asserts both `getByText(evil, { exact: true })` AND `window.__pwned === undefined`: ✓ (test 5, lines 65-79 of the test file)

## Self-Check: PASSED

## Threat Flags

None — no new security-relevant surface introduced beyond what the plan's `<threat_model>` already anticipated. The single trust boundary (server-supplied `items[].label` → DOM) is mitigated by Svelte's automatic text escaping and is proven by test 5. No new network paths, no new auth surface, no new file access, no schema changes.

---
*Phase: 13-datatable-enhancements*
*Completed: 2026-04-10*
