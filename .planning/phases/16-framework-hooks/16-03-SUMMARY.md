---
phase: 16
plan: "03"
status: complete
requirements: [FRAME-01, FRAME-03]
completed: 2026-04-21
---

# Phase 16 Plan 03 — Gallery-smoke crate + FRAME-03 symbol-table test

## Objective

Land the automated counterpart to Phase 17's future `gallery-demo` binary: a tiny `gallery-smoke` workspace-member crate that links `marionette` with `features = ["gallery"]`, registers one toy `#[gallery_demo(key = "smoke", name = "Smoke Check")] pub fn smoke() -> Node`, and asserts end-to-end that the `#[gallery_demo]` macro → linkme distributed slice → `registered_demos()` iterator pipeline round-trips cross-crate. Also pin the `#[gallery_demo]` compile-error messages via `trybuild` (FRAME-01 stability) and verify the FRAME-03 "zero demo symbols under default build" promise with a real `nm`-based symbol-table test.

## Requirements Addressed

- **FRAME-01** (macro misuse produces clear compile errors) — Plan 02 implemented the macro's signature/visibility checks; Plan 03 pins the exact error-message wording via four `trybuild` fixtures so future `rustc`/`syn`/`darling` upgrades cannot silently regress diagnostic quality without a `.stderr` snapshot mismatch.
- **FRAME-03** (zero demo symbols in default build; symbols appear under `--features gallery`) — Verified end-to-end by `backend/crates/marionette/tests/no_gallery_symbols.rs` which shells `cargo build -p marionette` twice (default + `--features gallery`) into isolated `--target-dir`s and inspects each rlib with `nm --demangle`.
- **FRAME-04** (end-to-end smoke test) — `gallery-smoke`'s `tests/registry_roundtrip.rs` registers a toy demo and asserts it appears in `registered_demos()` with correct `display_name` and stable iteration order.

## What Shipped

### New workspace-member crate: `backend/crates/gallery-smoke/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Minimal manifest: depends on `marionette` with `features = ["gallery"]`, dev-depends on `trybuild`. Declares own `gallery` feature that propagates to `marionette/gallery` (necessary for the `#[cfg(feature = "gallery")]` the macro emits to resolve positively in the consumer's namespace — verified empirically in RESEARCH §1). |
| `src/lib.rs` | Registers `#[gallery_demo(key = "smoke", name = "Smoke Check")] pub fn smoke() -> Node`. |
| `tests/registry_roundtrip.rs` | 4 tests: `force_link_smoke_demo`, `smoke_demo_is_registered`, `registry_is_alphabetically_ordered`, `registry_iteration_is_idempotent`. |
| `tests/ui_errors.rs` | `trybuild::TestCases::compile_fail("tests/ui/fail_*.rs")`. |
| `tests/ui/fail_not_pub.{rs,stderr}` | Private fn → expected compiler error naming the `pub` rule. |
| `tests/ui/fail_wrong_signature.{rs,stderr}` | Fn with arguments → expected compiler error naming the zero-args rule. |
| `tests/ui/fail_wrong_return.{rs,stderr}` | Fn returning `Vec<Node>` → expected compiler error naming the `Node` return rule. |
| `tests/ui/fail_applied_to_struct.{rs,stderr}` | Macro on a `struct` → expected compiler error naming the function rule. |
| `tests/ui/README.md` | One-paragraph doc explaining that `.stderr` files are rustc-version-sensitive + the `TRYBUILD=overwrite` recovery procedure. |

### New test in `marionette` crate

| File | Purpose |
|------|---------|
| `backend/crates/marionette/tests/no_gallery_symbols.rs` | FRAME-03 verification. Two `#[test]`s: `default_build_has_zero_demo_symbols` and `gallery_feature_build_has_demo_symbols`. Each shells `cargo build -p marionette` into its own `--target-dir` (`no-gallery-symbols-test-default/` and `no-gallery-symbols-test-gallery/` under `backend/target/`), then `nm --demangle`s the resulting `libmarionette.rlib` and greps for `gallery::DEMOS` / `__GALLERY_DEMO_` markers. |

### Workspace manifest

| File | Change |
|------|--------|
| `backend/Cargo.toml` | Added `"crates/gallery-smoke"` to `[workspace] members`. `linkme` + `trybuild` dependencies were already in `[workspace.dependencies]` from Plan 01. |

### Tweaks to Plan 02's macro

| File | Change |
|------|--------|
| `backend/crates/marionette-macros/src/gallery_demo.rs` | Minor fix (8 LoC delta) to the emitted linkme path — the macro now consistently emits `::marionette::gallery::__linkme::distributed_slice(::marionette::gallery::DEMOS)` per RESOLVED Q2 in RESEARCH.md. |

## Commits

1. `5cd559e` — `feat(16-03): scaffold gallery-smoke crate + fix macro linkme re-export`
2. `dc39c58` — `test(16-03): add registry round-trip integration tests (FRAME-04)`
3. `729b09b` — `test(16-03): trybuild compile-fail fixtures for #[gallery_demo] misuse (FRAME-01)`
4. *(merge)* `9afef92` — `chore: merge executor worktree (16-03 gallery-smoke partial — Tasks 1-3 of 4)`
5. `90695e0` — `test(16-03): FRAME-03 symbol-table test with isolated target-dir`

## Verification

Every acceptance criterion from the plan is green:

| Check | Command | Result |
|-------|---------|--------|
| gallery-smoke builds | `cargo build -p gallery-smoke` | ✓ |
| Registry roundtrip | `cargo test -p gallery-smoke --test registry_roundtrip` | ✓ 4/4 |
| Trybuild UI-errors | `cargo test -p gallery-smoke --test ui_errors` | ✓ 4/4 fixtures |
| FRAME-03 symbol test (both subtests) | `cargo test -p marionette --test no_gallery_symbols` | ✓ 2/2 |
| Default marionette build | `cargo build -p marionette` | ✓ green (no regressions) |
| Feature-on build | `cargo build -p marionette --features gallery` | ✓ green |
| Clippy on Phase 16 code | `cargo clippy -p marionette-macros -p gallery-smoke --all-features --tests -- -D warnings` + `cargo clippy -p marionette --lib --all-features --test no_gallery_symbols -- -D warnings` | ✓ zero warnings |

### Key-Files Created

- `backend/crates/gallery-smoke/Cargo.toml`
- `backend/crates/gallery-smoke/src/lib.rs`
- `backend/crates/gallery-smoke/tests/registry_roundtrip.rs`
- `backend/crates/gallery-smoke/tests/ui_errors.rs`
- `backend/crates/gallery-smoke/tests/ui/fail_*.{rs,stderr}` (4 pairs)
- `backend/crates/gallery-smoke/tests/ui/README.md`
- `backend/crates/marionette/tests/no_gallery_symbols.rs`

## Deviations From Plan

Two deviations, both forced by reality and documented here:

1. **Deferred symbol grep was too broad on first run.** The initial `no_gallery_symbols.rs` greped for any `gallery_demo` substring, which matched the always-compiled `marionette::gallery::registered_demos` fn (D-B4: stub returning empty iterator under default so consumers don't need cfg-guards). That gave a false FRAME-03 violation. Fixed by narrowing the grep to the true markers of registration: `gallery::DEMOS` (the `#[linkme] static` — only exists under feature) and `__GALLERY_DEMO_` (per-entry statics the proc macro emits). Documented inline in the test file's `demo_symbol_matches` helper.

2. **Executor agent was terminated mid-run.** The Wave 2 background agent completed Tasks 1-3 (gallery-smoke scaffold + round-trip tests + trybuild fixtures) and pushed them to its worktree branch, but the task runtime terminated it before Task 4 (`no_gallery_symbols.rs`) and SUMMARY.md could be committed. The orchestrator merged the 3 completed worktree commits, then implemented Task 4 inline (commit `90695e0`) and wrote this SUMMARY.md. All acceptance criteria ultimately passed. No plan scope was lost — just execution mode shifted for the final task.

## Deferred Items

See `.planning/phases/16-framework-hooks/deferred-items.md` (created by the executor agent) for any items it flagged. Review during Phase 17 planning.

## Hand-Off Notes for Phase 17 (DEMO-01 sweep)

- The macro emits its linkme annotation via `::marionette::gallery::__linkme::distributed_slice(::marionette::gallery::DEMOS)`. Phase 17 annotations like `#[gallery_demo(key = "button")] pub fn gallery_demo() -> Node { … }` inside `marionette/src/builders/standard.rs` will resolve these paths inside the `marionette` crate itself — which means the cfg-gate logic must hold even for intra-crate annotations (it does; the macro wraps both the fn and the static under the same `#[cfg(feature = "gallery")]`).
- `gallery-smoke` is **permanent** per D-D3 — do not retire it in Phase 17. It is the automated regression guard for the framework hooks (linkme cross-crate wiring, macro error messages, FRAME-03 symbol absence). Phase 17's `gallery-demo` binary covers the visual/human-verification side; `gallery-smoke` covers the `#[test]`-driven side.
- REQUIREMENTS.md §CRATE-01 describes `gallery-demo` as "the 5th Cargo workspace entry". With `gallery-smoke` landing in Phase 16 as the 5th member, `gallery-demo` becomes the 6th. Phase 17 planner should update CRATE-01's wording or simply accept the ordinal shift.
