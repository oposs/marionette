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

### Pre-existing frontend ESLint errors (67 errors, 1 warning)

- **Symptom:** `cd frontend && npm run lint` reports 68 problems (67
  errors, 1 warning) across `frontend/src/**/*` and
  `frontend/tests/e2e/**`.
- **Source:** Pre-existing. Confirmed baseline (stash-revert + lint) on
  2026-04-22 during Plan 17-05 Task 6d: identical 68 problems on a tree
  with no Plan 17-05 changes applied. None of the lint errors touch
  files this plan modified (`+layout.svelte`, `defaults.ts`,
  `ModalSurface.svelte`, `navigate.rs`, `modal.rs`, `show.rs`,
  `data_table.rs`).
- **Scope boundary:** Plan 17-05's own files are lint-clean. The
  workspace-wide lint run is dominated by pre-existing issues
  (unused eslint-disable directives, `no-useless-assignment`, etc.).
- **Resolution path:** Dedicated lint-cleanup plan (v1.3+) or paired
  with the clippy toolchain-drift cleanup above.

### Pre-existing ConfirmDialog browser-test failures (4 tests)

- **Symptom:** `npx vitest run --config vitest-browser.config.ts
  src/lib/components/popup/ConfirmDialog.browser-test.ts` reports 4/4
  failures (`Cannot read properties of null` on `dialog-title`,
  `dialog-footer` queries).
- **Source:** Pre-existing — fails identically on the baseline (stashed
  the ModalSurface.svelte change and re-ran on 2026-04-22; same 4/4
  failures). Documented in `.planning/STATE.md` §Blockers/Concerns as
  "5 popup browser-test failures" carried from v1.1.
- **Scope boundary:** The ConfirmDialog tests do NOT exercise
  ModalSurface.svelte — they wrap `ConfirmDialog` in a local
  `ConfirmDialogTestWrapper.svelte` with `<Dialog.Root open={true}>`
  directly. Plan 17-05 Task 4 touches only ModalSurface.svelte; the 4
  failures predate the change.
- **Resolution path:** Popup browser-test stabilization phase (v1.3+ or
  a dedicated test-infra plan). ModalSurface browser tests still pass
  (3/3) after Task 4's change.
