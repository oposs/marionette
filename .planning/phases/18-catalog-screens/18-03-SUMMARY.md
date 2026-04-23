---
phase: 18-catalog-screens
plan: 03
subsystem: backend
tags: [gallery, fixtures, data-table, fetch-rows, chrono, lcg, synthetic-data, serde]

# Dependency graph
requires:
  - phase: 17-gallery-crate-skeleton
    provides: "gallery-demo crate (lib + bin), fetch-rows handler registered, /demo/data-table bind convention"
  - phase: 13-datatable
    provides: "fetch-rows action + virtualized infinite scroll in frontend DataTable component"
provides:
  - "gallery_demo::fixtures::synthetic_rows(n) — deterministic LCG-seeded Row generator (shared with Phase 19 EXER-03)"
  - "Row struct + Status enum (JSON: snake_case fields, lowercase status, ISO date joined_at)"
  - "handle_demo_fetch_rows source-dispatch: demo-rows (legacy 5 rows) + catalog-synthetic-rows (paginated 500)"
  - "Per-row actions array [Edit, Delete, Duplicate] for CAT-03 DataTable row-actions column"
  - "PatchOperation::Set emission keyed at /demo/catalog-data-table/rows/{id} (object-map shape matches DataTable.svelte contract)"
affects: [18-06-cat-03-datatable, 19-exer-03-scale]

# Tech tracking
tech-stack:
  added:
    - "chrono 0.4 (workspace-inherited) — NaiveDate + Duration for deterministic date arithmetic"
  patterns:
    - "Hand-rolled LCG (Numerical-Recipes constants) instead of rand crate — keeps gallery-demo crate-weight minimal; deterministic = reproducible tests"
    - "Source-dispatch match on untrusted payload.source string with fall-through to ActionError::BadPayload (no dynamic dispatch, no SQL, no file paths)"
    - "Saturating arithmetic on offset+limit: start.saturating_add(limit).min(collection.len()) — no panic on u32::MAX offset"
    - "Per-row JSON augmentation (actions array injected post-serialize) — server-authored, client cannot forge"

key-files:
  created:
    - "backend/crates/gallery-demo/src/fixtures.rs (97 LOC + 5 tests)"
  modified:
    - "backend/crates/gallery-demo/Cargo.toml (chrono.workspace dep)"
    - "backend/crates/gallery-demo/src/lib.rs (pub mod fixtures)"
    - "backend/crates/gallery-demo/src/handlers/fetch_rows.rs (rewrite: source-dispatch + 6 tests)"

key-decisions:
  - "LCG over rand: zero extra dep, deterministic, reproducible — tests don't need fixed seeds or feature-gated dev-deps"
  - "Row struct extracted to shared fixtures.rs upfront (not inlined in fetch_rows.rs) so Phase 19 EXER-03 can consume synthetic_rows(10_000) without refactor"
  - "demo-rows legacy arm duplicates 5-row fixture locally instead of sharing with show.rs::seed_table_rows — avoids cross-module coupling (D-4-C locks show.rs untouched)"
  - "actions array uses ComponentAction shape {label, action:{type,name}} directly as serde_json::Value — matches UI-SPEC §CAT-03 contract without introducing a new strongly-typed struct (keeps Row pure data)"
  - "id in response path cast to u64 (from row.get('id')?.as_u64()?) — filter_map silently drops rows without numeric id, matching CRM's fetch_rows defensive pattern"

patterns-established:
  - "Deterministic LCG generator: seed 0x1234_5678_9ABC_DEF0, constants 1_664_525 / 1_013_904_223 — constants MUST NOT change (would invalidate all downstream tests)"
  - "Source-dispatch handlers: single match on payload.source to (path_prefix, rows), then uniform per-row Set loop at bottom — separates data selection from patch emission"

requirements-completed: []

# Metrics
duration: 11m
completed: 2026-04-23
---

# Phase 18 Plan 03: Synthetic-Row Generator + fetch-rows Source Dispatch Summary

**Shared LCG-seeded synthetic-row generator (`synthetic_rows(n)`) plus extended `fetch-rows` handler with `catalog-synthetic-rows` source arm — unblocks CAT-03 DataTable (Plan 18-06) with paginated actions-augmented rows at `/demo/catalog-data-table/rows/{id}`.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-04-23T16:01:41Z
- **Completed:** 2026-04-23T16:12:42Z
- **Tasks:** 2
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments

- Shared `gallery_demo::fixtures` module with deterministic LCG-based `synthetic_rows(n)` generator (same `n` → identical `Vec<Row>`), `Row { id, name, email, status, score, joined_at }`, and lowercase-serialized `Status` enum.
- Extended `handle_demo_fetch_rows` into a source-dispatching handler: legacy `"demo-rows"` unchanged; new `"catalog-synthetic-rows"` slices `synthetic_rows(500)` by `offset`/`limit` and emits per-row `Set` ops at `/demo/catalog-data-table/rows/{id}`.
- Every catalog-synthetic row carries an `actions: [Edit, Delete, Duplicate]` array (UI-SPEC §CAT-03).
- 11 new tests (5 fixtures + 6 handler) all green; clippy `-D warnings` clean on all targets; workspace builds with `--all-features`.
- Zero regressions: Phase 17 `nav_auto_discovery` and `smoke_boot` integration tests still pass; `show.rs::seed_table_rows` intentionally untouched (D-4-C).

## Task Commits

1. **Task 1: fixtures.rs + chrono dep + lib.rs plumbing** — `964c7fb` (feat)
2. **Task 2: fetch_rows.rs source-dispatch rewrite + 6 tests** — `0084801` (feat)

## Files Created/Modified

- `backend/crates/gallery-demo/src/fixtures.rs` **(created)** — Shared `Row` struct + `Status` enum + `synthetic_rows(n)` LCG generator + 5 unit tests. Copy target for Phase 19 EXER-03.
- `backend/crates/gallery-demo/Cargo.toml` — Added `chrono.workspace = true` (inherits `serde` feature from workspace; placed alphabetically adjacent to `axum.workspace`).
- `backend/crates/gallery-demo/src/lib.rs` — Added `pub mod fixtures;` declaration (alphabetical position, above `pub mod handlers`).
- `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` — Full rewrite. `FetchRowsPayload { source, offset, limit }` struct; `match payload.source` dispatch to `("demo-rows", legacy_5_rows)` / `("catalog-synthetic-rows", slice(offset..offset+limit) with actions injected)` / `BadPayload`; uniform per-row `Set` emission loop. 6 new async unit tests constructing a minimal `HandlerContext` via `MockDatabase` + anonymous `Session`.

## Diff Summary

| File | Change | Lines |
|------|--------|-------|
| `Cargo.toml` | 1 line added (`chrono.workspace = true`) | +1 |
| `lib.rs` | 1 line added (`pub mod fixtures;`) | +1 |
| `fixtures.rs` | full-file create (types + generator + 5 tests) | +124 |
| `handlers/fetch_rows.rs` | rewrite (source-dispatch + 6 tests) | +223 / −24 |

**Test counts:** 5 in `fixtures::tests` + 6 in `handlers::fetch_rows::tests` = 11 new tests. Pre-existing `handlers::show::tests` (3) still green.

## Sample Row JSON

`synthetic_rows(10)[0]` — cross-reference for Plan 18-06 (CAT-03 DataTable) column mapping:

```json
{
  "id": 1,
  "name": "Paul Davis",
  "email": "paul.davis@example.com",
  "status": "pending",
  "score": 444,
  "joined_at": "2024-12-01"
}
```

Full shape per request (catalog-synthetic-rows path) adds:

```json
"actions": [
  { "label": "Edit",      "action": { "type": "click", "name": "gallery-demo/noop" } },
  { "label": "Delete",    "action": { "type": "click", "name": "gallery-demo/noop" } },
  { "label": "Duplicate", "action": { "type": "click", "name": "gallery-demo/noop" } }
]
```

## Decisions Made

- **Cargo.toml placement** — `chrono.workspace = true` placed immediately after `axum.workspace = true`, not "between axum and marionette" as the plan narrative suggested (the actual Cargo.toml orders `marionette` + `marionette-protocol` + `marionette-macros` + `gallery-smoke` first, then the workspace-inherited block). Alphabetical ordering inside the workspace-inherited block is preserved. Acceptance criterion (`grep -c 'chrono.workspace = true' ... == 1`) satisfied.
- **`pub mod fixtures;` placement** — Placed above `pub mod handlers;` (alphabetical `f < h`), not "between handlers and home" as the plan narrative suggested. Matches the common Rust `pub mod` alphabetical convention. Acceptance criterion (`grep -n 'pub mod fixtures;' ... >= 1 match`) satisfied.
- **`HandlerContext` test construction** — Mirrored the pattern from `backend/crates/marionette/src/router.rs::tests::make_ctx` (canonical): `ActionMessage` literal with `id: Some("t1".into())`, `MockDatabase::new(Sqlite).into_connection()` for `db`, and a role-less anonymous `Session`. No new harness invented.
- **`default_limit` fn kept local to fetch_rows.rs** — Not shared with crm-demo's identical helper because the two handlers live in unrelated crates and the CRM version caps at `MAX_LIMIT = 100` (gallery's doesn't need the cap because the upstream `all.len() = 500` is already the natural cap — T-18-03-02 mitigation).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug / clippy::cast_possible_truncation] Added allow on the `as usize` casts inside the LCG map closure**
- **Found during:** Task 1 (clippy -D warnings run)
- **Issue:** Three `rng() as usize` casts (for name/surname/status lookup) and two `rng() % N as i32 / i64` casts (for score/days) triggered `clippy::cast_possible_truncation` + `clippy::cast_possible_wrap`. The plan already had fine-grained `#[allow(...)]` on the sc/days casts but not on the index-lookup casts.
- **Fix:** Hoisted the allow to a fn-level `#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]` on `synthetic_rows` — covers all five casts uniformly. The truncation is provably safe (all moduli are ≤16 or ≤1000, far below `usize::MAX`).
- **Files modified:** `backend/crates/gallery-demo/src/fixtures.rs`
- **Verification:** `cargo clippy -p gallery-demo --all-targets -- -D warnings` passes.
- **Committed in:** `964c7fb` (Task 1 commit)

**2. [Rule 1 — Bug / clippy::unreadable_literal] Added digit separators to LCG constants**
- **Found during:** Task 1 (clippy -D warnings run)
- **Issue:** `wrapping_mul(1664525).wrapping_add(1013904223)` triggered `clippy::unreadable_literal`.
- **Fix:** Rewrote as `wrapping_mul(1_664_525).wrapping_add(1_013_904_223)` — no numerical change (Rust literal separators are purely syntactic), LCG output bit-identical.
- **Files modified:** `backend/crates/gallery-demo/src/fixtures.rs`
- **Verification:** `cargo test -p gallery-demo --lib fixtures` still green (determinism test would have caught any numerical drift); clippy clean.
- **Committed in:** `964c7fb` (Task 1 commit)

**3. [Rule 1 — Bug / clippy::missing_panics_doc] Added `# Panics` doc + allow on `synthetic_rows` and `handle_demo_fetch_rows`**
- **Found during:** Task 1 + Task 2 (clippy -D warnings run)
- **Issue:** `synthetic_rows` calls `.expect("valid date")` on `NaiveDate::from_ymd_opt`, and `handle_demo_fetch_rows` calls `.expect("Row serializes")` on `serde_json::to_value(r)`. Both trigger `clippy::missing_panics_doc`.
- **Fix:** Added a `# Panics` rustdoc section on `synthetic_rows` (noting the panic is unreachable because the base date is compile-time known valid). On the async handler, used `#[allow(clippy::missing_panics_doc)]` alongside the existing `unused_async` allow — the `Row → serde_json::Value` conversion is infallible for our shape (primitive + String + NaiveDate all have `Serialize` impls that never error).
- **Files modified:** `backend/crates/gallery-demo/src/fixtures.rs`, `backend/crates/gallery-demo/src/handlers/fetch_rows.rs`
- **Verification:** `cargo clippy -p gallery-demo --all-targets -- -D warnings` passes.
- **Committed in:** `964c7fb` (fixtures) and `0084801` (handler)

---

**Total deviations:** 3 auto-fixed (3 clippy lints), all Rule 1 (pre-existing `clippy::pedantic` in the gallery-demo crate root's `lib.rs` enforces a stricter bar than the plan's example code anticipated).
**Impact on plan:** Zero scope creep — all three are cosmetic clippy pacifications that preserve the exact generator behavior and handler semantics. Functional correctness unchanged; every plan acceptance-criterion grep hits the expected count.

## Issues Encountered

None — both tasks progressed RED→GREEN on first compile after the clippy adjustments.

## Threat Flags

None — no new network endpoints, auth paths, file access, or schema changes beyond what the plan's `<threat_model>` already enumerated.

## Verification

All plan verification commands executed and passed:

- `cargo test -p gallery-demo` → **14 passed** (5 fixtures + 6 fetch_rows + 3 show) + 2 integration tests (`nav_auto_discovery`, `smoke_boot`) still green. `0 failed`.
- `cargo clippy -p gallery-demo --all-targets -- -D warnings` → clean.
- `cargo build --workspace --all-features` → `Finished dev profile [unoptimized + debuginfo] target(s) in 53.09s`.

Plan acceptance criteria (all satisfied):

- `grep -c 'chrono.workspace = true' backend/crates/gallery-demo/Cargo.toml` → `1` ✓
- `grep -n 'pub mod fixtures;' backend/crates/gallery-demo/src/lib.rs` → 1 match ✓
- `grep -c 'pub fn synthetic_rows' backend/crates/gallery-demo/src/fixtures.rs` → `1` ✓
- `grep -c '#\[serde(rename_all = "lowercase")\]' backend/crates/gallery-demo/src/fixtures.rs` → `1` ✓
- `grep -c '"catalog-synthetic-rows"' backend/crates/gallery-demo/src/handlers/fetch_rows.rs` → `5` (≥2 required) ✓
- `grep -n 'crate::fixtures::synthetic_rows' backend/crates/gallery-demo/src/handlers/fetch_rows.rs` → 1 match ✓
- `grep -n '/demo/catalog-data-table/rows' backend/crates/gallery-demo/src/handlers/fetch_rows.rs` → 4 matches (≥1 required) ✓
- `grep -c 'Edit'` + `grep -c 'Delete'` → `2` + `2` (≥1 each) ✓

## Known Stubs

None — `synthetic_rows` returns fully-populated rows and `handle_demo_fetch_rows` emits real `PatchOperation::Set` ops. No empty arrays, no placeholder strings, no TODO markers.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **Unblocks Plan 18-06 (CAT-03 DataTable)** — the CAT-03 screen can now `DataTable::bind("/demo/catalog-data-table/rows")` and trigger `fetch-rows` with `{source: "catalog-synthetic-rows"}` to receive 50-row pages with the actions column populated.
- **Downstream-ready for Phase 19 EXER-03** — the shared `fixtures::synthetic_rows(n)` can be called with `n = 10_000` without any change to the module; only the handler's `match` arm's hardcoded `500` needs to be bumped (or a new `"exer-synthetic-rows"` source added alongside) when Phase 19 lands.
- **No regressions** — `"demo-rows"` source still serves the exact 5-row fixture at `/demo/data-table/rows/{id}` that Phase 17's `data-table` leaf demo depends on. Integration tests confirm.

## Self-Check: PASSED

Verified post-SUMMARY:

- `backend/crates/gallery-demo/src/fixtures.rs` — **FOUND** ✓
- Task 1 commit `964c7fb` — **FOUND** (`git log --oneline | grep 964c7fb`) ✓
- Task 2 commit `0084801` — **FOUND** (`git log --oneline | grep 0084801`) ✓

---
*Phase: 18-catalog-screens*
*Completed: 2026-04-23*
