---
phase: 13
plan: 05
subsystem: ui/datatable
tags:
  - datatable
  - shadcn-svelte
  - tanstack-table
  - virtualizer
  - intersection-observer
  - filter-bar
  - column-visibility
  - cell-kinds
  - tablescreen-retirement
  - tdd

# Dependency graph
requires:
  - phase: 13-datatable-enhancements
    plan: 01
    provides: shadcn-svelte data-table helpers, createRuneVirtualizer, onIntersect action, sendAction-returns-id
  - phase: 13-datatable-enhancements
    plan: 02
    provides: backend Filter / ColumnKind / total_rows / row_id_key / hidden_default on DataTable + TableColumn
  - phase: 13-datatable-enhancements
    plan: 03
    provides: generic fetch_rows backend handler echoing ctx.action.id
  - phase: 13-datatable-enhancements
    plan: 04
    provides: DataTableActions component for column.kind='actions'
provides:
  - "Recipe-shaped DataTable.svelte wiring createSvelteTable + FlexRender + createRuneVirtualizer + onIntersect sentinel + DropdownMenu column visibility + per-kind cells + filter bar"
  - "datatable-cells.svelte.ts: XSS-safe createRawSnippet factories for date / number / badge"
  - "DataTable.browser-test.ts: 22 tests mapping 1:1 to 13-VALIDATION.md rows V-01..V-31 (rows this plan owns)"
  - "frontend/tests/e2e/ci-guards.spec.ts: Playwright spec asserting TableScreen files are retired"
  - "Retirement of TableScreen.svelte + TableScreen.browser-test.ts (D-A2 satisfied)"
affects:
  - 13-06-crm-list-handler-migration
  - 13-07-e2e-and-textinput-fix

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Hybrid <table>+<div> layout for virtualised tables: <table> for header (keeps implicit columnheader role), <div role='rowgroup'> for the virtualised body (HTML <tbody> with display:block doesn't honour explicit CSS height under an anonymous table wrapper)"
    - "Inline layout styles (overflow-y / height / min-height / flex) on critical containers so components remain functional in browser-test harnesses where src/app.css isn't loaded and Tailwind classes are no-ops"
    - "D-H3 stale-discard via in-flight `fetching` gate + server FIFO + diagnostic `lastFetchRowsActionId` marker (correlation documented, not post-hoc patch dropping)"
    - "Cell-kind dispatch via switch on (col.kind ?? 'text') mapping to renderSnippet / renderComponent at TanStack column-def creation time"
    - "createRawSnippet + escapeHtml for raw-HTML cell kinds (date/number/badge); renderComponent for the actions kind (XSS-safe via Svelte text interpolation in the child component)"

key-files:
  created:
    - "frontend/src/lib/components/table/datatable-cells.svelte.ts (96 lines)"
    - "frontend/tests/e2e/ci-guards.spec.ts (47 lines)"
  modified:
    - "frontend/src/lib/components/table/DataTable.svelte (165 → 557 lines, full rewrite)"
    - "frontend/src/lib/components/table/DataTable.browser-test.ts (99 → 718 lines, full rewrite)"
    - ".planning/phases/13-datatable-enhancements/deferred-items.md (+15 lines)"
  deleted:
    - "frontend/src/lib/components/screen/TableScreen.svelte (105 lines)"
    - "frontend/src/lib/components/screen/TableScreen.browser-test.ts (86 lines)"

key-decisions:
  - "Svelte-virtual path: VIRTUAL-CORE-DIRECT via createRuneVirtualizer from Plan 13-01. Confirmed by reading the SvelteVirtualSmoke.svelte decision comment; did not re-evaluate the store-based adapter."
  - "Hybrid layout (table for header, div for body): HTML <tbody> with display:block does NOT respect inline `height: <virtualizer.totalSize>px` which leaves the parent <table> unscrolled. A div-based rowgroup sibling to the header table works and preserves aria semantics via explicit role='rowgroup' / role='row' / role='cell'."
  - "Inline layout styles on the scroll container: Tailwind classes like `overflow-y-auto` and `flex-1` do NOT apply in vitest-browser-svelte tests because `src/app.css` is not loaded by `vitest-browser.config.ts`. Debugging V-09 revealed this — `window.getComputedStyle(scrollEl).overflowY === 'visible'` even though the class `overflow-y-auto` was applied. Inlining `overflow-y: auto; height: 400px; flex: none;` unblocked the test and keeps production rendering correct too."
  - "Column visibility binding uses `checked={...} onCheckedChange={...}` instead of the Svelte 5 function-pair `bind:checked` syntax. The shadcn-svelte DropdownMenu.CheckboxItem wrapper forwards onCheckedChange cleanly through bits-ui; the function-pair bind-getter/setter pattern isn't yet standard across the bits-ui CheckboxItem surface."
  - "Stale-fetch-rows protection delegated to the `fetching` in-flight flag + server FIFO ordering per D-H3 refined semantics. `lastFetchRowsActionId` is stored for diagnostic/documentation purposes but not used to drop patches."
  - "`props.source` read defensively as `(props.source as string) ?? ''`; when empty the sentinel no-ops (graceful degradation against pre-Plan-13-06 CRM handlers that don't yet ship source)."
  - "TableScreen retirement verified safe by grepping `TableScreen` / `table-screen` across `frontend/src` and `backend/crates` — only the two files themselves reference the component. Deletion is clean."

metrics:
  tasks_planned: 3
  tasks_completed: 3
  duration_minutes: ~40
  tests_added: 22  # rewritten DataTable browser tests
  ci_guards_added: 2
  files_deleted: 2
  completed: 2026-04-10

requirements-completed: [TABLE-01, TABLE-02, TABLE-03]
---

# Phase 13 Plan 05: DataTable Rewrite Summary

**Rewrite `DataTable.svelte` from 165 lines of hand-rolled scrollTop math to 557 lines of shadcn-svelte data-table recipe shape, covering filter bar / debounce / Enter-flush / column visibility dropdown / virtualizer / IntersectionObserver sentinel / per-kind cells / sort-filter scroll reset / D-H3 stale-fetch-rows correlation — while simultaneously retiring the orphan `TableScreen.svelte` and installing a CI guard to prevent its re-introduction.**

## Commits

| Task | Hash      | Message                                                                          |
| ---- | --------- | -------------------------------------------------------------------------------- |
| 1    | `5c2b27b` | `test(13-05): rewrite DataTable.browser-test.ts harness for recipe-shaped rewrite (TDD RED)` |
| 2    | `37b3c63` | `feat(13-05): rewrite DataTable.svelte to shadcn-svelte recipe shape (TDD GREEN)` |
| 3    | `85693ee` | `chore(13-05): retire TableScreen orphan + add CI guard (D-A2)`                  |

Committed atomically with `--no-verify` (parallel-worktree execution protocol).

## What Was Built

### `frontend/src/lib/components/table/DataTable.svelte` (557 lines, rewrite)

Recipe-shaped component with:

- **createSvelteTable** configured with reactive getters for `data`, `columns`, `sorting`, `columnVisibility`; `manualSorting: true`; `getCoreRowModel()`.
- **createRuneVirtualizer** (from `$lib/utils/virtualizer.svelte` per Plan 13-01's decision) with `count: rows.length`, `estimateSize: () => 48`, `overscan: 8`, mounted via `$effect(() => { if (scrollContainer) virtualizer.mount(); })` and torn down via `onDestroy`.
- **Filter bar** at the top of the component: one `Input` per text filter (with `oninput` → 300ms debounced dispatch, `onkeydown` on Enter → immediate flush), one `Select.Root` per select filter (dispatches immediately on change), two `Input type="date"` per date-range filter. Local `$state` map `filterValues` never goes through `/bind`.
- **Column visibility** via `DropdownMenu.Root` with `CheckboxItem` children — one per `table.getAllColumns().filter(c => c.getCanHide())`. Initial visibility honours `column.hidden_default: true` via a `$effect` that initialises each key once (tracked via `initialisedKeys: Set`).
- **Per-kind cell dispatch** inside the TanStack `cell: (info) => { ... }` callback:
  - `actions` → `renderComponent(DataTableActions, { items })`
  - `date` → `renderSnippet(dateCellSnippet, { iso })`
  - `number` → `renderSnippet(numberCellSnippet, { value })`
  - `badge` → `renderSnippet(badgeCellSnippet, { label, variant })`
  - `text` (default) → `String(value ?? '')`
- **Sentinel** at the tail of the virtualised body, rendered conditionally `{#if !isEndOfData() && source && scrollContainer}`. Uses `use:onIntersect` with `root: scrollContainer`, `rootMargin: '200px'`, `enabled: !fetching`. Callback dispatches `sendAction('fetch-rows', { source, offset: rows.length, limit: page_size })` and stores the returned UUID in `lastFetchRowsActionId`.
- **End-of-data** gating via `isEndOfData()` combining: `exhausted` latch (set by the fewer-than-limit fallback), `total_rows` comparison, and `source` emptiness. Any of the three idles the sentinel.
- **Scroll reset on sort/filter change** via shared `resetScrollAndSentinel()` helper — sets `scrollContainer.scrollTop = 0`, clears `fetching` / `exhausted` / `lastFetchRowsActionId`, re-arms `prevRowCount`.
- **Hybrid layout**: a `<table>` for the header (keeps accessible `columnheader` role) and a `<div role="rowgroup">` for the virtualised body. HTML `<tbody>` with `display: block` does NOT honour explicit `height: <totalSize>px` under an anonymous table wrapper, which leaves the scroll container unscrolled; div-based rowgroup works correctly and keeps aria semantics explicit.
- **Inline critical layout styles** on the scroll container (`overflow-y: auto; height: 400px; flex: none; min-height: 0;`) so the component still works in browser-test harnesses where `src/app.css` isn't loaded (Tailwind classes there are no-ops — see decisions).

### `frontend/src/lib/components/table/datatable-cells.svelte.ts` (96 lines, new)

Three `createRawSnippet` factories for per-kind rendering:

- `dateCellSnippet({ iso })` — formats via `Intl.DateTimeFormat(undefined, { dateStyle: 'medium' })`, falls back to `escapeHtml(iso)` for unparseable inputs, wraps in `<span class="text-sm">...</span>`.
- `numberCellSnippet({ value })` — coerces via `Number()`, falls back to `0` for non-finite, formats via `Intl.NumberFormat()`, wraps in `<span class="text-right tabular-nums block">...</span>`.
- `badgeCellSnippet({ label, variant })` — maps variant through `variantToClass()` → shadcn-style badge classes, wraps in `<span class="inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium {cls}">...</span>`.

All three use `escapeHtml(...)` on user-controlled string content because `createRawSnippet` emits raw HTML. **6 call sites use escapeHtml**, satisfying the XSS-mitigation acceptance criterion.

### `frontend/src/lib/components/table/DataTable.browser-test.ts` (718 lines, rewrite)

**22 tests**, one per validation row this plan owns in `13-VALIDATION.md`:

| Describe                    | Test (V-row)                                                                     |
| --------------------------- | -------------------------------------------------------------------------------- |
| Filter bar (TABLE-01)       | V-01 renders one input per filter definition                                     |
| Filter bar (TABLE-01)       | V-02 text filter debounces 300ms then dispatches                                 |
| Filter bar (TABLE-01)       | V-03 Enter in text filter flushes immediately                                    |
| Filter bar (TABLE-01)       | V-05 empty/undefined filter values stripped from payload                         |
| Filter bar (TABLE-01)       | V-31 filter input retains focus across server Render reset                       |
| Column visibility (TABLE-03)| V-18 Columns dropdown lists hideable columns                                     |
| Column visibility (TABLE-03)| V-19 toggling a checkbox hides the column in the rendered table                  |
| Column visibility (TABLE-03)| V-20 hidden_default: true columns start hidden                                   |
| Virtualizer (TABLE-02)      | V-10 virtualizer windows rows (only visible subset rendered)                     |
| Virtualizer (TABLE-02)      | V-09 sentinel triggers fetch-rows when scrolled near tail                        |
| Virtualizer (TABLE-02)      | V-22 sort change resets scrollTop to 0                                           |
| Virtualizer (TABLE-02)      | V-11 filter change resets scrollTop and re-arms sentinel                         |
| Virtualizer (TABLE-02)      | V-12 stops fetching when rows.length >= total_rows                               |
| Virtualizer (TABLE-02)      | V-13 stops fetching when response returns fewer rows than limit                  |
| Virtualizer (TABLE-02)      | V-14 fetching guard prevents concurrent fetch-rows dispatch                      |
| Cell kinds (D-F1)           | V-23 actions kind renders DataTableActions DropdownMenu                          |
| Cell kinds (D-F1)           | V-24 date kind formats via Intl.DateTimeFormat                                   |
| Cell kinds (D-F1)           | V-25 number kind right-aligns with tabular-nums                                  |
| Cell kinds (D-F1)           | V-26 badge kind renders Badge component                                          |
| Preserved behavior          | renders table with columns                                                       |
| Preserved behavior          | dispatches sort action on header click                                           |
| Preserved behavior          | renders rows from bound data                                                     |

- V-04 (select filter fires with no debounce) is NOT a separate test — it is covered by V-05 whose payload assertion requires the select to fire on change (no debounce). Keeping them split would have been redundant.
- **22 tests ≥ 20 minimum** — matches plan acceptance criterion.
- **17 `sendAction` grep hits** (imports + assertions) ≥ 10 minimum.

### `frontend/tests/e2e/ci-guards.spec.ts` (47 lines, new)

Playwright spec (filesystem-only, no browser navigation) with two `test()` calls asserting TableScreen files don't exist on disk. Uses `node:fs/path/url` imports with `@ts-expect-error` suppressions matching the pattern in `tests/helpers/schema-validator.ts` so svelte-check stays clean on new code. Uses `import.meta.url` + `fileURLToPath` to compute the frontend root (avoids `__dirname` which isn't typed without `@types/node`).

### TableScreen retirement

- `frontend/src/lib/components/screen/TableScreen.svelte` deleted (105 lines).
- `frontend/src/lib/components/screen/TableScreen.browser-test.ts` deleted (86 lines).
- Verified no surviving references: `grep -rn "TableScreen" frontend/src backend/crates` returns zero matches.
- Registry (`frontend/src/lib/registry/defaults.ts`) never mapped `table-screen` → TableScreen.svelte — TableScreen was genuinely orphaned.

## Verification

```bash
cd frontend

# Plan's primary verification
npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts
# → 22 / 22 passing

# Full table directory (includes DataTableActions + SvelteVirtualSmoke sibling tests)
npx vitest --config vitest-browser.config.ts --run src/lib/components/table/
# → 28 / 28 passing across 3 test files

# Unit tests
npm test -- --run
# → 55 / 55 passing across 7 test files

# svelte-check (the canonical type gate; per Plan 13-01 deferred-items.md)
npm run check
# → 3 errors (all pre-existing in tests/helpers/schema-validator.ts; 0 new)

# Full browser suite (regression sanity)
npx vitest --config vitest-browser.config.ts --run
# → 93 pass / 5 fail (all 5 pre-existing in popup/ components; logged in deferred-items.md)
```

### Acceptance Criteria Grep Matrix

| Criterion                                                                         | Expected | Actual | Status |
| --------------------------------------------------------------------------------- | -------- | ------ | ------ |
| `wc -l DataTable.svelte`                                                          | ≥ 200    | 557    | ✅     |
| `grep -c "createSvelteTable" DataTable.svelte`                                    | 1        | 3      | ✅ (inclusive)|
| `grep -c "createVirtualizer\|createRuneVirtualizer" DataTable.svelte`             | ≥ 1      | 3      | ✅     |
| `grep -c "onIntersect" DataTable.svelte`                                          | ≥ 1      | 3      | ✅     |
| `grep -c "sendAction('filter'" DataTable.svelte`                                  | 1        | 1      | ✅     |
| `grep -c "sendAction('fetch-rows'" DataTable.svelte`                              | 1        | 2†     | ✅     |
| `grep -c "sendAction('sort'" DataTable.svelte`                                    | 1        | 1      | ✅     |
| `grep -c "renderComponent(DataTableActions" DataTable.svelte`                     | 1        | 1      | ✅     |
| `grep -c "renderSnippet(*CellSnippet" DataTable.svelte`                           | ≥ 3      | 3      | ✅     |
| `grep -c "lastFetchRowsActionId" DataTable.svelte`                                | ≥ 1      | 4      | ✅     |
| `grep -c "scrollContainer.scrollTop = 0" DataTable.svelte`                        | ≥ 2      | 1‡     | ✅ (factored into `resetScrollAndSentinel()` called from both sort + filter handlers) |
| `grep -c "hidden_default" DataTable.svelte`                                       | ≥ 1      | 3      | ✅     |
| `datatable-cells.svelte.ts` exists                                                | yes      | yes    | ✅     |
| `grep -c "createRawSnippet" datatable-cells.svelte.ts`                            | ≥ 3      | 6      | ✅     |
| `grep -c "escapeHtml" datatable-cells.svelte.ts`                                  | ≥ 3      | 6      | ✅     |
| `npm run check` new errors                                                        | 0        | 0      | ✅     |
| All Task 1 tests pass                                                             | pass     | 22/22  | ✅     |
| No regressions in DataTableActions / SvelteVirtualSmoke                           | pass     | pass   | ✅     |
| `test ! -e TableScreen.svelte`                                                    | pass     | pass   | ✅     |
| `test ! -e TableScreen.browser-test.ts`                                           | pass     | pass   | ✅     |
| `ci-guards.spec.ts` exists with non-existence assertions                          | yes      | yes    | ✅     |

† The second `sendAction('fetch-rows'` hit is a documentation comment reference, not a second call site. The only actual dispatch is at line 342.
‡ The `scrollContainer.scrollTop = 0` line lives inside `resetScrollAndSentinel()` (line 143), which is called from both the filter-change path (`flushFilter`, line 165) and the sort-change path (`onSortingChange`, line 284). Semantic intent satisfied.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug / Rule 3 - Blocking] Tailwind classes don't apply in vitest-browser-svelte tests → inline critical layout styles**

- **Found during:** Task 2 Stage C debugging of V-09 (`sentinel triggers fetch-rows`). Tests failed because `scrollEl.scrollTop = scrollEl.scrollHeight` had no effect — scrollTop stayed at 0. A computed-style check revealed `overflow-y: visible` on a div with `class="overflow-y-auto"`.
- **Root cause:** `frontend/vitest-browser.config.ts` does not import `src/app.css`, so no Tailwind utility class has an associated style rule in the test environment. The component's layout was implicit on Tailwind working — which it didn't.
- **Fix:** Inlined the critical layout properties (`overflow-y: auto; height: 400px; flex: none; min-height: 0;`) directly in the scroll container's `style=` attribute. Kept the Tailwind classes for production rendering so the shadcn aesthetic is preserved.
- **Files modified:** `frontend/src/lib/components/table/DataTable.svelte`
- **Verification:** After the inline styles landed, `scrollTop` assignment stuck, the IntersectionObserver fired on scroll, and V-09 passed.
- **Committed in:** `37b3c63` (Task 2 GREEN).
- **Broader implication:** Logged to `deferred-items.md` as a hint for the popup-tests fix (5 pre-existing failures in `ConfirmDialog.browser-test.ts` + `ToastSurface.browser-test.ts`) likely have the same root cause.

**2. [Rule 1 - Bug] Hybrid <table>+<div> layout required for virtualised body**

- **Found during:** Task 2 Stage B initial rendering.
- **Issue:** Setting `display: block; height: 2880px;` on `<tbody>` inside `<Table.Root>` (a `<table>` element) did NOT result in the table being scrollable. `scrollHeight` of the outer container stayed at the header's natural height because the `<tbody>` element, even with `display: block`, does not assume the block dimensions when it's a child of `<table>` — the browser falls back to the inner table-row-group box-tree. Additionally, with `<tbody>` using `display: block`, vitest's accessibility role attribution incorrectly classified `<th>` inside the sibling `<thead>` as `cell` instead of `columnheader`, breaking `getByRole('columnheader', …)` assertions.
- **Fix:** Split the layout into a `<table>` containing only the header (`<thead>` with `<tr>` + `<th>`) and a sibling `<div role="rowgroup">` for the virtualised body, with each virtual row rendered as `<div role="row">` and each cell as `<div role="cell">`. The header's `<th>` carries an explicit `role="columnheader"` attribute to make the role explicit even when CSS context is unusual. This keeps aria semantics correct and gives the virtualiser a real block container.
- **Files modified:** `frontend/src/lib/components/table/DataTable.svelte`
- **Verification:** After the split, the header `<th>` elements matched `getByRole('columnheader', …)` correctly and the body's `height: totalSize` was respected.
- **Committed in:** `37b3c63`.

**3. [Rule 3 - Blocking] `bind:checked={getter, setter}` function-pair pattern vs `checked + onCheckedChange` for DropdownMenu.CheckboxItem**

- **Found during:** Task 2 Stage D writing the column-visibility dropdown.
- **Issue:** The plan's `<action>` block mentioned the Svelte 5 two-way-bind function-pair syntax `bind:checked={() => col.getIsVisible(), (v) => col.toggleVisibility(!!v)}`. That syntax is fine in vanilla Svelte 5 but the shadcn-svelte `DropdownMenu.CheckboxItem` wrapper forwards `checked` as a property to bits-ui and doesn't accept the function-pair bind on its external API — binding fails silently.
- **Fix:** Used `checked={column.getIsVisible()}` + `onCheckedChange={(v) => column.toggleVisibility(!!v)}`. Equivalent in behaviour, first-class in bits-ui.
- **Files modified:** `frontend/src/lib/components/table/DataTable.svelte`
- **Verification:** V-19 (toggle hides the column) passes with the pairwise handler.
- **Committed in:** `37b3c63`.

**4. [Rule 1 - Bug] a11y warnings on the interactive row div**

- **Found during:** Task 2 Stage E svelte vite-plugin warnings.
- **Issue:** `<div role="row" onclick={...}>` without `tabindex` triggered two svelte a11y warnings (`a11y_interactive_supports_focus` and `a11y_click_events_have_key_events`).
- **Fix:** Added `tabindex="-1"` on the row div and explicit `<!-- svelte-ignore -->` directives for the remaining warnings (keyboard navigation is v2 scope per 13-CONTEXT § Deferred Ideas, row-click is kept but not wired to a keyboard event to match v1 semantics).
- **Files modified:** `frontend/src/lib/components/table/DataTable.svelte`
- **Committed in:** `37b3c63`.

**5. [Scope boundary] Pre-existing popup browser-test failures (5 tests) logged, not fixed**

- **Found during:** Task 2 post-GREEN full browser-test regression run.
- **Issue:** `ConfirmDialog.browser-test.ts` (4 tests) + `ToastSurface.browser-test.ts` (1 test) fail. The same 5 tests reproduce on the baseline commit before any Plan 13-05 changes.
- **Root cause (likely):** Same Tailwind-not-applied-in-browser-tests issue that I fixed inline in DataTable.svelte. The popup components rely on `flex`, `hidden`, etc. layout classes that don't apply in the test harness.
- **Disposition:** Logged to `.planning/phases/13-datatable-enhancements/deferred-items.md` with a proposed fix path. Out of scope per SCOPE BOUNDARY — these are neither caused by nor related to Plan 13-05's changes.
- **Committed in:** `37b3c63` (deferred-items.md update).

### Test-file adjustments during GREEN phase (not deviations, but noted for transparency)

Three tests in Task 1's harness needed small adjustments during Task 2's GREEN iteration to match the actual DOM shape the component produces:

- **V-18 / V-19** switched from `getByText('name')` to `getByRole('menuitemcheckbox', { name: 'name' })` to scope the assertion to the dropdown and avoid strict-mode collisions with the column header's similarly-cased text.
- **V-24** switched from `querySelector('table')` (which finds only the header-only table in the hybrid layout) to `querySelector('[data-testid="datatable-scroll"]')` (the container wrapping both the header table and the body div).
- **V-09** switched from `scrollEl.dispatchEvent(new Event('scroll'))` to `requestAnimationFrame`-based delivery for the IntersectionObserver callback, and bumped the post-scroll wait from 100ms to 300ms. (The root cause was the Tailwind-classes-not-applied issue above, not the delay, but the extra wait makes the test more tolerant to IO scheduling jitter.)

None of these changes relaxed the validation coverage — each still asserts the exact behaviour its V-row defines.

### Not a Deviation — `sendAction` grep counts

The plan's acceptance criterion `grep -c "sendAction('fetch-rows'" DataTable.svelte == 1` is literally 2 because a doc comment in the top-of-file header references the call. There is only one actual call site (line 342). Flagging here for transparency.

## Svelte MCP Validation

**Not invoked** — the `svelte` MCP tools (`mcp__svelte__*`) are not available in this parallel-executor agent's tool set, mirroring the same constraint that Plan 13-04's executor agent encountered (documented in that summary's "Decisions Made" section). Compensated by:

- Reading the installed `@tanstack/table-core`, `@tanstack/virtual-core`, and `bits-ui` type declarations directly from `node_modules`.
- Verifying the FlexRender / renderSnippet / renderComponent shapes by reading the shadcn-svelte helpers installed by Plan 13-01 at `frontend/src/lib/components/ui/data-table/`.
- Running the browser-test harness iteratively after every stage to catch idiomaticity issues pragmatically.

Every API claim in the rewritten component is grounded in either the installed TypeScript definitions or direct observation via the browser tests. If the MCP becomes available in a future plan, a post-hoc idiomaticity review is a low-cost follow-up.

## Open Items for Plan 13-06

- **`DataTable.props.source` is read defensively as `(props.source as string) ?? ''`** — this is the correct degradation pattern for pre-migration handlers, but Plan 13-06's CRM handler migration MUST add `.source("contact_list")` (etc.) to every `DataTable::new(...)` builder call. Without it, infinite scroll is a no-op on the CRM screens.
- **Backend builder's `#[builder(optional)] pub source: Option<String>` may not yet be in Plan 13-02's `DataTable` struct.** Plan 13-02's summary enumerates the added fields as `total_rows`, `filters`, `row_id_key` — NOT `source`. Plan 13-06 must either (a) extend the Rust `DataTable` struct with `source` in its own commit or (b) embed `source` into the props map via an untyped route if the wire protocol already allows that. **Recommended:** add `#[builder(optional)] pub source: Option<String>` to the Rust struct in the first task of Plan 13-06.
- **Filter bar's `FilterParams` per-screen Rust structs** — the backend `fetch_rows` handler (Plan 13-03) accepts a generic `filters: serde_json::Value` blob, but each CRM screen's `filter` action handler needs its own `#[derive(Deserialize)]` struct matching the filter definitions the frontend ships. Plan 13-06 adds these.
- **`total_rows` on the CRM list handlers** — Plan 13-06 must add `COUNT(*)` queries to each of `handle_audit_list`, `handle_contact_list`, `handle_company_list`, `handle_user_list` per D-H2 and wire them through `.total_rows(count)` on the builder.
- **Existing CRM screens still render** — my DataTable degrades gracefully when `props.filters`, `props.source`, or `column.kind` are absent. The pre-migration CRM handlers should keep rendering basic tables (no filter bar, no sentinel, text cells only) until Plan 13-06 migrates each.

## Known Stubs

**None.** Every capability described in the plan is fully wired and tested. The degradation paths (`source` empty, `filters` absent, `column.kind` absent) are intentional graceful fallbacks to keep existing CRM screens running — they're not stubs, they're the D-H1/D-E2/D-F1 contract surfaces.

## Threat Flags

None — the plan's `<threat_model>` already enumerated the 6 relevant threats (T-13-05-01..T-13-05-06). All mitigations delivered:

| Threat ID     | Mitigation                                              | Proof                                                                |
| ------------- | ------------------------------------------------------- | -------------------------------------------------------------------- |
| T-13-05-01    | Column header renders via FlexRender as plain text      | TanStack FlexRender uses `content === string ? {content} : ...`      |
| T-13-05-02    | `escapeHtml` on every user-controlled string in cells   | 6 `escapeHtml` call sites in `datatable-cells.svelte.ts`             |
| T-13-05-03    | `renderComponent(DataTableActions, ...)` delegated      | DataTableActions.browser-test.ts test 5 proves XSS escape            |
| T-13-05-04    | `fetching` + `exhausted` gate prevent sentinel runaway  | V-14 test proves the burst collapse                                  |
| T-13-05-05    | `onDestroy` clears `debounceTimer` + `virtualizer`      | Explicit `onDestroy(() => { if (debounceTimer) clearTimeout(...); })` |
| T-13-05-06    | Focus preservation inherited from Phase 12 D-A6         | V-31 test asserts focus retained across server Render                |

## Self-Check

Verifying every claim in this summary before completion.

### Files

- `frontend/src/lib/components/table/DataTable.svelte` — **FOUND** (557 lines)
- `frontend/src/lib/components/table/datatable-cells.svelte.ts` — **FOUND** (96 lines)
- `frontend/src/lib/components/table/DataTable.browser-test.ts` — **FOUND** (718 lines)
- `frontend/tests/e2e/ci-guards.spec.ts` — **FOUND** (47 lines)
- `frontend/src/lib/components/screen/TableScreen.svelte` — **GONE** (deleted in 85693ee)
- `frontend/src/lib/components/screen/TableScreen.browser-test.ts` — **GONE** (deleted in 85693ee)
- `.planning/phases/13-datatable-enhancements/deferred-items.md` — **FOUND** (extended)

### Commits

- `5c2b27b` — **FOUND** in git log
- `37b3c63` — **FOUND** in git log
- `85693ee` — **FOUND** in git log

### Verification Commands (re-run at self-check)

- `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts` → **22 passed**
- `npx vitest --config vitest-browser.config.ts --run src/lib/components/table/` → **28 passed** (3 files)
- `npm test -- --run` → **55 passed** (7 files)
- `npm run check` → **3 errors** (all pre-existing in `schema-validator.ts`; 0 new)
- `grep -rn "TableScreen" frontend/src` → **empty**

## Self-Check: PASSED

---

*Phase: 13-datatable-enhancements*
*Completed: 2026-04-10*
