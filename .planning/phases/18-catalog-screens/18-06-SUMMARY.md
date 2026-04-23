---
phase: 18-catalog-screens
plan: 06
subsystem: backend
tags: [gallery, catalog, data-table, virtualization, column-kinds, filters, cat-03]

# Dependency graph
requires:
  - phase: 13-datatable
    provides: "DataTable + Filter + ColumnKind + TableColumn builders; frontend virtualized filter/column-viz DataTable.svelte"
  - phase: 17-gallery-crate-skeleton
    provides: "gallery-demo crate + #[gallery_demo] linkme auto-nav + /demo/<key>/<slot> bind convention"
  - plan: 18-03
    provides: "fixtures::synthetic_rows(500) shared generator + catalog-synthetic-rows fetch-rows source arm (paginated rows 51-500) + Edit/Delete/Duplicate actions injection"
provides:
  - "catalog::data_table::gallery_demo() → Vec<Node> — CAT-03 catalog screen composing DataTable with 7 columns (every ColumnKind) + 3 filters (text/select/date-range) + 500-row synthetic source"
  - "#[gallery_demo(key = \"catalog-data-table\", name = \"Catalog: Data Table\")] nav entry (auto-discovered via linkme DEMOS slice)"
  - "seed_for_key(\"catalog-data-table\") — object-map of rows 1-50 at /demo/catalog-data-table/rows keyed by stringified id, with actions injection matching fetch_rows.rs shape"
  - "catalog_rows_initial_object_map() private helper — bridges fixtures::synthetic_rows(50) + per-row actions injection for initial render"
affects: [18-08-uat, 19-exer-03-scale]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Catalog screen pattern consumed verbatim from Plan 18-04/18-05 (outer Container with locked OUTER_CLASS + title Heading level=1 + intro Text + composed child component; build_with_children flattens the 4-tuple)"
    - "Every-ColumnKind demonstration: Number (id, score), Text default (name, email), Badge (status), Date (joined_at), Actions (actions) — single DataTable exercises the full frontend per-kind snippet table"
    - "hidden_default=true on status + actions so the column-visibility dropdown has visible toggle affordance on first paint (UI-SPEC §CAT-03 lines 472-482)"
    - "Initial render seed + paginated fetch-rows share identical JSON shape (both inject the same [Edit, Delete, Duplicate] actions array) — no visual seam at the 50-row page boundary when IntersectionObserver fires for rows 51-500"
    - "seed_for_key arms remain order-sensitive: catalog-data-table landed before catalog-forms (matches ordinal order in the match arm block) and before the `_ =>` wildcard fall-through"

key-files:
  created:
    - "backend/crates/gallery-demo/src/catalog/data_table.rs (230 LOC + 8 unit tests)"
  modified:
    - "backend/crates/gallery-demo/src/catalog/mod.rs (add pub mod data_table)"
    - "backend/crates/gallery-demo/src/handlers/show.rs (add catalog-data-table arm + catalog_rows_initial_object_map helper + 3 unit tests)"

key-decisions:
  - "display_name (not name) is the correct DemoEntry field — plan's registered_demos_includes_catalog_data_table test used e.name, corrected to e.display_name per the macro emission (see marionette-macros/src/gallery_demo.rs:82 + marionette/src/gallery.rs:36). Matches Plan 18-04 catalog::buttons test pattern."
  - "DataTable.bind is a top-level component field (serialized as `bind` at the Component struct level, not nested in props) — test assertions use `t[\"bind\"]`, not `t[\"props\"][\"bind\"]`. Same pattern already used by data_table.rs leaf demo tests."
  - "Text-default columns (name, email) serialize with kind field ABSENT (not null) because TableColumn has #[serde(skip_serializing_if = \"Option::is_none\")] on kind. Tests use .is_none_or(serde_json::Value::is_null) to accept both missing and explicit null — robust against serializer config changes."
  - "Three new show.rs tests (not two as plan suggested) — added catalog_data_table_seed_spans_full_first_page to explicitly assert the 50-row boundary (ids 1 and 50 present, 51 absent, every row has 3 actions). Gives explicit regression coverage against a future off-by-one in catalog_rows_initial_object_map."
  - "catalog_rows_initial_object_map is a module-private fn (no pub) — it exists solely to break up the `seed_for_key` match arm body and keep D-4-C's seed_table_rows untouched. Downstream callers use seed_for_key, not the helper."

patterns-established:
  - "CAT catalog screen pattern is now stable across 3 plans (18-04 buttons, 18-05 forms, 18-06 data-table): outer Container (id='catalog-<name>-container' or '-root') with OUTER_CLASS 'flex flex-col gap-6 p-6' containing title H1 + intro Text + one or more composed components. Root id is asserted as v[0].0 by every test suite."
  - "Seed-path + bind-path alignment contract (G-05 lesson) is now enforced THREE times in the gallery: catalog-forms test asserts every /demo/catalog-forms/<suffix> bind has a matching seed entry; catalog-data-table seed matches the /demo/catalog-data-table/rows path the DataTable binds; both arms sit in the same seed_for_key match block for reviewer visibility."

requirements-completed: [CAT-03]

# Metrics
duration: 8m 30s
completed: 2026-04-23
---

# Phase 18 Plan 06: CAT-03 Data Table Catalog Screen Summary

**Shipped CAT-03 — one `DataTable` exercising every `ColumnKind` + 3-filter bar + column-visibility toggle + virtualized fetch-rows pagination against the shared 500-row synthetic generator. Initial render seeds rows 1-50 with actions injected; scroll triggers `catalog-synthetic-rows` fetch-rows arm for rows 51-500 via Plan 18-03's pipeline.**

## Performance

- **Duration:** ~8 min 30s
- **Started:** 2026-04-23T17:32:31Z
- **Completed:** 2026-04-23T17:40:59Z
- **Tasks:** 2
- **Files modified:** 3 (1 created, 2 modified)

## Accomplishments

- Created `catalog/data_table.rs` — single `DataTable` with 7 columns exercising every `ColumnKind` variant (Number, Text default, Badge, Date, Actions), 3 filters (Text name-search, Select status-filter, DateRange joined-range), `.source("catalog-synthetic-rows")`, `.bind("/demo/catalog-data-table/rows")`, `.row_id_key("id")`, `.page_size(50)`, `.total_rows(500)`.
- Registered the nav entry `Catalog: Data Table` with key `catalog-data-table` via `#[gallery_demo]` — auto-discovered by the linkme `DEMOS` slice, verified by the existing `nav_auto_discovery` integration test.
- Extended `seed_for_key` with a `catalog-data-table` arm seeding the first 50 rows of `fixtures::synthetic_rows(500)` as an object-map keyed by stringified id at `/demo/catalog-data-table/rows`. Every seeded row carries an `actions: [Edit, Delete, Duplicate]` array matching the fetch-rows handler's shape — initial render and paginated pages share identical JSON with no visible seam at the page boundary.
- 11 new tests all green (8 catalog::data_table + 3 handlers::show) on top of the pre-existing 43-test baseline — 54 unit tests total in gallery-demo, 0 failed. Integration tests (`nav_auto_discovery`, `smoke_boot`) still pass.
- `cargo clippy -p gallery-demo --all-targets --all-features -- -D warnings` clean.
- `cargo build --workspace --all-features` green (crm-demo + gallery-demo + all workspace members link cleanly).

## Task Commits

1. **Task 1: catalog/data_table.rs + mod.rs wire-up** — `24e095d` (feat)
2. **Task 2: seed_for_key catalog-data-table arm + catalog_rows_initial_object_map helper** — `240aceb` (feat)

## Files Created/Modified

- `backend/crates/gallery-demo/src/catalog/data_table.rs` **(created, 230 LOC + 8 unit tests)** — CAT-03 catalog fn. Single outer `Container` wrapping title H1 + intro Text + the composed DataTable. The DataTable is a leaf (no descendants to flatten), so `build_with_children` emits exactly 4 nodes: `[container-root, title, intro, table]`.
- `backend/crates/gallery-demo/src/catalog/mod.rs` — Added `pub mod data_table;` in alphabetical position between `buttons` and `forms`.
- `backend/crates/gallery-demo/src/handlers/show.rs` — Added the `catalog-data-table` arm to `seed_for_key`, inserted before the `catalog-forms` arm (the match arm order mirrors ordinal plan number: 18-04 buttons → 18-05 forms → 18-06 data-table, which is also alphabetical). Added the `catalog_rows_initial_object_map` private helper. Added 3 unit tests covering row shape + actions injection, generator alignment, and first-page boundary.

## Diff Summary

| File | Change | Lines |
|------|--------|-------|
| `catalog/mod.rs` | 1 line added (`pub mod data_table;`) | +1 |
| `catalog/data_table.rs` | full-file create (230 LOC incl. 8 tests) | +230 |
| `handlers/show.rs` | 1 match arm (+12) + helper fn (+20) + 3 tests (+73) | +118 |

**Test counts:** 8 new `catalog::data_table::tests` + 3 new `handlers::show::tests` = 11 new. Pre-existing: 43 → post-plan: 54 total unit tests, +2 integration tests (unchanged) = 56 tests total in gallery-demo.

## Column Definitions

| # | key | label | kind (JSON) | hidden_default | Rationale |
|---|-----|-------|-------------|----------------|-----------|
| 1 | `id` | `ID` | `number` | — | Right-aligned + `Intl.NumberFormat` per Phase 13 D-F1 |
| 2 | `name` | `Name` | *(absent — Text default)* | — | Default text cell; the most visually prominent field |
| 3 | `email` | `Email` | *(absent — Text default)* | — | Default text cell |
| 4 | `status` | `Status` | `badge` | `true` | Renders shadcn `Badge`; hidden so column-viz dropdown has a toggle target |
| 5 | `score` | `Score` | `number` | — | Right-aligned numeric |
| 6 | `joined_at` | `Joined` | `date` | — | `Intl.DateTimeFormat`-formatted ISO date |
| 7 | `actions` | *(empty label)* | `actions` | `true` | DropdownMenu of `{label, action}` items; hidden so the toggle is obvious |

Text-default columns (`name`, `email`) serialize with the `kind` field OMITTED from JSON (per `#[serde(skip_serializing_if = "Option::is_none")]` on `TableColumn.kind`).

## Filter Bar

| # | kind (JSON) | id | label | placeholder / options |
|---|-------------|-----|-------|-----------------------|
| 1 | `text` | `name-search` | `Name` | placeholder: `Filter by name…` |
| 2 | `select` | `status-filter` | `Status` | options: `[{value:"active", label:"Active"}, {value:"inactive", label:"Inactive"}, {value:"pending", label:"Pending"}]` |
| 3 | `date-range` | `joined-range` | `Joined` | — (two date inputs; span unset) |

## Sample Initial-Seed JSON (row id=1)

`seed_for_key("catalog-data-table")["demo"]["catalog-data-table"]["rows"]["1"]` — deterministic per the LCG-seeded generator (same as Plan 18-03 SUMMARY sample):

```json
{
  "id": 1,
  "name": "Paul Davis",
  "email": "paul.davis@example.com",
  "status": "pending",
  "score": 444,
  "joined_at": "2024-12-01",
  "actions": [
    { "label": "Edit",      "action": { "type": "click", "name": "gallery-demo/noop" } },
    { "label": "Delete",    "action": { "type": "click", "name": "gallery-demo/noop" } },
    { "label": "Duplicate", "action": { "type": "click", "name": "gallery-demo/noop" } }
  ]
}
```

Full 50-row seed is an object-map `{"1": ..., "2": ..., ..., "50": ...}` at `/demo/catalog-data-table/rows`. Rows 51-500 arrive via `catalog-synthetic-rows` fetch-rows (Plan 18-03) when the IntersectionObserver sentinel fires; they share the exact same shape including the `actions` injection.

## Decisions Made

- **`display_name` bug in the plan's test template (Rule 1 fix)** — Plan's `registered_demos_includes_catalog_data_table` test used `assert_eq!(e.name, "Catalog: Data Table")` but the `DemoEntry` field is `display_name` (see `marionette::gallery::DemoEntry` at `backend/crates/marionette/src/gallery.rs:36`; the macro emits `display_name: #display_name` at `marionette-macros/src/gallery_demo.rs:82`). Corrected inline during Task 1 implementation; mirrors the `catalog::buttons::tests::registered_demos_includes_catalog_buttons` assertion pattern.
- **`bind` serialization placement** — Plan's test sketch assumed `t["props"]["bind"]`, but `DataTable.bind` is inherited from the `ComponentBuilder` macro as a top-level `Component.bind` field (not a prop). Test asserts `t["bind"] == "/demo/catalog-data-table/rows"`. Verified against the `marionette-protocol::Component` struct shape.
- **Text-default column `kind` absence** — Tests assert `kind` is absent (not `null`) for `name` + `email` columns because `TableColumn.kind` has `#[serde(skip_serializing_if = "Option::is_none")]`. Used `.is_none_or(serde_json::Value::is_null)` to be robust against future serializer config changes that might start emitting explicit nulls.
- **Added a third seed test (`catalog_data_table_seed_spans_full_first_page`)** — Plan specified 2 tests; added one more giving explicit regression coverage against a future off-by-one in `catalog_rows_initial_object_map`. Asserts id "1" and "50" present, "51" absent, and every row has a 3-entry actions array (catches silent regressions in the action injection loop).
- **Match-arm placement (not hand-picked alphabetic)** — Placed the `catalog-data-table` arm immediately BEFORE `catalog-forms` in the `seed_for_key` match block. The three catalog arms now appear in order `catalog-buttons` (pre-existing from 18-04) → `catalog-data-table` (this plan) → `catalog-forms` (pre-existing from 18-05), which is both alphabetical AND their natural seeding order (empty / rows / value-table). Comment reflects the rationale (ordinal/alphabetical).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug / clippy::redundant_closure_for_method_calls] Replaced closure arg with method reference**
- **Found during:** Task 1 (clippy -D warnings run, post-test-pass)
- **Issue:** Three tests used `.is_none_or(|v| v.is_null())` which tripped `clippy::redundant_closure_for_method_calls` (enforced by the crate's `#![warn(clippy::pedantic)]`).
- **Fix:** Replaced with `.is_none_or(serde_json::Value::is_null)` — identical semantics, clippy-clean. No test behaviour change; determinism preserved.
- **Files modified:** `backend/crates/gallery-demo/src/catalog/data_table.rs`
- **Verification:** `cargo clippy -p gallery-demo --all-targets --all-features -- -D warnings` passes.
- **Committed in:** `24e095d` (Task 1 commit — caught before the commit, fixed inline)

**2. [Rule 1 — Bug correction in plan's test template] `e.name` → `e.display_name`**
- **Found during:** Task 1 (while authoring tests against the actual `DemoEntry` struct)
- **Issue:** Plan's `registered_demos_includes_catalog_data_table` test body asserted `assert_eq!(e.name, "Catalog: Data Table")`, but the field is `display_name` (confirmed at `marionette/src/gallery.rs:36` and via the catalog::buttons existing test).
- **Fix:** Used `e.display_name` — matches the macro's emission and the sibling plan's test convention.
- **Files modified:** `backend/crates/gallery-demo/src/catalog/data_table.rs`
- **Verification:** Test compiles + passes (`registered_demos_includes_catalog_data_table ... ok`).
- **Committed in:** `24e095d` (inline during initial write — never hit disk as `e.name`)

---

**Total deviations:** 2 auto-fixed (1 clippy lint + 1 plan-template bug), both Rule 1. Zero scope creep — both are surface corrections that preserve plan intent. Every plan acceptance-criterion grep hits the expected count.

## Issues Encountered

None — Task 1 RED→GREEN on first run; Task 2 RED→GREEN on first run. The only build blip was the 3 clippy closure lints, auto-fixed and reverified before commit.

## Threat Flags

None. The threat model enumerated in the plan already covers every new surface introduced by this plan:
- T-18-06-01 (DoS via 500 rows) — mitigated by virtualization + Plan 18-03's `limit.min(500)` cap in the handler (not re-verified here; unchanged).
- T-18-06-02 (Info disclosure) — synthetic generator, no PII; unchanged.
- T-18-06-03 (Column-visibility tampering) — frontend-local state, no server persistence.
- T-18-06-04 (Client-side filter bypass) — filters are UI-only, no server-side access gated by filter values.
- T-18-06-05 (XSS via row fields) — row fields are server-authored synthetic data; Svelte's default interpolation escapes.

No new network endpoints, auth paths, file access, or schema changes.

## Verification

All plan verification commands executed and passed:

- `cargo test -p gallery-demo --lib catalog::data_table` → **8 passed** ✓
- `cargo test -p gallery-demo --lib handlers::show` → **8 passed** (5 pre-existing + 3 new) ✓
- `cargo test -p gallery-demo` → **54 unit + 2 integration passed** ✓ (43 pre-existing + 11 new)
- `cargo clippy -p gallery-demo --all-targets --all-features -- -D warnings` → clean ✓
- `cargo build --workspace --all-features` → `Finished dev profile in 45.92s` ✓

Plan acceptance criteria (all satisfied):

- `test -f backend/crates/gallery-demo/src/catalog/data_table.rs` → exists ✓
- `grep -c 'key = "catalog-data-table"' …/catalog/data_table.rs` → `1` ✓
- `grep -c 'catalog-synthetic-rows' …/catalog/data_table.rs` → `4` (≥1 required) ✓
- `grep -c '/demo/catalog-data-table/rows' …/catalog/data_table.rs` → `4` (≥1 required) ✓
- `grep -c 'ColumnKind::Badge' …/catalog/data_table.rs` → `1` ✓
- `grep -c 'ColumnKind::Actions' …/catalog/data_table.rs` → `1` ✓
- `grep -c 'ColumnKind::Date' …/catalog/data_table.rs` → `1` ✓
- `grep -c 'Filter::text\|Filter::select\|Filter::date_range' …/catalog/data_table.rs` → `3` (≥3 required) ✓
- `grep -c '"catalog-data-table" =>' …/handlers/show.rs` → `1` ✓
- `grep -c 'catalog_rows_initial_object_map' …/handlers/show.rs` → `3` (≥2 required) ✓
- `grep -c 'crate::fixtures::synthetic_rows' …/handlers/show.rs` → `2` (≥1 required) ✓

## Known Stubs

None — every field on every column carries a real kind/hidden_default/label; every filter has a real id + label (+ placeholder or options); every seeded row has a fully populated Row payload + actions array; `total_rows=500` + `page_size=50` + `row_id_key="id"` are all concrete. No TODO markers, no empty arrays, no placeholder text.

## User Setup Required

None — no environment variables, no external services, no DB migrations.

## Next Phase Readiness

- **Plan 18-08 UAT** will verify end-to-end:
  - Nav entry `Catalog: Data Table` renders via `gallery-show` action with seeded rows visible on first paint (no flash-of-empty).
  - IntersectionObserver sentinel fires `fetch-rows` with `source=catalog-synthetic-rows`, receives rows 51-500, appends without visual seam.
  - Column-visibility dropdown surfaces `status` + `actions` (both hidden by default) and toggles work.
  - Text filter `name-search` filters client-side; Select `status-filter` filters by status enum; DateRange `joined-range` filters by `joined_at`.
  - All 5 ColumnKind variants render correctly (Number right-aligned / Text / Badge shadcn pill / Date formatted / Actions dropdown menu).
  - Mobile (≤640px) + desktop (≥1024px) viewports both render the filter bar + scroll-to-fetch behaviour.
- **Phase 19 EXER-03 ready** — The shared `fixtures::synthetic_rows(n)` can be called with `n=10_000` without change; only a new fetch-rows source arm needs adding alongside `catalog-synthetic-rows`.
- **No regressions** — Phase 17 `data-table` leaf demo at `/demo/data-table/rows` untouched (uses D-4-C-locked `seed_table_rows()` and `demo-rows` fetch-rows arm).

## Self-Check: PASSED

Verified post-SUMMARY:

- `backend/crates/gallery-demo/src/catalog/data_table.rs` — **FOUND** ✓
- `backend/crates/gallery-demo/src/catalog/mod.rs` — **FOUND** (existing, modified) ✓
- `backend/crates/gallery-demo/src/handlers/show.rs` — **FOUND** (existing, modified) ✓
- Task 1 commit `24e095d` — **FOUND** (`git log --oneline | grep 24e095d`) ✓
- Task 2 commit `240aceb` — **FOUND** (`git log --oneline | grep 240aceb`) ✓

---
*Phase: 18-catalog-screens*
*Completed: 2026-04-23*
