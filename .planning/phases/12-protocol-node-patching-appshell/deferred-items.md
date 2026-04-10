# Phase 12 Deferred Items

Items discovered during execution that are out of scope for the current plan.

## From 12-01-scaffolding

### Pre-existing TypeScript errors in tests/helpers/schema-validator.ts

`npm run check` reports 3 errors about `fs`, `path`, `url` module resolution:
- `tests/helpers/schema-validator.ts:4` — Cannot find module 'fs'
- `tests/helpers/schema-validator.ts:5` — Cannot find module 'path'
- `tests/helpers/schema-validator.ts:6` — Cannot find module 'url'

These predate Phase 12 (present before the shadcn Sidebar install). Likely missing `@types/node` or the test helper tsconfig needs `"types": ["node"]`. Not caused by Plan 12-01 changes.

## From 12-02-protocol-crate

### Pre-existing clippy pedantic failures in crm-demo

**Discovered during:** Task 2 verification — `cargo clippy --workspace -- -D warnings`

**Scope:** Entirely in `backend/crates/crm-demo/` (not touched by Plan 12-02)

**Count:** 76 errors across ~20 distinct lint categories, including:
- `clippy::struct_field_names` (audit_log, company, contact, interaction, listmonk_cache, listmonk_sync, note, user — all models)
- `clippy::too_many_lines` (8 functions: 106, 109, 123, 159, 199, 200, 321, 388 lines)
- `clippy::map_unwrap_or` / `map_unwrap_or_else`
- `clippy::cast_possible_truncation` (i64 → i32)
- `clippy::implicit_clone`
- `clippy::doc_markdown` (WsSession, SeaORM, etc. not in backticks)
- `clippy::needless_borrows_for_generic_args`
- `clippy::manual_let_else`
- `clippy::collapsible_if`
- `clippy::manual_pattern_char_comparison`
- `clippy::useless_format`
- `clippy::very_complex_type`

**Root cause:** Toolchain drift — the pinned clippy version (1.93.0) introduced new lints that weren't in effect when this code was originally written. None of these are caused by Plan 12-02's changes.

**Verification:** `git stash && cargo clippy -p crm-demo -- -D warnings` (pre-Plan-12-02 state) reproduces all 76 errors.

**In-scope crates are clean:** `cargo clippy -p marionette-protocol -p marionette -- -D warnings` exits 0 after Plan 12-02.

**Recommended resolution:** Dedicated lint-cleanup plan in Phase 12 or early Phase 13 — mechanical fixes only, no behavior changes. Alternative: add targeted `#[allow(...)]` at crate root with a TODO for each category.
