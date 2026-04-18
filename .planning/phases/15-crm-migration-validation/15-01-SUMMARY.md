---
phase: 15-crm-migration-validation
plan: 01
subsystem: database
tags: [sea-orm, sqlite, migration, contact-entity, crm, persistence]

# Dependency graph
requires:
  - phase: 14-formscreen-enhancements
    provides: "ContactFormData struct accepting country/notes/opt_in fields (marked dead-code pending Phase 15 schema)"
provides:
  - "m20260418_000011_extend_contact migration adding 3 columns to contact table"
  - "Extended contact::Model with contact_country / contact_notes / contact_opt_in fields"
  - "Seed data populating new columns for named contacts (Alice/Bob/Carol)"
  - "handle_contact_save persists all three new fields on insert AND update paths"
  - "Integration test contact_round_trips_country_notes_opt_in proving end-to-end round-trip"
affects: [15-02-form-shell, 15-03-validation-helpers, 15-04..05-handler-sweeps, milestone-v1.1-close]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SeaORM extend-table migration via independent execute_unprepared ALTER statements (per 15-RESEARCH.md Pitfall #5)"
    - "Integration test pattern using sqlite::memory: + Migrator::up for entity round-trip verification"

key-files:
  created:
    - "backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs"
    - ".planning/phases/15-crm-migration-validation/deferred-items.md"
  modified:
    - "backend/crates/crm-demo/src/migration/mod.rs"
    - "backend/crates/crm-demo/src/entities/contact.rs"
    - "backend/crates/crm-demo/src/seed.rs"
    - "backend/crates/crm-demo/src/handlers/contact.rs"
    - "backend/crates/crm-demo/src/handlers/listmonk.rs"

key-decisions:
  - "Integration test placed inside #[cfg(test)] mod tests at bottom of handlers/contact.rs (plan-sanctioned fallback) — crm-demo is a pure binary crate with no lib target, so tests/contact_persistence.rs cannot import internal modules"
  - "Update branch uses `active.contact_X = Set(data.X.clone())` assignment form (mirrors existing pattern on the `found.into()` ActiveModel), not struct-literal form — matches the plan's action text even though the acceptance_criteria grep was pattern-specific to the struct-literal form"
  - "Generated contacts get None/None/false for the three new columns to keep the 117-row generated seed small; UAT-facing data variety lives entirely on the three named contacts"

patterns-established:
  - "SeaORM column-add migration: one execute_unprepared per ALTER TABLE ADD COLUMN; down drops in reverse order"
  - "Boolean column in SQLite: INTEGER NOT NULL DEFAULT 0 mapped to `bool` (not Option<bool>) in the entity Model"

requirements-completed: [COMP-03]

# Metrics
duration: ~30min
completed: 2026-04-18
---

# Phase 15 Plan 01: Contact schema extension + save-path wiring Summary

**Contact entity now persists country/notes/opt_in end-to-end via a SeaORM extend-table migration + handle_contact_save wiring, closing Phase 14 Known Stub #1.**

## Performance

- **Duration:** ~30 min (both tasks)
- **Started:** 2026-04-18T06:56:41Z (phase execution start per STATE.md)
- **Completed:** 2026-04-18T07:10:00Z
- **Tasks:** 2 / 2
- **Files created:** 2 (migration + deferred-items log)
- **Files modified:** 5

## Accomplishments

- New SeaORM migration `m20260418_000011_extend_contact` adds three columns (`contact_country TEXT NULL`, `contact_notes TEXT NULL`, `contact_opt_in INTEGER NOT NULL DEFAULT 0`) with an honest reverse-order down migration.
- `contact::Model` extended with `contact_country: Option<String>`, `contact_notes: Option<String>`, `contact_opt_in: bool` between `contact_company` and timestamps.
- Seed data: Alice → CH + opt-in + Q2 note; Bob → US + opt-out; Carol → opt-in + long-form Q3 note. Generated contacts default to None/None/false.
- `ContactFormData`: removed `#[allow(dead_code)]` attrs from country/notes/opt_in; retained `#[serde(default)]` for payload tolerance.
- `handle_contact_save`: both insert (struct-literal `Set(data.X.clone())`) and update (`active.contact_X = Set(...)` assignment) paths now persist all three fields.
- `contact_round_trips_country_notes_opt_in` tokio test: inserts a contact with CH/note/true, queries back by ID, asserts all three round-trip correctly.
- Country-select node-patch behaviour from Phase 12 D-A6 left completely untouched per the plan constraint.

## Task Commits

Each task was committed atomically with --no-verify (parallel worktree mode):

1. **Task 1: Migration + entity + seed (+ handler stub for compile-gate)** — `f2fe3e2` (feat)
2. **Task 2: Handler wiring + integration test (+ listmonk.rs test helper fix)** — `42a173d` (feat)
3. **Deferred items log** — `9ff4b8d` (docs)

Final SUMMARY commit: appended after this file is written.

## Files Created/Modified

- `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs` — new migration (up: 3× ADD COLUMN; down: 3× DROP COLUMN in reverse)
- `backend/crates/crm-demo/src/migration/mod.rs` — registers new migration module + Box::new in migrations() vec
- `backend/crates/crm-demo/src/entities/contact.rs` — Model struct extended with 3 new fields
- `backend/crates/crm-demo/src/seed.rs` — named_contacts tuple widened; Alice/Bob/Carol seeded with realistic spread; generated contacts default
- `backend/crates/crm-demo/src/handlers/contact.rs` — ContactFormData dead-code attrs removed; insert + update paths wire new fields; tokio round-trip test added to existing `#[cfg(test)] mod tests`
- `backend/crates/crm-demo/src/handlers/listmonk.rs` — sample_contact test helper extended with None/None/false defaults (Rule 3 auto-fix)
- `.planning/phases/15-crm-migration-validation/deferred-items.md` — logs pre-existing marionette clippy warnings

## Decisions Made

- **Test location fallback.** Plan primary target was `tests/contact_persistence.rs`, but crm-demo has no `lib.rs` (pure binary crate), so an integration test cannot import `crm_demo::entities::contact`. The plan's fallback path — placing the test inside the existing `#[cfg(test)] mod tests { ... }` in `handlers/contact.rs` — was taken. This keeps the test adjacent to the save handler it covers.
- **Update-branch wiring pattern.** Plan action text explicitly prescribes `active.contact_country = Set(data.country.clone())` assignment form (mirroring the pre-existing `active.contact_name = Set(...)` pattern on the fetched ActiveModel). Used that form. The acceptance_criteria grep `"contact_country: Set(data.country"` (colon-separated struct-literal) was slightly over-fit to the insert-branch form; both paths nevertheless correctly wire all three fields (`grep -cE "(contact_country|contact_notes|contact_opt_in)\s*[:=]\s*Set\("` returns 9 — 3 insert + 3 update + 3 in the test).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] handle_contact_save ActiveModel literal missing new fields after Task 1 Model extension**
- **Found during:** Task 1 (`cargo check -p crm-demo` compile gate)
- **Issue:** Adding `contact_country / contact_notes / contact_opt_in` to `Model` broke the one struct-literal `contact::ActiveModel { … }` in `handle_contact_save` (E0063). The update branch uses `found.into()` + assignments so was unaffected.
- **Fix:** Added the three fields as `NotSet` placeholders in Task 1's commit so the crate compiles; Task 2 replaced them with real `Set(data.X.clone())` / `Set(data.opt_in.unwrap_or(false))` wiring per plan spec.
- **Files modified:** `backend/crates/crm-demo/src/handlers/contact.rs` (insert branch ActiveModel)
- **Verification:** `cargo check -p crm-demo` exits 0 after Task 1; `cargo test -p crm-demo` all 33 tests pass after Task 2.
- **Committed in:** `f2fe3e2` (Task 1), finalized in `42a173d` (Task 2)

**2. [Rule 3 - Blocking] listmonk.rs sample_contact test helper missing new fields**
- **Found during:** Task 2 (`cargo test -p crm-demo contact_round_trips_country_notes_opt_in` compile gate)
- **Issue:** `handlers/listmonk.rs:474 fn sample_contact` constructs a `contact::Model { … }` literal inside `#[cfg(test)]`. Task 1's Model extension broke it (E0063). This only surfaced at `cargo test` time because release builds don't compile `#[cfg(test)]` code.
- **Fix:** Added `contact_country: None, contact_notes: None, contact_opt_in: false` to the literal. Preserves test semantics (listmonk tests don't care about these fields).
- **Files modified:** `backend/crates/crm-demo/src/handlers/listmonk.rs`
- **Verification:** All 28 unit tests (including 5 listmonk async tests) pass; 5 integration tests pass.
- **Committed in:** `42a173d` (Task 2)

---

**Total deviations:** 2 auto-fixed (both Rule 3 blocking — cascading compile-gate failures from Task 1's Model extension; all scope-bound to files directly affected by the schema change).
**Impact on plan:** No scope creep. Both fixes are the minimum needed to keep the crate compiling after the planned schema extension. Plan acceptance criteria all pass.

## Issues Encountered

- **Acceptance-grep pattern mismatch.** Task 2's acceptance_criteria grep `contact_country: Set(data.country` expected `≥2` (insert + update) but the plan's action text specifies the update branch use `active.contact_X = Set(...)` assignment form — which the grep pattern doesn't match. Both save paths are correctly wired (verified via broader `grep -cE "(contact_country|contact_notes|contact_opt_in)\s*[:=]\s*Set\("` returning 9 matches: 3 insert + 3 update + 3 test). Plan's action text (authoritative) is satisfied; acceptance grep is documented as over-fit.

## User Setup Required

None — no external service configuration required.

## Threat Surface Scan

No new trust boundaries introduced beyond those documented in the plan's `<threat_model>`. Both threats (T-15-03-PLAN01 Tampering/Injection and T-15-03-PLAN01-b Information Disclosure) remain in their planned dispositions:

- **T-15-03-PLAN01 (mitigate):** All three new fields persist via SeaORM `Set()` / parameterised binding (no raw SQL from user input). Verified in both insert struct-literal and update assignment paths.
- **T-15-03-PLAN01-b (accept):** `contact_notes` free-text column visible only to users with existing contact-edit auth (admin-only). No cross-tenant leakage vector; pre-deployment posture per project memory.

Validation-path invariant preserved: no `/_errors/{bind}` bind paths are user-constructed in this plan (validation rewiring happens in Plan 03/04; this plan leaves `handle_contact_save`'s existing `ActionError::BadPayload` return path untouched).

## Known Stubs

None. All three new fields are wired end-to-end (form payload → ActiveModel → DB → Model → form repopulate). The only temporary stub — `NotSet` placeholders added to the insert ActiveModel at the end of Task 1 — was fully replaced in Task 2 with real `Set(data.X.clone())` wiring.

## Self-Check

### Files created (expect FOUND)

- FOUND: `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs`
- FOUND: `.planning/phases/15-crm-migration-validation/deferred-items.md`

### Commits (expect FOUND)

- FOUND: `f2fe3e2` — Task 1 (migration + entity + seed + compile-gate stub)
- FOUND: `42a173d` — Task 2 (handler wiring + integration test + listmonk fix)
- FOUND: `9ff4b8d` — deferred-items log

### Test gate

- `cargo test -p crm-demo contact_round_trips_country_notes_opt_in` → **1 passed**
- `cargo test -p crm-demo` (full suite) → **28 unit + 5 integration = 33 passed, 0 failed**
- `cargo check -p crm-demo` → **exits 0**

## Self-Check: PASSED

## Next Phase Readiness

- Schema extension unblocks any Phase 15 plan that wants to read country/notes/opt_in from the DB (Plan 03/04 validation-helper plans don't depend on it; Plan 05+ handler sweeps consume it transparently).
- UAT evidence (Chrome-MCP per D-H1) for this plan belongs to the contact-edit screen folder — deferred to Phase 15's UAT plan (Plan 07 per phase structure).
- Phase 15 Plan 06 (Flowbite/doc cleanup) should pick up the pre-existing marionette `doc_markdown` clippy warnings logged in `deferred-items.md`.

---
*Phase: 15-crm-migration-validation*
*Completed: 2026-04-18*
