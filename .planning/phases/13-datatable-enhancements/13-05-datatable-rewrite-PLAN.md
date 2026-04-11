---
phase: 13
plan: 05
type: execute
wave: 3
depends_on: [13-01, 13-02, 13-04]
files_modified:
  - frontend/src/lib/components/table/DataTable.svelte
  - frontend/src/lib/components/table/DataTable.browser-test.ts
  - frontend/src/lib/components/table/datatable-cells.svelte.ts
  - frontend/src/lib/components/screen/TableScreen.svelte
  - frontend/src/lib/components/screen/TableScreen.browser-test.ts
  - frontend/tests/e2e/ci-guards.spec.ts
autonomous: true
requirements: [TABLE-01, TABLE-02, TABLE-03]
must_haves:
  truths:
    - "DataTable renders a filter bar at the top with one shadcn primitive per filter definition (text → Input, select → Select, date-range → two date Inputs)"
    - "Text-filter input is debounced 300ms; pressing Enter flushes immediately; selects fire on change with no debounce"
    - "Filter dispatch uses a single `sendAction('filter', payload)` call with empty/undefined values stripped from payload"
    - "DataTable owns all filter state in local `$state` — no `/bind` round-trip through the data store"
    - "Column visibility 'Columns' DropdownMenu lives in DataTable's top region and toggles columns via TanStack's `column.getCanHide()` + `column.toggleVisibility()`"
    - "Initial column visibility honors `column.hidden_default: true` — those columns start hidden"
    - "Column visibility state is per-mount only (no persistence)"
    - "Rows render via `createSvelteTable` + `FlexRender` + `@tanstack/svelte-virtual` virtualizer (OR the virtual-core-direct fallback recorded in Plan 01's smoke test)"
    - "IntersectionObserver sentinel at the virtualizer tail (via `use:onIntersect` from `$lib/actions/viewport`) dispatches `sendAction('fetch-rows', { source, offset, limit })`"
    - "`fetch-rows` dispatch stores the returned action id in `lastFetchRowsActionId` local state; patches whose echoed `id` doesn't match are NOT applied (but the data store handles this naturally since the id-mismatch drop lives in a data-store patch filter — see implementation note)"
    - "When `total_rows` prop is set, the sentinel idles once `rows.length >= total_rows`"
    - "When `total_rows` is absent, the sentinel idles once a fetch-rows response returns fewer rows than the requested `limit`"
    - "Sort change fires `sendAction('sort', { column, direction })` and resets scrollTop to 0"
    - "Filter change (via `flushFilter`) also resets scrollTop to 0 and re-arms the sentinel"
    - "column.kind='actions' renders `DataTableActions` (Plan 04 component) via `renderComponent`"
    - "column.kind='badge' / 'date' / 'number' each render via their per-kind snippet (createRawSnippet + renderSnippet)"
    - "Old `frontend/src/lib/components/screen/TableScreen.svelte` and its browser-test are DELETED"
    - "A CI guard asserts those files do not exist"
  artifacts:
    - path: "frontend/src/lib/components/table/DataTable.svelte"
      provides: "Rewritten DataTable with filter bar, virtualizer, sentinel, column visibility, per-kind cells"
      min_lines: 200
    - path: "frontend/src/lib/components/table/datatable-cells.svelte.ts"
      provides: "createRawSnippet helpers for date/number/badge"
    - path: "frontend/src/lib/components/table/DataTable.browser-test.ts"
      provides: "Rewritten test suite covering filter bar, debounce, column visibility, sentinel, per-kind cells, stale discard"
  key_links:
    - from: "DataTable.svelte filter bar"
      to: "sendAction('filter', ...)"
      via: "flushFilter()"
      pattern: "sendAction\\(['\"]filter['\"]"
    - from: "DataTable.svelte sentinel"
      to: "sendAction('fetch-rows', ...)"
      via: "onIntersect callback"
      pattern: "sendAction\\(['\"]fetch-rows['\"]"
    - from: "DataTable.svelte actions column"
      to: "DataTableActions component"
      via: "renderComponent"
      pattern: "renderComponent\\(DataTableActions"
    - from: "DataTable.svelte virtualizer"
      to: "createVirtualizer OR createRuneVirtualizer (Plan 01's decision)"
      via: "import from @tanstack/svelte-virtual OR $lib/utils/virtualizer.svelte"
      pattern: "createVirtualizer|createRuneVirtualizer"
---

<objective>
Rewrite `DataTable.svelte` from the current 165-line hand-rolled scrollTop-math implementation to the canonical shadcn-svelte data-table recipe shape with every Phase 13 capability wired in: filter bar, column visibility, virtualizer + IntersectionObserver sentinel, per-kind cell rendering, sort reset semantics, and stale-fetch-rows discard via action-id correlation. Delete the orphan `TableScreen.svelte` in the same plan because it's the only file whose existence is a blocker for the phase's "retire TableScreen" decision (D-A2).

Purpose: This is the centerpiece plan. Every success criterion for Phase 13 is embodied by this component. Plan 06 (CRM handler migration) depends on the new JSON prop shape this plan consumes.

Output: Rewritten `DataTable.svelte`, new `datatable-cells.svelte.ts` helper module, rewritten `DataTable.browser-test.ts` covering ~20 assertions per 13-VALIDATION.md verification map, deleted `TableScreen.svelte` + its test, and a CI guard that asserts TableScreen cannot be re-introduced.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/phases/13-datatable-enhancements/13-VALIDATION.md
@.planning/phases/13-datatable-enhancements/13-01-scaffolding-PLAN.md
@.planning/phases/13-datatable-enhancements/13-02-backend-builder-PLAN.md
@.planning/phases/13-datatable-enhancements/13-04-datatable-actions-component-PLAN.md
@.planning/codebase/CONVENTIONS.md
@.planning/codebase/TESTING.md
@frontend/src/lib/components/table/DataTable.svelte
@frontend/src/lib/components/table/DataTable.browser-test.ts
@frontend/src/lib/components/table/SvelteVirtualSmoke.svelte
@frontend/src/lib/components/table/DataTableActions.svelte
@frontend/src/lib/transport/dispatcher.ts
@frontend/src/lib/actions/viewport.ts
@frontend/src/lib/store/data.svelte.ts

<interfaces>
<!-- Executor MUST read all of these before touching a single line. -->

Serialized DataTable props shape (from Plan 02's `data_table_phase13_example_serializes_correctly` test):
```json
{
  "type": "data-table",
  "props": {
    "columns": [
      { "key": "name", "label": "Name", "sortable": true },
      { "key": "email", "label": "Email" },
      { "key": "created", "label": "Created", "kind": "date", "sortable": true },
      { "key": "actions", "label": "", "kind": "actions" },
      { "key": "internal_id", "label": "ID", "hidden_default": true }
    ],
    "filters": [
      { "id": "search", "kind": "text", "label": "Search", "placeholder": "Filter contacts..." },
      { "id": "company", "kind": "select", "label": "Company", "options": [{"value":"","label":"All"},{"value":"1","label":"Acme"}] },
      { "id": "created", "kind": "date-range", "label": "Created date" }
    ],
    "total_rows": 237,
    "row_id_key": "id",
    "page_size": 50,
    "source": "contact_list"
  },
  "bind": "/contacts"
}
```

IMPORTANT: `source` is NOT yet in Plan 02's Rust `DataTable` struct. It's a required extra field the frontend reads to pass to the `fetch-rows` backend handler (per D-H1). Plan 06 (CRM migration) ships it via `.source("contact_list")` — but Plan 02's `DataTable` struct ALREADY extends via `row_id_key`. Either Plan 02 needs a late `source` field addition, or Plan 06 passes it some other way.

**Resolution for this plan:** Read `props.source` defensively (`(props.source as string) ?? ''`). If Plan 02 didn't add it, Plan 06 must add it — leave a comment in the DataTable code noting the dependency.

**CRITICAL:** Update Plan 02 post-hoc if `source` is missing: add a `#[builder(optional)] pub source: Option<String>` field to the DataTable struct. Do this at the top of Task 1 if the grep shows it's missing.

Current `DataTable.svelte` reads `props.totalRows` (camelCase) at line 31 — this is WRONG. Plan 02 serializes `total_rows` snake_case. This plan fixes that by switching to `props.total_rows`.

Existing `TableColumn`-like read pattern in current DataTable.svelte:
```typescript
let columns = $derived((props.columns as ColumnDef[]) ?? []);
// columns[i].key, columns[i].label, columns[i].sortable
```

Plan 01 extended `sendAction` to return `string`:
```typescript
const id: string = sendAction('fetch-rows', { source, offset, limit });
```

Plan 01 created `onIntersect` action at `$lib/actions/viewport.ts`:
```typescript
export interface OnIntersectOptions {
  onEnter: () => void;
  root?: Element | null;
  rootMargin?: string;
  threshold?: number | number[];
  enabled?: boolean;
}
export function onIntersect(node: Element, options: OnIntersectOptions);
```

Plan 04 created `DataTableActions` component at `$lib/components/table/DataTableActions.svelte`:
```svelte
<script lang="ts">
  export interface ActionItem { label: string; action: ComponentAction; }
  let { items = [] }: { items?: ActionItem[] } = $props();
</script>
```

Plan 01's SvelteVirtualSmoke.svelte records whether the store-based `createVirtualizer` from `@tanstack/svelte-virtual` works on Svelte 5, OR whether to use the virtual-core-direct fallback at `$lib/utils/virtualizer.svelte.ts`. READ the decision comment at the top of `SvelteVirtualSmoke.svelte` and use the same path in DataTable.

Existing `getData(surface, bind)` pattern (from `data.svelte.ts`):
```typescript
let rawData = $derived(bind ? (getData(surface, bind) as Record<string, Record<string, unknown>>) ?? {} : {});
let rows = $derived(Object.entries(rawData));
```

TanStack Table imports (from Plan 01's shadcn-svelte install):
```typescript
import { createSvelteTable, FlexRender, renderSnippet, renderComponent } from '$lib/components/ui/data-table/index.js';
import { getCoreRowModel, type ColumnDef, type SortingState, type VisibilityState } from '@tanstack/table-core';
import { createVirtualizer } from '@tanstack/svelte-virtual';  // OR the fallback
```

Existing Table primitive imports:
```typescript
import * as Table from '$lib/components/ui/table';
import { Input } from '$lib/components/ui/input';
import { Button } from '$lib/components/ui/button';
import * as Select from '$lib/components/ui/select';
import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
import { Badge } from '$lib/components/ui/badge';
```

Patch-reception correlation mechanism (for D-H3 stale discard):
- The dispatcher ALREADY correlates `PatchMessage.id` to the original `ActionMessage.id` via `confirmOptimistic(msg.id)` in `init.ts:38-51`
- Plan 03's `fetch_rows` handler echoes `ctx.action.id.clone()` into `PatchMessage.id`
- This plan's DataTable tracks `lastFetchRowsActionId` locally and needs to drop patches whose id doesn't match

**Implementation challenge:** The current data store applies patches via `applyPatch(msg)` in the init flow — DataTable doesn't intercept them today. To drop stale patches, DataTable needs a way to say "for this surface/bound collection, ignore patches whose id ≠ N."

**Two implementation options — pick one:**

**Option A (simpler, recommended):** Track `lastFetchRowsActionId` + `fetchInFlight: { id, offset, limit }` in DataTable state. Before dispatching a new sort/filter action, set `fetchInFlight = null` and clear `lastFetchRowsActionId`. When the data-store `rows` array grows (detected in `$effect`), check whether `fetchInFlight` is still "the latest." If the last sendAction's returned id matches, accept the new rows. If it doesn't match (because a fresh sort/filter has been dispatched in between), simply ignore the row-count change — BUT the data store will have already applied the patch. This option does NOT remove the stale rows from the store; it just ensures the DataTable UI doesn't react to them by resetting scroll/refetch state.

**Option B (stronger):** Extend the init.ts patch-handling layer to accept a per-surface filter function that DataTable can register to reject patches. More code, touches more files, probably overkill for one race condition.

**Chosen: Option A.** DataTable tracks state locally and ignores out-of-order UX signals. Document the limitation in a code comment: stale rows may briefly appear in the store before the next fresh Render replaces everything. This is acceptable per D-D2 (server is single source of truth; sort/filter triggers fresh Render).

**Revised stale-discard behavior:**
- `lastFetchRowsActionId: string | null = $state(null)` in DataTable
- `onFilterChange() / onSortChange()` clears it: `lastFetchRowsActionId = null`
- `sendAction('fetch-rows', ...)` returns an id — store it
- In the sentinel callback: BEFORE dispatching a new fetch-rows, if `lastFetchRowsActionId === null` (meaning a filter/sort just reset) OR the last request has been in flight too long, go ahead. Otherwise, gate on `fetching` flag.
- The `fetching` in-flight flag prevents duplicate concurrent fetches (same semantics as the current implementation)
- True stale-discard is largely unnecessary for filter/sort because server FIFO + fresh-Render guarantees ordering
- The one race we actually guard against: rapid scroll → two fetch-rows in flight → first completes with rows 50-99, second with rows 100-149. Both should land. The `fetching` flag gates the second dispatch until the first completes.

**Net: `lastFetchRowsActionId` tracking is more of a correctness invariant than an active dropper.** Still record it per D-H3 so the semantics are documented, and add a browser test that proves the `fetching` gate prevents overlapping dispatches.

Svelte MCP will be invoked during implementation to validate runes + virtualizer + createSvelteTable interop.
</interfaces>

<research_references>
- 13-RESEARCH.md §Pattern 1 (createSvelteTable config with Svelte 5 getters) — the exact state wiring pattern
- 13-RESEARCH.md §Pattern 2 (filter bar + 300ms debounce + stale-response guard)
- 13-RESEARCH.md §Pattern 3 (per-kind cell renderers via createRawSnippet + renderSnippet)
- 13-VALIDATION.md §Per-Task Verification Map rows 1-31 — this plan must land tests matching every row
- 13-CONTEXT.md §D-D1, D-D2, D-D3 (virtualizer + sentinel + reset semantics + total_rows contract)
- 13-CONTEXT.md §D-E1, D-E2 (column visibility)
- 13-CONTEXT.md §D-F1 (cell kinds)
- 13-CONTEXT.md §D-H3 (stale discard via action-id tracking)
</research_references>
</context>

<mcp_tool_usage>
Invoke the `svelte` MCP server (`mcp__svelte__*`) extensively during Task 2:
1. Before writing: query for Svelte 5 idioms for `createSvelteTable` with Svelte getters (`get data()`)
2. Before writing: query for the correct `FlexRender` snippet pattern for per-cell rendering
3. Before writing: query for the correct Svelte 5 `bind:checked={() => ..., (v) => ...}` two-way binding syntax used in the column visibility DropdownMenu.CheckboxItem
4. After writing: pass the full `DataTable.svelte` file to the MCP and ask for idiomaticity + correctness feedback. Fix any issues flagged.
5. After writing: query specifically for any `$effect` cleanup / memory-leak risks (e.g., the IntersectionObserver sentinel and debounce timers).

This is the most complex Svelte 5 component in the phase — DO NOT skip the svelte MCP validation.
</mcp_tool_usage>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Write the rewritten DataTable.browser-test.ts harness first (TDD RED phase)</name>
  <files>frontend/src/lib/components/table/DataTable.browser-test.ts</files>
  <read_first>
    - frontend/src/lib/components/table/DataTable.browser-test.ts (existing 99 lines — rewrite)
    - frontend/src/lib/components/table/DataTableActions.browser-test.ts (Plan 04 pattern reference)
    - frontend/src/lib/components/table/SvelteVirtualSmoke.browser-test.ts (Plan 01 pattern reference)
    - .planning/phases/13-datatable-enhancements/13-VALIDATION.md §Per-Task Verification Map (target assertion list)
    - .planning/codebase/TESTING.md §Browser component test structure
  </read_first>
  <behavior>
    Rewrite the test file to cover (minimum 20 assertions, mapping to 13-VALIDATION.md rows 1-31):

    - Test: `renders filter bar from props.filters` — verifies row 1
    - Test: `text filter debounced 300ms dispatches filter action` (use `vi.useFakeTimers()` + `vi.advanceTimersByTime(300)`) — verifies row 2
    - Test: `Enter in text filter flushes immediately` — verifies row 3
    - Test: `select filter fires with no debounce` — verifies row 4
    - Test: `empty filter values stripped from payload` — verifies row 5
    - Test: `filter input retains focus across server Render reset` (focus preservation) — verifies row 31
    - Test: `columns dropdown lists hideable columns` — verifies row 18
    - Test: `toggle hides column in table` — verifies row 19
    - Test: `hidden_default: true columns start hidden` — verifies row 20
    - Test: `virtualizer windows rows` (mount 200 rows, assert only ~15-25 rendered) — verifies row 10
    - Test: `sentinel triggers fetch-rows near tail` — verifies row 9
    - Test: `sort resets scrollTop to 0` — verifies row 22
    - Test: `filter reset resets scrollTop and re-arms sentinel` — verifies row 11
    - Test: `stops fetching when rows.length >= total_rows` — verifies row 12
    - Test: `stops fetching when response returns fewer rows than limit` — verifies row 13
    - Test: `fetching guard prevents concurrent fetch-rows dispatch` — verifies row 14 (adapted — see Option A above)
    - Test: `actions kind renders DataTableActions` — verifies row 23
    - Test: `date kind formats via Intl.DateTimeFormat` — verifies row 24
    - Test: `number kind right-aligns with tabular-nums` — verifies row 25
    - Test: `badge kind renders Badge component` — verifies row 26

    Each test follows the existing pattern:
    ```typescript
    vi.mock('$lib/transport/dispatcher', () => ({ sendAction: vi.fn(() => 'test-id-uuid') }));
    import { sendAction } from '$lib/transport/dispatcher';
    import DataTable from './DataTable.svelte';
    import { setFullState, resetStore } from '$lib/store/data.svelte';
    ```

    Some tests require setting up >100 rows of test data via `setFullState(surface, { rows: Object.fromEntries(new Array(200).fill(0).map((_, i) => [String(i), { id: String(i), name: `Row ${i}` }])) })`.
  </behavior>
  <action>
    Delete the old `DataTable.browser-test.ts` content and write a new file with ~20 tests as described in the behavior block. Each test MUST map to a row in 13-VALIDATION.md §Per-Task Verification Map.

    Use this skeleton structure:

    ```typescript
    import { render } from 'vitest-browser-svelte';
    import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
    import DataTable from './DataTable.svelte';
    import { setFullState, resetStore } from '$lib/store/data.svelte';

    vi.mock('$lib/transport/dispatcher', () => ({
      sendAction: vi.fn(() => 'test-uuid-1234'),
    }));
    import { sendAction } from '$lib/transport/dispatcher';

    beforeEach(() => {
      resetStore('test');
      vi.clearAllMocks();
      (sendAction as ReturnType<typeof vi.fn>).mockReturnValue('test-uuid-1234');
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    // Helper: build N rows keyed by id for the bound collection
    function buildRows(n: number, prefix = 'row') {
      const out: Record<string, Record<string, unknown>> = {};
      for (let i = 0; i < n; i++) {
        const id = `${prefix}-${i}`;
        out[id] = { id, name: `Name ${i}`, email: `n${i}@example.com`, created: '2026-04-01T12:00:00Z', count: i };
      }
      return out;
    }

    describe('Filter bar (TABLE-01)', () => {
      test('renders one input per filter definition (V-01)', async () => {
        const screen = await render(DataTable, {
          props: {
            props: {
              columns: [{ key: 'name', label: 'Name' }],
              filters: [
                { id: 'search', kind: 'text', label: 'Search', placeholder: 'Find...' },
                { id: 'company', kind: 'select', label: 'Company', options: [{ value: '', label: 'All' }, { value: '1', label: 'Acme' }] },
                { id: 'created', kind: 'date-range', label: 'Created' },
              ],
            },
            surface: 'test',
          },
        });
        await expect.element(screen.getByPlaceholderText('Find...')).toBeVisible();
        // select visible
        await expect.element(screen.getByText('Company')).toBeVisible();
        // date range: two date inputs
        // (query by labelled region)
      });

      test('text filter debounces 300ms (V-02)', async () => {
        vi.useFakeTimers();
        const screen = await render(DataTable, {
          props: {
            props: {
              columns: [{ key: 'name', label: 'Name' }],
              filters: [{ id: 'search', kind: 'text', label: 'Search' }],
            },
            surface: 'test',
          },
        });
        const input = screen.getByPlaceholderText('Search') || screen.getByLabelText('Search');
        await input.fill('Ali');
        // Not yet dispatched
        vi.advanceTimersByTime(200);
        expect(sendAction).not.toHaveBeenCalledWith('filter', expect.anything());
        // After 300ms total
        vi.advanceTimersByTime(100);
        expect(sendAction).toHaveBeenCalledWith('filter', { search: 'Ali' });
      });

      test('Enter in text filter flushes immediately (V-03)', async () => {
        vi.useFakeTimers();
        const screen = await render(DataTable, { /* ... */ });
        const input = screen.getByLabelText('Search');
        await input.fill('Alice');
        await input.press('Enter');
        // No debounce wait needed
        expect(sendAction).toHaveBeenCalledWith('filter', { search: 'Alice' });
      });

      test('select filter fires with no debounce (V-04)', async () => { /* ... */ });

      test('empty values stripped from filter payload (V-05)', async () => {
        vi.useFakeTimers();
        const screen = await render(DataTable, {
          props: {
            props: {
              columns: [{ key: 'name', label: 'Name' }],
              filters: [
                { id: 'search', kind: 'text', label: 'Search' },
                { id: 'company', kind: 'select', label: 'Company', options: [{ value: '', label: 'All' }] },
              ],
            },
            surface: 'test',
          },
        });
        await screen.getByLabelText('Search').fill('Alice');
        vi.advanceTimersByTime(300);
        // Only search should be in the payload; empty company omitted
        expect(sendAction).toHaveBeenCalledWith('filter', { search: 'Alice' });
      });

      test('filter input retains focus across server Render reset (V-31)', async () => {
        // Mount with filters, focus the search input, then swap the bound
        // collection to simulate a fresh Render — the filter input should
        // still be focused afterward. Mirrors the Phase 12 focus-preservation
        // pattern in frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts
      });
    });

    describe('Column visibility (TABLE-03)', () => {
      test('Columns dropdown lists hideable columns (V-18)', async () => { /* ... */ });
      test('Toggling a checkbox hides the column (V-19)', async () => { /* ... */ });
      test('hidden_default: true starts hidden (V-20)', async () => { /* ... */ });
    });

    describe('Virtualizer + infinite scroll (TABLE-02)', () => {
      test('virtualizer windows rows (V-10)', async () => {
        setFullState('test', { rows: buildRows(200) });
        const screen = await render(DataTable, {
          props: {
            props: { columns: [{ key: 'name', label: 'Name' }] },
            bind: '/rows',
            surface: 'test',
          },
        });
        // Only a small window of rows should be in the DOM
        const cells = await screen.container.querySelectorAll('[role="cell"]');
        expect(cells.length).toBeGreaterThan(0);
        expect(cells.length).toBeLessThan(200);
      });

      test('sentinel triggers fetch-rows near tail (V-09)', async () => {
        setFullState('test', { rows: buildRows(60) });
        const screen = await render(DataTable, {
          props: {
            props: {
              columns: [{ key: 'name', label: 'Name' }],
              total_rows: 237,
              page_size: 50,
              source: 'test_list',
            },
            bind: '/rows',
            surface: 'test',
          },
        });
        // Scroll to the bottom to intersect the sentinel
        const scrollEl = screen.container.querySelector('[data-testid="datatable-scroll"]') as HTMLElement;
        expect(scrollEl).toBeTruthy();
        scrollEl.scrollTop = 10000;
        scrollEl.dispatchEvent(new Event('scroll'));
        await new Promise((r) => setTimeout(r, 100));
        expect(sendAction).toHaveBeenCalledWith(
          'fetch-rows',
          expect.objectContaining({ source: 'test_list', offset: 60 })
        );
      });

      test('sort resets scrollTop to 0 (V-22)', async () => { /* ... */ });

      test('filter change resets scrollTop (V-11)', async () => { /* ... */ });

      test('stops fetching when rows.length >= total_rows (V-12)', async () => { /* ... */ });

      test('stops fetching when response returns fewer rows than limit (V-13)', async () => { /* ... */ });

      test('fetching guard prevents concurrent fetch-rows dispatch (V-14 adapted)', async () => { /* ... */ });
    });

    describe('Cell kinds (D-F1)', () => {
      test('actions kind renders DataTableActions (V-23)', async () => {
        setFullState('test', {
          rows: {
            r1: {
              id: 'r1',
              name: 'Alice',
              actions: [{ label: 'Edit', action: { type: 'click', name: 'edit' } }],
            },
          },
        });
        const screen = await render(DataTable, {
          props: {
            props: {
              columns: [
                { key: 'name', label: 'Name' },
                { key: 'actions', label: '', kind: 'actions' },
              ],
            },
            bind: '/rows',
            surface: 'test',
          },
        });
        // The DataTableActions trigger button should be in the DOM
        await expect.element(screen.getByLabelText('Row actions')).toBeVisible();
      });

      test('date kind formats via Intl.DateTimeFormat (V-24)', async () => { /* ... */ });

      test('number kind right-aligns (V-25)', async () => { /* ... */ });

      test('badge kind renders Badge component (V-26)', async () => { /* ... */ });
    });

    describe('Sort (preserved from v1)', () => {
      test('dispatches sort action on header click', async () => { /* adapted from existing test 60-76 */ });

      test('dispatches select-row on row click', async () => { /* adapted from existing test 78-99 */ });
    });
    ```

    Fill in EVERY `/* ... */` stub with real assertion code targeting the corresponding verification row. Do NOT leave placeholders.

    Run the test file against the OLD `DataTable.svelte` (since Task 2 hasn't rewritten it yet) — ALL new tests SHOULD FAIL with `expect received ... but was not called` or similar. This is the RED phase. Commit the failing test file.

    If the old DataTable happens to pass some tests by coincidence (e.g., basic rendering), that's fine — the bulk should fail.
  </action>
  <verify>
    <automated>cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts 2>&1 | tail -40</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/src/lib/components/table/DataTable.browser-test.ts` contains at least 20 `test(...)` calls
    - `grep -c "test(" frontend/src/lib/components/table/DataTable.browser-test.ts` >= 20
    - Each test comment references a validation row number (`V-01` through `V-31`)
    - `grep -c "V-0\|V-1\|V-2\|V-3" frontend/src/lib/components/table/DataTable.browser-test.ts` >= 20
    - Running the test file reports MAJORITY tests failing (expected in RED phase) OR all passing after Task 2 (also acceptable)
    - `cd frontend && npx tsc --noEmit` exits 0 — the test file compiles even if tests fail at runtime
    - `grep -c "sendAction" frontend/src/lib/components/table/DataTable.browser-test.ts` >= 10 (dispatcher is the primary assertion target)
  </acceptance_criteria>
  <done>Test harness is written, compiles, and fails with meaningful "not called" / "not visible" errors pointing at the unimplemented behaviors Task 2 will deliver.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Rewrite DataTable.svelte + datatable-cells.svelte.ts to pass the test harness (TDD GREEN phase)</name>
  <files>
    frontend/src/lib/components/table/DataTable.svelte,
    frontend/src/lib/components/table/datatable-cells.svelte.ts
  </files>
  <read_first>
    - Everything in `<read_first>` for Task 1 PLUS:
    - frontend/src/lib/components/table/SvelteVirtualSmoke.svelte §the "Path chosen" decision comment (determines whether to import from `@tanstack/svelte-virtual` directly or use the `$lib/utils/virtualizer.svelte.ts` fallback)
    - frontend/src/lib/components/ui/data-table/index.js (or .ts) — verify the exact exports (`createSvelteTable`, `FlexRender`, `renderSnippet`, `renderComponent`)
    - frontend/src/lib/components/ui/dropdown-menu/index.ts — verify CheckboxItem API
    - Plan 04's DataTableActions.svelte (for the `items` prop shape)
    - Task 1's DataTable.browser-test.ts (the passing target)
    - .planning/phases/13-datatable-enhancements/13-RESEARCH.md §Pattern 1 §Pattern 2 §Pattern 3 (the canonical recipe shape)
  </read_first>
  <behavior>
    Make ALL tests in Task 1's `DataTable.browser-test.ts` pass. The component MUST satisfy every `must_haves.truths` entry in this plan's frontmatter.
  </behavior>
  <action>
    **Step 0 — Read the svelte-virtual path decision.** Grep:
    ```bash
    grep -A5 "Path chosen" frontend/src/lib/components/table/SvelteVirtualSmoke.svelte
    ```
    If the decision is STORE-BASED, import `createVirtualizer` from `@tanstack/svelte-virtual`. If VIRTUAL-CORE-DIRECT, import `createRuneVirtualizer` from `$lib/utils/virtualizer.svelte`. The rest of this plan uses the placeholder `$virtualizer` to mean "whatever the chosen path exposes."

    **Step 1 — Write `datatable-cells.svelte.ts`.**

    Create `frontend/src/lib/components/table/datatable-cells.svelte.ts`:

    ```typescript
    import { createRawSnippet } from 'svelte';

    /**
     * Per-kind cell snippet factories for DataTable. Each returns a snippet
     * that can be passed to FlexRender via renderSnippet. Phase 13 D-F1.
     */

    export const dateCellSnippet = createRawSnippet<[{ iso: string }]>((getArgs) => {
      return {
        render: () => {
          const { iso } = getArgs();
          if (!iso) return '<span></span>';
          const formatted = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(iso));
          // Svelte's createRawSnippet returns HTML text; we handcraft an
          // escaped span. Since `formatted` is produced by Intl.DateTimeFormat
          // with a non-user-controlled date string, it's safe.
          return `<span class="text-sm">${escapeHtml(formatted)}</span>`;
        },
      };
    });

    export const numberCellSnippet = createRawSnippet<[{ value: number }]>((getArgs) => {
      return {
        render: () => {
          const { value } = getArgs();
          const formatted = new Intl.NumberFormat().format(Number(value) || 0);
          return `<span class="text-right tabular-nums block">${escapeHtml(formatted)}</span>`;
        },
      };
    });

    export const badgeCellSnippet = createRawSnippet<[{ label: string; variant?: string }]>((getArgs) => {
      return {
        render: () => {
          const { label, variant } = getArgs();
          const cls = variantToClass(variant);
          return `<span class="inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ${cls}">${escapeHtml(String(label ?? ''))}</span>`;
        },
      };
    });

    function variantToClass(variant?: string): string {
      switch (variant) {
        case 'success':
        case 'default':
          return 'bg-primary text-primary-foreground';
        case 'destructive':
        case 'error':
          return 'bg-destructive text-destructive-foreground';
        case 'outline':
          return 'border border-input bg-background';
        case 'secondary':
        default:
          return 'bg-secondary text-secondary-foreground';
      }
    }

    /** Minimal HTML-escape — createRawSnippet emits raw HTML, so we escape anything user-controlled. */
    function escapeHtml(s: string): string {
      return s
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
    }
    ```

    **Note on XSS:** `createRawSnippet.render()` returns a raw HTML string. User-controlled strings flowing through these snippets MUST be HTML-escaped. For `badge`, the `label` could come from server data — escape it. For `date`, the input is an ISO string formatted by `Intl.DateTimeFormat`, which returns safe locale-formatted text — still escape defensively. For `number`, the input is numeric.

    Actions kind does NOT use `createRawSnippet` — it uses `renderComponent(DataTableActions, { items })` which is XSS-safe by virtue of Svelte's text interpolation in the component itself.

    **Step 2 — Query the svelte MCP.** Ask:

    > "In Svelte 5, I need to wire a shadcn-svelte `createSvelteTable` with:
    > 1. `get data()` and `get columns()` reactive getters pointing to `$derived` values
    > 2. `state: { get sorting(), get columnVisibility() }` with onSortingChange/onColumnVisibilityChange using the updater-function pattern
    > 3. `manualSorting: true`, `getCoreRowModel()`
    > 4. A `@tanstack/svelte-virtual` createVirtualizer that windows the rows
    > 5. An `IntersectionObserver` sentinel at the virtualizer tail via `use:onIntersect`
    > 6. A debounced text-input filter bar with Enter-to-flush
    > 7. A DropdownMenu.CheckboxItem column visibility toggle using `bind:checked={() => col.getIsVisible(), (v) => col.toggleVisibility(!!v)}`
    >
    > Give me the full component skeleton with correct runes, `$effect` cleanup for timers/observers, and snippet-based FlexRender cell rendering."

    Use the MCP's output as the authoritative reference. DO NOT guess syntax.

    **Step 3 — Rewrite `frontend/src/lib/components/table/DataTable.svelte`.**

    Target structure (approximately 250 lines):

    ```svelte
    <script lang="ts">
      import {
        createSvelteTable,
        FlexRender,
        renderSnippet,
        renderComponent,
      } from '$lib/components/ui/data-table/index.js';
      import {
        getCoreRowModel,
        type ColumnDef,
        type SortingState,
        type VisibilityState,
      } from '@tanstack/table-core';
      import { createVirtualizer } from '@tanstack/svelte-virtual';
      // OR — if Plan 01 chose the fallback:
      // import { createRuneVirtualizer } from '$lib/utils/virtualizer.svelte';

      import * as Table from '$lib/components/ui/table';
      import { Input } from '$lib/components/ui/input';
      import { Button } from '$lib/components/ui/button';
      import * as Select from '$lib/components/ui/select';
      import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
      import ChevronUp from '@lucide/svelte/icons/chevron-up';
      import ChevronDown from '@lucide/svelte/icons/chevron-down';

      import { getData } from '$lib/store/data.svelte';
      import { sendAction } from '$lib/transport/dispatcher';
      import { onIntersect } from '$lib/actions/viewport';
      import type { ComponentAction } from '$lib/transport/messages';
      import DataTableActions from './DataTableActions.svelte';
      import {
        dateCellSnippet,
        numberCellSnippet,
        badgeCellSnippet,
      } from './datatable-cells.svelte';

      // -- Props --
      type SduiColumn = {
        key: string;
        label: string;
        sortable?: boolean;
        kind?: 'text' | 'badge' | 'actions' | 'date' | 'number';
        hidden_default?: boolean;
      };

      type FilterDef =
        | { id: string; kind: 'text'; label: string; placeholder?: string; span?: number }
        | { id: string; kind: 'select'; label: string; options: { value: string; label: string }[]; span?: number }
        | { id: string; kind: 'date-range'; label: string; span?: number };

      let {
        props = {},
        bind,
        action,
        surface,
      }: {
        props: Record<string, unknown>;
        bind?: string;
        action?: ComponentAction;
        surface: string;
      } = $props();

      // -- Derived from props --
      const sduiColumns = $derived((props.columns as SduiColumn[]) ?? []);
      const filterDefs = $derived((props.filters as FilterDef[]) ?? []);
      const totalRows = $derived((props.total_rows as number) ?? 0);
      const pageSize = $derived((props.page_size as number) ?? 50);
      const rowIdKey = $derived((props.row_id_key as string) ?? 'id');
      const source = $derived((props.source as string) ?? '');

      // -- Bound collection rows --
      const rawData = $derived(
        bind ? ((getData(surface, bind) as Record<string, Record<string, unknown>>) ?? {}) : {}
      );
      const rows = $derived(Object.values(rawData));

      // -- Filter state (local, NOT /bind) --
      let filterValues = $state<Record<string, unknown>>({});
      let debounceTimer: ReturnType<typeof setTimeout> | undefined;

      function flushFilter() {
        if (debounceTimer !== undefined) {
          clearTimeout(debounceTimer);
          debounceTimer = undefined;
        }
        const payload: Record<string, unknown> = {};
        for (const [k, v] of Object.entries(filterValues)) {
          if (v === '' || v === undefined || v === null) continue;
          if (typeof v === 'object' && v !== null && 'from' in v) {
            const fr = (v as { from?: string; to?: string });
            if (!fr.from && !fr.to) continue;
          }
          payload[k] = v;
        }
        // Reset scroll + sentinel state on filter change
        if (scrollContainer) scrollContainer.scrollTop = 0;
        fetching = false;
        exhausted = false;
        lastFetchRowsActionId = null;
        sendAction('filter', payload);
      }

      function scheduleFilter(delay = 300) {
        if (debounceTimer !== undefined) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
          debounceTimer = undefined;
          flushFilter();
        }, delay);
      }

      function handleTextChange(id: string, value: string) {
        filterValues[id] = value;
        scheduleFilter(300);
      }

      function handleSelectChange(id: string, value: string) {
        filterValues[id] = value;
        flushFilter();
      }

      function handleTextKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
          e.preventDefault();
          flushFilter();
        }
      }

      function handleDateRangeChange(id: string, side: 'from' | 'to', value: string) {
        const existing = (filterValues[id] as { from?: string; to?: string } | undefined) ?? {};
        filterValues[id] = { ...existing, [side]: value };
        scheduleFilter(300);
      }

      // -- TanStack column defs (derived from SDUI columns + cell kinds) --
      const columnDefs = $derived<ColumnDef<Record<string, unknown>>[]>(
        sduiColumns.map((c) => ({
          id: c.key,
          accessorKey: c.key,
          header: c.label,
          enableSorting: c.sortable ?? false,
          enableHiding: true,
          cell: (info) => {
            const value = info.row.original[c.key];
            switch (c.kind ?? 'text') {
              case 'actions': {
                const items = (value as Array<{ label: string; action: ComponentAction }>) ?? [];
                return renderComponent(DataTableActions, { items });
              }
              case 'date':
                return renderSnippet(dateCellSnippet, { iso: String(value ?? '') });
              case 'number':
                return renderSnippet(numberCellSnippet, { value: Number(value ?? 0) });
              case 'badge':
                return renderSnippet(badgeCellSnippet, { label: String(value ?? ''), variant: 'default' });
              case 'text':
              default:
                return String(value ?? '');
            }
          },
        }))
      );

      // -- TanStack state --
      let sorting = $state<SortingState>([]);
      let columnVisibility = $state<VisibilityState>({});

      // Initialize hidden-by-default columns from props
      $effect(() => {
        const initial: VisibilityState = {};
        for (const c of sduiColumns) {
          if (c.hidden_default === true) initial[c.key] = false;
        }
        columnVisibility = initial;
      });

      const table = createSvelteTable({
        get data() { return rows; },
        get columns() { return columnDefs; },
        state: {
          get sorting() { return sorting; },
          get columnVisibility() { return columnVisibility; },
        },
        onSortingChange: (updater) => {
          const next = typeof updater === 'function' ? updater(sorting) : updater;
          sorting = next;
          const primary = next[0];
          if (primary) {
            // Reset scroll/fetch state on sort change (D-D2)
            if (scrollContainer) scrollContainer.scrollTop = 0;
            fetching = false;
            exhausted = false;
            lastFetchRowsActionId = null;
            sendAction('sort', { column: primary.id, direction: primary.desc ? 'desc' : 'asc' });
          }
        },
        onColumnVisibilityChange: (updater) => {
          columnVisibility = typeof updater === 'function' ? updater(columnVisibility) : updater;
        },
        manualSorting: true,
        getCoreRowModel: getCoreRowModel(),
      });

      // -- Virtualizer --
      let scrollContainer: HTMLDivElement | undefined = $state();
      const virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
        count: rows.length,
        getScrollElement: () => scrollContainer ?? null,
        estimateSize: () => 48,
        overscan: 8,
      });
      // If Plan 01 chose VIRTUAL-CORE-DIRECT, replace the above with:
      // const virtualizer = createRuneVirtualizer(() => ({
      //   count: rows.length,
      //   getScrollElement: () => scrollContainer ?? null,
      //   estimateSize: () => 48,
      //   overscan: 8,
      // }));

      // -- Infinite scroll sentinel state --
      let fetching = $state(false);
      let exhausted = $state(false);
      let lastFetchRowsActionId: string | null = $state(null);

      function isEndOfData(): boolean {
        if (exhausted) return true;
        if (totalRows > 0 && rows.length >= totalRows) return true;
        return false;
      }

      function handleSentinelEnter() {
        if (fetching || isEndOfData() || !source) return;
        fetching = true;
        const offset = rows.length;
        const limit = pageSize;
        const id = sendAction('fetch-rows', { source, offset, limit });
        lastFetchRowsActionId = id;
      }

      // Clear the fetching flag when new rows land (row count changed since
      // last dispatch). If fewer-than-limit arrived, mark exhausted.
      let prevRowCount = $state(0);
      let expectedLimit = $state(0);
      $effect(() => {
        const count = rows.length;
        if (fetching && count > prevRowCount) {
          const delta = count - prevRowCount;
          fetching = false;
          if (expectedLimit > 0 && delta < expectedLimit && totalRows === 0) {
            exhausted = true;
          }
        }
        prevRowCount = count;
      });

      // Reset fetching + exhausted when the rows collection fully resets
      // (server replaced the entire /contacts map — typically a sort/filter
      // response). Detect via a full-replacement signal: rows is shorter
      // after a fresh Render. Simpler approach: re-arm on sort/filter
      // (already done above in onSortingChange + flushFilter).

      // -- Sort handler called from header (via TanStack) --
      // (see onSortingChange above)

      // -- Row click (preserved from v1) --
      function handleRowClick(row: Record<string, unknown>) {
        if (action) {
          const id = String(row[rowIdKey] ?? '');
          sendAction(action.name ?? 'select-row', { id }, action.target);
        }
      }
    </script>

    <div class="flex flex-col gap-4 h-full">
      <!-- Top region: filter bar + column visibility -->
      <div class="flex items-center gap-2 flex-wrap">
        {#each filterDefs as f (f.id)}
          {#if f.kind === 'text'}
            <Input
              class="max-w-sm"
              placeholder={f.placeholder ?? f.label}
              aria-label={f.label}
              value={String(filterValues[f.id] ?? '')}
              oninput={(e) => handleTextChange(f.id, (e.currentTarget as HTMLInputElement).value)}
              onkeydown={handleTextKeydown}
            />
          {:else if f.kind === 'select'}
            <Select.Root
              type="single"
              value={String(filterValues[f.id] ?? '')}
              onValueChange={(v) => handleSelectChange(f.id, v ?? '')}
            >
              <Select.Trigger class="w-[180px]" aria-label={f.label}>
                {f.label}
              </Select.Trigger>
              <Select.Content>
                {#each f.options as opt (opt.value)}
                  <Select.Item value={opt.value}>{opt.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          {:else if f.kind === 'date-range'}
            <Input
              type="date"
              aria-label={`${f.label} from`}
              value={((filterValues[f.id] as { from?: string } | undefined)?.from) ?? ''}
              oninput={(e) => handleDateRangeChange(f.id, 'from', (e.currentTarget as HTMLInputElement).value)}
              onkeydown={handleTextKeydown}
            />
            <Input
              type="date"
              aria-label={`${f.label} to`}
              value={((filterValues[f.id] as { to?: string } | undefined)?.to) ?? ''}
              oninput={(e) => handleDateRangeChange(f.id, 'to', (e.currentTarget as HTMLInputElement).value)}
              onkeydown={handleTextKeydown}
            />
          {/if}
        {/each}

        <!-- Column visibility -->
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props: trigProps })}
              <Button {...trigProps} variant="outline" class="ms-auto">Columns</Button>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end">
            {#each table.getAllColumns().filter((c) => c.getCanHide()) as column (column.id)}
              <DropdownMenu.CheckboxItem
                class="capitalize"
                bind:checked={() => column.getIsVisible(), (v) => column.toggleVisibility(!!v)}
              >
                {column.id}
              </DropdownMenu.CheckboxItem>
            {/each}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>

      <!-- Virtualized scroll container -->
      <div
        bind:this={scrollContainer}
        data-testid="datatable-scroll"
        class="overflow-y-auto flex-1 min-h-0 border rounded-md"
      >
        <div
          style="height: {virtualizer.getTotalSize?.() ?? 0}px; position: relative;"
          data-testid="datatable-inner"
        >
          <Table.Root>
            <Table.Header>
              {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
                <Table.Row>
                  {#each headerGroup.headers as header (header.id)}
                    <Table.Head
                      class={header.column.getCanSort() ? 'cursor-pointer hover:bg-accent' : ''}
                      onclick={() => header.column.getCanSort() && header.column.toggleSorting()}
                    >
                      <FlexRender content={header.column.columnDef.header} context={header.getContext()} />
                      {#if header.column.getIsSorted() === 'asc'}
                        <ChevronUp class="size-4 inline ms-1" />
                      {:else if header.column.getIsSorted() === 'desc'}
                        <ChevronDown class="size-4 inline ms-1" />
                      {/if}
                    </Table.Head>
                  {/each}
                </Table.Row>
              {/each}
            </Table.Header>
            <Table.Body>
              {#each virtualizer.getVirtualItems?.() ?? [] as vi (vi.key)}
                {@const row = table.getRowModel().rows[vi.index]}
                {#if row}
                  <Table.Row
                    data-index={vi.index}
                    style="position: absolute; top: 0; left: 0; width: 100%; height: {vi.size}px; transform: translateY({vi.start}px);"
                    class={action ? 'cursor-pointer' : ''}
                    onclick={() => handleRowClick(row.original)}
                  >
                    {#each row.getVisibleCells() as cell (cell.id)}
                      <Table.Cell class="px-4 py-3 text-sm">
                        <FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
                      </Table.Cell>
                    {/each}
                  </Table.Row>
                {/if}
              {/each}
            </Table.Body>
          </Table.Root>

          <!-- Sentinel for infinite scroll (sits at the tail of the virtual list) -->
          {#if !isEndOfData() && source}
            <div
              style="position: absolute; bottom: 0; left: 0; height: 1px; width: 100%;"
              use:onIntersect={{ onEnter: handleSentinelEnter, root: scrollContainer ?? null, rootMargin: '200px', enabled: !fetching }}
            ></div>
          {/if}
        </div>
      </div>
    </div>
    ```

    **CRITICAL IMPLEMENTATION CALLOUTS:**

    1. **The virtualizer API for Svelte 5 may require `$virtualizer` store auto-subscription OR direct rune access.** This depends on Plan 01's decision. Adapt the `virtualizer.getTotalSize()` and `virtualizer.getVirtualItems()` calls to match. If `createVirtualizer` returns a Svelte store, use `$virtualizer.getTotalSize()` and `$virtualizer.getVirtualItems()`. If it returns a `$state`-driven rune wrapper, use direct field access.

    2. **The `bind:checked={() => ..., (v) => ...}` two-way-binding function-pair syntax** is a Svelte 5 pattern. Verify with the svelte MCP. If it doesn't work with the current `bits-ui` DropdownMenu.CheckboxItem, fall back to `checked={column.getIsVisible()} onCheckedChange={(v) => column.toggleVisibility(!!v)}`.

    3. **`$effect` cleanup for the debounce timer:** Add a cleanup at the top-level `$effect` or rely on `$effect.root` semantics. A simple approach: add `$effect(() => () => { if (debounceTimer) clearTimeout(debounceTimer); });` as a component-lifetime cleanup.

    4. **Sentinel re-arming on filter/sort reset:** The `onIntersect` action's `update()` method is called when the options object changes. The `enabled: !fetching` dependency means the observer re-arms when `fetching` flips. This is sufficient for the re-arm semantic.

    5. **Initial visibility `$effect`:** The `$effect` that initializes hidden columns runs on mount (and when `sduiColumns` changes). This may fight with user-toggled state if `sduiColumns` later re-derives. If the test `toggle hides column` fails because the `$effect` stomps over user changes, convert the initialization to a `$state` default that runs ONCE on first mount, not on every re-derive.

    **Step 4 — Run the tests.**

    ```bash
    cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts
    ```

    Iterate until ALL tests from Task 1 pass. This will take multiple svelte-MCP queries + code tweaks + test runs.

    **Step 5 — Run the svelte MCP validation pass.** Pass the final `DataTable.svelte` + `datatable-cells.svelte.ts` to the MCP and request idiomaticity/correctness feedback. Fix any issues raised.
  </action>
  <verify>
    <automated>cd frontend && npx tsc --noEmit && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts src/lib/components/table/DataTableActions.browser-test.ts src/lib/components/table/SvelteVirtualSmoke.browser-test.ts 2>&1 | tail -60</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/src/lib/components/table/DataTable.svelte` is rewritten — `wc -l` >= 200 lines
    - `grep -c "createSvelteTable" frontend/src/lib/components/table/DataTable.svelte` == 1
    - `grep -c "createVirtualizer\|createRuneVirtualizer" frontend/src/lib/components/table/DataTable.svelte` >= 1
    - `grep -c "onIntersect" frontend/src/lib/components/table/DataTable.svelte` >= 1
    - `grep -c "sendAction('filter'" frontend/src/lib/components/table/DataTable.svelte` == 1
    - `grep -c "sendAction('fetch-rows'" frontend/src/lib/components/table/DataTable.svelte` == 1
    - `grep -c "sendAction('sort'" frontend/src/lib/components/table/DataTable.svelte` == 1
    - `grep -c "renderComponent(DataTableActions" frontend/src/lib/components/table/DataTable.svelte` == 1
    - `grep -c "renderSnippet(dateCellSnippet\|renderSnippet(numberCellSnippet\|renderSnippet(badgeCellSnippet" frontend/src/lib/components/table/DataTable.svelte` >= 3
    - `grep -c "lastFetchRowsActionId" frontend/src/lib/components/table/DataTable.svelte` >= 1 (D-H3 tracking)
    - `grep -c "scrollContainer.scrollTop = 0" frontend/src/lib/components/table/DataTable.svelte` >= 2 (sort reset + filter reset)
    - `grep -c "hidden_default" frontend/src/lib/components/table/DataTable.svelte` >= 1
    - `frontend/src/lib/components/table/datatable-cells.svelte.ts` exists
    - `grep -c "createRawSnippet" frontend/src/lib/components/table/datatable-cells.svelte.ts` >= 3
    - `grep -c "escapeHtml" frontend/src/lib/components/table/datatable-cells.svelte.ts` >= 3 (XSS mitigation)
    - `cd frontend && npx tsc --noEmit` exits 0
    - `cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTable.browser-test.ts` passes ALL tests (every `test(...)` from Task 1)
    - No regressions in `DataTableActions.browser-test.ts` or `SvelteVirtualSmoke.browser-test.ts`
    - svelte MCP's post-write validation pass found no issues (or any issues were fixed)
  </acceptance_criteria>
  <done>DataTable.svelte is the recipe-shaped implementation; every Task 1 test passes; no regressions in adjacent components.</done>
</task>

<task type="auto">
  <name>Task 3: Delete TableScreen.svelte + its test; add CI guard</name>
  <files>
    frontend/src/lib/components/screen/TableScreen.svelte,
    frontend/src/lib/components/screen/TableScreen.browser-test.ts,
    frontend/tests/e2e/ci-guards.spec.ts
  </files>
  <read_first>
    - frontend/src/lib/components/screen/TableScreen.svelte (confirm it exists, 105 lines)
    - frontend/src/lib/components/screen/TableScreen.browser-test.ts (confirm 86 lines)
    - frontend/src/lib/registry/defaults.ts (confirm TableScreen is NOT registered — research verified this)
    - frontend/tests/e2e/ (pattern for an existing spec to use as the template for the new guard spec)
    - backend/crates/crm-demo/src/handlers/ (grep for any use of TableScreen — if ANY handler still references it via some builder, STOP and defer this task to Plan 06 which migrates those handlers)
  </read_first>
  <action>
    **Step 1 — Safety check.** Before deleting, run:

    ```bash
    grep -rn "TableScreen\|table-screen" frontend/src/ backend/crates/ 2>/dev/null
    ```

    Expected findings after this grep:
    - `frontend/src/lib/components/screen/TableScreen.svelte` (the file itself)
    - `frontend/src/lib/components/screen/TableScreen.browser-test.ts` (the test, imports it)
    - NOTHING in `frontend/src/lib/registry/defaults.ts`
    - NOTHING in any other frontend file
    - Possibly findings in `backend/crates/crm-demo/src/handlers/` (if handlers emit the `table-screen` component type string — check the text `"table-screen"` or `TableScreen::new`)

    IF any backend handler references `table-screen` or builds a `TableScreen` component via a Rust builder, ABORT this task. It means TableScreen is live, not an orphan, and deletion would break Plan 06. Leave the files in place, note the conflict in the task-complete commit message, and update `.planning/phases/13-datatable-enhancements/deferred-items.md` with a note.

    If the grep confirms only the two files reference it AND `defaults.ts` does NOT register it (research says this is the case — verify once more), proceed.

    **Step 2 — Delete the files.**

    ```bash
    rm frontend/src/lib/components/screen/TableScreen.svelte
    rm frontend/src/lib/components/screen/TableScreen.browser-test.ts
    ```

    **Step 3 — Run the full frontend test suite** to confirm no breakage:

    ```bash
    cd frontend
    npx tsc --noEmit
    npm test -- --run
    npx vitest --config vitest-browser.config.ts --run
    ```

    If anything fails, one of two things is happening:
    - A stale import somewhere — find it and remove it
    - OR the safety check above missed a usage — restore the files and abort

    **Step 4 — Create a CI guard spec.**

    Create `frontend/tests/e2e/ci-guards.spec.ts`:

    ```typescript
    // CI guards for Phase 13 invariants. These run as a cheap filesystem check
    // (no browser) via playwright — the goal is to catch regressions if a
    // future refactor accidentally re-adds retired files.

    import { test, expect } from '@playwright/test';
    import { existsSync } from 'node:fs';
    import { resolve } from 'node:path';

    const FRONTEND_ROOT = resolve(__dirname, '../..');

    test.describe('Phase 13 CI guards', () => {
      test('TableScreen.svelte is retired (D-A2)', () => {
        const p = resolve(FRONTEND_ROOT, 'src/lib/components/screen/TableScreen.svelte');
        expect(existsSync(p)).toBe(false);
      });

      test('TableScreen.browser-test.ts is retired (D-A2)', () => {
        const p = resolve(FRONTEND_ROOT, 'src/lib/components/screen/TableScreen.browser-test.ts');
        expect(existsSync(p)).toBe(false);
      });
    });
    ```

    Playwright treats this as a regular spec file even though it makes no browser calls. It will run during `npx playwright test`.

    If the existing playwright config excludes non-browser-driven tests, use a different approach: create a simple node script or add an assertion to the existing test suite. Fallback: run the check as a Makefile step or a pre-commit hook. The goal is: ANY test run in CI catches re-introduction.

    **Step 5 — Run the CI guard.**

    ```bash
    cd frontend && npx playwright test tests/e2e/ci-guards.spec.ts
    ```

    If the playwright config rejects this test (e.g., because it tries to spin up a dev server for a test that doesn't need one), simplify: remove the `test.describe` wrapper and make the test bodies no-op unless the file exists. Or use `test.skip` if the check should be environment-conditional. The acceptance criterion is: the test EXISTS and would fail if TableScreen came back.
  </action>
  <verify>
    <automated>cd frontend && test ! -e src/lib/components/screen/TableScreen.svelte && test ! -e src/lib/components/screen/TableScreen.browser-test.ts && test -e tests/e2e/ci-guards.spec.ts && npx tsc --noEmit && npx vitest --config vitest-browser.config.ts --run 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `test ! -e frontend/src/lib/components/screen/TableScreen.svelte` exits 0
    - `test ! -e frontend/src/lib/components/screen/TableScreen.browser-test.ts` exits 0
    - `frontend/tests/e2e/ci-guards.spec.ts` exists with at least one `test(...)` asserting non-existence of TableScreen files
    - `cd frontend && npx tsc --noEmit` exits 0 (no stale imports referencing TableScreen)
    - `cd frontend && npx vitest --config vitest-browser.config.ts --run` passes all browser tests (no TableScreen regressions)
    - `grep -rn "TableScreen" frontend/src/ 2>/dev/null` returns NO matches
  </acceptance_criteria>
  <done>TableScreen orphan is gone; CI guard prevents its reintroduction; no regressions.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Server-supplied `props.columns[].label`, `props.filters[]`, `props.source`, row data | Untrusted. Any field could contain malicious strings. |
| User keyboard/mouse input → filter state | Local-only; no server trust needed. |
| IntersectionObserver callback → sendAction | Client-initiated; normal action dispatch auth path. |
| `source` prop passed to `fetch-rows` backend | Must match the server-side whitelist in Plan 03. If a malicious server sent a bad source, the backend rejects; if a malicious client tried to set one, the server validates. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-05-01 | Tampering (XSS via column header) | `FlexRender content={header.column.columnDef.header}` where header is the server-supplied label string | mitigate | `column.label` is used as the `header` prop which TanStack + FlexRender treats as a string and interpolates as text (not HTML). Verify by test: pass `<script>` in a column label and assert the DOM doesn't contain a script element. |
| T-13-05-02 | Tampering (XSS via badge/date/number snippets) | `createRawSnippet` returns raw HTML strings | mitigate | `datatable-cells.svelte.ts` `escapeHtml` helper escapes all user-controlled string inputs before building the HTML. Test: pass `<img onerror>` as a badge label and assert it renders as literal text. |
| T-13-05-03 | Tampering (XSS via actions column) | Delegated to Plan 04 | mitigate | `renderComponent(DataTableActions, { items })` — Plan 04's XSS test already proves `item.label` is escaped via Svelte text interpolation. |
| T-13-05-04 | DoS (Runaway fetch-rows) | Sentinel triggers infinite dispatches | mitigate | `fetching` in-flight flag + `exhausted` latch + `isEndOfData()` gate. Test `fetching guard prevents concurrent fetch-rows dispatch` proves the gate holds. |
| T-13-05-05 | DoS (Timer leak) | Debounce `setTimeout` never clears on unmount | mitigate | Component-level `$effect` cleanup clears `debounceTimer` on destroy. Manual inspection + svelte MCP pass covers leak risks. |
| T-13-05-06 | I (Focus disclosure) | Re-render clobbers focused filter input | mitigate | Phase 12's surface store already does fine-grained reactivity. Test `filter input retains focus across server Render reset` (V-31) proves focus is preserved — inherited from Phase 12 D-A6. |

No HIGH severity threats. XSS surfaces are all mitigated by escape helpers or delegated to already-tested components.
</threat_model>

<verification>
```bash
cd frontend
npx tsc --noEmit
npx vitest --config vitest-browser.config.ts --run src/lib/components/table/
npm test -- --run
```

All three MUST exit 0. In particular, `DataTable.browser-test.ts` must report ALL tests passing (at least 20).

Also run the CI guard:
```bash
cd frontend && npx playwright test tests/e2e/ci-guards.spec.ts 2>&1 || true
```
If playwright cannot run without a dev server, skip at runtime — but the spec file must exist with the right assertions.
</verification>

<success_criteria>
- `DataTable.svelte` is rewritten using the shadcn-svelte recipe shape (createSvelteTable + FlexRender + createVirtualizer + IntersectionObserver sentinel + per-kind cell renderers + column visibility DropdownMenu + inline filter bar)
- All ~20 new browser tests in `DataTable.browser-test.ts` pass (every `V-NN` from 13-VALIDATION.md is covered)
- `datatable-cells.svelte.ts` exists with XSS-safe `createRawSnippet` helpers for date/number/badge
- `DataTableActions` is used via `renderComponent` for the `actions` kind
- Stale-fetch-rows concerns are mitigated by `fetching` in-flight gate + `lastFetchRowsActionId` tracking per D-H3 (the field is stored; the semantic guarantee comes from the fetching flag + server FIFO, documented in a code comment)
- `TableScreen.svelte` and its browser-test are DELETED
- CI guard in `tests/e2e/ci-guards.spec.ts` asserts the deletion
- No regressions in any other browser test or unit test
- svelte MCP validation pass found no idiomatic issues (or they were fixed)
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-05-datatable-rewrite-SUMMARY.md` recording:
- Final line counts: `DataTable.svelte`, `datatable-cells.svelte.ts`, `DataTable.browser-test.ts`
- Which svelte-virtual path was used (STORE-BASED vs VIRTUAL-CORE-DIRECT) and any adapter-specific adjustments made
- svelte MCP feedback summary (the most useful corrections it surfaced)
- Any test from 13-VALIDATION.md §Per-Task Verification Map that was NOT implemented and why (expected: all 20 in Wave 2 should be green after this plan; the Wave 3 backend-integration tests come in Plan 06 and Plan 07)
- Open items for Plan 06 to pick up (e.g., "DataTable expects `props.source: string` — Plan 06 CRM migration must set it via the backend DataTable builder")
</output>
