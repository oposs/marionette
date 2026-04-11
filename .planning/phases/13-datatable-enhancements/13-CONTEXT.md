# Phase 13: DataTable Enhancements - Context

**Gathered:** 2026-04-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 13 enhances the existing `DataTable` SDUI component with three production-grade capabilities:

1. **Server-driven filter bar** — A filter bar built into DataTable that hosts text inputs, selects, and date-range pickers. Filter changes dispatch a single `filter` action to the server with all active filter values.
2. **Infinite scroll via IntersectionObserver sentinel** — Progressive server-side row loading triggered by a sentinel element near the virtual list tail.
3. **User-controlled column visibility** — A `Columns` DropdownMenu in DataTable's top region letting users show/hide columns at runtime.

In addition, Phase 13 implements three structural decisions that fall out of adopting the canonical shadcn-svelte data-table recipe:

- **Adoption of `@tanstack/table-core` + `@tanstack/svelte-virtual`** as the underlying state machine and row virtualizer, replacing the current hand-rolled `scrollTop` math + `ROW_HEIGHT` slicing in `frontend/src/lib/components/table/DataTable.svelte`.
- **Retirement of the orphan `frontend/src/lib/components/screen/TableScreen.svelte`** (and its browser-test). Phase 12 CONTEXT explicitly assigned this orphan's resolution to Phase 13. CRM handlers migrate to compose `Container([Heading, …toolbar Buttons, DataTable])` directly, mirroring the recipe's own pattern (which has no "screen wrapper" abstraction).
- **Cell rendering via `column.kind`** — A new optional `kind: 'text' | 'badge' | 'actions' | 'date' | 'number'` field on `TableColumn` resolves the latent "actions column renders [object Object]" bug in the current DataTable and gives backend handlers a typed way to express rich cells without shipping Svelte components over the wire.

**What this phase is NOT:**

- Not row selection / bulk actions (TABLE-07 / v2)
- Not per-row dropdown action menus as a separate column type beyond the `actions` cell kind described above (TABLE-04 covers richer per-row menus in v2)
- Not row-count status bars or "Showing X of Y" UI (TABLE-05 / v2 — though the optional `totalRows` prop laid down here makes that trivial later)
- Not empty-state illustrations (TABLE-06 / v2 — recipe's "No results" text suffices for v1)
- Not the CRM screen-by-screen filter cleanup (Phase 15 — Phase 13 migrates the four CRM list handlers JUST enough to retire TableScreen and exercise the new DataTable shape end-to-end)
- Not column visibility persistence across reloads (no localStorage, no server round-trip — per-mount state only)
- Not sticky headers, keyboard navigation, or sentinel-aware accessibility tuning beyond what TanStack/`svelte-virtual` ship by default (can be revisited in v2 if needed)

</domain>

<decisions>
## Implementation Decisions

### Architecture: shadcn-svelte data-table recipe adoption

- **D-A1:** **Adopt the shadcn-svelte data-table recipe verbatim.** Install `@tanstack/table-core`, `@tanstack/svelte-virtual`, and the shadcn-svelte `data-table` helper package (which provides `createSvelteTable`, `FlexRender`, `renderSnippet`, `renderComponent`). Rewrite `frontend/src/lib/components/table/DataTable.svelte` to mirror the recipe's structure: filter bar at top, `Table` primitives in the middle, `DropdownMenu.CheckboxItem` "Columns" toggle in the top region, virtualized rows via `createVirtualizer`, and an `IntersectionObserver` sentinel at the list tail for progressive prefetch. The current hand-rolled virtual scroll logic (ROW_HEIGHT, scrollTop math, manual buffer) is replaced by `@tanstack/svelte-virtual`. Hand-rolling UI is explicitly off the table per user preference — adopt the framework recipe.

- **D-A2:** **Retire `TableScreen.svelte`.** Delete `frontend/src/lib/components/screen/TableScreen.svelte` and its browser-test. The filter bar moves INSIDE DataTable (per recipe). CRM handlers (`audit`, `company`, `contact`, `user`) migrate to build a plain `Container` with `[Heading, …toolbar Buttons, DataTable]` children, which is exactly how the shadcn recipe composes title + toolbar + table in a page component. This resolves the orphan that Phase 12 CONTEXT explicitly assigned to Phase 13. No "thin shell" intermediary — speculative reuse for future non-table screens goes against the project's anti-pattern memory (no abstractions for hypothetical futures), and Phase 14 builds FormScreen separately.

### Filter bar realization

- **D-B1:** **Plain form-pattern filter bar; TanStack stays filter-agnostic.** DataTable hosts a regular `<form>` filter bar at the top using shadcn `Input`, `Select`, and `Button` primitives. Local filter state is a Svelte `$state` `Record<filterId, value>`. On change/Enter, DataTable dispatches `sendAction('filter', filterValues)`. TanStack Table is configured WITHOUT `manualFiltering` — it never sees filter values. TanStack handles columns/visibility/sort/virtualization only. Rationale: Marionette filters are server queries, not row predicates, and Marionette CRM filters often don't map to columns (global "search" spans multiple columns; "tag_filter_text" filters over an unrendered relation; date-range targets one column with two inputs). Forcing them into TanStack's column-filter API would require contortions for every non-trivial filter.

- **D-B2:** **Structured `filters[]` props array on DataTable.** `DataTable.props.filters: [{ id, kind: 'text' | 'select' | 'date-range', label, placeholder?, options?, span? }]`. DataTable internally maps each entry to the matching shadcn primitive (`Input` for text, `Select` for select, two `Input`s for date-range). New schema dimension lives in `spec/schemas/data.yaml`. Not a free-form NodeRenderer composition — the tight contract is worth more than the flexibility of arbitrary filter widgets, and we have no concrete use case for unusual filter types in the CRM.

- **D-B3:** **Backend builder gets fluent filter helpers.** `Filter::text(id)`, `Filter::select(id, options)`, `Filter::date_range(id)` constructors. `DataTable::new(cols).filter(Filter::text("search")).filter(Filter::select("company", opts)).filter(Filter::date_range("date")).build()`. Type-safe, autocomplete-friendly, consistent with Phase 12's AppShell builder conventions.

### Filter trigger + dispatch

- **D-C1:** **Debounced live-filter, no Apply button.** Text inputs dispatch the merged filter state to the server after a **300ms debounce** on each change. Pressing **Enter** inside a text input flushes immediately (skips the debounce). **Selects fire on change with no debounce** (selecting a value is a deliberate action). Date-range from/to inputs debounce like text. Matches the shadcn recipe pattern (recipe `Input` dispatches `setFilterValue` on every `oninput`). No "Apply Filters" button — the current TableScreen pattern is explicitly retired.

- **D-C2:** **Stale-response discard via request sequence numbers.** Live filtering means request `N` for "Alic" may complete after request `N+1` for "Alice". DataTable MUST guard against this. The implementation strategy is for the planner to decide, but the constraint is non-negotiable: stale responses to superseded queries must not overwrite the current rows. The planner must investigate whether Marionette's existing dispatcher (touched in Phase 12 patch wiring) already provides per-action sequence numbers or whether DataTable needs to track its own. The simplest viable shape: each `filter` dispatch increments a local `seq`; incoming patches that don't match the latest `seq` for this DataTable are ignored.

- **D-C3:** **Single `filter` action with flat values map.** Action name: `filter`. Payload: `{ filter_id_1: value_1, filter_id_2: value_2, … }` where each value matches the filter kind — string for text, string for select, `{ from, to }` object for date-range. Empty/undefined values are omitted from the payload (server interprets absence as "no filter"). Server handler deserializes via `#[derive(Deserialize)] FilterParams` matching the existing `Payload<T>::from_context` pattern in CRM handlers.

- **D-C4:** **DataTable owns local filter state; no `/bind` round-trip.** Filter values live in component-local Svelte `$state`, NOT bound through the protocol data store via `/bind`. Initial values come from `props.filterDefaults` (or empty). On dispatch, DataTable serializes its local state directly to the action payload. Removes the current TableScreen pattern of binding filters to `/contactFilters/*` paths through the data store. Matches the recipe's "uncontrolled w.r.t. external state" approach.

### Infinite scroll + reset semantics

- **D-D1:** **`@tanstack/svelte-virtual` `createVirtualizer` + sentinel inside the virtualizer.** Replace the current hand-rolled scrollTop math with `@tanstack/svelte-virtual`'s `createVirtualizer`. The virtualizer windows the DOM. An `IntersectionObserver` watches a sentinel element placed at (or N rows before) the virtual list tail. When the sentinel intersects the viewport, DataTable dispatches `sendAction('fetch-rows', { offset, limit })`. On server response (data patch appending rows), the virtualizer recomputes its range and the sentinel re-anchors to the new tail. Canonical TanStack ecosystem pairing — both libraries are TanStack and well-documented together. The fetching guard (currently `let fetching = $state(false)`) carries over conceptually but now lives inside the sentinel observer callback rather than a `$effect` watching scrollTop.

- **D-D2:** **Sort/filter reset = server sends Render replacement; client just resets scrollTop.** When the user changes a filter or sort, the server's handler builds a NEW `RenderMessage` for the table's surface (or a `PatchMessage` with a data `set` op replacing the bound collection wholesale at its root path). When the bound collection swaps out from under DataTable (Svelte 5 reactivity), DataTable resets `scrollTop` to 0 and re-arms the sentinel. Single source of truth = server. Client never mutates the protocol data store directly. Matches Marionette's existing convention (every CRM list re-render goes through a fresh `Render`). The first chunk arrives in the same response, sized by the existing `DataTable.props.page_size`.

- **D-D3:** **Both end-of-data contracts supported: `total_rows` when known, fewer-than-limit fallback otherwise.** DataTable accepts an OPTIONAL `total_rows` prop. If set, the sentinel idles when `rows.length >= total_rows` (and a row-count summary like "237 contacts" becomes possible — useful for v2 TABLE-05). If unset, the sentinel idles when a `fetch-rows` response returns fewer rows than the requested `limit`. Server picks the contract per screen — expensive `COUNT(*)` queries only happen when `total_rows` matters. Both code paths must be tested. Document the contract in DataTable's prop docs and in the spec for the `data-table` component type.

### Column visibility

- **D-E1:** **`DropdownMenu` trigger in DataTable's top region; per-mount state, no persistence.** DataTable renders a "Columns" DropdownMenu trigger button in its top region (next to or after the filter bar, per the recipe layout). Each column appears as a `DropdownMenu.CheckboxItem` driven by a local Svelte `$state` `Map<columnKey, boolean>`. Initial visibility comes from props (default: all visible unless backend marks `col.hidden_default = true`). Toggling persists for the lifetime of the component mount only — navigating away and back resets to backend defaults. No localStorage, no server round-trip. If a future phase wants persistence, it can add it without disturbing this layer.

- **D-E2:** **Backend `hidden_default` per column.** `TableColumn` gains an optional `hidden_default: bool` field. CRM handlers can ship sensible defaults per screen (e.g., hide the audit log payload column by default). Users can still toggle it visible via the dropdown.

### Cell rendering

- **D-F1:** **`column.kind` enum with built-in renderers.** `TableColumn` protocol shape gains an optional `kind: 'text' | 'badge' | 'actions' | 'date' | 'number'` field (default `'text'`). DataTable maps each kind to a per-kind Svelte snippet that becomes the column's TanStack `cell` callback via `createRawSnippet` + `renderSnippet` (recipe pattern). Specifically:
  - `text` — `String(value)` (default)
  - `badge` — shadcn `Badge` component with optional variant map (e.g., `{ success: 'default', error: 'destructive' }`)
  - `actions` — expects `rowData[col.key]` to be an array of `{ label, action }` objects, renders as a `DropdownMenu` (matches existing CRM contact/company row-action shape and matches the recipe's `data-table-actions` component)
  - `date` — formats via `Intl.DateTimeFormat`
  - `number` — right-aligns with `Intl.NumberFormat`

  Solves the latent "actions column renders `[object Object]`" bug in current DataTable. Extensible — new kinds add a snippet on the TS side and an enum variant on the Rust side. Phase 13 implements all five initial kinds.

### Backend builder ergonomics

- **D-G1:** **Fluent additions on the existing `DataTable` struct in `builders/standard.rs`.** Extend the existing `#[derive(ComponentBuilder)] DataTable` struct to:
  ```rust
  pub struct DataTable {
      pub columns: Vec<TableColumn>,
      #[builder(optional)] pub page_size: Option<u32>,
      #[builder(optional)] pub total_rows: Option<u64>,
      #[builder(optional)] pub filters: Option<Vec<Filter>>,
      #[builder(optional)] pub row_id_key: Option<String>,
  }
  ```
  with `Filter` and `ColumnKind` enums in the same module. `TableColumn` gains `kind: Option<ColumnKind>` and `hidden_default: Option<bool>`. Fluent helpers: `Filter::text(id)`, `Filter::select(id, options)`, `Filter::date_range(id)`; on `TableColumn`: `.kind(ColumnKind::Badge)`, `.hidden_default(true)`; on `DataTable`: `.filter(...)`, `.total_rows(n)`, `.row_id_key("id")`. Same builder pattern as the rest of the marionette crate. Schema additions in `spec/schemas/data.yaml` mirror the Rust types.

### Claude's Discretion

- Exact debounce implementation (Svelte action vs `setTimeout` in `$effect`) — planner picks
- Sentinel placement: at the very last virtual row vs. N rows before the end (small UX tradeoff — flicker vs. premature fetch)
- DropdownMenu "Columns" button placement within the top region (next to filter bar, after it, or right-aligned) — visual decision for the planner / UI-spec phase
- Specific badge variant map defaults (which colors map to which conventional status names)
- Whether the sentinel uses `IntersectionObserver` directly or a small wrapper Svelte action
- Exact migration order for the four CRM handlers (audit, company, contact, user) — can be done as one plan or split per handler
- Whether to keep the existing `DataTable.browser-test.ts` and update assertions, or rewrite from scratch given the recipe-based rewrite

### Post-research refinements (added 2026-04-11 after 13-RESEARCH.md)

These decisions supplement the original CONTEXT and are derived from discoveries during research. Locked — the planner treats them as non-negotiable.

- **D-H1: Generic server-side `fetch-rows` handler keyed on component `source` id.** **Research discovered that the frontend's existing `sendAction('fetch-rows', ...)` dispatch is dead code today — no backend handler is registered for it, so infinite scroll has never actually fired end-to-end.** Phase 13 adds a new generic `fetch_rows` handler that takes `payload: { source: string, offset: u32, limit: u32 }` and internally dispatches to the right query function based on `source` (e.g., `'contact_list'`, `'company_list'`, `'audit_list'`, `'user_list'`). Frontend dispatches `sendAction('fetch-rows', { source, offset, limit })`. DataTable.props gains an implicit or explicit `source` identifier the backend embeds at render time. One handler covers all four CRM list screens; future list screens add a new branch.

- **D-H2: Enable `total_rows` for ALL four CRM list handlers at launch.** Add a `COUNT(*)` query (with the same `WHERE` clause as the page query) to each of `handle_audit_list`, `handle_contact_list`, `handle_company_list`, `handle_user_list`. SQLite COUNT is cheap at CRM scale (hundreds of rows). Ensures consistent contract across screens and exercises the `total_rows` code path. v2's row-count status bar (TABLE-05) becomes a one-liner later. The fewer-than-limit fallback path still exists in DataTable as a safety net AND is exercised in unit tests (so both code paths stay live).

- **D-H3: Stale-response discard via DataTable-local action-id tracking.** Research confirmed the server guarantees FIFO ordering on a single WebSocket connection (see `backend/crates/marionette/src/ws.rs` `read_loop` — actions dispatched serially, responses written through a single mpsc channel to a single writer task). This means the text-filter race ("Alic" vs "Alice") is physically impossible — filter responses arrive in order. The ONLY real race is `fetch-rows` interleaved with `filter`/`sort`. The fix is:
  1. Extend `frontend/src/lib/transport/dispatcher.ts` `sendAction` to RETURN the generated `action.id` (string) to the caller.
  2. The new backend `fetch_rows` handler echoes `ctx.action.id.clone()` into the `PatchMessage.id` field (already the established correlation convention — `confirmOptimistic` in `init.ts` already consumes this shape).
  3. DataTable tracks `lastFetchRowsActionId: string | null` in local `$state`. When a filter or sort change fires, it clears the tracked id. When a `fetch-rows` dispatch fires, it stores the returned id. The patch handler compares incoming patch `id` to the tracked id and drops the patch if they don't match.
  No protocol changes, no new schemas, no new dispatcher infrastructure. Component-scoped fix.

- **D-H4: Fold the TextInput `input_type` bug fix + seed-data bump into Phase 13.** Two small side-concerns surface:
  1. **TextInput `input_type` bug** — flagged in Phase 12 research: `TextInput.svelte` reads `props.type` but the backend serializes `props.input_type`, so password fields render as text. One-task fix (align the Svelte component to `props.input_type` with a fallback). Added as a small plan (e.g., Plan 13-0X). Rationale: Phase 13 already touches form-adjacent components; FormScreen Phase 14 shouldn't inherit a stale bug.
  2. **Seed data volume** — Phase 13's infinite-scroll E2E test needs more than `page_size` rows in at least one CRM entity (contacts is the obvious choice) to actually exercise the sentinel. `backend/crates/crm-demo/src/seed.rs` must be bumped to produce e.g. `>2 × page_size` contacts. Added as a small task (could live inside the infinite-scroll E2E plan or as a sibling seed plan).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project state and prior phases
- `.planning/REQUIREMENTS.md` §DataTable — TABLE-01 (filter bar), TABLE-02 (IntersectionObserver sentinel), TABLE-03 (column visibility), and v2 deferrals TABLE-04..07
- `.planning/PROJECT.md` — milestone goals and v1.1 scope
- `.planning/ROADMAP.md` Phase 13 — phase goal, depends-on (Phase 12), success criteria
- `.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md` — defines node-patching semantics, surface architecture, and explicitly hands the orphan `TableScreen.svelte` resolution to Phase 13
- `.planning/phases/12-protocol-node-patching-appshell/12-VERIFICATION.md` — Phase 12 success criteria and verification, especially the data-patch / surface story DataTable's row prefetch will rely on

### Protocol specs that Phase 13 mutates
- `spec/PROTOCOL.md` — JSON Pointer data binding, RenderMessage / PatchMessage shapes, surface model. The new filter spec, column kind enum, and totalRows contract must be documented here.
- `spec/schemas/data.yaml` — schema additions for the new `DataTable` props (filters, totalRows, hidden_default, kind)
- `spec/schemas/message.yaml` — verify no message-level changes are needed (filter actions and fetch-rows actions are normal `ActionMessage`/`PatchMessage` traffic; document if any changes ARE needed)

### Frontend code that Phase 13 rewrites or deletes
- `frontend/src/lib/components/table/DataTable.svelte` — REWRITTEN to recipe shape
- `frontend/src/lib/components/table/DataTable.browser-test.ts` — updated to assert recipe shape, filter dispatch, sentinel behavior, column visibility
- `frontend/src/lib/components/screen/TableScreen.svelte` — DELETED
- `frontend/src/lib/components/screen/TableScreen.browser-test.ts` — DELETED
- `frontend/src/lib/components/ui/table/*` — shadcn Table primitives (already installed in Phase 10/11), used by recipe
- `frontend/src/lib/components/ui/input/`, `select/`, `button/`, `dropdown-menu/`, `badge/` — shadcn primitives the new DataTable composes
- `frontend/src/lib/store/data.svelte.ts` — bound collection access pattern (read-only in DataTable)
- `frontend/src/lib/transport/dispatcher.ts` — `sendAction` API + check for any existing per-action sequence/correlation tracking that helps with D-C2

### Backend code that Phase 13 extends or migrates
- `backend/crates/marionette/src/builders/standard.rs` — `DataTable` struct extension, `Filter` and `ColumnKind` enums, fluent helpers
- `backend/crates/marionette/src/builders/` — other builder files for context on the established `#[derive(ComponentBuilder)]` pattern
- `backend/crates/crm-demo/src/handlers/audit.rs` — migrate to retire TableScreen
- `backend/crates/crm-demo/src/handlers/contact.rs` — migrate to retire TableScreen (largest handler — has the most complex filter shape: search + company + tags + date range)
- `backend/crates/crm-demo/src/handlers/company.rs` — migrate to retire TableScreen
- `backend/crates/crm-demo/src/handlers/user.rs` — migrate to retire TableScreen

### External library docs (research-phase reading)
- https://shadcn-svelte.com/docs/components/data-table — the recipe (full Svelte version, including filter, visibility, sort, pagination — Phase 13 ignores pagination since infinite scroll replaces it)
- https://tanstack.com/table/v8/docs — TanStack Table Core API, especially the manual mode flags (we use `manualSorting` for sort but NOT `manualFiltering`) and the column visibility API
- https://tanstack.com/virtual/latest/docs/framework/svelte/svelte-virtual — `@tanstack/svelte-virtual` `createVirtualizer` API
- https://tanstack.com/virtual/latest/docs/introduction — virtualizer concepts, sentinel patterns, infinite scroll examples

### Codebase intel (for established conventions)
- `.planning/codebase/CONVENTIONS.md` — Svelte component naming, test naming
- `.planning/codebase/STRUCTURE.md` — frontend layout
- `.planning/codebase/STACK.md` — current dep landscape (verify what's already installed)
- `.planning/codebase/TESTING.md` — browser-test patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`frontend/src/lib/components/ui/table/*`** — shadcn Table primitives (`Table.Root`, `Header`, `Body`, `Row`, `Head`, `Cell`) are already installed and used by current DataTable. The recipe uses these unchanged.
- **`frontend/src/lib/components/ui/dropdown-menu/`** — shadcn DropdownMenu (used elsewhere in CRM for row actions). The "Columns" toggle and the `actions` cell-kind both reuse this.
- **`frontend/src/lib/components/ui/input/`**, **`select/`**, **`button/`**, **`badge/`** — all shadcn primitives required by the recipe and the cell kinds, already in the project from Phase 10/11.
- **`frontend/src/lib/store/data.svelte.ts` `getData(surface, bind)` / `getAllData(surface)`** — DataTable already reads keyed collection rows via this. Pattern stays.
- **`frontend/src/lib/transport/dispatcher.ts` `sendAction`** — DataTable already dispatches `sort` and `select-row` actions via this. Same pattern for `filter` and `fetch-rows`.
- **The existing `rowIdKey` prop and keyed-collection row pattern** in current DataTable.svelte (`Object.entries(rawData)`) — keeps row identity for TanStack's `getRowId` config and for node-patching support added in Phase 12.
- **Backend `Payload<T>::from_context` pattern** in `crm-demo/src/handlers/` — current convention for typed action payload deserialization. The new `filter` action handlers reuse this.
- **The `nav_active_patch(...)` helper** used in `contact.rs:450` and other CRM handlers — no change, still applies after the Container migration.

### Established Patterns

- **Adjacency-list composition** — Marionette's "build a Container with children" pattern (used everywhere, including Phase 12's AppShell) is exactly how the migrated CRM handlers will compose `[Heading, …toolbar Buttons, DataTable]` instead of `TableScreen`.
- **`#[derive(ComponentBuilder)]` macro pattern** in `builders/standard.rs` — every existing builder uses it; the extended `DataTable` follows the same recipe.
- **Fluent slot/method builders** — Phase 12's hand-written `AppShell` builder establishes the convention for builders with non-trivial helper methods. The Phase 13 `DataTable` extension follows the auto-derived path (since slot semantics aren't needed) but mirrors the same call-site shape.
- **Fine-grained reactivity preserves focus across patches** — Phase 12's surface store rewrite (D-A6) is the reason a debounced filter input retains focus while patches replace its sibling rows. DataTable's filter bar inherits this for free as long as the filter inputs are addressed by stable IDs.
- **Surface-scoped data binding** — DataTable reads its bound collection from a single surface; Phase 12's per-surface state model means filter resets in one DataTable don't disturb another.

### Integration Points

- **Where `TableScreen` is used today** — search `backend/crates/crm-demo/src/handlers/` for `TableScreen::new` (or whatever the builder is named) to enumerate every call site. All four CRM list handlers (audit, company, contact, user) are affected.
- **Where the `data-table` component type is registered on the frontend** — `frontend/src/lib/registry/defaults.ts` currently maps `'data-table'` → `DataTable.svelte`. The mapping stays; only the implementation changes.
- **`spec/schemas/data.yaml`** — additions for the new `DataTable` props. Schema validation tests in Phase 12's protocol-conformance E2E (added in 12-08) will catch schema/runtime drift.
- **The existing `'sort'`, `'select-row'`, and `'fetch-rows'` actions** dispatched from DataTable — these stay. The new `'filter'` action is a peer.
- **The dispatcher's request/response correlation** (touched in Phase 12 patch wiring) — needs investigation for D-C2 (stale-response discard). May need a small extension if no per-action sequence number exists today.

</code_context>

<specifics>
## Specific Ideas

- **"Hand-rolling things when it comes to UI design is not ideal"** — explicit user feedback during the Area 1 discussion. Phase 13 adopts the framework recipe rather than reinventing it, even though the recipe pulls in TanStack as a runtime dependency.
- **Match the shadcn data-table recipe LAYOUT exactly:** filter bar at top, "Columns" dropdown trigger button to the right of the filter bar, table rounded-md border below, virtualized row rendering, sentinel at the tail. The recipe is the visual contract.
- **Solve the latent "actions column renders `[object Object]`" bug** — current CRM contact and company tables ship per-row action arrays in the data, but DataTable.svelte just `String()`s them into the cell. The new `kind: 'actions'` cell renderer makes those finally render as DropdownMenus, matching the recipe's `data-table-actions` component.
- **Filter UX feels live** — no Apply button. Type → 300ms → results update. Modern data-grid feel. Matches Linear/Notion-class search experiences.

</specifics>

<deferred>
## Deferred Ideas

- **Row selection + bulk actions** (TABLE-07) — v2. The recipe shows it; we're skipping it for v1.1.
- **Per-row dropdown action menus as a separate column type** (TABLE-04) — partially covered by `kind: 'actions'` but not the full menu+modal flow for "Edit/Delete/Move…" — v2.
- **Row-count status bar "Showing X of Y"** (TABLE-05) — the optional `total_rows` prop laid down in D-D3 makes this a one-liner addition later. v2.
- **Empty-state illustration with friendly message** (TABLE-06) — v2. Recipe's "No results" text suffices.
- **Column visibility persistence across reloads** — out of scope. No localStorage, no server round-trip. Future phase if user demand surfaces.
- **Sticky table header during virtualized scroll** — possibly a v2 polish item. TanStack Virtual supports it but the planner can decide whether to wire it up in Phase 13.
- **Keyboard navigation (arrow keys to move row focus, Enter to fire row action)** — v2. Not in success criteria.
- **Sentinel-aware accessibility tuning beyond defaults** (`aria-rowcount`, screen-reader behavior) — v2.
- **CRM-wide filter bar audit / consistency cleanup** — Phase 15 (CRM Migration & Validation). Phase 13 only migrates the four list handlers JUST enough to retire `TableScreen` and exercise the new DataTable end-to-end.
- **Loading skeleton during sort/filter reset** — out of scope; the data swap is fast enough that a skeleton would flash. Revisit if real measurements show a perceptible gap.

</deferred>

---

*Phase: 13-datatable-enhancements*
*Context gathered: 2026-04-11*
