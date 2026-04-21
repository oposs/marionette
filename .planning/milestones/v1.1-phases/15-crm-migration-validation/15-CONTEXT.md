# Phase 15: CRM Migration & Validation - Context

**Gathered:** 2026-04-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 15 is the v1.1 milestone closer. It does three things and nothing else:

1. **Sweep the remaining CRM form handlers onto the Phase-14 FieldSet composition.** Phase 14 migrated `handlers/contact.rs` (edit form only) as the canonical exercise. Phase 15 migrates the four remaining form surfaces using the same primitives and composition rules: `handlers/company.rs` (edit form), `handlers/user.rs` (edit form), `handlers/interaction.rs` (edit form), and the inline **tag-add** + **note-add** forms embedded in `handlers/contact.rs` (and the note-add form embedded in `handlers/company.rs`). No handler that renders a multi-field `Form` escapes Phase 15 still using the flat `Form::new().children([TextInput, ...])` shape.

2. **Close the Phase-14-deferred items that are explicitly earmarked for Phase 15.** Four concrete items:
   - **Contact DB schema gaps** (`contact.rs:84`, `contact.rs:453-460`): add `country`, `notes`, `opt_in` columns to the `contact` table via a new SeaORM migration and persist those fields in `handle_contact_save`. The form already accepts them; the backend drops them on the floor today.
   - **Per-field validation write-path** (Phase 14 UAT-03 + WR-02): save handlers that currently return `ActionError::BadPayload` (form-level toast) must emit per-field `SetData` patches to `/_errors/{bind}` so the Field.Error render path — proven sound in Phase 14 — actually fires end-to-end on real submission.
   - **Validation documentation drift** (Phase 14 WR-02): `spec/PROTOCOL.md` currently documents two validation shapes (§legacy `/contactForm/errors` array at ~ll. 804-819 and §new `/_errors/{bind}` at ~ll. 593-600). Delete the legacy section; keep `/_errors/{bind}` as the single canonical shape.
   - **Three leftover review items** from Phase 14 (IN-01, IN-02, WR-01): (a) gate `window.__mrnSetData` behind `import.meta.env.DEV` so the test hook doesn't ship to production builds, (b) replace the hand-rolled `Component` literal in `contact.rs:1577-1584` (toast path) with the Button builder, (c) either drop the empty-payload `sendAction` in `Form.svelte:29` or wire a real `submit`-action pathway.

3. **Lock the clean break from Flowbite with a CI guard and doc-alignment pass.** The codebase's runtime is already Flowbite-free (0 matches under `frontend/src/`, `backend/crates/`, `spec/`). But:
   - `CONCEPT.md` (lines ~260, 268, 630) and `TOOLING.md` (line 39) still mention Flowbite as the implementation vocabulary. Update these to reflect shadcn-svelte as the canonical choice; keep one deliberate "prior-art" note if the historical framing still reads well, otherwise delete.
   - `frontend/tests/e2e/ci-guards.spec.ts` currently asserts `TableScreen.*` deletion. Extend it to (a) grep-assert zero `flowbite` tokens under runtime paths (`frontend/src/**`, `backend/crates/**`, `spec/**`, top-level `CONCEPT.md`/`TOOLING.md`) and (b) keep the existing FormScreen deletion guard confirmed by Phase 14.

Also in scope, opportunistically: introduce the Rust-side `form_shell()` helper that Phase 14 D-A2 explicitly left open ("Phase 15 can introduce a Rust-side helper if the repetition becomes painful across ≥3 screens") — the threshold is tripped once company/user/interaction join contact on the new shape. Keep it a thin builder helper; not a resurrected frontend `FormScreen`.

**What this phase is NOT:**

- NOT a protocol change. No new SDUI component types beyond what Phase 14 shipped. If a form migration exposes a missing primitive, file it as deferred and reshape the handler.
- NOT a DataTable rewrite. Phase 13 already migrated the four list handlers; Phase 15 leaves `audit`, `company`, `contact`, `user` list screens alone unless the Flowbite/doc sweep finds a drift.
- NOT a fix for the 5 pre-existing popup browser-test failures in `popup/ConfirmDialog.browser-test.ts` (4) and `popup/ToastSurface.browser-test.ts` (1). These are Phase 11+ residue unrelated to CRM migration and gating Phase 15 on them would expand the phase past its purpose. Logged in 13-deferred-items and Phase 12 deferred-items; stay deferred to a dedicated popup-fix follow-up.
- NOT a toolchain change. The `@types/node` resolution for `tests/helpers/schema-validator.ts` (Phase 12/13/14 deferral) gets the minimum fix — rewrite imports to `node:fs`/`node:path`/`node:url` so `svelte-check` stops complaining — but we do NOT wire `@types/node` into the frontend `tsconfig.json` or broaden the check-surface.
- NOT the milestone-close ceremony. `/gsd-complete-milestone` for v1.1 runs after Phase 15 ships. PROJECT.md "Current State" and REQUIREMENTS.md Validated sections get rolled forward in that ceremony, not here.
- NOT wizard forms, persistent column-visibility, breadcrumbs, or any other v2 feature (SHELL-05..07, TABLE-04..07, FORM-03, FORM-04).
- NOT a crm-demo clippy cleanup (76-86 pedantic warnings from toolchain drift, logged in Phase 12 deferred-items). Run `cargo clippy -p marionette-protocol -p marionette -- -D warnings` (in-scope crates) — pass. Do not gate Phase 15 on `-p crm-demo` pedantics.

</domain>

<decisions>
## Implementation Decisions

### Area A — CRM form migration scope

- **D-A1: Full form-handler sweep — company, user, interaction, plus inline tag-add + note-add forms.** Every call site that builds a `Form::new().children([…leaf components…])` in `backend/crates/crm-demo/src/handlers/` gets rewritten to the Phase 14 composition shape: `Container → Heading + back Button → Form → FieldSet+FieldSeparator+… → action row`. Concretely:
  - `handlers/company.rs:190-215` — company edit form. Fields: `name`, `website`, `address`. Group into a single `FieldSet` ("Company details") on v1; two-set split (Details / Address) if `full_width` becomes natural for `address` on desktop.
  - `handlers/user.rs:217-260` — user edit form. Fields: `name`, `email`, `password`, `role` select. Group into `FieldSet`("Account") containing `[name, email, password]` + `FieldSet`("Permissions") containing `[role, preferred_contact_method]` (the latter is the new RadioGroup — see D-E1).
  - `handlers/interaction.rs:63-109` — interaction edit form. Fields: `type` select → migrate to `RadioGroup` (see D-E1 — this is the natural home), `subject`, `date`, `notes`. Group into `FieldSet`("Interaction") containing all four, with `notes` marked `full_width`.
  - `handlers/contact.rs:716-760` — inline **tag-add** + **note-add** forms. These are one-field forms; they don't need `FieldSet` grouping but must (a) go through the same `Form.svelte` boundary (already true), (b) use the new `TextInput` with `description` where the label benefits, and (c) consume the per-field error shape (D-D1).
  - `handlers/company.rs:318-330` — inline note-add form in company view. Same treatment.
- **D-A2: Login form (`auth.rs`) stays out of scope.** Login is a REST POST endpoint (`handle_login`), not an action-style `Form`. The frontend login page (`frontend/src/routes/+page.svelte`) renders a standalone SvelteKit form, not an SDUI handler. The TextInput `input_type` fix from Phase 14 D-E1 already covers the password field rendering. Touch only if the CI Flowbite guard surfaces a residue there.
- **D-A3: No new frontend SDUI components in this phase.** If a migration exposes a gap (e.g., handlers/interaction needs a time-picker and `TextInput type="datetime-local"` isn't enough), file it as deferred and reshape the handler to work with existing primitives. Do not introduce `DatePicker`, `Combobox`, `FileInput`, or other recipe components mid-sweep.

### Area B — DRY helper (form_shell) and inline-composition pattern

- **D-B1: Introduce a thin Rust-side `form_shell()` helper in `backend/crates/marionette/src/builders/standard.rs`.** Phase 14 D-A2 set the threshold at ≥3 screens; Phase 15 has 4 form handlers (contact, company, user, interaction) repeating the `Container([Heading, back_button, Form, …])` envelope. A zero-cost helper that assembles those four fixed children from positional args (`form_shell(heading, back_action, form_node, form_descendants)`) keeps the handler code terse without reviving a frontend `FormScreen` abstraction. No opinionated styling — it literally just builds the adjacency-list shape that every handler currently repeats. Handlers that want custom envelope shapes (e.g., contact.rs's edit form with extra metadata) compose by hand as before.
- **D-B2: Migrate contact.rs to use `form_shell()` as part of the sweep.** Phase 14 D-A2 was careful to leave contact.rs inline ("mirrors Phase 13's pattern"). Now that the helper exists, refactor contact.rs to consume it for consistency — this is the only backward-facing edit to Phase 14 work, and it's additive (shape stays identical, construction sites compress).
- **D-B3: No frontend counterpart.** The helper is Rust-side only. `frontend/src/lib/components/screen/` stays empty (both FormScreen and TableScreen are retired). The SDUI protocol sees no change.

### Area C — DB schema + contact field persistence

- **D-C1: New SeaORM migration `m20260418_000011_extend_contact.rs`.** Adds three columns to the `contact` table:
  - `contact_country TEXT NULL` — free-form string (ISO-3166-1 alpha-2 like "CH" or "DE", but no DB-level enum; existing country-select options enumerate).
  - `contact_notes TEXT NULL` — long-text, default NULL.
  - `contact_opt_in INTEGER NOT NULL DEFAULT 0` — boolean as 0/1 (SQLite convention; SeaORM handles `bool`).
- **D-C2: Update `entities/contact.rs` Model struct** to include `contact_country: Option<String>`, `contact_notes: Option<String>`, `contact_opt_in: bool`. Update seed data in `seed.rs` to populate a realistic spread (some opt-ins, some country-nulls) so the UAT screens show non-trivial data.
- **D-C3: Wire `handle_contact_save` to persist the three fields.** Drop the `#[serde(default)] #[allow(dead_code)]` attributes on `ContactFormData.country`, `ContactFormData.notes`, `ContactFormData.opt_in` and set them on the `ActiveModel`. Keep the country-select node-patch behaviour (Phase 12 D-A6) identical — only the save path changes.
- **D-C4: Down migration must round-trip.** The migration's `down` drops the three columns. Phase 15 is pre-deployment; no prod data to preserve; keep `down` honest.

### Area D — Per-field validation wiring

- **D-D1: Save handlers emit per-field `SetData` patches to `/_errors/{bind}` on validation failure, not form-level `BadPayload` toasts.** Concretely: `handle_contact_save`, `handle_company_save`, `handle_user_save`, `handle_interaction_save` each move from `Err(ActionError::BadPayload("…"))` to a `PatchMessage` containing one `SetData` op per invalid field. The `bind` path maps to the field's protocol path (e.g., `/contactForm/name`), and the error value is a plain string — the Field.Error component already consumes `/_errors/{bind}` as a string (`TextInput.svelte:26` pattern). Existing `ActionError::BadPayload` stays the fallback for "request is so malformed we can't tell which field is wrong" (JSON parse failures, missing form_bind, etc.).
- **D-D2: Delete the legacy `/contactForm/errors` array shape from `spec/PROTOCOL.md`.** Phase 14 WR-02 flagged two validation shapes documented side-by-side. `/_errors/{bind}` is the canonical shape since Phase 11; the legacy array pre-dates it and is nowhere exercised in the current code. Remove the legacy section (approx. `spec/PROTOCOL.md:804-819`), keep the `/_errors/{bind}` section (approx. `spec/PROTOCOL.md:593-600`), and add one explicit example of a multi-field validation patch to show how a handler should shape it.
- **D-D3: Provide a small Rust helper `validation_error_patch()` in `backend/crates/marionette/src/error.rs` (or a new `validation.rs`).** Takes an iterator of `(bind_path, message)` and returns a `PatchMessage` already assembled for the current surface. Keeps each handler's validation-emission site to 3-4 lines. No new error variant — the handler returns `Ok(response)` with the validation patch; the error channel stays reserved for protocol-layer failures.
- **D-D4: Existing toast path stays for non-field errors.** Server errors, auth failures, and database errors continue through `ActionError` → `ErrorMessage` → `/_errors` banner. The change in D-D1 is strictly about field-level validation.

### Area E — RadioGroup CRM demo + consistency touches

- **D-E1: Migrate `handlers/interaction.rs` `type` Select to RadioGroup.** The interaction type is a small fixed set (`call`, `email`, `meeting`, per `interaction.rs:196`) — the canonical RadioGroup fit. Backend builder: `RadioGroup::new("Type").options([("call", "Call"), …]).bind("/interactionForm/interaction_type").build()`. Mirror the existing Select's validation (must be one of the three values) server-side.
- **D-E2: Add `preferred_contact_method` RadioGroup to user edit form, no DB persistence.** Pure UI demo so we exercise RadioGroup in a place other than interaction. The handler accepts the field in the payload but discards it (explicit `#[allow(dead_code)]` with a comment that v2 persists it). This keeps RadioGroup visible in CRM without dragging another migration into Phase 15.
- **D-E3: Every migrated form gets a `description` on at least one field.** Phase 14 proved the `Field.Description` render path. Phase 15 exercises it in production handlers so the primitive is visible end-to-end (e.g., "Will appear on invoices" under `company.name`, "Used for password resets" under `user.email`).

### Area F — Flowbite residue guard + doc cleanup

- **D-F1: Zero Flowbite tokens in runtime paths — enforced by CI guard.** Extend `frontend/tests/e2e/ci-guards.spec.ts` with a new test block: `test('no Flowbite residue in runtime code', …)` that greps (case-insensitive) for `flowbite` under `frontend/src/**`, `backend/crates/**`, and `spec/**`. Expected match count: 0. The grep runs via `child_process` against the repo root (the existing spec already uses `node:fs` for filesystem checks — same pattern).
- **D-F2: Update CONCEPT.md and TOOLING.md to reflect shadcn-svelte.** `CONCEPT.md:260`, `CONCEPT.md:268`, `CONCEPT.md:630` currently use "Flowbite" as the example web-side vocabulary for the "base components" discussion. Rewrite those lines to say "shadcn-svelte" (or "a shadcn-svelte-like component set") and keep the cross-platform framing intact. `TOOLING.md:39` replaces "Flowbite Svelte - Tailwind CSS component library" with "shadcn-svelte - Tailwind CSS + bits-ui component library". One historical footnote at the end of CONCEPT.md is acceptable ("Earlier iterations used Flowbite; v1.1 migrated to shadcn-svelte for …") — use discretion on whether to include it.
- **D-F3: Update `.planning/codebase/STACK.md`.** It's a governance doc, not runtime, so the CI guard doesn't touch it — but it's still read by downstream agents. Rewrite the frontend stack section to describe shadcn-svelte. Same pass for any REQUIREMENTS.md §Validated text once v1.1 closes (that's milestone-close, not Phase 15, but adjacent).
- **D-F4: ci-guards.spec.ts retains its FormScreen + TableScreen deletion asserts.** Don't regress Phase 13 D-A2 or Phase 14 D-A1. Add one more retired-file assert for anything Phase 15 deletes (unlikely — the sweep is additive).

### Area G — Scope-closure cleanup (Phase 14 leftovers)

- **D-G1: Gate `window.__mrnSetData` behind `import.meta.env.DEV` (Phase 14 IN-02).** `frontend/src/lib/init.ts:92-102` currently assigns the test hook unconditionally. Wrap the assignment in `if (import.meta.env.DEV)` so production builds don't ship the hook. Keep the companion `__mrnSendAction` hook under the same gate (consistency). UAT and E2E suites run in dev mode, so they keep working.
- **D-G2: Fix `Form.svelte:29` empty-payload `sendAction` (Phase 14 WR-01).** Today `Form.svelte` dispatches `sendAction(form.action, {}, surface)` with an empty payload when the action is set. No handler currently wires `action` on `Form` (save flows dispatch through the Save Button), so this is latent. Resolve by either (a) passing the collected form values as the payload, matching what a real submit would send, or (b) removing the dispatch altogether if no handler uses it. Planner picks after reading the Form contract in spec/PROTOCOL.md — preference is (a) because it makes `Form action="…"` a viable submit path, not dead code.
- **D-G3: Replace hand-rolled `Component` literal in `contact.rs:1577-1584` with Button builder (Phase 14 IN-01).** Small mechanical swap — the existing Button builder supports the shape the literal constructs; no behavioral change.
- **D-G4: Fix `@types/node` gap by switching to `node:` prefix imports.** Rewrite `tests/helpers/schema-validator.ts:4-6` imports from `'fs'`/`'path'`/`'url'` to `'node:fs'`/`'node:path'`/`'node:url'` (Node 18+ native). Remove the `@ts-expect-error` suppressions in `frontend/tests/e2e/ci-guards.spec.ts:21-26` that exist for the same reason. Do NOT add `@types/node` to `devDependencies` — the `node:` prefix is the lighter fix and `svelte-check` respects it.

### Area H — UAT rigor + test coverage

- **D-H1: Chrome-MCP UAT per migrated screen.** Mirrors Phase 14's 14-uat-evidence pattern. One evidence folder per screen (`15-uat-evidence/company-edit/`, `user-edit/`, `interaction-edit/`, `contact-tag-add/`, `contact-note-add/`) with screenshots + JSON assertions + console logs. Scenario template reused from Phase 14 UAT driver; adapt per-screen field lists. User memory explicitly confirms Chrome-MCP is the preferred UAT mechanism (no walkthroughs handed to the user).
- **D-H2: New E2E spec per migrated screen.** Add `frontend/tests/e2e/company-edit.spec.ts`, `user-edit.spec.ts`, `interaction-edit.spec.ts`. Each covers: field rendering, FieldSet grouping visible, save flow, validation error display (exercises D-D1 per-field write-path). Inline tag-add / note-add forms get smoke coverage inside `contact-edit.spec.ts` (existing).
- **D-H3: Visual rebaseline for each migrated screen.** Extend `frontend/tests/visual/form.spec.ts` with new snapshot cases (`company-edit-form`, `user-edit-form`, `interaction-edit-form`) at desktop 1280×720 + mobile 375×800. Run with `--update-snapshots` to create the baseline; verify green on second run.
- **D-H4: Browser-tests for the new primitives stay as-is.** Phase 14 already covers TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch, FieldSet, FieldSeparator at the component level. No additions required.
- **D-H5: Known flakes stay flakes.** The 5 pre-existing `popup/ConfirmDialog.browser-test.ts` (4) + `popup/ToastSurface.browser-test.ts` (1) failures reproduce on `main` and remain deferred. Phase 15 verification treats them as pre-existing — same disposition as Phase 13/14.

### Area I — Claude's Discretion

Within Phase 15:

- Exact number of `FieldSet`s per form (e.g., company: one set or two) — base it on field count and semantic grouping. Preference: one `FieldSet` for ≤4 fields, two for 5+, mirror Phase 14's contact.rs pattern for consistency.
- Whether `form_shell()` takes a positional `action_row` parameter or reads it from the form's last child. Preference: positional, keeps the shape explicit.
- Exact column count for the new SeaORM migration's `down()` (one `drop_column` per field in reverse order).
- Specific class utility strings for any action row customization; follow Phase 14 D-D1 Option A (`Container class="flex gap-2 justify-end"`).
- Whether to split the scope-closure items (D-G1..D-G4) into their own plan or fold them into the sweep plans. Either is fine — they're each <30 lines of changes.
- UAT scenario mix per screen (Phase 14 used 6 scenarios for contact; Phase 15 can use 3-4 per screen since primitives are already proven).
- Whether to add a CONCEPT.md historical footnote about Flowbite (D-F2) or just delete all Flowbite references cleanly.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap / requirements

- `.planning/ROADMAP.md` §Phase 15 — goal, depends-on (Phase 13, Phase 14), success criteria (all 3 rows).
- `.planning/REQUIREMENTS.md` §Component Migration — COMP-03 ("CRM demo screens fully functional with new component implementations"); §Out of Scope — Superforms / Formsnap rejection (keeps server-side validation canonical).
- `.planning/PROJECT.md` — v1.1 milestone goal; §Key Decisions "Clean break from Flowbite".

### Prior phase decisions this phase inherits

- `.planning/phases/14-formscreen-enhancements/14-CONTEXT.md` — Phase 14 form composition. D-A1 (FormScreen retired), D-A2 (inline Container → Phase 15 revisits as D-B1), D-B1 (internal Field wrap), D-B3 (`.description(…)` builder helper), D-C1..C4 (FieldSet responsive grid + overrides), D-D1 (action row), D-E3/E4 (Textarea, RadioGroup, Switch installed).
- `.planning/phases/14-formscreen-enhancements/14-08-SUMMARY.md` §Known Stubs — country/notes/opt_in not persisted → D-C1..C3; §Phase 15 sections (full CRM migration, per-field validation, RadioGroup smoke) — directly inform D-A1, D-D1, D-E1.
- `.planning/phases/14-formscreen-enhancements/14-VERIFICATION.md` §Anti-Patterns — WR-01 (Form.svelte empty payload), WR-02 (validation doc drift), IN-01 (contact.rs Component literal), IN-02 (window.__mrnSetData gating).
- `.planning/phases/13-datatable-enhancements/13-CONTEXT.md` D-A2 — TableScreen retirement precedent; §Deferred "CRM-wide filter bar audit / consistency cleanup" → Phase 15 (but lists are already migrated; leave unless Flowbite residue found).
- `.planning/phases/13-datatable-enhancements/deferred-items.md` §NodeRenderer blur — resolved Phase 14 D-E2; §popup browser-test failures → stay deferred per D-H5.
- `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` §TextInput input_type — resolved Phase 14 D-E1; §clippy pedantic — stays deferred (D-H5 equivalent for Rust).

### Protocol specs that Phase 15 mutates

- `spec/PROTOCOL.md` §Validation errors — D-D2 deletes the legacy `/contactForm/errors` array section (~ll. 804-819) and keeps `/_errors/{bind}` as canonical (~ll. 593-600). Add one worked multi-field validation patch example.
- `spec/PROTOCOL.md` §Form — D-G2 may update Form dispatch semantics (payload shape on submit-action).
- `spec/schemas/data.yaml` — no new component types added. If D-D2 affects a schema, update accordingly.
- `spec/openapi.yaml` — regenerate only if a schema file changes.

### Code Phase 15 rewrites, adds, or extends

- **Rewritten** (CRM migration per D-A1):
  - `backend/crates/crm-demo/src/handlers/company.rs` — edit form + inline note-add.
  - `backend/crates/crm-demo/src/handlers/user.rs` — edit form.
  - `backend/crates/crm-demo/src/handlers/interaction.rs` — edit form (+ RadioGroup per D-E1).
  - `backend/crates/crm-demo/src/handlers/contact.rs` — inline tag-add + note-add; also refactor envelope to `form_shell()` per D-B2; persist country/notes/opt_in per D-C3; toast Component literal per D-G3.
- **Rewritten** (validation per D-D1):
  - Each of the above `handle_*_save` paths emits per-field `/_errors/{bind}` patches instead of `BadPayload`.
- **Added**:
  - `backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs` — three-column migration per D-C1.
  - `backend/crates/marionette/src/builders/standard.rs` — `form_shell()` helper per D-B1.
  - `backend/crates/marionette/src/error.rs` (or new `validation.rs`) — `validation_error_patch()` helper per D-D3.
  - `frontend/tests/e2e/company-edit.spec.ts`, `user-edit.spec.ts`, `interaction-edit.spec.ts` per D-H2.
  - New snapshot cases in `frontend/tests/visual/form.spec.ts` per D-H3.
  - `frontend/tests/e2e/ci-guards.spec.ts` — Flowbite residue grep per D-F1.
  - `.planning/phases/15-crm-migration-validation/15-uat-evidence/<screen>/*` per D-H1.
- **Edited**:
  - `backend/crates/crm-demo/src/entities/contact.rs` — Model struct fields per D-C2.
  - `backend/crates/crm-demo/src/seed.rs` — seed data for new columns per D-C2.
  - `backend/crates/crm-demo/src/migration/mod.rs` — register new migration.
  - `frontend/src/lib/init.ts:92-102` — gate `__mrnSetData` (and `__mrnSendAction`) behind `import.meta.env.DEV` per D-G1.
  - `frontend/src/lib/components/form/Form.svelte:29` — fix empty-payload sendAction per D-G2.
  - `frontend/tests/helpers/schema-validator.ts` — `node:` prefix imports per D-G4; remove `@ts-expect-error` in `ci-guards.spec.ts:21-26`.
  - `CONCEPT.md` (~ll. 260, 268, 630) per D-F2.
  - `TOOLING.md:39` per D-F2.
  - `.planning/codebase/STACK.md` frontend-stack section per D-F3.

### External library docs (only if new questions arise)

- https://shadcn-svelte.com/docs/components/radio-group — for D-E1's RadioGroup migration of interaction.type. Phase 14 14-06 already covers the primitive; this is a re-read if options/validation edge cases surface.
- https://docs.rs/sea-orm-migration/latest/sea_orm_migration/ — migration idioms for adding nullable columns on SQLite (D-C1).

### Codebase intel

- `.planning/codebase/CONVENTIONS.md` — Svelte 5 / Rust style.
- `.planning/codebase/STACK.md` — updated by D-F3 in this phase; read current state first.
- `.planning/codebase/TESTING.md` — browser-test / E2E / UAT patterns. Phase 14 `14-uat-evidence/` is the canonical UAT reference (D-H1).

### User preferences (from memory)

- `feedback_use_chrome_for_uat.md` — Chrome-MCP is the canonical UAT mechanism; don't hand walkthroughs to the user. → D-H1.
- `feedback_no_handrolling_ui.md` — adopt framework recipes over custom designs. → D-A3 (no new primitives), D-E1 (RadioGroup recipe).
- `feedback_pre_deployment_no_backcompat.md` — no migration shims, no tombstones, fix root causes. → D-C4 (honest down-migration), D-D2 (delete legacy validation section outright), D-G1..G4 (clean fixes, no fallback paths).
- `feedback_options_need_reasoning.md` — pros/cons/rationale for every option; check shadcn-svelte recipes first. → Reflected throughout; decisions cite the Phase 14 recipe rather than inventing shapes.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`backend/crates/marionette/src/builders/standard.rs`** — Phase 14's `FieldSet`, `FieldSeparator`, `Textarea`, `RadioGroup`, `Switch`, `Form`, `TextInput`, `SelectInput`, `Checkbox`, `Container`, `Heading`, `Button` builders. Phase 15 composes with them; `form_shell()` (D-B1) joins them.
- **`backend/crates/crm-demo/src/handlers/contact.rs:519-670`** — Phase 14's canonical composition for an edit form (3× `FieldSet`, 2× `FieldSeparator`, `Textarea`, `Switch`, action row). Every Phase 15 form migration is a structural copy of this shape with per-handler fields.
- **`backend/crates/crm-demo/src/handlers/contact.rs:1468-1550`** — Phase 12 D-A6 country-select node-patch demo. Stays untouched; the migration extends the schema so the country value round-trips to SQLite instead of being discarded on save.
- **`frontend/src/lib/components/form/{TextInput,SelectInput,Checkbox,Textarea,RadioGroup,Switch,FieldSet,FieldSeparator,Form}.svelte`** — all leaf components already wrap `Field.Field` internally (Phase 14 D-B1). No changes needed in frontend components for Phase 15.
- **`frontend/tests/uat/uat-driver.spec.ts`** + **`playwright.uat.config.ts`** — the Phase 14 UAT driver pattern. Phase 15 clones it per screen (D-H1), swapping field lists + assertions.
- **`frontend/tests/e2e/ci-guards.spec.ts`** — Phase 13's retired-file grep guard. Phase 15 extends it with Flowbite residue grep (D-F1).
- **`backend/crates/crm-demo/src/migration/m20260323_*.rs`** — existing SeaORM migration files. `m20260323_000004_create_contact.rs` is the base for D-C1's extension migration to follow.
- **`backend/crates/marionette/src/error.rs` `ActionError` + `ActionResult`** — existing error channel. Phase 15 extends the handler-response shape to include validation patches (D-D1/D-D3) without breaking ActionError's contract for protocol-layer failures.

### Established Patterns

- **Adjacency-list composition** — unchanged from Phase 14. `form_shell()` (D-B1) builds the same shape; handlers stay flat-children-first, grouped by FieldSet.
- **`#[derive(ComponentBuilder)]` fluent builders** — every builder uses this. `form_shell()` is a thin function helper (not a new builder) because it composes existing builders without introducing new props.
- **Validation via `/_errors/{bind}`** — frontend reads; Phase 15 makes the backend finally write to it (D-D1). The render path is proven (Phase 14 UAT-03).
- **SeaORM migration chain** — monotonic timestamp-prefixed modules, registered in `mod.rs`. Phase 15 appends one.
- **Per-surface node patches** — `PatchMessage.surface` (Phase 12 D-A3). Validation patches target the same surface as the form (default `main` for CRM).
- **Pre-deployment posture** — no back-compat shims (D-C4, D-D2, D-G1..G4). Consistent with PROJECT memory.
- **Chrome-MCP UAT evidence committed under `.planning/phases/XX/XX-uat-evidence/`** — Phase 14 precedent (D-H1).
- **CI guards as E2E file-existence asserts** — Phase 13 `ci-guards.spec.ts` pattern extended by D-F1.

### Integration Points

- **`backend/crates/crm-demo/src/migration/mod.rs`** — register `m20260418_000011_extend_contact` (D-C1).
- **`backend/crates/crm-demo/src/entities/contact.rs`** — append 3 fields to `Model` + `ActiveModel` (D-C2).
- **`backend/crates/crm-demo/src/seed.rs`** — populate new columns in the seed data.
- **`backend/crates/marionette/src/builders/standard.rs`** — export `form_shell()` (D-B1).
- **`backend/crates/marionette/src/error.rs` or `validation.rs`** — export `validation_error_patch()` (D-D3).
- **`frontend/src/lib/init.ts`** — dev-gate hooks (D-G1).
- **`frontend/src/lib/components/form/Form.svelte:29`** — submit-action payload wiring (D-G2).
- **`frontend/tests/e2e/ci-guards.spec.ts`** — Flowbite residue grep (D-F1).
- **`spec/PROTOCOL.md`** — delete legacy validation section, add worked example (D-D2).
- **`CONCEPT.md` + `TOOLING.md` + `.planning/codebase/STACK.md`** — Flowbite → shadcn-svelte (D-F2, D-F3).
- **Protocol version** — stays `"1.1.0"` (no protocol change in Phase 15).

</code_context>

<specifics>
## Specific Ideas

- **"The canonical Phase 14 composition is the template."** Every migrated form ends up structurally identical to `handlers/contact.rs:519-670` — the only variable is the field list per handler. When planning, use contact.rs as the working reference and compare each migrated handler back to it.
- **"Zero Flowbite residue — and make the CI prove it."** Runtime code is already clean. The CI guard (D-F1) is the load-bearing piece: it prevents a future handler from accidentally bringing Flowbite back (e.g., via a copied snippet from an old tutorial). The doc sweep (D-F2) is cosmetic but unlocks the guard semantically — once the guard runs, the docs stay consistent.
- **"Server-side validation is the canonical shape."** Per-field `/_errors/{bind}` patches (D-D1) make the Phase 14 Field.Error anatomy actually fire on bad submits. No Superforms, no client-side Zod — REQUIREMENTS.md §Out of Scope stands. The frontend was always ready; Phase 15 makes the backend match.
- **"RadioGroup gets its day."** Phase 14 shipped the primitive with zero CRM use. Phase 15 gives it two homes (interaction type per D-E1; user preferred_contact_method per D-E2) so the CRM demo exercises the full Field recipe. If interaction.type already works as a Select, migrating it to RadioGroup is additive polish — worth it because the demo's purpose is to showcase the full stack.
- **"`form_shell()` is a Rust helper, not a revived FormScreen."** Phase 14 was explicit about killing the frontend `FormScreen.svelte`. D-B1 doesn't walk that back — it just compresses the backend envelope boilerplate. The frontend sees the same adjacency-list shape either way.
- **"DB migration is unavoidable, make it small."** D-C1 adds exactly 3 columns to exactly 1 table. No cross-table refactors, no indexes (unless a query demands one — the columns are only read on the contact detail page, no new list filtering), no defaults that lie about the data model.
- **"Chrome-MCP UAT per screen, not a mega-session."** User memory says no walkthroughs. Phase 14 proved the per-screen Playwright/Chrome-MCP pattern scales — 5 screens × ~3 scenarios = 15 scenarios total, each with its own evidence folder. Parallelizable across plans.
- **"One plan per handler, one plan for the scope-closure bundle, one plan for the Flowbite/doc/CI pass, one plan for DB + validation helpers."** Planner's call — this is Claude's discretion per D-I — but a ~5-plan shape keeps each plan atomic and reviewable.

</specifics>

<deferred>
## Deferred Ideas

- **Persistence of `preferred_contact_method` on the user entity** — D-E2 ships the RadioGroup UI only. If future marketing or workflow features need the value, add a migration then.
- **Wizard / multi-step forms** (FORM-03) — v2.
- **Arbitrary per-field col-span / row-span beyond `full_width`** (FORM-04) — v2.
- **Persistent column visibility** (TABLE-03 persistence) — v2 (localStorage or server-state).
- **Row selection + bulk actions** (TABLE-07) — v2.
- **Empty-state illustrations** (TABLE-06) — v2; current "No results" text suffices.
- **Breadcrumbs, multiple sidebar variants, sidebar collapse persistence** (SHELL-05..07) — v2.
- **Superforms / Formsnap / client-side Zod validation** — REQUIREMENTS.md §Out of Scope; server-side `/_errors` is the canonical shape after Phase 15 D-D1.
- **crm-demo clippy pedantic sweep** — 76-86 warnings from toolchain drift, Phase 12 deferred-items. Mechanical; deserves its own plan outside the v1.1 milestone.
- **Popup browser-test fixes** — 5 failures (ConfirmDialog ×4, ToastSurface ×1). Phase 11/12/13 residue; dedicated popup-fix follow-up.
- **Login form migration to SDUI** — auth.rs is a REST endpoint, not an SDUI handler; migrating it is a protocol-surface question for a future phase if the login UX needs SDUI primitives.
- **`@types/node` adoption across frontend tsconfig** — only the minimum fix (D-G4 `node:` prefix) is in scope. Broader `@types/node` integration is a toolchain decision for v2.
- **DatePicker / Combobox / FileInput SDUI components** — not in Phase 15 scope (D-A3); add if a real handler needs one.
- **Milestone-close ceremony (PROJECT.md roll-forward, REQUIREMENTS.md Validated moves)** — `/gsd-complete-milestone` handles it after Phase 15 ships.

</deferred>

---

*Phase: 15-crm-migration-validation*
*Context gathered: 2026-04-18*
