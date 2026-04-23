---
phase: 18-catalog-screens
plan: 04
subsystem: ui
tags: [gallery, catalog, button, cat-01, shadcn, rust, linkme]

# Dependency graph
requires:
  - phase: 18-catalog-screens/18-01
    provides: "Button struct loading/icon/aria_label optional fields + Button.svelte reads variant/size/loading/icon/aria_label from SDUI props"
  - phase: 18-catalog-screens/18-01
    provides: "Tailwind app.css @source inline safelist with sm:grid-cols-4 and lg:grid-cols-4"
provides:
  - "CAT-01 Buttons & Actions catalog screen — 5 variants × 3 sizes × 4 states = 60 Button matrix"
  - "catalog/ module scaffold (mod.rs) for sibling CAT-02..CAT-05 plans to extend"
  - "Stable id convention `cb-<variant>-<size>-<state>` for catalog Button cells"
  - "`gallery` cargo feature flag on gallery-demo crate (forwards marionette/gallery)"
  - "`catalog-buttons` key registered via linkme DEMOS distributed slice; auto-discovered by AppShell nav"
affects: [18-05-forms, 18-06-data-table, 18-07-feedback, 18-08-typography]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Catalog fn pattern: pure fn() -> Vec<Node> using marionette::builders::{Button, Container, Heading, Text} + ComponentAction — fresh composition, does NOT invoke leaf gallery_demo()"
    - "Node-flatten pattern for composite trees: build_tree() on inner grid and Card returns (root, descendants); outer build_with_children() splices all descendants into a single flat Vec<Node>"
    - "Container-as-Card idiom: locked class string `rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4` — avoid Container::card(true) (wrong layout)"

key-files:
  created:
    - "backend/crates/gallery-demo/src/catalog/mod.rs"
    - "backend/crates/gallery-demo/src/catalog/buttons.rs"
  modified:
    - "backend/crates/gallery-demo/src/lib.rs"
    - "backend/crates/gallery-demo/src/handlers/show.rs"
    - "backend/crates/gallery-demo/Cargo.toml"

key-decisions:
  - "Added `gallery` feature flag to gallery-demo/Cargo.toml (default=on) — the #[gallery_demo] proc macro emits #[cfg(feature = \"gallery\")] on both the fn and the linkme static, so the consumer crate must define the feature for the cfg to resolve. Matches gallery-smoke's existing pattern. Without this, the macro compiles to a no-op and catalog-buttons never registers."
  - "Tests assert DemoEntry.display_name (the actual struct field) instead of plan's entry.name — plan spec had a typo. Same substantive check; no semantic change."
  - "Applied #[allow(clippy::match_same_arms)] to the seed_for_key match (not a single arm attribute — not supported) to keep the explicit `catalog-buttons` arm as documentation. Comment above the match explains the rationale so future catalog plans don't accidentally flatten the arms into the wildcard."

patterns-established:
  - "Catalog screen anatomy: outer Container(flex flex-col gap-6 p-6) → title H1 → intro Text → N per-category Cards. Each Card: Container(rounded-lg border …) → H3 legend → inner responsive grid Container."
  - "ID convention for matrix cells: `cb-<variant>-<size>-<state>` (variant/size are the iterated axes, state is one of {normal,disabled,loading,icon}). Sibling catalog plans should mirror `<family-key>-<axis1>-<axis2>-<state>` for consistency."

requirements-completed: [CAT-01]

# Metrics
duration: ~35min
completed: 2026-04-23
---

# Phase 18 Plan 04: CAT-01 Buttons & Actions Summary

**Full 60-Button variant × size × state matrix catalog screen (5 per-variant Cards × 12 cells each) wired through the linkme registry with stable `cb-<variant>-<size>-<state>` ids and `gallery-demo/noop` click actions.**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-04-23 (Wave 2 start)
- **Completed:** 2026-04-23T17:03:44Z
- **Tasks:** 2 (Task 1 scaffold; Task 2 TDD matrix implementation)
- **Files modified:** 5 (2 created, 3 modified)

## Accomplishments

- **60-Button matrix** in a single catalog screen: 5 variants (default/destructive/outline/ghost/link) × 3 sizes (sm/default/lg) × 4 states (normal/disabled/loading/icon-only) = 60 Button instances, each with a stable id, variant/size props, and the expected state wiring (disabled, loading, icon+aria_label).
- **Responsive layout matching UI-SPEC §CAT-01 verbatim**: outer `flex flex-col gap-6 p-6` Container with H1 title + intro Text + 5 Cards (`rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4`), each Card containing an H3 legend and a 4-column inner grid (`grid grid-cols-1 sm:grid-cols-4 lg:grid-cols-4 gap-3`).
- **Auto-discovery wired**: `catalog-buttons` key registers via linkme's `DEMOS` distributed slice; `registered_demos()` yields the entry with `display_name = "Catalog: Buttons"`; the existing nav_auto_discovery integration test passes without modification (it iterates the registry, not a hardcoded list).
- **Catalog scaffold ready for siblings 18-05..18-08**: `catalog/mod.rs` declared in lib.rs; sibling plans only need to add `pub mod <family>;` and create the matching file.

## Task Commits

1. **Task 1: Scaffold catalog module + CAT-01 zero-state seed** — `57c8a17` (feat)
2. **Task 2 RED: Failing tests for CAT-01 matrix** — `7d48f01` (test)
3. **Task 2 GREEN: 60-Button matrix implementation** — `293845e` (feat)

_Task 2 is TDD (`tdd="true"`); the two commits (test → feat) follow the RED → GREEN cycle. No REFACTOR commit — no duplication or cleanup needed after GREEN._

## Files Created/Modified

- `backend/crates/gallery-demo/src/catalog/mod.rs` **[created]** — Module declaration (`pub mod buttons;`) for the catalog sub-namespace. No glob re-exports (siblings will also have `gallery_demo` fns; globs would collide).
- `backend/crates/gallery-demo/src/catalog/buttons.rs` **[created]** — CAT-01 implementation (~150 lines): `#[gallery_demo(key="catalog-buttons", name="Catalog: Buttons")]` fn + `build_variant_card` helper + 6 unit tests.
- `backend/crates/gallery-demo/src/lib.rs` **[modified]** — Added `pub mod catalog;` alongside the existing `fixtures/handlers/home/state` modules (alphabetical insertion).
- `backend/crates/gallery-demo/src/handlers/show.rs` **[modified]** — Added `"catalog-buttons" => serde_json::json!({})` arm (pure-visual screen, no bind paths) + `#[allow(clippy::match_same_arms)]` on the outer match with explanatory comment.
- `backend/crates/gallery-demo/Cargo.toml` **[modified]** — Added `[features]` block with `default = ["gallery"]` and `gallery = ["marionette/gallery"]`. Necessary because the `#[gallery_demo]` macro gates both the fn and the linkme static on `#[cfg(feature = "gallery")]`.

## Decisions Made

- **Added `gallery` feature flag to gallery-demo/Cargo.toml** (Rule 3 — blocking issue). Without it, the macro-emitted cfg gate compiles to dead code and the catalog fn never registers. Gallery-demo's entire reason for existence is the gallery registry, so `default = ["gallery"]` is the right default. Matches gallery-smoke's existing pattern exactly.
- **Used `build_tree()` for inner grid and Card, not `build_with_children()`**. The plan's template showed `build_with_children()` everywhere, but the flatten semantics are cleaner when we need to carry descendants separately from the root tuple — `build_tree()` returns `(root, descendants)` natively, avoiding manual `.skip(1)` chains. No semantic difference; same final Vec<Node> shape.
- **Kept explicit `catalog-buttons` arm in `seed_for_key`** (not folded into the wildcard). Documents a known zero-state catalog key; sibling plans 18-05..18-08 will add real seed arms right next to it. The `#[allow(clippy::match_same_arms)]` + comment makes the intent explicit.
- **Tests assert `display_name`, not `name`**, because the plan spec's `entry.name` is a typo — the `DemoEntry` struct field is `display_name` (see `marionette/src/gallery.rs:36`). Same substantive check, just the actual field.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `gallery` feature to gallery-demo/Cargo.toml**
- **Found during:** Task 2 RED (first `cargo test` invocation)
- **Issue:** The `#[gallery_demo]` proc macro emits `#[cfg(feature = "gallery")]` on both the wrapped fn and the `linkme::distributed_slice` static. gallery-demo had no `[features]` section, so `feature = "gallery"` was never set, the cfg stripped both items, and tests reported `cannot find function gallery_demo in this scope`.
- **Fix:** Added `[features] default = ["gallery"], gallery = ["marionette/gallery"]` to gallery-demo/Cargo.toml. Matches gallery-smoke's existing pattern (the other consumer of the macro outside the marionette crate itself).
- **Files modified:** `backend/crates/gallery-demo/Cargo.toml`
- **Verification:** `cargo test -p gallery-demo --lib catalog::buttons` compiles and runs after the feature is added; 5/6 tests fail as RED expected (stub implementation).
- **Committed in:** `7d48f01` (Task 2 RED commit)

**2. [Rule 1 - Clippy lint] `#[allow(clippy::match_same_arms)]` on `seed_for_key` match**
- **Found during:** Task 2 GREEN (final `cargo clippy --all-targets -- -D warnings`)
- **Issue:** `"catalog-buttons" => serde_json::json!({})` and the wildcard `_ => serde_json::json!({})` share the same body. Clippy flags this as `match_same_arms`. Folding the arm into the wildcard would defeat the documentation intent (the explicit arm tells future catalog authors that `catalog-buttons` is a known zero-state key, and sibling plans 18-05..18-08 will add real seed arms right next to it).
- **Fix:** Added `#[allow(clippy::match_same_arms)]` on the `match key {` expression with a comment explaining the rationale (explicit known-zero-state documentation; wildcard-collapse would cause seed drift).
- **Files modified:** `backend/crates/gallery-demo/src/handlers/show.rs`
- **Verification:** `cargo clippy -p gallery-demo --all-targets -- -D warnings` exits 0 after the fix.
- **Committed in:** `293845e` (Task 2 GREEN commit)

---

**Total deviations:** 2 auto-fixed (1 Rule 3 blocking, 1 Rule 1 lint-gate).
**Impact on plan:** Both deviations were gate-fixes required to satisfy the plan's own verification criteria (`cargo test` green, `cargo clippy -- -D warnings` clean). No scope creep. The gallery-feature addition is a one-line Cargo.toml change that will also benefit sibling plans 18-05..18-08 (they use the same macro).

## Issues Encountered

- **Transient cargo filesystem race** during `cargo build --workspace --all-features`: `error: failed to write file … dep-graph.part.bin: Resource temporarily unavailable (os error 11)`. Retried once; second invocation completed cleanly (21.96s). This is a known cargo incremental-compilation issue under concurrent access, unrelated to the plan changes. No further action taken.

## Authentication Gates

None — this is a pure UI composition plan with no external services, no DB writes, no auth flow.

## Known Stubs

None. The catalog fn is a complete 60-cell matrix; every Button has all required props wired (variant/size/label/id + state-specific disabled/loading/icon+aria_label); every non-disabled/non-loading cell wires a real action (`gallery-demo/noop`, an existing handler).

## Threat Flags

None — no new network surface, no new auth paths, no new file access, no schema changes. All content is server-authored from closed `VARIANTS`/`SIZES` array literals; no user input flows into labels, ids, classes, or action names. Matches the plan's `<threat_model>` dispositions (all T-18-04-* threats accepted with existing mitigations).

## Self-Check: PASSED

Verified commits exist:
- `57c8a17` — FOUND (Task 1 scaffold)
- `7d48f01` — FOUND (Task 2 RED)
- `293845e` — FOUND (Task 2 GREEN)

Verified created files exist:
- `backend/crates/gallery-demo/src/catalog/mod.rs` — FOUND
- `backend/crates/gallery-demo/src/catalog/buttons.rs` — FOUND

Verified modified files carry expected changes:
- `backend/crates/gallery-demo/src/lib.rs` contains `pub mod catalog;` — FOUND
- `backend/crates/gallery-demo/src/handlers/show.rs` contains `"catalog-buttons" =>` — FOUND
- `backend/crates/gallery-demo/Cargo.toml` contains `gallery = ["marionette/gallery"]` — FOUND

Verified plan acceptance criteria:
- `grep -c 'key = "catalog-buttons"'` in buttons.rs = **1** (expected 1)
- `grep -c 'name = "Catalog: Buttons"'` in buttons.rs = **1** (expected 1)
- `grep -c '#\[must_use\]'` in buttons.rs = **1** (expected ≥1)
- `grep -c 'gallery_demo'` in marionette/src/builders/button.rs = **3** (unchanged; leaf demo untouched)
- `cargo test -p gallery-demo` = **22 passed, 0 failed** (6 new catalog::buttons + 14 existing + 2 integration)
- `cargo clippy -p gallery-demo --all-targets -- -D warnings` = **exit 0**
- `cargo build --workspace --all-features` = **exit 0**

## Next Plan Readiness

- Ready for Wave 3 (18-05 forms, 18-06 data-table). Sibling catalog plans add new `pub mod <family>;` lines to `catalog/mod.rs`, drop a new `catalog/<family>.rs` file, and add the matching `"catalog-<family>" =>` arm to `seed_for_key`.
- Stable id convention `cb-<variant>-<size>-<state>` documented here so sibling authors can use `<family>-<axis1>-<axis2>` consistently.
- Plan 18-08 Chrome MCP UAT verifies visual behaviour of disabled (opacity/cursor), loading (Loader2 spinner), and icon-only (plus icon + tooltip via aria-label) cells across all 5 variants × 3 sizes.

## Plan Output Contract Compliance

Per `<output>` in 18-04-PLAN.md:
- **File tree diff (3 modified + 1 new):** delivered — 2 new (`catalog/mod.rs`, `catalog/buttons.rs`) + 3 modified (`lib.rs`, `handlers/show.rs`, `Cargo.toml`). Plan counted `mod.rs` as "new" and `buttons.rs` as the main implementation; Cargo.toml bump is deviation-driven.
- **Total cell count assertion result:** **60** (asserted by the `exactly_sixty_button_instances` unit test; all 60 Buttons have `type: "button"` and are counted post-flatten).
- **List of id prefixes for sibling authors:**
  - Root: `catalog-buttons-root`
  - Title/intro: `catalog-buttons-title`, `catalog-buttons-intro`
  - Card roots: `catalog-buttons-card-<variant>` (one per variant)
  - Per-variant legend: `catalog-buttons-<variant>-legend`
  - Per-variant inner grid: `catalog-buttons-<variant>-grid`
  - Button cells: `cb-<variant>-<size>-<state>` where state ∈ {normal, disabled, loading, icon}
- **Plan 18-08 Chrome MCP UAT note:** explicitly carried in the Next Plan Readiness section above.

---
*Phase: 18-catalog-screens*
*Plan: 04*
*Completed: 2026-04-23*
