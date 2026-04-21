---
phase: 16-framework-hooks
plan: 01
subsystem: framework
tags: [rust, linkme, feature-flag, distributed-slice, registry, gallery, proc-macro-adjacent]

# Dependency graph
requires:
  - phase: 15-crm-migration-validation
    provides: v1.1 closeout — clean base for v1.2 framework-hooks work
provides:
  - marionette::gallery::DemoEntry public struct (always compiled)
  - marionette::gallery::DEMOS linkme distributed slice (feature-gated)
  - marionette::gallery::registered_demos() iteration API (always compiled, empty under default)
  - marionette::gallery::__linkme hidden re-export for downstream macro emission
  - marionette::gallery::Node re-export of builders::Node
  - gallery cargo feature on marionette crate (`gallery = ["dep:linkme"]`)
  - linkme = "0.3" + trybuild = "1" workspace dependencies
affects:
  - 16-02 (gallery_demo attribute macro — emits tokens referencing marionette::gallery::{DemoEntry, DEMOS, __linkme})
  - 16-03 (gallery-smoke crate — imports registered_demos + trybuild fixtures)
  - 17-gallery-demos (built-in component demos register via macro into DEMOS)
  - 18+ (gallery-demo binary iterates registered_demos for nav)

# Tech tracking
tech-stack:
  added: [linkme 0.3, trybuild 1 (workspace dep only, not yet used)]
  patterns:
    - "cargo feature gate with dep: prefix: `feature_name = [\"dep:optional_crate\"]`"
    - "linkme distributed slice backing a typed registry static"
    - "std::sync::OnceLock memoization for one-time sort + dedup"
    - "Pure-helper extraction for unit-testability of cfg-gated code"
    - "Hidden re-export of downstream library for macro-emitted absolute paths"

key-files:
  created:
    - backend/crates/marionette/src/gallery.rs
  modified:
    - backend/Cargo.toml
    - backend/crates/marionette/Cargo.toml
    - backend/crates/marionette/src/lib.rs

key-decisions:
  - "D-A1 locked: linkme (not inventory) backs the registry — typed static, zero runtime cost, explicit mental model"
  - "D-A2 locked: stable iteration order owned by marionette via sort-at-iteration-time, not by linkme's platform-dependent linker order"
  - "D-A3 locked: duplicate-key collisions debug_assert! panic in debug, tracing::warn! keep-first in release"
  - "D-A4 locked: sort + dedup memoized via std::sync::OnceLock — O(n log n) once per process"
  - "D-B1 locked: DEMOS distributed slice is fully cfg(feature = gallery)-gated"
  - "D-B3 locked: gallery feature uses dep:linkme prefix to suppress implicit feature name"
  - "D-B4 locked: registered_demos() always compiles; returns empty iterator under default build"
  - "D-C2 locked: DemoEntry = { key, render, display_name } — no component_type field, no source-location field"
  - "Module path: marionette::gallery (submodule), not marionette::* (top-level) — matches marionette::builders::* organization"
  - "OnceLock (stdlib, stable since 1.70) chosen over external once_cell for zero-dep sync primitive"

patterns-established:
  - "First optional-dep cargo feature in the marionette workspace — establishes `dep:`-prefix idiom for subsequent feature gates"
  - "First use of std::sync::OnceLock in the codebase — pattern for lazily-computed, thread-safe, read-mostly caches"
  - "First distributed-slice-backed registry — pattern for future auto-discovery surfaces (e.g., workflow step types, migration entries)"
  - "cfg_attr(not(any(feature=X, test)), allow(dead_code)) — pattern for pure helpers reachable only through feature-gated or test paths"

requirements-completed: [FRAME-02]

# Metrics
duration: 18min
completed: 2026-04-21
---

# Phase 16 Plan 01: Gallery Registry Foundation Summary

**linkme-backed DEMOS distributed slice, DemoEntry type, memoized-sort registered_demos() iterator, plus gallery cargo feature gate — the auto-discovery spine for every v1.2 gallery consumer.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-04-21T20:17:30Z
- **Completed:** 2026-04-21T20:35:31Z
- **Tasks:** 2
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments

- `marionette::gallery` module created with `DemoEntry` (always compiled), `DEMOS` distributed slice (feature-gated), and `registered_demos()` iterator API (always compiled, empty under default).
- `gallery` cargo feature landed on the `marionette` crate with the post-1.60 `dep:linkme` prefix idiom — `cargo tree -p marionette` verifies linkme is absent from the default compile graph and present under `--features gallery`.
- Stable iteration order enforced by `sort_entries()` pure helper (alphabetical by `key`) + `std::sync::OnceLock`-memoized sorted cache.
- Duplicate-key collision detection: `debug_assert!` panic in debug builds, `tracing::warn!` + keep-first dedup in release.
- `linkme` re-exported at `marionette::gallery::__linkme` so the `#[gallery_demo]` macro (Plan 02) can emit absolute paths without downstream consumers needing their own `linkme` dep.
- 4 inline unit tests cover sort order, empty input, duplicate-panic-in-debug, and `registered_demos` idempotence.
- Clippy pedantic green on `--all-features`; `cargo doc --features gallery --no-deps` renders the public API cleanly.

## Task Commits

1. **Task 1: Wire workspace + crate Cargo manifests for gallery feature** — `cfe0f9f` (feat)
   - Added `linkme = "0.3"` + `trybuild = "1"` to `backend/Cargo.toml` workspace.dependencies
   - Added `linkme = { workspace = true, optional = true }` + `[features] gallery = ["dep:linkme"]` to `backend/crates/marionette/Cargo.toml`
2. **Task 2: Create marionette::gallery module** — `958c2fc` (feat)
   - New `backend/crates/marionette/src/gallery.rs` (~175 lines incl. tests)
   - `pub mod gallery;` inserted alphabetically into `backend/crates/marionette/src/lib.rs`

## Files Created/Modified

- **`backend/crates/marionette/src/gallery.rs`** (NEW, ~175 LoC) — `DemoEntry`, `DEMOS`, `registered_demos()`, `sort_entries()` helper, `OnceLock`-memoized cache, 4 unit tests.
- **`backend/Cargo.toml`** (MODIFIED) — two new entries under `[workspace.dependencies]`: `linkme = "0.3"` and `trybuild = "1"`.
- **`backend/crates/marionette/Cargo.toml`** (MODIFIED) — optional `linkme` dep + new `[features] gallery = ["dep:linkme"]` section.
- **`backend/crates/marionette/src/lib.rs`** (MODIFIED) — one-line insertion of `pub mod gallery;` alphabetically between `extractors` and `migration`.
- **`backend/Cargo.lock`** (regenerated by cargo) — locks `linkme v0.3.36` + `linkme-impl v0.3.36`.

## Decisions Made

- **`#[must_use]` dropped from `registered_demos()`**: clippy pedantic flags `double_must_use` because `impl Iterator` already carries `#[must_use]` implicitly. Removing the redundant outer attribute keeps the API intent without clippy noise.
- **`#[cfg_attr(not(any(feature = "gallery", test)), allow(dead_code))]` on `sort_entries()`**: under default `cargo build` (no tests, no gallery feature), `sort_entries()` is unreachable (only called from `#[cfg(feature = "gallery")] fn build_sorted`). The plan didn't specify this attribute but the default build would otherwise emit a `dead_code` warning that clippy pedantic would promote to an error. This is a targeted, correctness-preserving allow — not a workspace-wide silencer.
- **No `#![warn(clippy::pedantic)]` at module top**: the `marionette` crate root already carries `#![warn(clippy::pedantic)]` (see `lib.rs:1`), so `gallery.rs` inherits it. Duplicating the attribute at module scope is redundant per patterns in `builders/node.rs`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy `double_must_use` error on `registered_demos()`**
- **Found during:** Task 2 clippy verification (`cargo clippy -p marionette --all-features -- -D warnings`)
- **Issue:** Plan specified `#[must_use] pub fn registered_demos() -> impl Iterator<...>`. Clippy pedantic flags this because `impl Iterator` already has `#[must_use]` via its trait bound, producing `-D clippy::double_must_use`.
- **Fix:** Removed the redundant `#[must_use]` attribute. The `impl Iterator` return type carries the must-use intent implicitly.
- **Files modified:** `backend/crates/marionette/src/gallery.rs` (line 101 area)
- **Verification:** `cargo clippy -p marionette --all-features -- -D warnings` exits 0.
- **Committed in:** `958c2fc` (Task 2 commit — fix was applied before commit)

**2. [Rule 3 - Blocking] `dead_code` warning on `sort_entries()` under default build**
- **Found during:** Task 2 initial `cargo build -p marionette` (default features)
- **Issue:** Under default build (no `gallery` feature, not a test build), `sort_entries()` is unreachable because it's only called from the `#[cfg(feature = "gallery")] build_sorted` variant. This produces a `dead_code` warning; the crate's `#![warn(clippy::pedantic)]` would promote it to a hard error under full lint runs.
- **Fix:** Added `#[cfg_attr(not(any(feature = "gallery", test)), allow(dead_code))]` to `sort_entries()` — precisely scoped to the configurations where the fn is genuinely unreachable.
- **Files modified:** `backend/crates/marionette/src/gallery.rs` (line 54 area)
- **Verification:** `cargo build -p marionette` exits 0 with no warnings; `cargo clippy -p marionette -- -D warnings` exits 0.
- **Committed in:** `958c2fc` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug — clippy lint violation, 1 blocking — dead_code warning under default build)
**Impact on plan:** Both are minimal, localized annotations needed because the plan was written before cross-validating against clippy pedantic + default-build dead-code semantics. No scope creep; `must_haves` contract unchanged; public API bytes-identical to plan specification.

## Issues Encountered

- None beyond the two deviations above.

## Authentication Gates

None — this plan is pure Rust framework work with no external services or credentials involved.

## User Setup Required

None — no external service configuration required. The gallery feature is a build-time cargo flag only.

## Verification Results

All 7 overall verification steps from the plan pass:

1. `cargo build -p marionette` — green (default, no linkme).
2. `cargo build -p marionette --features gallery` — green (linkme compiled).
3. `cargo test -p marionette --lib gallery` — 4/4 tests pass.
4. `cargo clippy -p marionette --all-features -- -D warnings` — green.
5. `cargo doc -p marionette --features gallery --no-deps` — green (pre-existing private-link warning in `session.rs` unrelated to this plan is out of scope per Rule 3 scope boundary).
6. `cargo tree -p marionette -e normal` — no `linkme` in output (correct).
7. `cargo tree -p marionette -e normal --features gallery` — `linkme v0.3.36` present (correct).

## Threat Model Status

All four entries in the plan's threat register remain `accept`/`mitigate` as originally dispositioned:
- **T-16-01-01 (linkme static tampering):** accepted — followed canonical linkme idiom exactly.
- **T-16-01-02 (duplicate-key DoS):** mitigated — `dedup_by(|a, b| a.key == b.key)` in release, `debug_assert!` in debug per D-A3.
- **T-16-01-03 (display_name info disclosure):** accepted — nav label only, no PII path.
- **T-16-01-04 (feature-gate leak):** mitigated — `cargo tree -p marionette -e normal` verification confirms linkme absent from default graph.

No new threat surface introduced beyond the plan's register.

## Next Phase Readiness

- **Plan 16-02 unblocked:** The `#[gallery_demo]` attribute macro can now emit tokens referencing `::marionette::gallery::DemoEntry`, `::marionette::gallery::DEMOS`, and `::marionette::gallery::__linkme::distributed_slice`. All three paths resolve under `--features gallery`.
- **Plan 16-03 unblocked:** The `gallery-smoke` crate can declare `marionette = { path = "../marionette", features = ["gallery"] }` and import `marionette::gallery::{Node, registered_demos}` from this foundation.
- **No concerns or blockers carried forward.**

## Self-Check: PASSED

- `backend/crates/marionette/src/gallery.rs` — FOUND
- `backend/Cargo.toml` edits — FOUND (`linkme = "0.3"` at line 33, `trybuild = "1"` at line 34)
- `backend/crates/marionette/Cargo.toml` edits — FOUND (optional linkme dep at line 22, `gallery` feature at line 29)
- `backend/crates/marionette/src/lib.rs` — FOUND (`pub mod gallery;` at line 9)
- Commit `cfe0f9f` — FOUND in git log
- Commit `958c2fc` — FOUND in git log
- All overall verification steps (build default, build gallery, test, clippy, doc, tree default, tree gallery) — PASSED

---
*Phase: 16-framework-hooks*
*Completed: 2026-04-21*
