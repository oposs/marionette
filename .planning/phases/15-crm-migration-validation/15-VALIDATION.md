---
phase: 15
slug: crm-migration-validation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-18
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 15-RESEARCH.md §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Frontend framework** | Vitest 4.x (browser mode via @vitest/browser-playwright) |
| **Frontend config file** | `frontend/vitest-browser.config.ts` + `frontend/vitest.config.ts` |
| **Frontend quick run command** | `cd frontend && npm run check && npx vitest --run` |
| **Frontend full suite command** | `cd frontend && npm run check && npx vitest --run && npx playwright test` |
| **Frontend UAT command** | `cd frontend && npx playwright test tests/uat/ --config tests/uat/playwright.uat.config.ts` |
| **Backend framework** | cargo test (Rust built-in) |
| **Backend config file** | `backend/Cargo.toml` (workspace) |
| **Backend quick run command** | `cd backend && cargo test -p marionette -p marionette-protocol` |
| **Backend full suite command** | `cd backend && cargo test --workspace && cargo clippy -p marionette-protocol -p marionette -- -D warnings` |
| **Estimated runtime** | Frontend quick ~15s; frontend full ~90s; backend quick ~20s; backend full ~60s |

---

## Sampling Rate

- **After every task commit:** Run the relevant quick command (frontend or backend depending on which crate/package was touched; run both if the task spans both)
- **After every plan wave:** Run the full suite on both frontend + backend
- **Before `/gsd-verify-work`:** Full suite + UAT must be green
- **Max feedback latency:** ~90 seconds (frontend full suite)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 1 | COMP-03 | — | New SeaORM migration adds contact.country/notes/opt_in columns; up+down both succeed on fresh SQLite | unit (backend) | `cd backend && cargo test -p crm-demo migration::` | ❌ W0 (new test) | ⬜ pending |
| 15-01-02 | 01 | 1 | COMP-03 | — | Contact entity Model exposes new fields; seed data populates them | unit (backend) | `cd backend && cargo test -p crm-demo entities::contact` | ❌ W0 (new test) | ⬜ pending |
| 15-01-03 | 01 | 1 | COMP-03 | — | handle_contact_save persists country/notes/opt_in to DB | integration (backend) | `cd backend && cargo test -p crm-demo --test contact_persistence` | ❌ W0 (new test) | ⬜ pending |
| 15-02-01 | 02 | 2 | COMP-03 | — | form_shell() helper composes Container([Heading, back_button, Form]) + children map | unit (backend) | `cd backend && cargo test -p marionette builders::form_shell` | ❌ W0 (new test) | ⬜ pending |
| 15-02-02 | 02 | 2 | COMP-03 | — | validation_error_patch() returns PatchMessage with SetData /_errors{bind} ops | unit (backend) | `cd backend && cargo test -p marionette validation::patch` | ❌ W0 (new test) | ⬜ pending |
| 15-03-01 | 03 | 3 | COMP-03 | — | handlers/company.rs edit form renders via FieldSet; save persists; per-field /_errors emission | integration (backend) + E2E (frontend) | `cd backend && cargo test -p crm-demo company_handler && cd frontend && npx playwright test tests/e2e/company-edit.spec.ts` | ❌ W0 (new spec) | ⬜ pending |
| 15-03-02 | 03 | 3 | COMP-03 | — | handlers/user.rs edit form renders via FieldSet incl. RadioGroup preferred_contact_method; save persists | integration + E2E | `cd backend && cargo test -p crm-demo user_handler && cd frontend && npx playwright test tests/e2e/user-edit.spec.ts` | ❌ W0 (new spec) | ⬜ pending |
| 15-04-01 | 04 | 3 | COMP-03 | — | handlers/interaction.rs edit form renders via FieldSet; interaction_type as RadioGroup; save persists | integration + E2E | `cd backend && cargo test -p crm-demo interaction_handler && cd frontend && npx playwright test tests/e2e/interaction-edit.spec.ts` | ❌ W0 (new spec) | ⬜ pending |
| 15-04-02 | 04 | 3 | COMP-03 | — | Inline tag-add + note-add forms migrated; use new TextInput description prop where helpful | E2E | `cd frontend && npx playwright test tests/e2e/contact-edit.spec.ts -g "inline"` | ✅ (extend existing) | ⬜ pending |
| 15-04-03 | 04 | 3 | COMP-03 | — | contact.rs edit form refactored to use form_shell(); no regression from Phase 14 UAT | E2E + UAT | `cd frontend && npx playwright test tests/e2e/contact-edit.spec.ts tests/uat/contact-edit-uat.spec.ts` | ✅ (extend existing) | ⬜ pending |
| 15-05-01 | 05 | 4 | COMP-03 | T-15-01 (test-hook leak) | window.__mrnSetData/__mrnSendAction gated behind import.meta.env.DEV; production builds exclude hooks | integration (frontend build) | `cd frontend && npm run build && ! grep -q "__mrnSetData" build/*.js` | ❌ W0 (new test) | ⬜ pending |
| 15-05-02 | 05 | 4 | COMP-03 | — | Form.svelte submit-action dispatches collected form values (not empty payload) | unit (frontend) | `cd frontend && npx vitest --run src/lib/components/form/Form.browser-test.ts -t "submit-action"` | ❌ W0 (extend existing) | ⬜ pending |
| 15-05-03 | 05 | 4 | COMP-03 | — | contact.rs:1577-1584 hand-rolled Component literal replaced with Button builder | unit (backend) | `cd backend && cargo test -p crm-demo --test toast_builder` | ❌ W0 (new test) | ⬜ pending |
| 15-05-04 | 05 | 4 | COMP-03 | — | tests/helpers/schema-validator.ts uses node: prefix imports; svelte-check passes without @ts-expect-error suppressions | static check | `cd frontend && npm run check` | ✅ (extend existing) | ⬜ pending |
| 15-06-01 | 06 | 4 | COMP-03 | T-15-02 (Flowbite regression) | CI grep asserts zero Flowbite tokens under frontend/src, backend/crates, spec/ | E2E file-read | `cd frontend && npx playwright test tests/e2e/ci-guards.spec.ts -g "Flowbite"` | ❌ W0 (new test block) | ⬜ pending |
| 15-06-02 | 06 | 4 | COMP-03 | — | CONCEPT.md + TOOLING.md + .planning/codebase/STACK.md updated — no Flowbite as primary vocabulary | static check | `! grep -l "Flowbite" CONCEPT.md TOOLING.md .planning/codebase/STACK.md \| grep -v ':.*historical\|prior-art'` | ❌ W0 (guard) | ⬜ pending |
| 15-06-03 | 06 | 4 | COMP-03 | — | spec/PROTOCOL.md legacy /contactForm/errors section removed; canonical /_errors/{bind} section includes worked multi-field example | static check | `! grep -c "contactForm/errors" spec/PROTOCOL.md` (count 0) | ✅ (spec exists) | ⬜ pending |
| 15-07-01 | 07 | 5 | COMP-03 | — | Visual rebaselines for company-edit, user-edit, interaction-edit forms (desktop + mobile) | visual | `cd frontend && npx playwright test tests/visual/form.spec.ts` | ❌ W0 (new snapshots) | ⬜ pending |
| 15-07-02 | 07 | 5 | COMP-03 | — | UAT evidence committed per screen (company-edit, user-edit, interaction-edit, contact-tag-add, contact-note-add) | UAT | `ls .planning/phases/15-crm-migration-validation/15-uat-evidence/*/` | ❌ W0 (new dirs) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `frontend/tests/e2e/company-edit.spec.ts` — Playwright E2E for company edit form migration
- [ ] `frontend/tests/e2e/user-edit.spec.ts` — Playwright E2E for user edit form + RadioGroup demo
- [ ] `frontend/tests/e2e/interaction-edit.spec.ts` — Playwright E2E for interaction edit form + RadioGroup
- [ ] Extend `frontend/tests/e2e/contact-edit.spec.ts` with inline tag-add + note-add coverage
- [ ] Extend `frontend/tests/e2e/ci-guards.spec.ts` — Flowbite grep guard test block
- [ ] `frontend/tests/visual/form.spec.ts` — 6 new snapshot blocks (3 screens × desktop + mobile)
- [ ] `frontend/tests/uat/company-edit-uat.spec.ts` (+ user-edit-uat, interaction-edit-uat, inline-forms-uat) — Chrome-MCP UAT drivers per screen
- [ ] `backend/crates/crm-demo/tests/contact_persistence.rs` — integration test for country/notes/opt_in save
- [ ] `backend/crates/crm-demo/tests/company_handler.rs`, `tests/user_handler.rs`, `tests/interaction_handler.rs` — integration tests for per-field /_errors emission
- [ ] `backend/crates/marionette/tests/builders_form_shell.rs` — unit tests for form_shell() helper (via existing `tests/` infra)
- [ ] `backend/crates/marionette/tests/validation_patch.rs` — unit tests for validation_error_patch() helper
- [ ] `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs` — new migration file
- [ ] `.planning/phases/15-crm-migration-validation/15-uat-evidence/{company-edit,user-edit,interaction-edit,contact-tag-add,contact-note-add}/` — 5 UAT evidence folders

No new framework install required — all infrastructure (Vitest, Playwright, cargo test, SeaORM migrations) is already in place from Phases 10-14.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Listmonk integration remains functional across CRM | COMP-03 | Requires a live Listmonk instance; not part of CI. Phase 9 already validated; Phase 15 only touches form handlers, not the Listmonk sync client | Start `make dev`; log in as admin; open a contact; confirm the Listmonk history tab loads; confirm subscribing/unsubscribing still hits Listmonk |
| Full CRM CRUD smoke across login → companies → contacts → interactions → audit log | COMP-03 criterion 1 | Validates success-criterion #1 end-to-end; Chrome-MCP UAT automates per-screen, but the full-story walkthrough is the final human-confirmable gate | Chrome-MCP script: login, create a company, create a contact under that company, add a note, add a tag, create an interaction, view the audit log, edit user role, confirm no console errors |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter (after Wave 0 complete)

**Approval:** pending
