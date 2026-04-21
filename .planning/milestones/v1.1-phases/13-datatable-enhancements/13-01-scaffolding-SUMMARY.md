---
phase: 13-datatable-enhancements
plan: 01
subsystem: ui
tags: [tanstack, svelte-virtual, virtual-core, shadcn-svelte, data-table, dropdown-menu, intersection-observer, dispatcher, seed]

# Dependency graph
requires:
  - phase: 12-protocol-node-patching-appshell
    provides: per-surface reactive data store + dispatcher correlation via ActionMessage.id
provides:
  - "@tanstack/table-core@8.21.3 and @tanstack/svelte-virtual@3.13.23 (+ transitive @tanstack/virtual-core@3.13.23) installed"
  - "shadcn-svelte data-table helper package at $lib/components/ui/data-table/ exporting createSvelteTable, FlexRender, renderSnippet, renderComponent"
  - "shadcn-svelte dropdown-menu primitive at $lib/components/ui/dropdown-menu/"
  - "Svelte 5 + virtual-core-direct wrapper at $lib/utils/virtualizer.svelte (createRuneVirtualizer) — smoke-tested and proven"
  - "sendAction returns the generated correlation UUID (string) so callers can track request/response correspondence"
  - "onIntersect Svelte action at $lib/actions/viewport wrapping IntersectionObserver with leading-edge latch + enabled toggle"
  - "120 seeded demo contacts (3 named + 117 generated) so infinite-scroll E2E has > 2 × page_size rows"
affects:
  - 13-02-backend-builder
  - 13-03-fetch-rows-handler
  - 13-04-datatable-actions-component
  - 13-05-datatable-rewrite
  - 13-06-crm-list-handler-migration
  - 13-07-e2e-and-textinput-fix

# Tech tracking
tech-stack:
  added:
    - "@tanstack/table-core@^8.21.3"
    - "@tanstack/svelte-virtual@^3.13.23"
    - "@tanstack/virtual-core@3.13.23 (transitive, used directly)"
    - "shadcn-svelte data-table helper (CLI-installed source files)"
    - "shadcn-svelte dropdown-menu primitive (CLI-installed source files)"
  patterns:
    - "Svelte 5 reactive wrappers around non-reactive headless libraries via $state tick counters in .svelte.ts modules"
    - "Idempotent mount()/destroy() contract for lifecycle-bound wrappers"
    - "Dispatcher returns correlation IDs for component-scoped request/response tracking (no new protocol messages needed)"
    - "Leading-edge latched IntersectionObserver (flip false→true only) to prevent duplicate dispatches during slow scrolls"

key-files:
  created:
    - "frontend/src/lib/components/ui/data-table/index.ts"
    - "frontend/src/lib/components/ui/data-table/data-table.svelte.ts"
    - "frontend/src/lib/components/ui/data-table/flex-render.svelte"
    - "frontend/src/lib/components/ui/data-table/render-helpers.ts"
    - "frontend/src/lib/components/ui/dropdown-menu/index.ts + 16 component files"
    - "frontend/src/lib/components/table/SvelteVirtualSmoke.svelte"
    - "frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts"
    - "frontend/src/lib/utils/virtualizer.svelte.ts"
    - "frontend/src/lib/actions/viewport.ts"
    - ".planning/phases/13-datatable-enhancements/deferred-items.md"
  modified:
    - "frontend/package.json (added TanStack deps)"
    - "frontend/package-lock.json"
    - "frontend/src/lib/transport/dispatcher.ts (sendAction now returns string)"
    - "frontend/src/lib/transport/dispatcher.test.ts (3 new return-value tests)"
    - "backend/crates/crm-demo/src/seed.rs (3 → 120 contacts)"

key-decisions:
  - "Svelte-virtual path: VIRTUAL-CORE-DIRECT. The store-based @tanstack/svelte-virtual adapter leaves {#each $store.getVirtualItems() ...} empty under Svelte 5 (issue TanStack/virtual#866). Fallback to @tanstack/virtual-core + $state tick counter proven in smoke test."
  - "sendAction return-type widened from void to string — backward compatible at every call site (all existing callers ignore the return value)."
  - "Seed retains Alice/Bob/Carol as named contacts so seed_tags/seed_notes/seed_interactions lookups by contact_name continue to work; 117 deterministic Seed Contact NNN rows append for pagination tests."
  - "Pre-existing tsc --noEmit errors in ui/badge, ui/button, and tests/helpers/schema-validator.ts are OUT OF SCOPE (logged to deferred-items.md); the project's effective type gate is svelte-check (`npm run check`), which shows no new errors."

patterns-established:
  - "Svelte 5 rune wrapper for headless TanStack libraries: expose getters that `void tick` (trivial dependency registration) and bridge library callbacks to `tick++` inside $state. Idempotent mount/destroy lifecycle."
  - "Dispatcher correlation pattern: callers capture `const id = sendAction(...)` and drop any incoming message whose id doesn't match the latest tracked id. No protocol changes."

requirements-completed: [TABLE-01, TABLE-02]

# Metrics
duration: ~8min
completed: 2026-04-11
---

# Phase 13 Plan 01: Scaffolding Summary

**Wave-0 infrastructure for the DataTable rewrite: TanStack deps installed, shadcn-svelte data-table and dropdown-menu primitives via CLI, Svelte 5 + virtual-core-direct wrapper proven via smoke test, sendAction extended with correlation-id return, onIntersect Svelte action, and 120 seeded contacts.**

## Performance

- **Duration:** ~8 min (plus ~3 min cargo build on cold cache)
- **Started:** 2026-04-11T18:50:00Z
- **Completed:** 2026-04-11T18:59:00Z
- **Tasks:** 3
- **Files created:** 28 (shadcn CLI output) + 5 (hand-written) + 1 (deferred log)
- **Files modified:** 4

## Accomplishments

- `@tanstack/table-core@8.21.3` and `@tanstack/svelte-virtual@3.13.23` pinned in `frontend/package.json` (plus transitive `@tanstack/virtual-core@3.13.23`).
- shadcn-svelte `data-table` helper package installed at `frontend/src/lib/components/ui/data-table/` exporting `createSvelteTable`, `FlexRender`, `renderSnippet`, `renderComponent` — verified against Plan 13-05's import contract.
- shadcn-svelte `dropdown-menu` primitive installed at `frontend/src/lib/components/ui/dropdown-menu/` with the full 17-file subtree (Root/Trigger/Content/CheckboxItem/etc.).
- **Svelte 5 + svelte-virtual compatibility decision made empirically.** Store-based adapter tested first, failed (row-0 never rendered even though `getTotalSize()` correctly reported 4000px — classic manifestation of TanStack/virtual#866). Fallback `createRuneVirtualizer` wrapper built around `@tanstack/virtual-core` directly, wired via a `$state` tick counter; smoke test passes. Decision recorded at the top of `SvelteVirtualSmoke.svelte` so Plan 13-05 uses the same path.
- `sendAction` in `frontend/src/lib/transport/dispatcher.ts` now returns the generated correlation UUID as `string`. Backward compatible: all 15+ existing call sites ignore the return value. Three new unit tests cover the new contract; all 11 dispatcher tests green.
- `onIntersect` Svelte action at `frontend/src/lib/actions/viewport.ts` wraps `IntersectionObserver` with a leading-edge latch (fires only on `false → true` transitions) and an `enabled` toggle for idling the sentinel once the table runs out of rows.
- `backend/crates/crm-demo/src/seed.rs` seeds 120 contacts (3 named + 117 generated). `cargo build -p crm-demo` and `cargo test -p crm-demo` both green (7 unit + 5 integration tests unchanged).

## Task Commits

1. **Task 1: Install TanStack deps + shadcn-svelte CLI adds** — `57a30c6` (chore)
2. **Task 2: Svelte-virtual smoke test + virtual-core-direct wrapper (TDD RED → GREEN)** — `87b17b6` (feat)
3. **Task 3: sendAction returns id + onIntersect action + seed 120 contacts (TDD)** — `95e6116` (feat)

Committed atomically with `--no-verify` (per parallel-worktree execution protocol; pre-commit hook contention avoided across sibling worktree agents).

## Files Created/Modified

### Created (hand-written)
- `frontend/src/lib/components/table/SvelteVirtualSmoke.svelte` — smoke-test component with svelte-virtual Svelte 5 decision comment at top. Uses `createRuneVirtualizer`.
- `frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts` — vitest-browser-svelte test asserting row-0 visible, row-99 absent, total height ≥ 4000px.
- `frontend/src/lib/utils/virtualizer.svelte.ts` — `createRuneVirtualizer` factory wrapping `@tanstack/virtual-core`'s `Virtualizer` class with Svelte 5 `$state` reactivity. Idempotent `mount()/destroy()`, reactive `totalSize` and `virtualItems` getters.
- `frontend/src/lib/actions/viewport.ts` — `onIntersect` Svelte action (exports `OnIntersectOptions` type and `onIntersect` function).
- `.planning/phases/13-datatable-enhancements/deferred-items.md` — logs pre-existing tsc errors out of scope for this plan.

### Created (shadcn-svelte CLI output)
- `frontend/src/lib/components/ui/data-table/` — 4 files (`index.ts`, `data-table.svelte.ts`, `flex-render.svelte`, `render-helpers.ts`)
- `frontend/src/lib/components/ui/dropdown-menu/` — 17 files (index.ts + 16 dropdown-menu-*.svelte primitives)

### Modified
- `frontend/package.json` — `dependencies` gains `@tanstack/table-core` `^8.21.3` and `@tanstack/svelte-virtual` `^3.13.23`.
- `frontend/package-lock.json` — +299 transitive packages.
- `frontend/src/lib/transport/dispatcher.ts` — `sendAction` return type `void` → `string`, JSDoc updated to explain D-H3 correlation use case.
- `frontend/src/lib/transport/dispatcher.test.ts` — 3 new tests appended to the existing `Message dispatcher` describe block.
- `backend/crates/crm-demo/src/seed.rs` — `seed_contacts` rewrites to preserve 3 named contacts (Alice/Bob/Carol) + 117 generated (`Seed Contact 000..116`). Seed log updated.

## Decisions Made

- **Svelte-virtual path: VIRTUAL-CORE-DIRECT.** The store-based adapter's `derived` store only fires `onChange` on the initial mount under Svelte 5 `{#each}`; the test rendered the outer `height: 4000px` container but the inner rows were empty. Switched to direct `Virtualizer` instantiation with a rune-based tick counter bridge. Documented in the component's top-of-file comment so Plan 13-05 follows the same path.
- **sendAction signature widening is safe.** `grep -rn "sendAction(" frontend/src` shows 15+ call sites across NavItem, ConfirmDialog, ModalSurface, FormScreen, TableScreen, Button, TextInput, SelectInput, Form, and DataTable — none of them bind the return value or annotate it as `void`. Widening `void → string` is non-breaking.
- **Named-contact preservation in seed.** `seed_tags`, `seed_notes`, and `seed_interactions` all look up contacts by name (Alice Johnson / Bob Smith / Carol Williams). Keeping those three stable and appending 117 generated contacts avoids cascading changes to the rest of the seed pipeline.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Svelte-virtual store adapter empty-table regression forced fallback path**
- **Found during:** Task 2 (SvelteVirtualSmoke TDD)
- **Issue:** The store-based attempt rendered an empty table — `$virtualizer.getTotalSize()` returned 4000 correctly but `$virtualizer.getVirtualItems()` returned `[]` inside `{#each}`. Classic manifestation of TanStack/virtual#866.
- **Fix:** Built `frontend/src/lib/utils/virtualizer.svelte.ts` as the planned Step B fallback. Rewrote `SvelteVirtualSmoke.svelte` to use `createRuneVirtualizer`. Updated the decision comment to record the actual chosen path.
- **Files modified:** frontend/src/lib/utils/virtualizer.svelte.ts, frontend/src/lib/components/table/SvelteVirtualSmoke.svelte
- **Verification:** `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/SvelteVirtualSmoke.browser-test.ts` → 1 passed.
- **Committed in:** `87b17b6` (Task 2 commit)

**2. [Rule 1 - Bug] Infinite effect-update-depth loop in first wrapper draft**
- **Found during:** Task 2 (first wrapper iteration)
- **Issue:** First draft of `createRuneVirtualizer` bumped `tick` from inside `mount()` AND inside `setOptions()`. Since the owning `$effect` in SvelteVirtualSmoke reads `scrollRef` (to decide when to call mount), and the template reads `vr.totalSize` / `vr.virtualItems` (which read `tick`), the mount-time `tick++` re-ran the owning effect, which called `mount()` again, etc. Svelte threw `effect_update_depth_exceeded`.
- **Fix:** Added an idempotent `mounted` guard in `mount()`, removed the direct `tick++` bumps from `mount()` and `setOptions()`. `tick` is now only bumped from `onChange`, which fires naturally once the ResizeObserver delivers its first rect measurement. Verified no loop, first virtualItems read happens asynchronously after mount.
- **Files modified:** frontend/src/lib/utils/virtualizer.svelte.ts
- **Verification:** Smoke test passes in 29ms with no errors or warnings.
- **Committed in:** `87b17b6` (Task 2 commit — squashed with the fallback introduction)

**3. [Rule 3 - Blocking] Pre-existing `tsc --noEmit` errors are out of scope**
- **Found during:** Task 1 acceptance-criteria check
- **Issue:** The plan's acceptance criterion says `npx tsc --noEmit` exits 0, but the baseline (before any plan changes) already fails with 9 errors in `ui/badge/index.ts`, `ui/button/index.ts`, and `tests/helpers/schema-validator.ts`. Verified by stashing changes and re-running.
- **Fix:** Logged pre-existing errors to `deferred-items.md` and adopted `npm run check` (svelte-check) as the effective type-gate for this plan. svelte-check reports only the 3 schema-validator errors (same as baseline, all pre-existing `@types/node` gaps) — my new files are clean.
- **Files modified:** .planning/phases/13-datatable-enhancements/deferred-items.md
- **Verification:** `npm run check` → 3 errors, all pre-existing, none in files created/modified by this plan.
- **Committed in:** `57a30c6` (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 Rule 1 bug, 2 Rule 3 blocking).
**Impact on plan:** All three deviations were anticipated in the plan's text or research (Step B fallback, tsc vs svelte-check ambiguity). No scope creep. The decision-comment block in `SvelteVirtualSmoke.svelte` explicitly documents the fallback outcome for Plan 13-05.

## Issues Encountered

- **Svelte 5 + svelte-virtual store adapter is actually broken under runes in `{#each}`** — confirmed with a minimal reproducer. The smoke test is now a living regression check; if a future svelte-virtual release fixes the adapter, the plan-05 implementation can revisit the store path and delete `virtualizer.svelte.ts`.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **Plan 13-02 (backend builder):** unblocked. Can extend `DataTable`/`TableColumn`/add `Filter`/`ColumnKind` in `backend/crates/marionette/src/builders/standard.rs` independently.
- **Plan 13-03 (fetch-rows handler):** unblocked. Can read the new 120-contact seed and rely on `ActionMessage.id` echoing for the DataTable stale-discard flow.
- **Plan 13-04 (DataTableActions component):** unblocked. Can consume `sendAction` knowing it returns a string.
- **Plan 13-05 (DataTable rewrite):** the critical dependency plan. Must import `createRuneVirtualizer` from `$lib/utils/virtualizer.svelte` (NOT `createVirtualizer` from `@tanstack/svelte-virtual`), must use `FlexRender`/`createSvelteTable`/`renderSnippet`/`renderComponent` from `$lib/components/ui/data-table/index.ts`, must use `onIntersect` from `$lib/actions/viewport.ts`, and must capture the string returned by `sendAction('fetch-rows', ...)` into `lastFetchRowsActionId` for stale-patch discard.
- **Plan 13-06 (CRM list handler migration):** unblocked. Can rely on 120 seeded contacts.
- **Plan 13-07 (E2E + TextInput fix):** unblocked.

## Self-Check

Verifying claims before completion.

### Files
- `frontend/src/lib/components/ui/data-table/index.ts` — FOUND
- `frontend/src/lib/components/ui/dropdown-menu/index.ts` — FOUND
- `frontend/src/lib/components/table/SvelteVirtualSmoke.svelte` — FOUND
- `frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts` — FOUND
- `frontend/src/lib/utils/virtualizer.svelte.ts` — FOUND
- `frontend/src/lib/actions/viewport.ts` — FOUND
- `frontend/src/lib/transport/dispatcher.ts` — MODIFIED (return-type change)
- `frontend/src/lib/transport/dispatcher.test.ts` — MODIFIED (3 new tests)
- `backend/crates/crm-demo/src/seed.rs` — MODIFIED (120 contacts)
- `.planning/phases/13-datatable-enhancements/deferred-items.md` — FOUND

### Commits
- `57a30c6` — FOUND in `git log`
- `87b17b6` — FOUND in `git log`
- `95e6116` — FOUND in `git log`

### Verification
- `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/SvelteVirtualSmoke.browser-test.ts` → 1 passed
- `npx vitest --run src/lib/transport/dispatcher.test.ts` → 11 passed
- `npm test -- --run` → 61 passed across 8 test files (full frontend unit suite)
- `cargo build -p crm-demo` → OK
- `cargo test -p crm-demo` → 7 unit + 5 integration tests passed
- `npm run check` (svelte-check) → 3 pre-existing errors in tests/helpers/schema-validator.ts, 0 new errors

## Self-Check: PASSED

## Threat Flags

None — no new security-relevant surface introduced by this plan. The `onIntersect` action is a DOM-level observer with no network or auth implications; `sendAction` return-value widening is a purely local API change.

---
*Phase: 13-datatable-enhancements*
*Completed: 2026-04-11*
