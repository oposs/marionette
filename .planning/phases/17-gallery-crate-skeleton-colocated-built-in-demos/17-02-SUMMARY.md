---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 02
subsystem: api
tags: [rust, refactor, builders, marionette, component-split, re-export-shim]

# Dependency graph
requires:
  - phase: 16-framework-hooks
    provides: "#[gallery_demo] macro, registered_demos() + DemoEntry contract, gallery cargo feature — unchanged by this plan"
provides:
  - "25 per-component builder files under backend/crates/marionette/src/builders/ (button.rs, text_input.rs, select.rs + SelectOption, checkbox.rs, container.rs, grid.rs, heading.rs, text.rs, side_nav.rs, nav_item.rs, nav_group.rs, surface_mount.rs, form.rs, textarea.rs, radio_group.rs + RadioOption, switch.rs, field_set.rs, field_separator.rs, data_table.rs + TableColumn/ColumnKind/Filter, modal.rs, toast.rs, confirm_dialog.rs, spinner.rs, error_display.rs, composites.rs for form_shell)"
  - "Re-export hub in builders/mod.rs declaring all per-component submodules + pub use re-exports"
  - "Preserved builders/standard.rs as a Option-A pub use super::{...::*} shim — all 12 external callers of marionette::builders::standard::* keep building unchanged"
  - "Natural per-component home for Plan 17-04's #[gallery_demo] sibling functions"
affects: [17-04-colocated-built-in-demos, 18-catalog-screens, 19-exerciser-screens]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-component file organization: one ComponentBuilder struct + its colocated props types + its own tests per .rs file"
    - "Option A re-export shim: pure-code reorganization behind a pub use surface that preserves deprecation-free external import paths"
    - "Meta-test at module hub: all_19_standard_types lives in builders/mod.rs #[cfg(test)] as the canonical 'every builder compiles' check"

key-files:
  created:
    - "backend/crates/marionette/src/builders/button.rs"
    - "backend/crates/marionette/src/builders/text_input.rs"
    - "backend/crates/marionette/src/builders/select.rs"
    - "backend/crates/marionette/src/builders/checkbox.rs"
    - "backend/crates/marionette/src/builders/container.rs"
    - "backend/crates/marionette/src/builders/grid.rs"
    - "backend/crates/marionette/src/builders/heading.rs"
    - "backend/crates/marionette/src/builders/text.rs"
    - "backend/crates/marionette/src/builders/side_nav.rs"
    - "backend/crates/marionette/src/builders/nav_item.rs"
    - "backend/crates/marionette/src/builders/nav_group.rs"
    - "backend/crates/marionette/src/builders/surface_mount.rs"
    - "backend/crates/marionette/src/builders/form.rs"
    - "backend/crates/marionette/src/builders/textarea.rs"
    - "backend/crates/marionette/src/builders/radio_group.rs"
    - "backend/crates/marionette/src/builders/switch.rs"
    - "backend/crates/marionette/src/builders/field_set.rs"
    - "backend/crates/marionette/src/builders/field_separator.rs"
    - "backend/crates/marionette/src/builders/data_table.rs"
    - "backend/crates/marionette/src/builders/modal.rs"
    - "backend/crates/marionette/src/builders/toast.rs"
    - "backend/crates/marionette/src/builders/confirm_dialog.rs"
    - "backend/crates/marionette/src/builders/spinner.rs"
    - "backend/crates/marionette/src/builders/error_display.rs"
    - "backend/crates/marionette/src/builders/composites.rs"
  modified:
    - "backend/crates/marionette/src/builders/mod.rs"
    - "backend/crates/marionette/src/builders/standard.rs"

key-decisions:
  - "Option A (shim-preserved standard.rs) over Option B (retire and edit 12 callers) — zero-caller-edit blast radius; CONTEXT.md §Claude's Discretion codified this choice"
  - "Meta-test all_19_standard_types relocated to builders/mod.rs hub (not any single component file) — it is the module-wide 'every builder compiles' contract"
  - "Cross-module import: data_table.rs imports SelectOption via use super::select::SelectOption (Filter::Select carries SelectOptions)"
  - "Cross-module import: composites.rs uses super::container::Container; Button/Form/Heading imports live only inside the test module (via super::super::{button,form,heading}) so the library surface of composites.rs stays minimal"

patterns-established:
  - "Per-component file: //! doc header + single-line module use of marionette_macros::ComponentBuilder + struct + optional impl + #[cfg(test)] mod tests — 25 instances in this plan"
  - "Meta-test at hub: builders/mod.rs owns the 'every builder' smoke test; per-component files own their own specific tests"
  - "Shim-preserved public API: builders/standard.rs reduces to `pub use super::{...::*};` — callers keep existing imports without #[deprecated] markers"

requirements-completed: []

# Metrics
duration: ~45min
completed: 2026-04-22
---

# Phase 17 Plan 02: Per-Component File Refactor of builders/standard.rs Summary

**Split 1398-line standard.rs into 25 per-component files; preserved marionette::builders::standard::* via Option A re-export shim; zero external caller edits; 71 lib tests still green.**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-04-22T09:24Z
- **Completed:** 2026-04-22T10:10Z
- **Tasks:** 2 (Task 1 split+shim; Task 2 workspace verification sweep — pure verification, no file edits)
- **Files modified:** 27 (2 rewritten: mod.rs + standard.rs; 25 newly created per-component files)

## Accomplishments

- `backend/crates/marionette/src/builders/standard.rs` (1398 lines, 19 ComponentBuilder structs + 4 props structs + `form_shell` helper + 42 tests) split into **25 per-component files** under `backend/crates/marionette/src/builders/`.
- `builders/mod.rs` rewritten as a re-export hub declaring every per-component submodule + `pub mod standard` (the shim) + `pub use <module>::*` for every submodule. Hosts the `all_19_standard_types` meta-test under `#[cfg(test)] mod tests`.
- `builders/standard.rs` reduced to a **16-line pure re-export shim** (`pub use super::{button::*, text_input::*, ...};`) — zero struct definitions, zero tests, zero helpers remain.
- All **12 external callers** of `marionette::builders::standard::*` (crm-demo × 7, gallery-smoke × 3 including two `.rs` + `.stderr` fixtures, marionette's own `app_shell.rs` line 244, crm-demo integration test × 1) continue to compile **unchanged** via the shim.
- **71 lib tests** preserved — same count and result as pre-refactor baseline. Tests distributed across 12 per-component `#[cfg(test)] mod tests` blocks + 1 composites test + 1 meta-test in mod.rs.

## File → Struct(s) Colocated

| File                     | Struct(s) + colocated props                                    | Tests moved |
|--------------------------|----------------------------------------------------------------|-------------|
| button.rs                | `Button`                                                       | 3 (button_builder, optional_fields_omitted, visibility_binding) |
| text_input.rs            | `TextInput`                                                    | 5 (text_input_builder, text_input_serializes_description, text_input_serializes_full_width, text_input_omits_description_when_not_set, custom_id) |
| select.rs                | `SelectOption` + `Select`                                      | 3 (select_serializes_description_and_full_width, select_serializes_placeholder_and_disabled, select_omits_new_optionals_when_not_set) |
| checkbox.rs              | `Checkbox`                                                     | 3 (checkbox_serializes_description_and_full_width, checkbox_omits_new_optionals_when_not_set, checkbox_preserves_existing_disabled_field) |
| container.rs             | `Container`                                                    | 2 (container_builder_with_children, children_method) |
| grid.rs                  | `Grid`                                                         | 1 (grid_with_optional_fields) |
| heading.rs               | `Heading`                                                      | 0 |
| text.rs                  | `Text`                                                         | 0 |
| side_nav.rs              | `SideNav`                                                      | 0 |
| nav_item.rs              | `NavItem`                                                      | 0 |
| nav_group.rs             | `NavGroup`                                                     | 0 |
| surface_mount.rs         | `SurfaceMount`                                                 | 2 (surface_mount_builder, surface_mount_builder_custom_id) |
| form.rs                  | `Form`                                                         | 0 |
| textarea.rs              | `Textarea`                                                     | 3 (textarea_basic_serialization, textarea_full_serialization, textarea_rows_is_u32) |
| radio_group.rs           | `RadioOption` + `RadioGroup`                                   | 2 (radio_group_serializes_options_and_optionals, radio_group_basic_serialization) |
| switch.rs                | `Switch`                                                       | 2 (switch_basic_serialization, switch_full_serialization) |
| field_set.rs             | `FieldSet`                                                     | 2 (field_set_basic_serialization, field_set_full_serialization) |
| field_separator.rs       | `FieldSeparator`                                               | 1 (field_separator_serializes_with_no_props) |
| data_table.rs            | `TableColumn` + `ColumnKind` + `Filter` + `DataTable`          | 11 (filter_text_*, filter_select_*, filter_date_range_*, column_kind_*, table_column_*, data_table_fluent_filters_accumulate, data_table_omits_*, data_table_phase13_example_*, data_table_source_*) |
| modal.rs                 | `Modal`                                                        | 0 |
| toast.rs                 | `Toast`                                                        | 0 |
| confirm_dialog.rs        | `ConfirmDialog`                                                | 0 |
| spinner.rs               | `Spinner`                                                      | 0 |
| error_display.rs         | `ErrorDisplay`                                                 | 0 |
| composites.rs            | `form_shell` helper fn (cross-struct: Container + test-only Button/Form/Heading) | 1 (form_shell_assembles_container_with_heading_back_form) |
| **mod.rs (meta)**        | —                                                              | 1 (all_19_standard_types — whole-module smoke test at hub) |
| **TOTAL**                |                                                                | **42** (unchanged from pre-refactor) |

## Cross-Module Imports Introduced

Only two cross-module imports inside the new per-component files — all others are self-contained:

1. **`data_table.rs`** → `use super::select::SelectOption;` — because `Filter::Select` variant carries `Vec<SelectOption>`.
2. **`composites.rs`** → `use super::container::Container;` (library surface) plus `use super::super::{button::Button, form::Form, heading::Heading};` inside the `#[cfg(test)] mod tests` module (test-only). The `form_shell` fn's public signature accepts pre-built `(String, Component)` tuples, so Button/Form/Heading aren't needed in the non-test library path.

## Shim Strategy (Option A) — git-diff-stat Summary

```
 backend/crates/marionette/src/builders/button.rs              | NEW  57
 backend/crates/marionette/src/builders/checkbox.rs            | NEW  64
 backend/crates/marionette/src/builders/composites.rs          | NEW 152
 backend/crates/marionette/src/builders/confirm_dialog.rs      | NEW  13
 backend/crates/marionette/src/builders/container.rs           | NEW  61
 backend/crates/marionette/src/builders/data_table.rs          | NEW 370
 backend/crates/marionette/src/builders/error_display.rs       | NEW  11
 backend/crates/marionette/src/builders/field_separator.rs     | NEW  34
 backend/crates/marionette/src/builders/field_set.rs           | NEW  71
 backend/crates/marionette/src/builders/form.rs                | NEW  15
 backend/crates/marionette/src/builders/grid.rs                | NEW  27
 backend/crates/marionette/src/builders/heading.rs             | NEW  15
 backend/crates/marionette/src/builders/mod.rs                 | M  ~100 (rewrite)
 backend/crates/marionette/src/builders/modal.rs               | NEW  15
 backend/crates/marionette/src/builders/nav_group.rs           | NEW  11
 backend/crates/marionette/src/builders/nav_item.rs            | NEW  13
 backend/crates/marionette/src/builders/radio_group.rs         | NEW 102
 backend/crates/marionette/src/builders/select.rs              | NEW 111
 backend/crates/marionette/src/builders/side_nav.rs            | NEW  11
 backend/crates/marionette/src/builders/spinner.rs             | NEW  12
 backend/crates/marionette/src/builders/standard.rs            | M   -1382 +16 (shrunk to shim)
 backend/crates/marionette/src/builders/surface_mount.rs       | NEW  45
 backend/crates/marionette/src/builders/switch.rs              | NEW  60
 backend/crates/marionette/src/builders/text.rs                | NEW  12
 backend/crates/marionette/src/builders/text_input.rs          | NEW  88
 backend/crates/marionette/src/builders/textarea.rs            | NEW  91
 backend/crates/marionette/src/builders/toast.rs               | NEW  14
 27 files changed  (2 modified + 25 created) — ALL inside builders/
 0  files changed outside  backend/crates/marionette/src/builders/
```

External caller files (`crm-demo/src/**`, `gallery-smoke/src/lib.rs`, `gallery-smoke/tests/ui/*.rs` + their `.stderr` fixtures, `marionette/src/builders/app_shell.rs`) are **byte-for-byte unchanged**. The Option A shim absorbs the refactor's surface-area impact.

## Test-Migration Notes

- Each test was categorized by its PRIMARY subject struct (the type it constructs/asserts on) and moved to that file's `#[cfg(test)] mod tests` block.
- Tests that touch multiple builders (e.g., `container_builder_with_children`, `children_method`) landed with their primary subject (`container.rs` — they build Container and use Heading/Button/TextInput incidentally). These files import siblings via `use super::super::heading::Heading;` etc. inside the test module.
- `form_shell_assembles_container_with_heading_back_form` landed in `composites.rs` (its primary subject — the form_shell helper).
- `all_19_standard_types` (the 19-way meta-test) landed in `builders/mod.rs` `#[cfg(test)] mod tests` block — it doesn't belong under any single component and represents the whole-module smoke guarantee.
- Zero tests were lost or renamed; pre- and post-refactor test counts match exactly (71 lib tests; 42 of those in builders/).

## Decisions Made

- **Shim-preservation (Option A) chosen** as the `standard.rs` disposition. Canonical user-preference rationale: `feedback_pre_deployment_no_backcompat.md` says "no back-compat shims, fix root causes" — but this is NOT a back-compat shim. It is a **clean co-equal public API path**: both `marionette::builders::Button` AND `marionette::builders::standard::Button` resolve to the same item, by design. No `#[deprecated]` markers; no "will be removed in v1.3" language. Paying a 12-file import churn tax across crm-demo + gallery-smoke for zero functional gain would violate scope discipline.
- **Meta-test `all_19_standard_types` hosted at mod.rs**, not in any single component file. It is the module-wide smoke test ("every builder still compiles and emits the right `component.r#type`"), so the hub is its natural home. Per-component files host only their own assertions.
- **Cross-module import count minimized to two** — only `data_table.rs → select::SelectOption` (carried in `Filter::Select`) and `composites.rs → container::Container` (library surface). Button/Form/Heading in composites.rs are test-only (`super::super::*` inside the test module) — kept out of the public library path so composites.rs's top-level `use` block stays minimal.

## Deviations from Plan

None - plan executed exactly as written.

One minor cosmetic note: during the initial write of `composites.rs` I briefly included the full `use super::{button::Button, container::Container, form::Form, heading::Heading};` as an eager top-level import per the plan's `<interfaces>` guidance. Clippy accepted this (no warnings fired), but the library code path only needs `Container` — `Button`/`Form`/`Heading` are referenced only in the test module and in doc-comment examples. I trimmed the library-surface import to just `Container` and moved Button/Form/Heading into the test module's `use super::super::{button::Button, form::Form, heading::Heading};` block. Cleaner separation of test-only vs library imports. Not a deviation from plan intent — the plan's `<interfaces>` showed the broader import as an acceptable shape; I picked the narrower one. Both compile and pass clippy with `-D warnings` under `--all-features`.

## Issues Encountered

None - all 5 Task 2 verification commands green on first run:

| # | Command                                                                  | Result   |
|---|--------------------------------------------------------------------------|----------|
| 1 | `cargo test -p marionette --lib`                                         | 71/71 ok |
| 2 | `cargo test -p marionette --lib --features gallery`                      | 71/71 ok |
| 3 | `cargo test --workspace --exclude crm-demo --features gallery`           | all ok   |
| 4 | `cargo clippy -p marionette --lib --all-features -- -D warnings`         | 0 warns  |
| 5 | `cargo build -p crm-demo`                                                | ok       |

Sanity: `grep -rn 'use marionette::builders::standard::' backend/` returned **12 hits** — all 12 external callers preserved.

## User Setup Required

None - no external service configuration required.

## Hand-off to Plan 17-04

Every in-scope file now has a natural home for Plan 17-04's `#[cfg(feature = "gallery")] #[gallery_demo(key = "...")] pub fn gallery_demo() -> Vec<Node>` sibling:

- **Leaf demos** (Button, TextInput, Select, Checkbox, Grid, Heading, Text, Textarea, RadioGroup, Switch, Spinner, ErrorDisplay) — append `gallery_demo()` to the matching per-component file (e.g., `button.rs` gets `pub fn gallery_demo() -> Vec<Node> { ... }`).
- **Composite demos** (Form, FieldSet, DataTable, Modal, ConfirmDialog, Toast) — same: append `gallery_demo()` to the matching per-component file; compose via nested `gallery_demo()` calls to other builders.
- **AppShell demo** — hand-designed, appended to `backend/crates/marionette/src/builders/app_shell.rs` (untouched by this plan).
- **Skip list** (SurfaceMount, NavItem, NavGroup, FieldSeparator, SideNav, Container, TableColumn per CONTEXT.md §D-B2) — their files have NO `gallery_demo()` sibling; skip-list rationale documented in GALLERY-DEMOS.md (to be authored in Plan 17-03).

Cross-module nesting for composite demos (e.g., `Form::gallery_demo()` nesting `TextInput::gallery_demo()`) works via the public re-exports in `builders/mod.rs` — `use super::text_input::gallery_demo as text_input_demo;` or similar. No additional infrastructure needed.

## Task Commits

1. **Task 1: Split standard.rs into 25 per-component files + rewrite mod.rs + shim standard.rs** — `68d86b6` (refactor)
2. **Task 2: Workspace-wide test + clippy sweep** — no file modifications; all 5 verification commands green on the Task-1 commit. Per plan "This task modifies NO files", no separate commit.

**Plan metadata commit:** pending (this SUMMARY.md commit below).

## Next Phase Readiness

- Ready for Plan 17-04 (19-demo sweep). Per-component files are the landing pads.
- Plan 17-01 runs in parallel in a sibling worktree (modifying `gallery.rs`'s `DemoEntry.render` signature from `fn() -> Node` to `fn() -> Vec<Node>`). Zero file overlap with Plan 17-02 — these plans are parallel-safe.
- Plan 17-03 (gallery-demo crate skeleton + GALLERY-DEMOS.md author-facing contract) consumes this refactor indirectly — its call sites that mount demos into the AppShell iterate `registered_demos()`, not the builders directly. No expected friction from this plan.

## Self-Check: PASSED

- All 25 per-component files FOUND at `backend/crates/marionette/src/builders/*.rs`
- `builders/mod.rs` + `builders/standard.rs` FOUND (rewritten shim)
- Commit `68d86b6` (Task 1) FOUND in git log
- SUMMARY.md FOUND at `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-02-SUMMARY.md`

---
*Phase: 17-gallery-crate-skeleton-colocated-built-in-demos*
*Completed: 2026-04-22*
