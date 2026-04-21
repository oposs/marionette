# Phase 13: DataTable Enhancements - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-11
**Phase:** 13-datatable-enhancements
**Areas discussed:** Filter bar shape, Filter trigger + actions, Scroll + data lifecycle, Columns + backend builder

**Discussion notes:** The session pivoted twice based on user feedback. First pivot: the user asked for reasoning to be attached to every option rather than bare labels, and pointed out that shadcn-svelte likely already has a recipe for this problem. Initial questions were withdrawn and re-posed with full pros/cons after fetching the shadcn-svelte data-table recipe via MCP. Second pivot: after seeing the first reshaped options, the user explicitly stated "handrolling things when it comes to ui design is not ideal" — the architecture choice was then committed to "adopt the shadcn-svelte data-table recipe verbatim" without further options-balancing, and follow-on questions were narrowed to issues the recipe doesn't decide for us.

These pivots are recorded as feedback memories and apply to future discuss-phase sessions.

---

## Filter bar shape

### Architecture (committed without a poll after the second pivot)

| Option | Description | Selected |
|--------|-------------|----------|
| Hand-rolled with shadcn visuals | Keep DataTable.svelte custom; mirror recipe layout but build state machinery by hand. Smallest diff, no new deps, fits server-driven model — but hand-rolls UI which the user explicitly rejected. | |
| Full shadcn-svelte recipe with TanStack | Install @tanstack/table-core + @tanstack/svelte-virtual + shadcn-svelte data-table helpers. Rewrite DataTable.svelte to mirror the recipe verbatim. | ✓ |
| Hybrid (TanStack for columns/visibility only) | Install TanStack but use only getCoreRowModel + VisibilityState. Mixed paradigm — worst kind of legibility hit per the user's "no hand-roll" preference. | |

**User's choice:** Full recipe. Rationale: explicit feedback that hand-rolling UI is not acceptable, recipe is the canonical pattern in both shadcn ecosystems (Svelte and React), TanStack's `manualSorting` mode + plain form-pattern filter bar resolves the server-driven mismatch without fighting the framework.

### TableScreen.svelte fate

| Option | Description | Selected |
|--------|-------------|----------|
| Retire it; handlers compose Container directly | Delete TableScreen + browser-test. CRM handlers build Container([Heading, …toolbar, DataTable]). Filter bar moves into DataTable. | ✓ |
| Thin TableScreen shell as page-chrome primitive | Keep TableScreen as title + toolbar + DataTable wrapper. Filter bar moves out. | |
| Keep TableScreen, defer to Phase 15 | No filter bar move; DataTable adds both filter bar AND coexists with TableScreen's. Two filter bars stack. | |

**User's choice:** Retire. Rationale: matches the shadcn recipe's structure (no "screen" abstraction in the recipe), resolves the orphan Phase 12 explicitly assigned to Phase 13, avoids speculative reuse for hypothetical future non-table screens.

### Filter bar realization (server dispatch path)

| Option | Description | Selected |
|--------|-------------|----------|
| Plain form-pattern filter bar (TanStack stays filter-agnostic) | DataTable hosts a `<form>` with shadcn Input/Select/Button. Local Svelte $state. On change → sendAction('filter', values). manualFiltering NOT used. | ✓ |
| TanStack columnFilters in manual mode | Set manualFiltering: true. Each filter binds to a column id via setFilterValue. Global search uses globalFilter. Range filters encode {from, to}. | |
| Hybrid (TanStack where natural, sidecar for the rest) | Per-column filters via TanStack; cross-column / range filters via sidecar local state. Two state stores. | |

**User's choice:** Plain form-pattern. Rationale: Marionette filters are server queries, not row predicates. Many CRM filters don't map to columns (global search, tag-text, date range). Recipe-aligned visually without contorting TanStack's API.

### Filter declaration shape

| Option | Description | Selected |
|--------|-------------|----------|
| Structured filters[] props array | filters: [{id, kind, label, options?, span?}]. DataTable maps each kind to a shadcn primitive. Rust builder gets fluent Filter::text/select/date_range helpers. | ✓ |
| Dynamic child nodes via NodeRenderer | Backend builds TextInput / Select etc. and references their IDs in filterIds. DataTable lays them out via NodeRenderer. | |
| Hybrid (structured kinds + node escape hatch) | Built-in kinds for the 90% case + extraFilterIds for unusual widgets. | |

**User's choice:** Structured props array. Rationale: tight Rust-typed contract, autocomplete-friendly handler call sites, no concrete need for arbitrary filter widgets.

---

## Filter trigger + actions

### Trigger timing

| Option | Description | Selected |
|--------|-------------|----------|
| Debounced live-filter, no Apply button | 300ms debounce on text inputs, Enter flushes immediately, selects fire on change. Modern data-grid feel. Requires stale-response discipline. | ✓ |
| Explicit Apply button (current TableScreen pattern) | Filter inputs are local state; explicit submit dispatches the action. | |
| Per-filter opt-in via filter spec | Each filter declaration picks its trigger mode. Two trigger styles can coexist on one screen. | |

**User's choice:** Debounced live-filter. Rationale: modern UX, recipe-aligned, no extra clicks. Stale-response discipline noted as a hard requirement (D-C2).

### Action contract

| Option | Description | Selected |
|--------|-------------|----------|
| Single 'filter' action with flat values map | One action name. Payload is {filter_id: value}. Empty values omitted. | ✓ |
| Per-field 'filter-changed' action | One action per field change. Server reconstructs full state from session. | |
| Backend declares per-table action name | DataTable.props.filter_action override. | |

**User's choice:** Single 'filter' action. Rationale: matches Marionette's stateless action handler convention, simplest server-side deserialization, current TableScreen also uses 'filter'.

### State plumbing

| Option | Description | Selected |
|--------|-------------|----------|
| DataTable owns local Svelte $state, dispatches directly | Component-local state, no /bind round-trip. | ✓ |
| /bind round-trip through protocol data store | Filter inputs bound to /contactFilters/* paths. Current TableScreen pattern. | |

**User's choice:** Local $state. Rationale: filters are transient UI state, not protocol data. Recipe doesn't bind filter inputs externally either.

---

## Scroll + data lifecycle

### Virtualization + sentinel

| Option | Description | Selected |
|--------|-------------|----------|
| @tanstack/svelte-virtual + sentinel inside the virtualizer | createVirtualizer windows the DOM. IntersectionObserver sentinel at the virtual list tail triggers fetch-rows. | ✓ |
| Sentinel without virtualization (DOM grows unbounded) | All loaded rows in DOM. Doesn't scale to large datasets. | |
| Hand-rolled scrollTop math + sentinel | Keep current virtual scroll, replace only the prefetch trigger. Hand-rolls UI. | |

**User's choice:** TanStack Virtual + sentinel. Rationale: canonical TanStack pairing, replaces hand-rolled virtualization, satisfies TABLE-02's IntersectionObserver mandate.

### Reset semantics on sort/filter

| Option | Description | Selected |
|--------|-------------|----------|
| Server sends Render replacement; client clears collection | New RenderMessage or full collection-set patch from server. Client just resets scrollTop. | ✓ |
| Client clears local collection optimistically | Client mutates store immediately, waits for server. | |
| Hybrid (reset flag + custom patch op) | New patch op semantics for "clear and refill." | |

**User's choice:** Server-side replacement. Rationale: server is single source of truth, matches Marionette's existing CRM pattern (every list re-render goes through Render), no client-side data store mutation.

### End-of-data signal

| Option | Description | Selected |
|--------|-------------|----------|
| total_rows in props + count comparison | Server provides total_rows; sentinel idles when reached. | |
| Explicit end-of-data marker / fewer-than-limit fallback | Server doesn't track totalRows; fewer-than-limit response means done. | |
| Both contracts supported | Optional total_rows; fewer-than-limit fallback when unset. Server picks per screen. | ✓ |

**User's choice:** Both contracts. Rationale: server can pick per screen — expensive COUNT(*) only when total_rows actually matters, falls back to fewer-than-limit otherwise.

---

## Columns + backend builder

### Column visibility UX + persistence

| Option | Description | Selected |
|--------|-------------|----------|
| DropdownMenu in DataTable header; per-mount state | Local Svelte $state Map. No persistence. Backend hidden_default for sensible defaults. | ✓ |
| Persist via localStorage keyed by table id | Cross-session persistence, side-channel outside protocol. | |
| Persist via server round-trip | Server stores user preference. Substantial backend work for v1 nicety. | |

**User's choice:** Per-mount state. Rationale: matches recipe verbatim, no localStorage side channel, no premature persistence work, future phase can add it later.

### Cell rendering

| Option | Description | Selected |
|--------|-------------|----------|
| column.kind enum with built-in renderers | Optional kind: text/badge/actions/date/number per column. createRawSnippet/renderSnippet map kinds to Svelte snippets. | ✓ |
| Format strings only (column.format template) | Backend ships format string. Doesn't solve actions column. | |
| Cell template via nested SDUI nodes per row | Each cell is an SDUI node. Massive payload bloat. | |

**User's choice:** kind enum. Rationale: solves the latent "actions column renders [object Object]" bug, recipe-aligned snippet pattern, extensible enum.

### Rust builder shape

| Option | Description | Selected |
|--------|-------------|----------|
| Fluent additions on existing struct | Extend DataTable in builders/standard.rs with new fields and helpers. Filter and ColumnKind enums in same module. | ✓ |
| Separate DataTable + DataTableConfig structs | Move filter/visibility/total_rows into a separate config struct. | |
| Generic SDUI props builder (no typed enums) | Raw serde_json::Value via .prop() escape hatch. Loses type safety. | |

**User's choice:** Fluent additions on existing struct. Rationale: same builder pattern as the rest of the marionette crate, type-safe filter declarations, autocomplete-friendly call sites, consistent with Phase 12's AppShell builder conventions.

---

## Claude's Discretion

- Exact debounce implementation (Svelte action vs setTimeout in $effect)
- Sentinel placement: at the very last virtual row vs N rows before the end
- DropdownMenu "Columns" button placement within the top region
- Specific badge variant map defaults
- Whether the sentinel uses IntersectionObserver directly or a wrapper Svelte action
- Migration order for the four CRM handlers
- Whether to update the existing DataTable.browser-test.ts or rewrite it

## Deferred Ideas

- Row selection + bulk actions (TABLE-07, v2)
- Per-row action menu beyond actions cell-kind (TABLE-04, v2)
- Row count status bar (TABLE-05, v2 — total_rows prop makes it a one-liner)
- Empty state illustrations (TABLE-06, v2)
- Column visibility persistence (no localStorage / no server round-trip in v1)
- Sticky header during virtualized scroll
- Keyboard navigation
- Sentinel-aware accessibility tuning
- CRM-wide filter bar consistency (Phase 15)
- Loading skeleton during reset
