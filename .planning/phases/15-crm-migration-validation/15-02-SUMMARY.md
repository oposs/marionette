---
phase: 15-crm-migration-validation
plan: 02
subsystem: builders
tags: [rust, builders, form-shell, validation, marionette, tdd]

# Dependency graph
requires:
  - phase: 14-formscreen-enhancements
    provides: FieldSet/FieldSeparator/RadioGroup primitives + /_errors{bind} render path that Plan 15-02's helpers wrap
provides:
  - "form_shell() free function in backend/crates/marionette/src/builders/standard.rs — composes Container([heading, back_button, form_child]) envelope + flat HashMap<String, Component> output"
  - "validation_error_patch() helper in new backend/crates/marionette/src/validation.rs — generic (B, M): Into<String> iterator → ProtocolMessage::Patch with SetData ops targeting /_errors{bind}"
  - "pub mod validation; registration in backend/crates/marionette/src/lib.rs"
affects: [15-03-company-user-interaction-forms, 15-04-contact-refactor, 15-05-per-field-validation-wiring]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN per task — failing test committed as `test(…)` before any `feat(…)` impl"
    - "Free-function envelope helper pattern (no new builder struct) — `form_shell` composes existing fluent builders via positional tuple args"
    - "Generic Into<String> boundary for validation iterators — callers pass &str/String/owned tuples without explicit conversion"

key-files:
  created:
    - "backend/crates/marionette/src/validation.rs"
    - ".planning/phases/15-crm-migration-validation/deferred-items.md"
  modified:
    - "backend/crates/marionette/src/builders/standard.rs"
    - "backend/crates/marionette/src/lib.rs"

key-decisions:
  - "form_shell signature kept positional (5 args) — matches Phase 13 inline style; fluent FormShellBuilder rejected as over-engineered for 4 future call sites"
  - "validation_error_patch returns ProtocolMessage::Patch (not PatchMessage) — matches the Ok(vec![...]) convention of handler returns; id: None because ws.rs::propagate_id fills it on send"
  - "No new ActionError variant per D-D3 — handlers return Ok(vec![validation_error_patch(...)]) for field validation; BadPayload stays for protocol-layer failures"
  - "validation module registered at crate root without pub use re-export — callers import marionette::validation::validation_error_patch (keeps ActionError's namespace clean)"

patterns-established:
  - "Envelope composition via free function — handlers stay flat-children-first, helper handles the repeated Container([heading, back, form]) shape"
  - "Per-field validation as SetData patches at /_errors{bind} — helper prefixes the bind literal; callers pass server-derived path strings only (threat T-15-03-PLAN02 mitigation doc-enforced)"

requirements-completed: [COMP-03]

# Metrics
duration: 11min
completed: 2026-04-18
---

# Phase 15 Plan 02: Form Shell + Validation Helpers Summary

**Two Rust-side composition helpers — `form_shell()` assembles the canonical CRM form envelope; `validation_error_patch()` shapes per-field validation failures into `/_errors{bind}` SetData patches — unlocking the Wave-2 handler rewrites.**

## Performance

- **Duration:** 11 min (09:00 → 09:11 local)
- **Started:** 2026-04-18T07:00:03Z
- **Completed:** 2026-04-18T07:11:14Z
- **Tasks:** 2 (both TDD)
- **Files modified:** 2
- **Files created:** 1 (validation.rs)

## Accomplishments

- `form_shell(root_id, heading, back_button, form_child, form_descendants)` free function — wraps `Container([heading.1, back_button.1, form_child.1])` via `build_with_children`, flattens everything into `HashMap<String, Component>` ready for `RenderMessage.nodes`. Re-exported by `pub use standard::*;` in `builders/mod.rs`.
- `validation_error_patch<I, B, M>(surface, errors)` generic helper — single iterator pass produces one `PatchOperation::Set { path: "/_errors{bind}", value: String(msg) }` per tuple; returns `ProtocolMessage::Patch`. `id: None` so `ws.rs::propagate_id` owns correlation.
- `validation` module registered via `pub mod validation;` between `session` and `ws` in `lib.rs`.
- 4 new unit tests (1 × form_shell + 3 × validation_error_patch) — all green; full marionette lib suite 67/67 pass; crm-demo downstream compile clean.

## Task Commits

Each task ran the full TDD RED/GREEN cycle:

1. **Task 1 RED:** test for `form_shell_assembles_container_with_heading_back_form` — `02cfe9c` (test)
2. **Task 1 GREEN:** `form_shell()` implementation + deferred-items log — `e4362dd` (feat)
3. **Task 2 RED:** 3 tests for `validation_error_patch` (single / multi / empty) + module registration — `45b92c5` (test)
4. **Task 2 GREEN:** `validation_error_patch()` implementation — `71fda33` (feat)

No REFACTOR commits needed — both implementations fit within the planned doc-comment + body + single pass.

## Files Created/Modified

- **`backend/crates/marionette/src/builders/standard.rs`** — appended `form_shell()` free function (45 lines + doc comment) between `ErrorDisplay` and the `#[cfg(test)] mod tests` block; appended the `form_shell_assembles_container_with_heading_back_form` unit test to the existing tests mod.
- **`backend/crates/marionette/src/validation.rs`** — new file (125 lines). Module-level docstring codifies the threat T-15-03-PLAN02 mitigation invariant (server-derived literal bind paths only); one `pub fn validation_error_patch<I, B, M>`; 3 unit tests.
- **`backend/crates/marionette/src/lib.rs`** — added `pub mod validation;` between `pub mod session;` and `pub mod ws;` (alphabetical position).
- **`.planning/phases/15-crm-migration-validation/deferred-items.md`** — new file logging 6 pre-existing `clippy::doc_markdown` warnings in `standard.rs` lines 43/55/59/94/113/223 (pre-existing toolchain drift, not introduced by Plan 15-02).

## Decisions Made

- **Positional `form_shell` signature.** Plan's Area-I discretion allowed positional vs. builder struct; chose positional to mirror Phase 13's handler composition style and because 5 args remain readable. Fluent `FormShellBuilder` rejected as over-engineered for 4 call sites.
- **`validation_error_patch` returns `ProtocolMessage`, not `PatchMessage`.** Matches `ActionResult = Result<Vec<ProtocolMessage>, ActionError>`, letting handlers write `Ok(vec![validation_error_patch(...)])` without an extra `ProtocolMessage::Patch(...)` wrap.
- **Separate `validation` module — no `pub use` re-export at crate root.** Keeps `ActionError`'s namespace clean; full path `marionette::validation::validation_error_patch` is explicit at call sites and cannot collide with future helpers.
- **Defensive `nodes.insert` re-inserts in `form_shell`.** `Container::build_with_children` already includes the three child tuples, so the subsequent `nodes.insert(heading.0, heading.1)` etc. are idempotent. Kept as documentation against future macro-emission changes.

## Deviations from Plan

Plan executed as specified. Noting one minor scope-boundary call:

**1. [Scope — pre-existing] 6 `clippy::doc_markdown` warnings in `standard.rs`**

- **Found during:** Task 1 acceptance-criteria verification (`cargo clippy -p marionette -- -D warnings`)
- **Issue:** Pre-existing `#![warn(clippy::pedantic)]` warnings at lines 43, 55, 59, 94, 113, 223 — `FieldSet` / `RadioGroup` identifiers need backticks in doc comments.
- **Investigation:** Verified the warnings exist on a pristine checkout of `ffd3c76d31fe858721d921ced9f576b2a3796fd3` (worktree base) — same 6 errors, 0 added by this plan. Matches the "crm-demo has 76-86 pre-existing clippy pedantic warnings (toolchain drift)" entry already tracked in `.planning/STATE.md §Blockers/Concerns`; the `marionette` crate has the same drift on a smaller scale.
- **Decision:** Logged to `.planning/phases/15-crm-migration-validation/deferred-items.md` per the executor's SCOPE BOUNDARY rule (only auto-fix issues directly caused by the current task's changes). The fix is a mechanical backtick sweep that belongs to the deferred `marionette` pedantic cleanup, not to a helper-add plan.
- **Impact on acceptance criterion:** `cargo clippy -p marionette -- -D warnings` exits non-zero due to these 6 pre-existing warnings. Plan 15-02 added ZERO new clippy warnings (verified by grepping `validation.rs:` and `standard.rs:57[0-9]-6[0-9][0-9]` from clippy output — empty). If the Phase 15 verifier re-runs the exact acceptance-criterion command and fails it on these 6, the fix is five lines of trivial backtick additions across the same file; recommend folding into Plan 15-07 (closure / cleanup).

**Auto-fixes applied to new code:**
- During Task 1 GREEN: my own doc comment introduced 11 `doc_markdown` / `doc_overindented_list_items` warnings; fixed inline before committing (tightened identifier-heavy argument list, wrapped `FieldSet`/`back_button`/`form_child` in backticks) — these were caused by my change, so Rule 1 applied.
- During Task 2 GREEN: my doc comment introduced 5 `doc_overindented_list_items` warnings; fixed inline before committing.

**Total deviations:** 0 plan-scope deviations; 1 pre-existing issue deferred with written rationale.
**Impact on plan:** None — primary verification (unit tests green, helpers callable from crm-demo) passes cleanly; deferred item is orthogonal to the plan's objective.

## Issues Encountered

None. One transient confusion untangled: after initially running `git stash` to inspect pre-existing clippy state, two stashes were created (one tracked, one untracked-only). Identified the correct stash via `git stash show -p` and dropped the empty one — working tree restored without loss.

## Known Stubs

None. Both helpers are production-complete — they are intentionally thin (no stubs, no TODOs, no placeholder data). Wave-2 plans (15-03, 15-04, 15-05) will consume them directly; there is no "wire later" step.

## Threat Flags

None. The plan's explicit threat `T-15-03-PLAN02` (Tampering/Injection via `validation_error_patch` bind paths) is mitigated as specified: helper doc-comment forbids user-derived bind paths, and the module-level security-invariant paragraph codifies the callers' contract. No new trust boundaries or untracked surface introduced.

## Self-Check: PASSED

**Files exist:**
- FOUND: backend/crates/marionette/src/builders/standard.rs (contains `pub fn form_shell`)
- FOUND: backend/crates/marionette/src/validation.rs (contains `pub fn validation_error_patch`)
- FOUND: backend/crates/marionette/src/lib.rs (contains `pub mod validation;`)
- FOUND: .planning/phases/15-crm-migration-validation/deferred-items.md

**Commits exist on main:**
- FOUND: 02cfe9c (Task 1 RED — `test(15-02)` for form_shell)
- FOUND: e4362dd (Task 1 GREEN — `feat(15-02)` form_shell implementation)
- FOUND: 45b92c5 (Task 2 RED — `test(15-02)` for validation_error_patch)
- FOUND: 71fda33 (Task 2 GREEN — `feat(15-02)` validation_error_patch implementation)

**Acceptance criteria:**
- `grep -c "pub fn form_shell" standard.rs` → 1
- `grep -c "form_shell_assembles_container_with_heading_back_form" standard.rs` → 1
- `grep -c "pub fn validation_error_patch" validation.rs` → 1
- `grep -c "pub mod validation" lib.rs` → 1
- `grep -c "validation_error_patch_shapes_single_error|…multi_field|…empty_iter_returns_empty_patch" validation.rs` → 3
- `cargo test -p marionette --lib` → 67 passed, 0 failed (includes the 4 new tests)
- `cargo check -p crm-demo` → clean compile (helpers reachable from downstream crate)

## TDD Gate Compliance

Both tasks ran the full TDD cycle with visible RED → GREEN commits in git log:
- Task 1: `test(15-02)` RED commit 02cfe9c immediately followed by `feat(15-02)` GREEN commit e4362dd.
- Task 2: `test(15-02)` RED commit 45b92c5 immediately followed by `feat(15-02)` GREEN commit 71fda33.

REFACTOR was not required — both implementations landed at the planned surface without needing cleanup.

## Next Phase Readiness

- Wave-2 plans (15-03 company/user/interaction form rewrites; 15-04 contact.rs refactor) can `use marionette::builders::form_shell;` and `use marionette::validation::validation_error_patch;` immediately.
- No protocol changes, no schema changes, no ActionError variants added — downstream compile remains stable.
- Deferred pre-existing clippy warnings in `standard.rs` do not block Wave-2 plans (they compile; only `-D warnings` CI-mode surfaces them). Fold into the Plan 15-07 closure pass or a dedicated pedantic cleanup.

---
*Phase: 15-crm-migration-validation*
*Plan: 02*
*Completed: 2026-04-18*
