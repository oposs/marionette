# Phase 18 Deferred Items

Items discovered during Phase 18 execution that are out of scope per the GSD executor Scope Boundary rule (only auto-fix issues DIRECTLY caused by the current task's changes).

## 18-01 — Pre-existing clippy failures in `marionette` crate

**Discovered:** 2026-04-23, Plan 18-01 Task 1 verification step.

**State:** `cargo clippy -p marionette --features gallery --all-targets -- -D warnings` exits non-zero with 31 `error:` lines at the feature-branch base commit `f64783b` (verified via a temp worktree checkout). All 31 errors exist BEFORE any Plan 18-01 change; Plan 18-01's Task 1 introduces zero new clippy findings (`31 → 31` error count identical).

**Breakdown of pre-existing errors:**
- 3 × `function is never used` warnings in `crates/marionette/tests/macro_tests.rs` (async placeholder fixtures `save_contact`, `edit_profile`, `delete_user`) — `-D warnings` escalates the `dead_code` lint to error.
- 7 × `item in documentation is missing backticks` in builder doc comments — clippy pedantic `doc_markdown`.
- 21 × `binding to `_` prefixed variable with no side-effect` in `crates/marionette/src/gallery.rs` FRAME-03 symbol-availability test (the `let _x = …;` assignments that exist specifically to force symbol-referencing at compile time) — clippy pedantic `no_effect_underscore_binding`. The `_` prefix is semantically load-bearing here (these are intentional no-ops asserting symbol existence), so fixing requires a per-statement `#[allow(clippy::no_effect_underscore_binding)]` or a refactor to a distinct marker helper.

**Why deferred:** All errors are in files Plan 18-01 does not touch (gallery.rs, macro_tests.rs, builder doc comments unrelated to Button). Phase 17 shipped with these warnings tolerated; Plan 18-01's acceptance criteria `cargo clippy -p marionette --features gallery --all-targets -- -D warnings` exits 0 cannot be satisfied without rewriting Phase 17 infrastructure, which is explicitly out of Plan 18-01 scope.

**Plan 18-01 verification substitute:** Plan 18-01's Button changes were verified clean via:
- `cargo test -p marionette --features gallery --lib builders::button` — all 6 tests pass (3 new + 3 existing).
- Delta clippy check: `31 → 31` error count confirms zero new findings.

**Candidate resolution:** v1.3+ cleanup plan (pairs naturally with the existing "97 clippy pedantic warnings in crm-demo from toolchain drift" item noted in STATE.md Blockers/Concerns).
