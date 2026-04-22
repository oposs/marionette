# Phase 17 — Deferred Items

Tracked here: issues discovered DURING Phase 17 execution that are NOT within
the current plan's scope. Out-of-scope per the executor rule "Only auto-fix
issues DIRECTLY caused by the current task's changes."

## From Plan 17-05 execution (2026-04-22)

### Pre-existing crm-demo clippy::pedantic failures (97 errors)

- **Symptom:** `cargo clippy --workspace --features gallery -- -D warnings` fails
  with ~97 pedantic errors in `backend/crates/crm-demo/src/` (e.g.,
  `clippy::too_many_lines` on `main()`, `clippy::doc_markdown` on several
  function docs).
- **Source:** Pre-existing toolchain drift, documented in `.planning/STATE.md`
  §Blockers/Concerns ("some clippy pedantic warnings in crm-demo from toolchain
  drift"). Not introduced by Plan 17-05's changes.
- **Scope boundary:** Plan 17-05 touches only
  `backend/crates/gallery-demo/src/handlers/{navigate,modal,show}.rs`,
  `backend/crates/marionette/src/builders/data_table.rs`, and
  `frontend/src/lib/components/popup/ModalSurface.svelte`. These all pass
  clippy with `-D warnings` in isolation (`cargo clippy -p <crate>`).
- **Resolution path:** Separate cleanup plan or a dedicated toolchain-drift
  phase. The plan's workspace-wide verification command (
  `cargo clippy --workspace --features gallery -- -D warnings`) was run but
  cannot pass until those unrelated pedantic issues are addressed; per-plan
  clippy on the touched crates does pass.
