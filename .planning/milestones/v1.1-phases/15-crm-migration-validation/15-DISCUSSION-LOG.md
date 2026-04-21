# Phase 15: CRM Migration & Validation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-18
**Phase:** 15-crm-migration-validation
**Mode:** `--auto` (all gray areas auto-selected; recommended defaults chosen)
**Areas discussed:** A — CRM form sweep scope; B — DRY helper; C — DB schema + contact field persistence; D — Per-field validation wiring; E — RadioGroup CRM demo; F — Flowbite residue + doc cleanup; G — Scope-closure cleanup (Phase 14 leftovers); H — UAT rigor + test coverage; I — Claude's Discretion

---

## Area A — CRM form migration scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full sweep | Migrate company.rs, user.rs, interaction.rs, inline tag-add, inline note-add (both contact.rs and company.rs) | ✓ |
| Partial sweep | Only user.rs + company.rs edit forms; inline forms deferred | |
| Minimal sweep | Swap components in place; no FieldSet restructure | |
| Include login migration | Move auth.rs login from REST endpoint to SDUI form | |

**Auto-selection:** "Full sweep" — Phase 14 D-A2 and 14-08 SUMMARY explicitly name these handlers as Phase 15 scope. Login stays out (D-A2 in CONTEXT.md) because it's a REST endpoint, not an SDUI form; migrating it is a protocol-surface decision unrelated to "CRM migration complete".
**Rationale:** Phase 15's success criterion #1 is "All CRM screens render and function correctly" — leaving any handler on the old shape breaks that. The inline tag-add + note-add forms are cheap to migrate once the pattern is set; including them now prevents drift where the main forms use FieldSet and inline widgets don't.

---

## Area B — DRY helper (form_shell)

| Option | Description | Selected |
|--------|-------------|----------|
| Introduce `form_shell()` Rust helper | Thin backend builder helper: `form_shell(heading, back_action, form, descendants)` | ✓ |
| Keep inline composition | Each handler repeats `Container([Heading, Button, Form, …])` verbatim | |
| Revive frontend FormScreen | Re-add `FormScreen.svelte` as a first-class SDUI component | |
| Fluent builder method | Add `.shell(heading, back)` method to `Form` builder | |

**Auto-selection:** "Introduce `form_shell()` Rust helper" — Phase 14 D-A2 explicitly set the threshold: "Phase 15 can introduce a Rust-side helper if the repetition becomes painful across ≥3 screens." Phase 15 has 4 form handlers repeating the same envelope.
**Rationale:** Rust-side only keeps the SDUI protocol unchanged. A function helper (not a builder) avoids coupling — handlers that need a custom envelope compose by hand. Reviving `FormScreen.svelte` contradicts Phase 14 D-A1 (hard deletion). A fluent builder method ties the shape to `Form` which is a `<form>` boundary abstraction, not a screen abstraction — wrong level.

---

## Area C — DB schema + contact field persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Add 3 columns (country, notes, opt_in) via new migration | SeaORM `m20260418_000011_extend_contact.rs` + entity + seed + save handler | ✓ |
| Drop the 3 form fields | Remove country select, notes textarea, opt_in switch from contact.rs | |
| JSON blob column | One `contact_metadata JSON` column holding all three fields | |
| Defer to v2 | Keep form rendering; don't persist (status quo) | |

**Auto-selection:** "Add 3 columns" — Phase 14 14-08 SUMMARY Known Stubs explicitly deferred this to Phase 15 ("DB migration explicitly deferred to Phase 15 to avoid scope creep"). Dropping the fields would walk back a Phase 14 UI decision. A JSON blob contradicts the existing schema style (every contact attribute has its own column). Deferring to v2 leaves a stub visible to demo viewers.
**Rationale:** Three `NULL`-able columns on one table is the smallest possible change. Pre-deployment posture (no migration concerns) means down-migrations can be honest. Mirrors existing `m20260323_*` convention.

---

## Area D — Per-field validation wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Emit per-field `SetData` patches to `/_errors/{bind}` + `validation_error_patch()` helper | Save handlers build a PatchMessage with one op per bad field | ✓ |
| New `ActionError::FieldErrors(Vec<(path, msg)>)` variant | Error channel carries per-field info; dispatcher splits it | |
| Keep form-level `BadPayload` toast | Status quo; Field.Error never fires | |
| Server-side Superforms-like schema | Wrap a JSON schema validator around each handler | |

**Auto-selection:** "Emit per-field SetData patches + helper" — Phase 14 UAT-03 rewrite + 14-08 SUMMARY explicitly name this as Phase 15 scope. The Field.Error render path is proven; only the write-path is missing.
**Rationale:** Using existing `PatchMessage` / `SetData` infrastructure means no protocol change. A new `ActionError` variant would drag dispatcher changes and split the error channel in two awkward ways. Keeping `BadPayload` leaves a visible UAT stub (14-08 SUMMARY's UAT-03b confirms the backend currently does the wrong thing). Server-side schema validation contradicts REQUIREMENTS.md §Out of Scope (Superforms/Formsnap rejection applies to both sides of the fence — simple `/_errors` patches from handler logic is the canonical pattern).

---

## Area E — RadioGroup CRM demo placement

| Option | Description | Selected |
|--------|-------------|----------|
| Interaction type + user preferred_contact_method | Two RadioGroup uses; interaction type persists, user preference is UI-only | ✓ |
| Interaction type only | One migration; no user-form RadioGroup | |
| User role as RadioGroup | Migrate user.role from Select to RadioGroup | |
| Leave RadioGroup unused in CRM | Primitive ships via browser-tests only | |

**Auto-selection:** "Interaction type + user preferred_contact_method" — interaction type is the natural fit (small fixed set, mutually exclusive, matches RadioGroup semantics). Adding a second use demonstrates the primitive outside a single handler. `preferred_contact_method` needs no DB migration (form-only).
**Rationale:** 14-08 SUMMARY §"Phase 15 (RadioGroup smoke)" explicitly suggests `preferred_contact_method`. Migrating user.role carries risk (role is a permissions boundary and changes affect auth checks) — wrong home for a UI demo. Leaving RadioGroup unused in CRM means the demo doesn't exercise it end-to-end, undercutting the "full stack showcase" purpose.

---

## Area F — Flowbite residue + doc cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| CI grep guard + update CONCEPT.md/TOOLING.md/STACK.md | Lock clean break in CI; update user-facing docs to shadcn-svelte | ✓ |
| CI grep guard only | Leave docs' historical Flowbite mentions; just guard runtime code | |
| Update docs only | No CI guard | |
| Leave as-is | Code is clean; docs are "historical" | |

**Auto-selection:** "CI grep guard + doc updates" — Success Criterion #2 says "Zero Flowbite references remain anywhere in the codebase". Runtime is already clean but the user-facing docs (CONCEPT.md, TOOLING.md) still describe Flowbite as the frontend vocabulary. Without the CI guard, a future handler can accidentally regress (e.g., copy a tutorial snippet).
**Rationale:** Pre-deployment posture favors complete cleanup, no half-updates. The grep guard is cheap (one file-read + regex) and catches drift in PRs before review. One historical footnote in CONCEPT.md is fine for context; scattered mentions in TOOLING.md and STACK.md aren't.

---

## Area G — Scope-closure cleanup (Phase 14 leftovers)

| Option | Description | Selected |
|--------|-------------|----------|
| Include G1-G4 in Phase 15 | Gate `__mrnSetData` (IN-02) + fix Form.svelte empty-payload (WR-01) + replace Component literal (IN-01) + `node:` prefix imports | ✓ |
| G1 + G2 only | Skip the low-severity IN-01 + the toolchain fix G4 | |
| Defer all | Pure CRM sweep; leave leftovers for follow-ups | |

**Auto-selection:** "Include G1-G4" — all four are small (<30 lines each), all are flagged in Phase 14 REVIEW/VERIFICATION, all block a clean "v1.1 milestone closes here" story. Bundling them with the CRM sweep means one coherent Phase 15 close-out instead of three separate follow-ups.
**Rationale:** Pre-deployment posture rejects lingering `#[allow(dead_code)]` / `// TODO Phase 15` attributes; these fixes remove them. IN-02 (production-leaked test hook) is a real security/hygiene concern — shouldn't wait. G4 (`node:` prefix) lets us delete 3 `@ts-expect-error` suppressions in ci-guards.spec.ts, cleaner code.

---

## Area H — UAT rigor + test coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Chrome-MCP UAT per migrated screen + E2E spec per screen + visual rebaseline | Full Phase 14 pattern, 5 screens | ✓ |
| Sampled UAT (contact + one other) + E2E per screen | Fewer UAT folders; same E2E coverage | |
| Existing E2E suite only | No new specs; rely on Phase 13/14 coverage | |
| Manual walkthrough handed to user | No automation; user verifies | |

**Auto-selection:** "Per-screen Chrome-MCP UAT + E2E + visual rebaseline" — user memory explicitly says Chrome-MCP is the UAT mechanism and no walkthroughs handed to the user. Phase 14 proved the per-screen pattern scales.
**Rationale:** The sampled approach creates blind spots in migration-parity — every screen has slightly different field-count and grouping and needs its own visual baseline. Existing E2E suite only covers contact; the whole point of Phase 15 is the other four screens. Manual walkthroughs violate `feedback_use_chrome_for_uat.md`.

---

## Area I — Claude's Discretion (items explicitly deferred to planning)

Listed in CONTEXT.md `<decisions>` §Area I. No option matrix — these are resolved during `/gsd-plan-phase` based on task decomposition. Examples: exact `FieldSet` count per form (one for ≤4 fields, two for 5+), whether scope-closure items ride in their own plan or fold into the sweep plans, UAT scenario mix per screen.

---

## Scope-creep items redirected to Deferred Ideas

- Persistence of `preferred_contact_method` on user entity → v2 (no marketing/workflow features demand it yet).
- Wizard forms, arbitrary col-span, persistent column visibility, row selection, empty-state illustrations, breadcrumbs, multiple sidebar variants → v2 (REQUIREMENTS.md §v2).
- Superforms/Formsnap/client-side Zod → Out of Scope (REQUIREMENTS.md §Out of Scope).
- crm-demo clippy pedantic sweep → dedicated follow-up plan outside v1.1.
- Popup browser-test failures → dedicated popup-fix follow-up.
- Login form migration to SDUI → future phase if login UX demands SDUI primitives.
- `@types/node` broader adoption → v2 toolchain decision; only `node:` prefix fix is in Phase 15.
- DatePicker / Combobox / FileInput SDUI components → add when a handler needs one.

---

## Claude's Discretion

- Number of `FieldSet`s per handler (field-count based).
- Whether `form_shell()` takes action_row positionally or reads it from Form's last child.
- Whether scope-closure items (D-G1..G4) are their own plan or folded.
- UAT scenario mix per screen (3-4 suggested; Phase 14 used 6 for contact).
- Whether to include a historical Flowbite footnote in CONCEPT.md (D-F2).
- Exact SeaORM migration down() ordering.

---

*Audit log finalized: 2026-04-18*
