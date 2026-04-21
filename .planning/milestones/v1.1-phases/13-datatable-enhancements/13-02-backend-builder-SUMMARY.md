---
phase: 13
plan: 02
subsystem: backend/builders
tags: [backend, builders, datatable, filter, column-kind, serde]
requires:
  - marionette::builders::standard (existing DataTable struct)
  - marionette_macros::ComponentBuilder derive
provides:
  - Filter enum (Text/Select/DateRange)
  - ColumnKind enum (Text/Badge/Actions/Date/Number)
  - Extended TableColumn (kind, hidden_default, Default impl)
  - Extended DataTable (total_rows, filters, row_id_key)
  - DataTableBuilder::filter() hand-written append-setter
affects:
  - backend/crates/marionette/src/builders/standard.rs
  - backend/crates/crm-demo/src/handlers/audit.rs
  - backend/crates/crm-demo/src/handlers/company.rs
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/crm-demo/src/handlers/user.rs
tech-stack:
  added: []
  patterns:
    - "Hand-written append-setter on derived ComponentBuilder (mirrors Phase 12 AppShellBuilder slot helpers)"
    - "serde tagged enum with kebab-case kind values for wire contract"
    - "#[derive(Default)] on TableColumn to allow ..Default::default() spread in existing struct literals"
key-files:
  created: []
  modified:
    - backend/crates/marionette/src/builders/standard.rs
    - backend/crates/crm-demo/src/handlers/audit.rs
    - backend/crates/crm-demo/src/handlers/company.rs
    - backend/crates/crm-demo/src/handlers/contact.rs
    - backend/crates/crm-demo/src/handlers/user.rs
decisions:
  - "Implemented D-G1 fluent additions on existing DataTable struct without splitting into a new type"
  - "Added #[derive(Default)] on TableColumn and spread ..Default::default() into 36 existing struct literals to keep crm-demo compiling; full fluent migration deferred to Plan 13-06"
  - "Hand-wrote DataTableBuilder::filter() as append-setter to work around the derive macro's replace-semantics for Option<Vec<T>>"
metrics:
  tasks_planned: 2
  tasks_completed: 2
  duration_minutes: ~12
  completed: 2026-04-10
requirements: [TABLE-01, TABLE-02, TABLE-03]
---

# Phase 13 Plan 02: Backend Builder Extension Summary

**One-liner:** Extended the Rust `DataTable` builder with `Filter`/`ColumnKind` enums, hand-written append-style `.filter()` helper, and `total_rows`/`row_id_key`/`hidden_default` fields so CRM handlers can declare the full Phase 13 DataTable wire shape fluently.

## Commits

| Task | Hash      | Message                                                                        |
| ---- | --------- | ------------------------------------------------------------------------------ |
| 1    | `34174de` | `feat(13-02): extend DataTable builder with Filter, ColumnKind, total_rows, row_id_key` |
| 2    | `d9f1463` | `test(13-02): add end-to-end DataTable inventory test for Phase 13 shape`        |

## What Was Built

### `Filter` tagged enum (`standard.rs`)

Serde-tagged enum with three variants:

```rust
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Filter {
    Text { id, label?, placeholder?, span? },
    Select { id, label?, options, span? },
    DateRange { id, label?, span? },
}
```

Constructors: `Filter::text(id)`, `Filter::select(id, options)`, `Filter::date_range(id)`.
Setters: `.label(...)`, `.placeholder(...)` (text-only), `.span(...)`.
Wire-side `kind` tag serializes `text` / `select` / `date-range` — matching the D-B2 contract.

### `ColumnKind` enum (`standard.rs`)

```rust
#[serde(rename_all = "lowercase")]
pub enum ColumnKind { Text, Badge, Actions, Date, Number }
```

Serializes as lowercase strings (`"text"`, `"badge"`, `"actions"`, `"date"`, `"number"`). `Copy + Eq + PartialEq` so consumers can compare variants.

### Extended `TableColumn`

Added two optional fields (`kind: Option<ColumnKind>`, `hidden_default: Option<bool>`), `#[derive(Default)]`, and four ergonomic builder methods: `TableColumn::new(k, l)`, `.sortable()`, `.kind(...)`, `.hidden_default(...)`.

### Extended `DataTable`

Three new `#[builder(optional)]` fields — `total_rows: Option<u64>`, `filters: Option<Vec<Filter>>`, `row_id_key: Option<String>`.

### Hand-written `DataTableBuilder::filter()`

```rust
impl DataTableBuilder {
    #[must_use]
    pub fn filter(mut self, f: Filter) -> Self {
        match self.filters.as_mut() {
            Some(existing) => existing.push(f),
            None => self.filters = Some(vec![f]),
        }
        self
    }
}
```

Same append-pattern Phase 12 used for `AppShellBuilder::slot()`. The derived setter for `filters` (which is `Option<Vec<Filter>>`) would be a REPLACE-setter and would clobber prior calls — the hand-written method APPENDS so `.filter(...).filter(...).filter(...)` composes correctly.

### Macro-generated setter names (confirmed)

The `#[derive(ComponentBuilder)]` macro exposes each `#[builder(optional)]` field as a same-named setter that takes `Option<T>`'s inner `T`:

| Field           | Setter call site                 |
| --------------- | -------------------------------- |
| `page_size`     | `.page_size(50u32)`              |
| `total_rows`    | `.total_rows(237u64)`            |
| `row_id_key`    | `.row_id_key("id")` (string impl Into) |
| `filters`       | `.filters(Vec<Filter>)` (replace) — prefer `.filter(f)` |

All test expectations validated these names.

## Realistic JSON Shape (for Plan 05 reader)

The `data_table_phase13_example_serializes_correctly` test produces a `data-table` component whose `props` serialize as:

```json
{
  "columns": [
    { "key": "name", "label": "Name", "sortable": true },
    { "key": "email", "label": "Email" },
    { "key": "created", "label": "Created", "sortable": true, "kind": "date" },
    { "key": "actions", "label": "", "kind": "actions" },
    { "key": "internal_id", "label": "ID", "hidden_default": true }
  ],
  "page_size": 50,
  "total_rows": 237,
  "filters": [
    { "kind": "text", "id": "search", "label": "Search", "placeholder": "Filter contacts..." },
    { "kind": "select", "id": "company", "label": "Company",
      "options": [
        { "value": "", "label": "All companies" },
        { "value": "1", "label": "Acme" }
      ]
    },
    { "kind": "date-range", "id": "created", "label": "Created date" }
  ],
  "row_id_key": "id"
}
```

Key notes for Plan 05's frontend reader:

- `columns[].kind` is OMITTED on plain-text columns (don't assume it's always present — fall back to `'text'`).
- `columns[].hidden_default` is OMITTED unless explicitly set to `true`.
- `filters[].kind` uses the kebab form `date-range` (not `dateRange` or `date_range`).
- `filters[].label` / `.placeholder` / `.span` are all OMITTED when unset; `options` is always present on `select`.
- Top-level `total_rows`, `row_id_key`, `filters`, `page_size` are ALL snake_case and ALL omitted when unset.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking clippy lint] Switched `impl Default for TableColumn` to `#[derive(Default)]`**

- **Found during:** Task 1 verify step (`cargo clippy -p marionette --lib -- -D warnings`)
- **Issue:** clippy `derivable_impls` lint fired because the manual `impl Default` was entirely field-by-field defaults.
- **Fix:** Replaced the hand-written `impl Default` block with `#[derive(Default)]` on the struct; behavior identical.
- **Files modified:** `backend/crates/marionette/src/builders/standard.rs`
- **Commit:** `34174de`

No other deviations — the plan was executed exactly as written, including the scoped migration of `TableColumn { .. }` struct literals in crm-demo handlers (36 call sites across 4 files).

### Deferred Issues (out of scope)

- `cargo clippy -p crm-demo -- -D warnings` reports 76 pre-existing pedantic warnings (doc_markdown on `WsSession`, `too_many_lines` on `main`, etc). None are caused by or related to this plan's changes; they pre-date Phase 13. Not fixed — out of scope per SCOPE BOUNDARY rule. The plan's acceptance criterion only requires `cargo clippy -p marionette -- -D warnings` clean (which it is) and `cargo build -p crm-demo` success (which it is).

## CRM Handler Migration Summary

| File                                          | TableColumn literals patched |
| --------------------------------------------- | ---------------------------: |
| `backend/crates/crm-demo/src/handlers/audit.rs`   | 6 |
| `backend/crates/crm-demo/src/handlers/contact.rs` | 16 |
| `backend/crates/crm-demo/src/handlers/company.rs` | 9 |
| `backend/crates/crm-demo/src/handlers/user.rs`    | 5 |
| **Total**                                     | **36** |

Each patch is the minimal `..Default::default()` spread at the end of the struct literal. Plan 13-06 will migrate these call sites to the fluent `TableColumn::new(..).sortable()` API and wire up real `kind` / `hidden_default` / filter declarations per screen.

## Verification

```text
cd backend
cargo test -p marionette --lib builders::standard::tests   # 20 passed, 0 failed (7 new Task 1 tests + 1 new Task 2 test)
cargo clippy -p marionette --lib -- -D warnings            # clean
cargo build -p crm-demo                                    # clean
cargo test -p marionette --lib                             # 41 passed total
```

All plan acceptance greps:

| Criterion                                                           | Expected | Actual |
| ------------------------------------------------------------------- | -------: | -----: |
| `grep -c "pub enum Filter" standard.rs`                             | 1        | 1      |
| `grep -c "pub enum ColumnKind" standard.rs`                         | 1        | 1      |
| `grep -c "impl DataTableBuilder" standard.rs`                       | 1        | 1      |
| `grep -c "pub fn filter(mut self, f: Filter)" standard.rs`          | 1        | 1      |
| `grep -c 'rename_all = "kebab-case"' standard.rs`                   | ≥1       | 1      |
| `grep -c 'rename_all = "lowercase"' standard.rs`                    | ≥1       | 1      |

## Known Stubs

None. Every type and method added in this plan is fully wired and exercised by an inline test. The optional fields (`kind`, `hidden_default`, `total_rows`, `filters`, `row_id_key`) are new wire-shape capabilities — CRM handlers will start populating them in Plan 13-06, but the Rust-side contract they consume is complete and tested here.

## Self-Check: PASSED

**Commits verified present in git log:**

- `34174de` — FOUND
- `d9f1463` — FOUND

**Files verified present on disk:**

- `backend/crates/marionette/src/builders/standard.rs` — FOUND
- `backend/crates/crm-demo/src/handlers/audit.rs` — FOUND
- `backend/crates/crm-demo/src/handlers/contact.rs` — FOUND
- `backend/crates/crm-demo/src/handlers/company.rs` — FOUND
- `backend/crates/crm-demo/src/handlers/user.rs` — FOUND

**Grep contracts verified** (see table above): 6/6 match expected counts.

**Plan success criteria:** all 6 plan-level success criteria green (Filter/ColumnKind/DataTableBuilder::filter exist, ergonomic TableColumn builder methods exist, Phase 13 end-to-end inventory test green, crm-demo compiles, zero clippy pedantic warnings in marionette).
