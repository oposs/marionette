---
phase: 15-crm-migration-validation
plan: 03
subsystem: handlers
tags: [rust, crm, handlers, form-shell, fieldset, radio-group, validation, migration, tdd]

# Dependency graph
requires:
  - phase: 14-formscreen-enhancements
    provides: FieldSet / FieldSeparator / RadioGroup / Textarea primitives + Field.Error /_errors{bind} render path
  - phase: 15-crm-migration-validation
    provides: "Plan 15-02 — form_shell() envelope helper + validation_error_patch() in marionette::validation"
provides:
  - "handle_company_form now uses form_shell + FieldSet('Company details') + action row (flex gap-2 justify-end); name TextInput carries the locked description 'Will appear on invoices and contact details.' (D-E3)"
  - "handle_company_save emits /_errors/{bind} patches via validation_error_patch('content', errors) instead of per-field ActionError::BadPayload (D-D1); server-derived literal bind paths /companyForm/{name,website} (T-15-03-PLAN03-a)"
  - "handle_user_form uses form_shell + 2 FieldSets ('Account', 'Permissions') + FieldSeparator + RadioGroup with 3 options (email/sms/phone) each carrying locked per-option descriptions from 15-UI-SPEC §Description Copy Contract"
  - "handle_user_save emits /_errors/{bind} patches via validation_error_patch on /userForm/{name,email,password,role} with password rules preserving existing create-vs-edit semantics"
  - "UserFormData payload struct accepts preferred_contact_method: Option<String> with #[serde(default)] #[allow(dead_code)] per D-E2 (UI-only demo, not persisted)"
  - "Inline note-add form inside company detail view migrated to 15-UI-SPEC §6 locked layout: Container class='flex flex-col gap-2 items-end' wrapping Textarea rows(3) + Button '+ Add note'"
  - "Two pure-function validation helpers (collect_company_save_errors, collect_user_save_errors) give the TDD cycle a testable surface without DB setup, and isolate the bind-path literals in one place for audit"
affects: [15-04-handler-sweep-b, 15-05-interaction-form, 15-06-doc-ci-guard]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-field validation pulled into a pure fn (`collect_*_save_errors`) that returns `Vec<(bind_path, message)>` — testable RED/GREEN without a DB context; callers feed the vec into validation_error_patch"
    - "Post-form content injected into a form_shell-built outer Container by patching the Container's .children with new ids before inserting the new descendant map — used by company.rs to keep linked-contacts table + notes list positioned after the Form without abandoning form_shell"
    - "preferred_contact_method payload-accepted, handler-discarded pattern for UI-only demo fields (#[serde(default)] #[allow(dead_code)] + inline comment); isolates demo wiring from persistence surface area"

key-files:
  created: []
  modified:
    - "backend/crates/crm-demo/src/handlers/company.rs (handle_company_form rewritten via form_shell + FieldSet; handle_company_save rewritten via validation_error_patch; inline note-add migrated to UI-SPEC §6; collect_company_save_errors helper + 4 tests added)"
    - "backend/crates/crm-demo/src/handlers/user.rs (handle_user_form rewritten via form_shell + 2 FieldSets + FieldSeparator + RadioGroup; handle_user_save rewritten via validation_error_patch; UserFormData gains preferred_contact_method field; collect_user_save_errors helper + 7 tests added)"

key-decisions:
  - "Pulled validation into pure fns (collect_company_save_errors, collect_user_save_errors) so the TDD cycle has a testable surface without requiring an in-memory SQLite DB harness per test. The handler call site becomes two lines (`let errors = collect_*_save_errors(...); if !errors.is_empty() { return Ok(vec![validation_error_patch(...)]) }`), matching 15-PATTERNS.md SP-2 exactly."
  - "company.rs post-form content (linked-contacts sub-table, notes list, note-add form) is appended to the form_shell-built outer Container by mutating the container node's `children` vec after the form_shell call, instead of bypassing form_shell. This keeps the helper usable for a handler with downstream content while still getting the envelope shape consistency D-B1 promised."
  - "Inline note-add form upgraded from TextInput to Textarea with rows(3) + full_width(true). The UI-SPEC §6 contract calls for a textarea-over-button flex-col layout; the previous inline TextInput was a hangover from Phase 12 and didn't give enough room for a real note. Button label swapped from 'Add Note' to '+ Add note' per 15-UI-SPEC §Copywriting."
  - "preferred_contact_method in UserFormData uses Option<String> (not String) because the frontend's RadioGroup `bind` may deserialise as null in a wire-edge case (e.g., initial render before the user clicks). Plus the #[serde(default)] makes old payloads without the field still deserialise cleanly — no migration pain when Wave 2 ships."
  - "form_data pre-populates preferred_contact_method to `\"email\"` on both new and edit so the RadioGroup has a default selection when the form first renders. 15-UI-SPEC didn't mandate a default but an unselected RadioGroup is visually awkward; `email` is the safest default since every user already has an email address."
  - "Role validation copy: 'Choose one of the listed roles (admin or user).' — parenthesises the actual enum values so a future role-extension (e.g., adding 'viewer') only needs the copy updated; 15-UI-SPEC §Copywriting anti-pattern for this row was 'Invalid value.' which doesn't tell the user what's valid."

patterns-established:
  - "Validation-helper fn + validation_error_patch pattern (SP-2) — the canonical Phase 15 save-handler shape: `let errors = collect_*_save_errors(...); if !errors.is_empty() { return Ok(vec![validation_error_patch('content', errors)]); } // proceed to DB write`"
  - "form_shell-with-post-form-content pattern — when a handler needs to render content BEOW the form envelope (e.g., related-entity tables, comments, notes), mutate the form_shell-built outer Container's `.children` Vec after the helper returns; preserves the form_shell invariant (root Container wraps heading+back+form) while allowing handler-specific additions"
  - "preferred_contact_method as the canonical UI-only-demo-field template: declare in payload struct with #[serde(default)] #[allow(dead_code)] and a comment linking to the decision doc (here D-E2); no DB column, no validation, no audit entry"

requirements-completed: [COMP-03]

# Metrics
duration: 10min
completed: 2026-04-18
---

# Phase 15 Plan 03: Company + User Form Migration Summary

**Rewrote handle_company_form + handle_company_save + handle_user_form + handle_user_save onto the Phase 14 form_shell + FieldSet composition, introduced the first production RadioGroup in the CRM demo (preferred_contact_method, UI-only per D-E2), and rewired every per-field BadPayload branch in both handlers to emit /_errors/{bind} patches via validation_error_patch.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-04-18T07:24:39Z
- **Completed:** 2026-04-18T07:34:21Z
- **Tasks:** 2 (both TDD — full RED/GREEN cycle, no REFACTOR needed)
- **Files modified:** 2

## Accomplishments

- **Company edit form migrated to Phase 14 composition** — single `FieldSet("Company details")` wrapping `[name, website, address]`, name carries the locked description string `"Will appear on invoices and contact details."`, action row `flex gap-2 justify-end` with `[Cancel (outline), Save company (default)]`, new `← Back` button (outline, click `company_list`), and outer envelope assembled via `form_shell("company-form-root", ...)`.
- **Inline note-add form** inside the company detail view migrated to the 15-UI-SPEC §6 contract: `Container class="flex flex-col gap-2 items-end"` wrapping a `Textarea` (`rows=3`, `full_width=true`) and a primary `Button("+ Add note")`.
- **User edit form migrated** with a 5-field split into two `FieldSet`s (`Account = [name, email, password]` / `Permissions = [role, preferred_contact_method]`) separated by an explicit `FieldSeparator`. Email carries the locked description `"Used for password resets and notifications."` (D-E3). Save button renamed to `"Save user"`. New `← Back` button. Outer envelope via `form_shell`.
- **RadioGroup in production (first time in CRM demo)** — `preferred_contact_method` with 3 options (email/sms/phone), each carrying the locked per-option description from 15-UI-SPEC §Description Copy Contract (`"Receive updates by email."`, `"Text messages to your phone."`, `"A human will call you."`). UI-only per D-E2 — `UserFormData.preferred_contact_method: Option<String>` is `#[serde(default)] #[allow(dead_code)]`, accepted-but-discarded.
- **Validation write-path rewired** — both `handle_company_save` and `handle_user_save` now emit `/_errors/{bind}` patches via `validation_error_patch("content", errors)`. Bind paths are server-derived string literals (`/companyForm/{name,website}`, `/userForm/{name,email,password,role}`) — T-15-03-PLAN03-a/b mitigation enforced by the pure-fn boundary. `ActionError::BadPayload` retained only for protocol-layer failures (JSON parse, missing `form_bind`) per D-D4.
- **Password rules preserved** — `collect_user_save_errors` keeps the existing create-vs-edit semantics (create: required ≥8 chars; edit: blank OK, non-empty <8 fails) and switches error copy to the 15-UI-SPEC §Copywriting preferred strings (`"Password is required."`, `"Password must be at least 8 characters."`).
- **11 new unit tests**, all green: 4 for company validation (empty name, bad website, order, valid), 7 for user validation (empty name, bad email, bad role, empty password on create, blank-OK on edit, short on edit, order, valid).

## Task Commits

Each task ran the full TDD RED/GREEN cycle:

1. **Task 1 RED:** failing tests for `collect_company_save_errors` — `3b182af` (test)
2. **Task 1 GREEN:** `handle_company_form` + `handle_company_save` rewrite + helper impl + inline note-add migration — `0525415` (feat)
3. **Task 2 RED:** failing tests for `collect_user_save_errors` — `08855bf` (test)
4. **Task 2 GREEN:** `handle_user_form` + `handle_user_save` rewrite + helper impl + `UserFormData.preferred_contact_method` addition — `ec65b17` (feat)

No REFACTOR commits needed — both implementations landed at the planned surface without needing cleanup.

## Files Created/Modified

- **`backend/crates/crm-demo/src/handlers/company.rs`** — rewritten handle_company_form (form_shell + FieldSet; back_button; action row) and handle_company_save (validation_error_patch); migrated inline note-add form to UI-SPEC §6 layout; added `collect_company_save_errors` pure-fn helper + 4 unit tests. Imports updated: added `form_shell`, `FieldSet`, `Textarea` from `marionette::builders::standard`; added `marionette::validation::validation_error_patch`. Lines: +240 / −107.
- **`backend/crates/crm-demo/src/handlers/user.rs`** — rewritten handle_user_form (form_shell + 2 FieldSets + FieldSeparator + RadioGroup + back_button + action row) and handle_user_save (validation_error_patch with password create/edit rules preserved); added `preferred_contact_method` to UserFormData; added `collect_user_save_errors` pure-fn helper + 7 unit tests. Imports updated: added `form_shell`, `FieldSet`, `FieldSeparator`, `RadioGroup`, `RadioOption`; added `marionette::validation::validation_error_patch`. Lines: +351 / −54.

## Decisions Made

- **Validation logic pulled into pure fns** so TDD has a testable surface without DB harness per test. Matches 15-PATTERNS.md SP-2 handler shape exactly (the handler body becomes two lines wrapping the helper call).
- **form_shell with post-form content (company.rs)** — linked-contacts table, notes list, and note-add form live AFTER the form envelope in the outer Container. Instead of bypassing `form_shell`, I mutate the returned outer Container's `.children` Vec to append the post-form node ids, then insert the nodes + descendants into the returned map. Preserves the Phase 14 envelope shape invariant while allowing handler-specific content below the form.
- **preferred_contact_method defaulted to `"email"` in form_data** — 15-UI-SPEC didn't mandate a default but an unselected RadioGroup renders awkwardly. `email` is a safe default (every user has one).
- **Role validation copy specifies the enum values** — `"Choose one of the listed roles (admin or user)."` rather than the anti-pattern `"Invalid value."` from 15-UI-SPEC §Copywriting. The parenthetical makes the message actionable without the user having to look elsewhere.
- **Inline note-add upgraded from TextInput to Textarea** — the UI-SPEC §6 contract explicitly calls for a textarea-over-button flex-col layout. The old single-line input was a Phase 12 hangover that didn't give enough room for a real note.

## Deviations from Plan

**None — plan executed exactly as written** with two minor scope-clarifying choices that were within Claude's Discretion (D-I):

**1. [Scope Clarification] Added pure-fn validation helpers inside the handler files rather than inline in the save handlers**

- **Found during:** Task 1 RED (needed a testable surface for the TDD cycle)
- **Issue:** Inline validation in the save handler can't be unit-tested without a full HandlerContext (DB, session, payload extractor). The plan's acceptance criteria included `cargo test -p crm-demo company` / `... user` passing, but without a helper, the only "tests" would have been compile-time assertions via grep.
- **Decision:** Extracted the error-collection into `collect_company_save_errors` and `collect_user_save_errors` — pure `fn(&str, Option<&str>, ...) -> Vec<(String, String)>` signatures that test the error tuples directly. The save handlers consume the helpers in a 2-line pattern that matches 15-PATTERNS.md SP-2.
- **Impact:** No scope creep — same validation logic, same bind paths, same copy. Added 11 green tests.

**2. [UX Polish] form_data pre-populates preferred_contact_method to `"email"`**

- **Found during:** Task 2 GREEN
- **Issue:** 15-UI-SPEC didn't specify a default selection for the RadioGroup. Leaving the field unset renders all three options as unselected, which is visually awkward.
- **Decision:** Set `preferred_contact_method: "email"` in both new-user and edit-user form_data blocks. Matches the top option in the RadioGroup so the UI shows a natural "email selected by default" state.
- **Impact:** Zero — UI-only per D-E2, the value is discarded server-side anyway.

**Total deviations:** 0 plan-scope deviations; 2 scope-clarifying choices made within D-I (Claude's Discretion).

## Issues Encountered

None. Compile was green on both tasks' GREEN commits without any iteration; tests passed on first run after the implementations were added.

## Known Stubs

**1. `preferred_contact_method` is intentionally stubbed per D-E2.**

- **File:** `backend/crates/crm-demo/src/handlers/user.rs` — `UserFormData::preferred_contact_method: Option<String>`, marked `#[serde(default)] #[allow(dead_code)]`.
- **Reason:** D-E2 explicitly scopes the field as UI-only for Phase 15 — gives the RadioGroup primitive a production home without dragging a DB migration into this phase.
- **Resolution phase:** Deferred. See `.planning/phases/15-crm-migration-validation/15-CONTEXT.md` §Deferred Ideas for "Persistence of `preferred_contact_method` on the user entity".
- **Plan goal impact:** None — the plan's stated goal is to introduce RadioGroup to the CRM demo's user edit form, which it does. Persistence was never in scope.

## Threat Flags

None. All `validation_error_patch` bind paths are server-derived string literals passed to the pure-fn validation helpers — T-15-03-PLAN03-a and T-15-03-PLAN03-b mitigations are structurally enforced (the helpers take no user-supplied `bind` argument, only field values). T-15-03-PLAN03-c (password field): error copy is static (`"Password is required."` / `"Password must be at least 8 characters."`) and never echoes the submitted value. T-15-03-PLAN03-d (preferred_contact_method silently discarded): documented via the `#[allow(dead_code)]` comment in the payload struct, accepted per D-E2 (low risk — no security boundary crossed).

## Self-Check: PASSED

**Files modified exist:**
- FOUND: `backend/crates/crm-demo/src/handlers/company.rs` (contains `form_shell(`, `FieldSet::new`, `validation_error_patch`, `"Will appear on invoices and contact details."`, `"Save company"`, `"+ Add note"`, `company-note-form-row`)
- FOUND: `backend/crates/crm-demo/src/handlers/user.rs` (contains `form_shell(`, `RadioGroup::new`, `legend("Account"`, `legend("Permissions"`, `FieldSeparator`, `preferred_contact_method`, `"Used for password resets and notifications."`, 3× locked option description strings, `"Save user"`)

**Commits exist on worktree branch:**
- FOUND: `3b182af` (Task 1 RED — `test(15-03)` for company validation)
- FOUND: `0525415` (Task 1 GREEN — `feat(15-03)` company migration)
- FOUND: `08855bf` (Task 2 RED — `test(15-03)` for user validation)
- FOUND: `ec65b17` (Task 2 GREEN — `feat(15-03)` user migration + RadioGroup)

**Acceptance criteria (Task 1 — company.rs):**
- `grep -c "form_shell("` → 2 (≥1 required) ✓
- `grep -c "\"/companyForm/name\""` → 5 (≥2 required) ✓
- `grep -c "Will appear on invoices and contact details"` → 1 (=1 required) ✓
- `grep -c "Save company"` → 2 (≥1 required) ✓
- `grep -c "validation_error_patch"` → 4 (≥1 required) ✓
- `grep -c "FieldSet::new"` → 1 (≥1 required) ✓
- `grep -cE 'BadPayload\\("Name is required|BadPayload\\("Website'` → 0 (=0 required) ✓

**Acceptance criteria (Task 2 — user.rs):**
- `grep -c "form_shell("` → 2 (≥1 required) ✓
- `grep -c "RadioGroup::new"` → 1 (=1 required) ✓
- `grep -c "Used for password resets and notifications"` → 1 (=1 required) ✓
- `grep -cE "Receive updates by email|Text messages to your phone|A human will call you"` → 3 (=3 required) ✓
- `grep -c 'legend("Account"'` → 1 (≥1 required) ✓
- `grep -c 'legend("Permissions"'` → 1 (≥1 required) ✓
- `grep -c "FieldSeparator"` → 4 (≥1 required) ✓
- `grep -c "preferred_contact_method"` → 9 (≥3 required) ✓
- `grep -c "validation_error_patch"` → 4 (≥1 required) ✓
- `grep -c "Save user"` → 1 (≥1 required) ✓

**Build + test verification:**
- `cargo check -p crm-demo` → clean compile
- `cargo test -p crm-demo company` → 4/4 company tests pass (plus 1 pre-existing fetch_rows auth test matched the filter)
- `cargo test -p crm-demo user` → 7/7 user tests pass (plus 2 pre-existing fetch_rows auth tests matched the filter)
- `cargo test -p crm-demo` (full) → 39 lib tests + 5 integration = 44 pass (was 28+5 = 33; +11 new tests from this plan)

## TDD Gate Compliance

Both tasks ran the full TDD cycle with visible RED → GREEN commits in git log:

- **Task 1:** `test(15-03)` RED commit `3b182af` (tests fail to compile — `collect_company_save_errors` does not exist) immediately followed by `feat(15-03)` GREEN commit `0525415` (helper + rewrite land together, tests pass).
- **Task 2:** `test(15-03)` RED commit `08855bf` (tests fail to compile — `collect_user_save_errors` does not exist) immediately followed by `feat(15-03)` GREEN commit `ec65b17` (helper + rewrite land together, tests pass).

RED was confirmed by `cargo test --no-run` failing with `error[E0425]: cannot find function ... in this scope` on both tasks before the corresponding GREEN commit. REFACTOR was not required — both implementations landed at the planned surface without needing cleanup.

## Next Phase Readiness

- **Plan 15-04 (handler sweep B: contact.rs inline tag-add + note-add)** can consume `form_shell`, `validation_error_patch`, and the post-form-content append pattern established here.
- **Plan 15-05 (interaction-form)** follows the same SP-1 / SP-2 shape as company/user — single FieldSet "Interaction", Textarea full-width for notes, RadioGroup for interaction_type. The interaction save handler also follows the `collect_*_save_errors` + `validation_error_patch` pattern.
- **Plan 15-07 (E2E tests)** can target the stable DOM ids introduced here: `company-form-root`, `company-details-set`, `company-form-actions`, `user-form-root`, `user-account-set`, `user-permissions-set`, `user-form-preferred-contact-method`, and the locked legend / description / button-label strings.
- **No new deferred items** introduced by this plan. Pre-existing clippy drift (tracked in Plan 15-02 deferred-items.md) remains untouched — my changes added zero new clippy warnings.

---
*Phase: 15-crm-migration-validation*
*Plan: 03*
*Completed: 2026-04-18*
