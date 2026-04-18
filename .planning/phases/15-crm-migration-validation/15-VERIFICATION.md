---
phase: 15-crm-migration-validation
verified: 2026-04-18T12:00:00Z
status: human_needed
score: 9/9 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run full Playwright E2E suite against live dev server"
    expected: "All E2E tests pass including company-edit, user-edit, interaction-edit, and contact-edit inline form tests; Flowbite CI guard reports 0 matches"
    why_human: "E2E specs require a live make dev instance; cannot run headless without the full backend running"
  - test: "Run visual baseline comparison for 6 new form snapshots"
    expected: "company-edit-form.png, company-edit-form-mobile.png, user-edit-form.png, user-edit-form-mobile.png, interaction-edit-form.png, interaction-edit-form-mobile.png all match baselines within maxDiffPixels: 200"
    why_human: "Visual snapshots require a running dev server to capture; diff comparison requires human confirmation the first-run baseline looks correct"
  - test: "Verify UAT evidence screenshots show correct rendered screens"
    expected: "desktop.png and mobile.png in each of the 5 15-uat-evidence folders (company-edit, user-edit, interaction-edit, contact-tag-add, contact-note-add) show the correct form layout, FieldSet legends, RadioGroup options where applicable, and per-field description text"
    why_human: "Screenshot content must be inspected visually; automated assertions.json shows passed: true but the actual rendered quality needs human review"
---

# Phase 15: CRM Migration & Validation — Verification Report

**Phase Goal:** The CRM demo runs entirely on the new component stack, proving the migration is complete and everything works end-to-end.
**Verified:** 2026-04-18T12:00:00Z
**Status:** human_needed (all automated checks pass; 3 items require live server)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | All 4 additional form handlers render via FieldSet composition (M1) | ✓ VERIFIED | `FieldSet\|form_shell` grep returns 9 in company.rs, 10 in user.rs, 22 in contact.rs; interaction.rs uses RadioGroup + Textarea + form_shell |
| 2 | Contact DB schema extended with country/notes/opt_in — all three fields persist round-trip (M2) | ✓ VERIFIED | Migration `m20260418_000011_extend_contact.rs` exists; entity Model has 3 new fields; save handler wires 9 `Set()` calls (3 insert + 3 update + 3 test); round-trip test passes at `contact.rs:1782` |
| 3 | Per-field validation via `/_errors/{bind}` write-path wired in every migrated save handler (M3) | ✓ VERIFIED | `validation_error_patch` call counts: company.rs: 4, user.rs: 4, interaction.rs: 7, contact.rs: 9, note.rs: 6; PROTOCOL.md documents single canonical shape + worked multi-field example |
| 4 | RadioGroup demonstrated in CRM (interaction.type + user.preferred_contact_method) (M4) | ✓ VERIFIED | interaction.rs `RadioGroup::new("Type", type_options)` at line 97; user.rs has 6 occurrences of `RadioGroup`; UAT assertions.json confirms 3 options with data-states for both screens |
| 5 | Zero Flowbite residue under frontend/src, backend/crates, spec — enforced by CI grep guard (M5) | ✓ VERIFIED | `grep -rn "flowbite" frontend/src backend/crates spec -i` exits with code 1 (no matches); ci-guards.spec.ts has `git grep -Iil 'flowbite'` guard |
| 6 | User-facing docs (CONCEPT.md, TOOLING.md, STACK.md) describe shadcn-svelte as the frontend vocabulary (M6) | ✓ VERIFIED | CONCEPT.md lines 260/268/630 use "shadcn-svelte"; TOOLING.md line 39 reads "shadcn-svelte - Tailwind CSS + bits-ui component library"; STACK.md line 47 lists "shadcn-svelte 1.2.7 + bits-ui 2.17.3 + @lucide/svelte 1.8.0" |
| 7 | spec/PROTOCOL.md documents only canonical `/_errors/{bind}` validation shape; worked multi-field example included (M7) | ✓ VERIFIED | Legacy `/contactForm/errors` array section deleted; `/_errors/{bind}` documented at lines 597-598; `#### Worked example: multi-field validation on form submit` section at line 602 shows PatchMessage with 2 SetData ops |
| 8 | Phase 14 review items resolved: `__mrnSetData` DEV-gated; Form.svelte submit-action honest; contact.rs toast Button; node: prefix imports (M8) | ✓ VERIFIED | init.ts has `if (typeof window !== 'undefined' && import.meta.env.DEV)` guard (3 occurrences); Form.svelte dispatches `getData(surface, bind)` payload; contact.rs uses `Button::new(&toast_label).id("toast-country-change").action(...)` at line 1646; schema-validator.ts uses `node:fs`/`node:path`/`node:url` |
| 9 | E2E + visual + UAT evidence for every migrated screen committed (M9) | ✓ VERIFIED (pending live server confirmation) | company-edit.spec.ts, user-edit.spec.ts, interaction-edit.spec.ts exist; contact-edit.spec.ts extended with 8 inline-form references; form.spec.ts has 8 references to new screen baselines; 6 PNG snapshots committed; all 5 UAT evidence folders contain desktop.png + mobile.png + assertions.json (passed: true) + console.log |

**Score:** 9/9 truths verified (M9 has automated evidence in place; live-server confirmation is a human step)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs` | SeaORM migration adding 3 columns | ✓ VERIFIED | File exists; contains ADD COLUMN for country/notes/opt_in and DROP COLUMN down migration |
| `backend/crates/crm-demo/src/migration/mod.rs` | Registers new migration | ✓ VERIFIED | 2 occurrences of `m20260418_000011_extend_contact` (mod declaration + Box::new) |
| `backend/crates/crm-demo/src/entities/contact.rs` | Model struct with 3 new fields | ✓ VERIFIED | `contact_country: Option<String>`, `contact_notes: Option<String>`, `contact_opt_in: bool` each appear exactly once |
| `backend/crates/crm-demo/src/seed.rs` | Seed data for new columns | ✓ VERIFIED | Alice: `Some("CH")`, `Some("Interested in Q2 enterprise tier.")`, `true`; Bob and Carol seeded with spread |
| `backend/crates/crm-demo/src/handlers/contact.rs` | handle_contact_save persists new fields; round-trip test inline | ✓ VERIFIED | 9 Set() calls for new fields (3 insert + 3 update + 3 test); test at line 1782; WR-02 fix at lines 457-459 reads from entity |
| `backend/crates/marionette/src/builders/standard.rs` | form_shell() helper | ✓ VERIFIED | `pub fn form_shell` at line 621; unit test at line 1354 |
| `backend/crates/marionette/src/validation.rs` | validation_error_patch() helper | ✓ VERIFIED | File exists; `pub fn validation_error_patch` present |
| `backend/crates/marionette/src/lib.rs` | validation module registered | ✓ VERIFIED | `pub mod validation` present |
| `frontend/src/lib/init.ts` | DEV-gated test hooks | ✓ VERIFIED | `import.meta.env.DEV` appears 3 times; both `__mrnSendAction` and `__mrnSetData` inside guard |
| `frontend/src/lib/components/form/Form.svelte` | Submit dispatches real payload | ✓ VERIFIED | `getData(surface, bind)` at line 38; fallback `{}` for no-bind branch |
| `frontend/tests/e2e/company-edit.spec.ts` | E2E spec for company edit form | ✓ VERIFIED | File exists; 2 × `__mrnSendAction`, 5 × FieldSet, 9 × validation/error |
| `frontend/tests/e2e/user-edit.spec.ts` | E2E spec for user edit form | ✓ VERIFIED | File exists; RadioGroup coverage confirmed |
| `frontend/tests/e2e/interaction-edit.spec.ts` | E2E spec for interaction edit form | ✓ VERIFIED | File exists; RadioGroup test at line 93 |
| `frontend/tests/visual/form.spec.ts` | 6 new visual baselines | ✓ VERIFIED | 8 references to company/user/interaction edit screen names; 6 PNG baseline files committed |
| `.planning/phases/15-crm-migration-validation/15-uat-evidence/` | 5 evidence folders | ✓ VERIFIED | All 5 folders (company-edit, user-edit, interaction-edit, contact-tag-add, contact-note-add) contain desktop.png + mobile.png + assertions.json (passed: true) + console.log |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `m20260418_000011_extend_contact.rs` | `migration/mod.rs` | `Box::new(…)` in migrations() vec | ✓ WIRED | 2 occurrences of migration name in mod.rs |
| `handle_contact_save` | `contact::ActiveModel` | `Set(data.country/notes/opt_in)` | ✓ WIRED | 9 Set() calls covering insert + update + test |
| `handle_company_save` | `validation_error_patch` | `collect_company_save_errors` pure fn | ✓ WIRED | 4 occurrences in company.rs |
| `handle_user_save` | `validation_error_patch` | `collect_user_save_errors` pure fn | ✓ WIRED | 4 occurrences in user.rs |
| `handle_interaction_save` | `validation_error_patch` | validation accumulator | ✓ WIRED | 7 occurrences in interaction.rs |
| `handle_note_save` | `validation_error_patch` | validation accumulator | ✓ WIRED | 6 occurrences in note.rs |
| `Form.svelte` | `getData(surface, bind)` | submit handler | ✓ WIRED | Line 38 reads bound subtree |
| `init.ts` | `import.meta.env.DEV` | outer if-guard | ✓ WIRED | Both hooks inside single DEV guard |
| E2E specs | `window.__mrnSendAction` | `page.evaluate` | ✓ WIRED | 2 occurrences in company-edit.spec.ts; same pattern in all 3 new specs |
| Visual baselines | PNG files | `toHaveScreenshot` | ✓ WIRED | 6 PNG files in `tests/__snapshots__/visual/form.spec.ts-snapshots/` |
| UAT specs | `15-uat-evidence/{screen}/` | `fs.writeFileSync` / `fs.mkdirSync` | ✓ WIRED | 5 evidence folders populated with required files |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `contact.rs handle_contact_form` | `found.contact_country/notes/opt_in` | `contact::Entity::find_by_id(cid)` SeaORM query | Yes — reads from DB after migration 11 | ✓ FLOWING |
| `validation_error_patch` | `errors: Vec<(bind, msg)>` | `collect_*_save_errors` pure fns (input validation, not DB) | Yes — server-derived literals | ✓ FLOWING |
| `form_shell` | `HashMap<String, Component>` | `build_with_children` macro output | Yes — composes real builder output | ✓ FLOWING |

---

### Behavioral Spot-Checks

Step 7b: SKIPPED for most checks (requires live dev server). The following were verified statically:

| Behavior | Method | Result | Status |
|----------|--------|--------|--------|
| Migration file has correct ADD COLUMN SQL | grep on migration file | "ALTER TABLE contact ADD COLUMN contact_country TEXT" found | ✓ PASS |
| WR-02 fix: edit form reads from entity | grep contact.rs:457-459 | `found.contact_country.as_deref().unwrap_or("")` found | ✓ PASS |
| Flowbite grep returns zero matches | `grep -rn "flowbite" frontend/src backend/crates spec -i` | Exit code 1 (no matches) | ✓ PASS |
| DEV gate on test hooks | grep init.ts | `import.meta.env.DEV` appears 3× wrapping both hooks | ✓ PASS |
| Form payload fix | grep Form.svelte | `getData(surface, bind)` dispatched as payload | ✓ PASS |
| Button builder for toast | grep contact.rs:1646 | `Button::new(&toast_label).id("toast-country-change")` found | ✓ PASS |
| node: prefix imports | grep schema-validator.ts | `node:fs`, `node:path`, `node:url` found | ✓ PASS |
| Legacy validation section removed from PROTOCOL.md | grep `/contactForm/errors` | No matches (clean) | ✓ PASS |
| Visual PNG baselines committed | filesystem check | 6 PNG files in `tests/__snapshots__/visual/form.spec.ts-snapshots/` | ✓ PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| COMP-03 | Plans 15-01 through 15-07 | CRM demo screens fully functional with new component implementations | ✓ SATISFIED | All 4 remaining form handlers migrated to FieldSet + form_shell; contact schema extended; per-field validation wired; Flowbite eliminated; E2E + visual + UAT evidence committed |

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `backend/crates/marionette/src/builders/standard.rs:656-658` | `form_shell` double-inserts heading/back_button/form_child into nodes map after `build_with_children` already includes them | ⚠️ Warning (WR-03) | Cosmetic — 6 unnecessary `.clone()` calls; functional result is correct; noted by code review as non-blocking |
| `frontend/tests/uat/*.spec.ts` | UAT specs use hardcoded `http://localhost:5173/` instead of relative `/` | ℹ️ Info (IN-04) | Breaks CI with ephemeral ports; UAT specs only run against a known local dev server so lower risk; non-blocking for Phase 15 |
| `backend/crates/crm-demo/src/handlers/*.rs:538-548` | `caller_id` falls back to `0` on unauthenticated sessions | ℹ️ Info (IN-02) | Defense-in-depth gap; auth middleware should reject first; noted by review; pre-deployment and non-blocking |

**No blockers found.** WR-01 (contact_persistence.rs) was resolved — test exists inline in `contact.rs:1782`. WR-02 (data loss on contact edit reload) was fixed in commit `2628a90`. WR-03 is cosmetic. 4 INFO items are non-blocking.

---

### Human Verification Required

#### 1. Full Playwright E2E Suite

**Test:** Start `make dev` in one terminal; run `cd frontend && npx playwright test` in another.
**Expected:** All E2E specs pass including `company-edit.spec.ts`, `user-edit.spec.ts`, `interaction-edit.spec.ts`, and the extended `contact-edit.spec.ts`; the Flowbite CI guard in `ci-guards.spec.ts` passes with 0 Flowbite token matches.
**Why human:** Requires a running backend+frontend dev server; cannot run headless against static files.

#### 2. Visual Baseline Sanity Check

**Test:** Open each of the 6 committed PNGs in `/home/oetiker/checkouts/marionette/frontend/tests/__snapshots__/visual/form.spec.ts-snapshots/`: `company-edit-form-chromium-linux.png`, `company-edit-form-mobile-chromium-linux.png`, `user-edit-form-chromium-linux.png`, `user-edit-form-mobile-chromium-linux.png`, `interaction-edit-form-chromium-linux.png`, `interaction-edit-form-mobile-chromium-linux.png`.
**Expected:** Each screenshot shows the correct form layout — FieldSet legends visible (Company details / Account + Permissions / Interaction), RadioGroup options rendered for user and interaction screens, description text under labelled fields, and action row right-aligned.
**Why human:** PNG content requires visual inspection; automated tooling only confirms file existence and pixel-diff tolerance, not semantic correctness.

#### 3. UAT Evidence Screenshot Review

**Test:** Open the 5 UAT evidence folders at `.planning/phases/15-crm-migration-validation/15-uat-evidence/`. For each, open `desktop.png` and `mobile.png`.
**Expected:** Forms render correctly with FieldSet grouping, RadioGroup options (for user-edit and interaction-edit), field descriptions, and action row. `assertions.json` `passed: true` should match what you see on screen.
**Why human:** Screenshot evidence is committed but content correctness requires human judgment.

---

### Gaps Summary

No gaps found. All 9 must-haves are verified in the codebase. Three human verification items relate to live-server E2E / visual testing and UAT screenshot review — standard final gates for any phase that produces rendered UI. The codebase delivers all required artifacts, wiring, and data flows.

**Open review items (non-blocking):**
- WR-03 (form_shell double-insert) — cosmetic, 6 redundant clones; functional result correct. Acceptable for Phase 15.
- IN-04 (hardcoded localhost URLs in UAT specs) — info level; UAT specs are developer-run only, not CI-gated. Can be fixed in a follow-up.
- IN-02 (caller_id silent fallback to 0) — info level; defense-in-depth gap, pre-deployment, no active exploit path.

---

_Verified: 2026-04-18T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
