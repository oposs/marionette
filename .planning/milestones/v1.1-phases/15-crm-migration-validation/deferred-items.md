# Phase 15 — Deferred Items

Items discovered during Phase 15 execution that are out of scope for the current plan(s) and are deferred to a dedicated follow-up.

## Pre-existing clippy pedantic doc_markdown warnings in `marionette` crate

**Discovered during:** Plans 15-01 and 15-02 (independently surfaced in each).

**File:** `backend/crates/marionette/src/builders/standard.rs`
**Severity:** Pedantic-lint only; runtime behaviour unaffected.
**Scope:** Pre-existing toolchain drift (mirrors the `crm-demo` pedantic backlog already logged in STATE.md §Blockers/Concerns).

6 `clippy::doc_markdown` warnings on pre-existing code:

| Line | Identifier missing backticks |
|------|-------------------------------|
| 43   | `FieldSet` (TextInput.full_width doc) |
| 55   | `RadioGroup` (RadioOption doc) |
| 59   | `RadioGroup` (RadioOption doc, second hit) |
| 94   | `FieldSet` (SelectInput.full_width doc) |
| 113  | `FieldSet` (Checkbox.full_width doc) |
| 223  | `FieldSet` (Textarea.full_width doc) |

**Reproduction:** `cd backend && cargo clippy -p marionette -- -D warnings` exits non-zero with the 6 warnings listed above. Without `-D warnings` the build is clean (warnings only).

**Origin:** Pre-existing; last touched by Phase 14 commits (`a599c84`, `5d58921`, `2c5856a`). Not introduced by Plan 15-01 or 15-02 (verified by checking out `ffd3c76d31fe858721d921ced9f576b2a3796fd3` on a scratch tree — same 6 errors).

**Why deferred:** Fixing them falls outside Plan 15-01's and 15-02's `<files_modified>` scope; the scope-boundary rule in `execute-plan.md` forbids sweeping unrelated pre-existing warnings into an unrelated plan. The fix is mechanical (wrap 6 identifiers in backticks).

**Action for future plan (candidate: Plan 15-06 doc sweep):** One commit, six backtick pairs, gated on `cargo clippy -p marionette -- -D warnings` exit 0.
