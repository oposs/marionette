---
phase: 15-crm-migration-validation
plan: 06
subsystem: docs
tags: [docs, ci-guard, flowbite-residue, protocol-spec, brand-voice, playwright, clippy]

# Dependency graph
requires:
  - phase: 14-crm-migration-validation
    provides: "FormScreen.svelte + FormScreen.browser-test.ts retired (D-A1) — preserved absence is what the new CI guard enforces"
  - phase: 13-datatable-enhancements
    provides: "TableScreen retirement + the original ci-guards.spec.ts TableScreen deletion guard that this plan extends"
  - plan: 15-03
    provides: "Handlers migrated to /_errors/{bind} validation shape — allowed Task 1 to delete the legacy /contactForm/errors documentation"
  - plan: 15-04
    provides: "Form composition recipes that wire /_errors/{bind} — documented by the new worked example"
provides:
  - "Flowbite-free runtime paths (frontend/src, backend/crates, spec, CONCEPT.md, TOOLING.md)"
  - "shadcn-svelte as the sole documented component library vocabulary across user-facing and governance docs"
  - "spec/PROTOCOL.md single-source-of-truth validation documentation (canonical /_errors/{bind} + worked multi-field example)"
  - "CI guard asserting zero Flowbite residue via git grep (prevents regression via copy-pasted tutorials)"
  - "CI guards for FormScreen.svelte and FormScreen.browser-test.ts retirement (extends existing TableScreen guards)"
  - "Cleaned marionette clippy doc_markdown backlog (6 warnings resolved)"
affects: [future-phases, docs-governance, protocol-evolution, ci-pipeline]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "CI filesystem/git-grep guards via execSync against REPO_ROOT (one level above FRONTEND_ROOT)"
    - "git grep exit code 1 (no matches) treated as success case in try/catch wrapper"
    - "Doc brand-voice sweep landing before CI grep guard ordering (reverse would turn CI red)"

key-files:
  created: []
  modified:
    - "CONCEPT.md (lines 260, 268, 630 — Flowbite → shadcn-svelte)"
    - "TOOLING.md (line 39 — Flowbite Svelte → shadcn-svelte + bits-ui)"
    - ".planning/codebase/STACK.md (line 47 — flowbite-svelte 1.31 → shadcn-svelte 1.2.7 + bits-ui 2.17.3 + @lucide/svelte 1.8.0)"
    - "spec/PROTOCOL.md (deleted lines 803-819 legacy Validation Errors as Data section; appended worked multi-field validation example under canonical Validation semantics)"
    - "frontend/tests/e2e/ci-guards.spec.ts (renamed describe to Phase 13/14/15, added REPO_ROOT, added FormScreen x2 + Flowbite residue guards)"
    - "backend/crates/marionette/src/builders/standard.rs (6 doc_markdown backtick fixes — FieldSet and RadioGroup)"

key-decisions:
  - "Clean-break Flowbite removal — no historical footnote in CONCEPT.md because the CI grep guard would flag it (consistent with pre-deployment no-backcompat posture)"
  - "CI guard order: doc sweep lands in Task 1 BEFORE the grep assertion in Task 2 — reverse order would turn CI red on first run"
  - "ASCII-table cell at CONCEPT.md line 268 rebuilt to preserve column width (shadcn-svelte is 15 chars vs Flowbite's 10; fit into the 17-char cell with 1 space padding)"
  - "New execSync import in ci-guards.spec.ts follows the existing @ts-expect-error suppression pattern — Plan 15-05 will remove all four suppressions together once node:* types land via schema-validator.ts cleanup"
  - "git grep pathspec scope matches the plan's runtime-path CI guard scope: frontend/src, backend/crates, spec, CONCEPT.md, TOOLING.md — STACK.md is in .planning (not in grep scope) but is still swept per D-F3"
  - "Folded pre-existing marionette clippy doc_markdown warnings into this doc-sweep plan (6 backtick fixes in standard.rs) — mechanical and semantically adjacent, logged as resolved in deferred-items.md"

patterns-established:
  - "REPO_ROOT = resolve(FRONTEND_ROOT, '..') for Playwright-based git grep guards"
  - "execSync try/catch with status === 1 success branch for 'no matches' git grep semantics"
  - "Worked-example subsection pattern under canonical spec sections (uses #### nested under ###) for concrete protocol documentation"

requirements-completed: [COMP-03]

# Metrics
duration: 5min
completed: 2026-04-18
---

# Phase 15 Plan 06: Docs-sweep + CI guard + Protocol validation surgery Summary

**Locked the Flowbite clean break with a git-grep CI guard, aligned CONCEPT.md / TOOLING.md / STACK.md to shadcn-svelte, and rewrote spec/PROTOCOL.md's validation section to document only the canonical `/_errors/{bind}` shape with a worked multi-field example.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-18T08:02:44Z
- **Completed:** 2026-04-18T08:07:51Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Three runtime-guarded docs (CONCEPT.md, TOOLING.md, STACK.md) have zero Flowbite residue; shadcn-svelte is now the only documented component-library vocabulary.
- spec/PROTOCOL.md documents one canonical validation shape — `/_errors/{bind}` with per-field/form-level semantics — plus a worked multi-field example showing a `PatchMessage` with two `SetData` ops against `/_errors/contactForm/{name,email}`. The legacy `/contactForm/errors` shape is gone.
- ci-guards.spec.ts now has 5 passing guards (was 2): TableScreen x2 (preserved), FormScreen x2 (new), Flowbite residue grep (new).
- Pre-existing marionette clippy `doc_markdown` backlog (6 warnings on `FieldSet` / `RadioGroup` identifiers in `standard.rs`) cleared — `cargo clippy -p marionette -- -D warnings` now exits 0.

## Task Commits

Each task was committed atomically (both with `--no-verify` per parallel-executor protocol):

1. **Task 1: Doc brand-voice sweep + PROTOCOL.md validation surgery** — `d6eea93` (docs)
2. **Task 2: CI guards extension (Flowbite residue + FormScreen deletion)** — `fc229ed` (test)

## Files Created/Modified

- `CONCEPT.md` — Three targeted replacements (cross-platform paragraph, ASCII-table cell, Phase 2 heading) + ASCII-table column-width preservation.
- `TOOLING.md` — Frontend stack bullet rewritten (Flowbite Svelte → shadcn-svelte + bits-ui).
- `.planning/codebase/STACK.md` — Frontend stack line updated to shadcn-svelte 1.2.7 + bits-ui 2.17.3 + @lucide/svelte 1.8.0 (version numbers from 15-UI-SPEC locked strings).
- `spec/PROTOCOL.md` — Legacy `### Validation Errors as Data` section deleted wholesale (heading + prose + YAML snippet + trailing paragraph). Appended `#### Worked example: multi-field validation on form submit` subsection under canonical `### Validation semantics`, containing prose + JSON code block (two Set ops against `/_errors/contactForm/{name,email}`) + paragraph clarifying `Ok(vec![patch])` vs `Err(ActionError::BadPayload)` + paragraph about error clearing on success render.
- `frontend/tests/e2e/ci-guards.spec.ts` — `test.describe` renamed to "Phase 13/14/15 CI guards"; added `REPO_ROOT` constant; added `execSync` import (with `@ts-expect-error` matching existing suppression pattern); added three new tests (FormScreen.svelte, FormScreen.browser-test.ts, Flowbite residue).
- `backend/crates/marionette/src/builders/standard.rs` — 6 doc comment fixes: `FieldSet` → `` `FieldSet` `` and `RadioGroup` → `` `RadioGroup` `` at lines 43, 55, 59, 94, 113, 223.

## Decisions Made

- **Clean-break Flowbite removal** — No historical footnote added in CONCEPT.md because the CI grep guard from Task 2 would flag it. Consistent with pre-deployment no-backcompat posture (user memory `feedback_pre_deployment_no_backcompat`). UI-SPEC §Doc Brand-Voice OQ3 explicitly allowed either choice; chose omission for clean-break CI alignment.
- **Task ordering** — Task 1 (doc sweep) must land before Task 2 (grep guard) — reverse would turn CI red on first run.
- **ASCII-table width preservation** — CONCEPT.md line 268 cell rebuilt: `│  (Flowbite)     │` (17-char content) → `│ (shadcn-svelte) │` (17-char content). Parallel alignment with "(Native)" and "(Remote-friendly)" cells maintained.
- **Fold clippy backlog into doc-sweep scope** — The 6 pre-existing `clippy::doc_markdown` warnings in `standard.rs` are mechanical backtick fixes and semantically adjacent to the doc-sweep theme. Included per the `<cross_plan_coordination>` directive. Marking them as resolved in `deferred-items.md` is tracked below in "Deferred Items Resolved".

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed missing frontend node_modules**

- **Found during:** Task 2 verification (`npx playwright test`)
- **Issue:** Worktree had no `frontend/node_modules` — Playwright couldn't resolve `@playwright/test` and the test could not run.
- **Fix:** Ran `npm install` in `frontend/`.
- **Files modified:** None committed (node_modules is gitignored).
- **Verification:** All 5 ci-guards passed afterward.
- **Committed in:** N/A (no file changes).

### Expanded Scope

**2. [Cross-plan coordination] Fold-in of marionette clippy doc_markdown warnings**

- **Driven by:** `<cross_plan_coordination>` directive in the execution prompt ("Also address the pre-existing 6 clippy doc_markdown warnings… these are small backtick fixes that fit in the doc-sweep scope").
- **Issue:** 6 pre-existing `clippy::doc_markdown` warnings on `standard.rs` (logged in `.planning/phases/15-crm-migration-validation/deferred-items.md`).
- **Fix:** Added backticks around `FieldSet` (4 locations) and `RadioGroup` (2 locations) in doc comments.
- **Files modified:** `backend/crates/marionette/src/builders/standard.rs`.
- **Verification:** `cd backend && cargo clippy -p marionette -- -D warnings` now exits 0.
- **Committed in:** `d6eea93` (part of Task 1 commit).

---

**Total deviations:** 1 auto-fixed (blocking) + 1 expanded scope (cross-plan coordinated).
**Impact on plan:** Blocking fix was environmental (worktree setup). Expanded scope was directed by the execution prompt and kept the deferred-items backlog drained. No scope creep beyond what was explicitly requested.

## Issues Encountered

- Only pre-existing `schema-validator.ts` svelte-check errors (3× "Cannot find module 'fs' / 'path' / 'url'" — tracked in Phase 13 deferred-items). Plan 15-05 is scheduled to clean these up.

## Deferred Items Resolved

- **6 clippy::doc_markdown warnings in `backend/crates/marionette/src/builders/standard.rs`** — resolved in Task 1 commit `d6eea93`. The `deferred-items.md` entry should be marked resolved (not in this plan's scope to edit the tracking file; State-update agent will handle if desired).

## User Setup Required

None — no external services touched.

## Next Phase Readiness

- D-F1 (CI grep guard) complete and green.
- D-F2 (user-facing doc sweep) complete.
- D-F3 (governance doc sweep) complete.
- D-D2 (protocol spec validation surgery) complete.
- Cross-plan dependency: ci-guards.spec.ts edits are additive (new test blocks only) and do not touch the three existing `@ts-expect-error` suppressions; Plan 15-05's removal of those suppressions (once node:* types land) will merge cleanly.
- Remaining Phase 15 work: Plan 15-07 (phase closure / verification).

## Self-Check: PASSED

Verification of claimed files and commits:

- `CONCEPT.md` FOUND (0 flowbite matches, 3 shadcn-svelte matches)
- `TOOLING.md` FOUND (0 flowbite matches, 1 shadcn-svelte match)
- `.planning/codebase/STACK.md` FOUND (0 flowbite matches, 1 shadcn-svelte match)
- `spec/PROTOCOL.md` FOUND (0 contactForm/errors matches, 1 "Worked example: multi-field validation" match)
- `frontend/tests/e2e/ci-guards.spec.ts` FOUND (1 "No Flowbite residue in runtime code" match, 1 "FormScreen.svelte is retired" match, 1 "FormScreen.browser-test.ts is retired" match, REPO_ROOT present, execSync import present)
- `backend/crates/marionette/src/builders/standard.rs` FOUND (`cargo clippy -p marionette -- -D warnings` exits 0)
- Commit `d6eea93` FOUND in git log (Task 1)
- Commit `fc229ed` FOUND in git log (Task 2)
- Playwright ci-guards test run: 5/5 passed

---

*Phase: 15-crm-migration-validation*
*Completed: 2026-04-18*
