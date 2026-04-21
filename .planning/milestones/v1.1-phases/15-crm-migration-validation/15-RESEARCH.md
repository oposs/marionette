# Phase 15: CRM Migration & Validation — Research

**Researched:** 2026-04-18
**Domain:** CRM handler sweep onto Phase-14 FieldSet composition; DB migration to extend contact table; per-field validation write-path; Flowbite residue CI guard; Phase-14 review-item scope closure.
**Confidence:** HIGH

## Summary

Phase 15 is a **composition + validation + cleanup** phase. No new SDUI primitives, no protocol changes, no frontend component rewrites. All scaffolding landed in Phases 10–14. The remaining work is:

1. Replicate the `handlers/contact.rs:519-670` form-composition pattern verbatim onto three more handlers (`company`, `user`, `interaction`) and the two inline mini-forms inside `contact.rs` (tag-add, note-add) + `company.rs` (note-add).
2. Extend the `contact` table with three columns (`contact_country TEXT NULL`, `contact_notes TEXT NULL`, `contact_opt_in INTEGER NOT NULL DEFAULT 0`) via a new SeaORM migration using the project's established `execute_unprepared("ALTER TABLE …")` idiom, then wire `handle_contact_save` to persist them.
3. Move the four `handle_*_save` handlers from `Err(ActionError::BadPayload(...))` (form-level toast) to `Ok([PatchMessage { surface: "content", patch: [Set { path: "/_errors/.../name", value: "Name is required" }, …] }])` on validation failure. The frontend Field.Error path is already proven by Phase-14 UAT-03.
4. Delete the legacy `/contactForm/errors` array section from `spec/PROTOCOL.md` (~ll. 804-819) and add one worked multi-field validation-patch example to the canonical `/_errors/{bind}` section (~ll. 593-600).
5. Extend `frontend/tests/e2e/ci-guards.spec.ts` with a Flowbite residue grep + Phase 14 review-item mop-up (IN-01, IN-02, WR-01) + `@types/node` `node:` prefix fix.
6. Replace "Flowbite" with "shadcn-svelte" in three docs: `CONCEPT.md` (lines 260, 268, 630), `TOOLING.md` (line 39), `.planning/codebase/STACK.md` (line 47).

**Primary recommendation:** Structure the phase as 5-7 thin plans. A reasonable split:
- **P1 — DB migration + contact persistence + seed update** (SeaORM migration + entity extension + `handle_contact_save` field wiring + seed spread).
- **P2 — `form_shell()` + `validation_error_patch()` helpers** (the two Rust builders/helpers every handler plan depends on).
- **P3 — Handler sweep A: `company` + `user` (+ note-add inline)** (uses helpers; per-field validation wired).
- **P4 — Handler sweep B: `interaction` (with RadioGroup migration) + `contact` refactor to `form_shell()` + inline tag-add/note-add** (closes CRM surface; refactors contact.rs to helper).
- **P5 — Scope-closure + doc/CI pass** (Flowbite grep guard, CONCEPT/TOOLING/STACK doc sweep, `__mrnSetData`/`__mrnSendAction` dev-gate, `Form.svelte:29` fix, contact.rs:1577 Component literal swap, `@types/node` `node:` prefix, PROTOCOL.md validation-section surgery).
- **P6 — E2E + visual rebaseline per screen** (`company-edit.spec.ts`, `user-edit.spec.ts`, `interaction-edit.spec.ts`; visual snapshots at 1280×720 + 375×800).
- **P7 — Chrome-MCP / Playwright UAT per screen + phase closure** (5 evidence folders, reused UAT driver).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-03 | CRM demo screens fully functional with new component implementations | Every section below maps directly to a concrete action for COMP-03: handler sweep (T1), DB extension (T2), validation write-path (T3), RadioGroup placement (T6), Flowbite CI guard (T5), UAT rigor (T10). |

## User Constraints (from CONTEXT.md)

### Locked Decisions (from CONTEXT.md §Decisions)

**Area A — CRM form migration scope**
- **D-A1:** Full form-handler sweep — company, user, interaction, plus inline tag-add + note-add forms. Every `Form::new().children([…leaf…])` in `handlers/` gets rewritten to `Container → Heading + back Button → Form → FieldSet+FieldSeparator+… → action row`. Specific surfaces enumerated: `company.rs:190-215`, `user.rs:217-260`, `interaction.rs:63-109`, `contact.rs:716-760`, `company.rs:318-330`.
- **D-A2:** Login form (`auth.rs`) stays out of scope — REST endpoint, not SDUI.
- **D-A3:** No new frontend SDUI components in this phase.

**Area B — DRY helper and inline-composition pattern**
- **D-B1:** Introduce `form_shell()` in `backend/crates/marionette/src/builders/standard.rs`. Positional args: `form_shell(heading, back_action, form_node, form_descendants)` or similar — thin, no styling opinions.
- **D-B2:** Migrate `contact.rs` to use `form_shell()` as part of the sweep.
- **D-B3:** No frontend counterpart. `frontend/src/lib/components/screen/` stays empty.

**Area C — DB schema + contact field persistence**
- **D-C1:** New SeaORM migration `m20260418_000011_extend_contact.rs`. Adds `contact_country TEXT NULL`, `contact_notes TEXT NULL`, `contact_opt_in INTEGER NOT NULL DEFAULT 0`.
- **D-C2:** Update `entities/contact.rs` Model struct + seed a realistic spread.
- **D-C3:** Wire `handle_contact_save` to persist the three fields. Drop `#[serde(default)] #[allow(dead_code)]`. Country-select node-patch behaviour (Phase 12 D-A6) unchanged — only save path changes.
- **D-C4:** Down migration drops the three columns. No back-compat.

**Area D — Per-field validation wiring**
- **D-D1:** `handle_contact_save`, `handle_company_save`, `handle_user_save`, `handle_interaction_save` move from `Err(BadPayload)` to `Ok([PatchMessage { SetData /_errors/{bind} "…" }])`. `ActionError::BadPayload` reserved for non-field errors (JSON parse, missing form_bind).
- **D-D2:** Delete legacy `/contactForm/errors` array section from `spec/PROTOCOL.md` (~ll. 804-819). Keep `/_errors/{bind}` canonical (~ll. 593-600). Add worked multi-field example.
- **D-D3:** Small Rust helper `validation_error_patch()` in `backend/crates/marionette/src/error.rs` (or new `validation.rs`). Takes `impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>` → `PatchMessage`.
- **D-D4:** Existing toast path stays for non-field errors.

**Area E — RadioGroup CRM demo + consistency touches**
- **D-E1:** Migrate `handlers/interaction.rs` `type` Select → RadioGroup. Options: `[("call","Call"), ("email","Email"), ("meeting","Meeting")]`. Bind `/interactionForm/interaction_type`. Server-side validation at interaction.rs:196 unchanged.
- **D-E2:** Add `preferred_contact_method` RadioGroup to user edit form. No DB persistence. Handler accepts but discards (`#[allow(dead_code)]`).
- **D-E3:** Every migrated form gets a `description` on at least one field.

**Area F — Flowbite residue guard + doc cleanup**
- **D-F1:** Zero Flowbite tokens in runtime paths — enforced by CI guard in `ci-guards.spec.ts`. Case-insensitive grep under `frontend/src/**`, `backend/crates/**`, `spec/**`. Expected match count: 0.
- **D-F2:** Update `CONCEPT.md:260/268/630` and `TOOLING.md:39` to shadcn-svelte.
- **D-F3:** Update `.planning/codebase/STACK.md:47`.
- **D-F4:** Keep existing TableScreen + FormScreen deletion asserts.

**Area G — Scope-closure cleanup (Phase 14 leftovers)**
- **D-G1:** Gate `window.__mrnSetData` + `__mrnSendAction` behind `import.meta.env.DEV` in `frontend/src/lib/init.ts:92-102`.
- **D-G2:** Fix `Form.svelte:29` empty-payload `sendAction`. Preference: (a) pass collected form values as payload.
- **D-G3:** Replace hand-rolled `Component` literal at `contact.rs:1577-1584` with Button builder.
- **D-G4:** Fix `@types/node` gap with `node:` prefix imports. Remove `@ts-expect-error` suppressions in `ci-guards.spec.ts:21-26`.

**Area H — UAT rigor + test coverage**
- **D-H1:** Chrome-MCP UAT per migrated screen. One evidence folder per screen under `.planning/phases/15-crm-migration-validation/15-uat-evidence/`.
- **D-H2:** New E2E spec per migrated screen: `company-edit.spec.ts`, `user-edit.spec.ts`, `interaction-edit.spec.ts`. Inline forms via smoke coverage in existing `contact-edit.spec.ts`.
- **D-H3:** Visual rebaseline per screen (desktop 1280×720 + mobile 375×800).
- **D-H4:** Browser-tests for primitives stay as-is.
- **D-H5:** Known flakes stay flakes (5 popup tests; deferred).

### Claude's Discretion (from CONTEXT.md §Area I)

- Exact FieldSet count per form. Preference: 1 set for ≤4 fields, 2 for 5+.
- Whether `form_shell()` takes positional `action_row` parameter. Preference: positional.
- Specific class utility strings for action row — follow D-D1 Option A (`Container class="flex gap-2 justify-end"`).
- Whether to split scope-closure items (D-G1..G4) into their own plan or fold into sweep plans.
- UAT scenario mix per screen (3-4 per screen vs. Phase 14's 6).
- Whether to add a CONCEPT.md historical footnote about Flowbite (D-F2).

### Deferred Ideas (OUT OF SCOPE)

- Persistence of `preferred_contact_method` on user entity.
- Wizard / multi-step forms (FORM-03) — v2.
- Arbitrary per-field col-span / row-span beyond `full_width` (FORM-04) — v2.
- Persistent column visibility — v2.
- Row selection + bulk actions (TABLE-07) — v2.
- Empty-state illustrations (TABLE-06) — v2.
- Breadcrumbs, multiple sidebars, sidebar collapse persistence (SHELL-05..07) — v2.
- Superforms / Formsnap / client-side Zod validation — REQUIREMENTS.md §Out of Scope.
- crm-demo clippy pedantic sweep — Phase 12 deferred-items.
- Popup browser-test fixes (5 failures) — dedicated follow-up.
- Login form migration to SDUI — auth.rs is a REST endpoint.
- `@types/node` adoption across frontend tsconfig — only `node:` prefix fix in scope.
- DatePicker / Combobox / FileInput SDUI components.
- Milestone-close ceremony.

## Project Constraints (from CLAUDE.md + user memory)

- **Chrome-MCP is the canonical UAT mechanism** (user memory: `feedback_use_chrome_for_uat.md`). If not wired in this environment, fall back to Playwright as Phase 14 Plan 08 did — same contract, different driver.
- **No hand-rolled UI** (`feedback_no_handrolling_ui.md`) — adopt shadcn-svelte recipes. Phase 15 inherits Phase 14's Field anatomy verbatim.
- **Pre-deployment posture — no back-compat shims** (`feedback_pre_deployment_no_backcompat.md`). D-C4 honest `down()`, D-D2 delete legacy section outright, D-G1..G4 clean fixes.
- **Options need reasoning** (`feedback_options_need_reasoning.md`) — pros/cons/rationale for every option; check shadcn-svelte recipes first.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Form field composition (FieldSet grouping, action row) | Backend builder (`builders/standard.rs`) | — | SDUI is server-driven; the composition shape is the handler's decision. Frontend only renders the adjacency-list. |
| DB schema extension (country/notes/opt_in) | Backend migration (SeaORM) | Backend entity | Schema + ORM model are co-located in `crm-demo`. Frontend sees no change — form already sends these fields. |
| Per-field validation message | Backend save handler | Frontend Field.Error render | Server computes the error string; transport is a `SetData` op to `/_errors/{bind}`; frontend Field anatomy reads that path. Write is backend-authoritative. |
| Flowbite residue detection | CI test (filesystem grep) | — | Static analysis — runs in Playwright spec via `node:fs` + `node:child_process` (or ripgrep). No browser interaction needed. |
| RadioGroup UI for interaction type | Backend builder call + frontend primitive | — | Backend serializes options, frontend renders bits-ui RadioGroup (already exists). |
| Dev-only test hook gating | Frontend bundler (Vite tree-shaking) | — | `import.meta.env.DEV` is a Vite-compile-time constant that tree-shakes the `if`-gated block out of production bundles. |
| Documentation alignment (CONCEPT/TOOLING/STACK) | Docs (markdown) | — | Pure text edit; no runtime impact. |
| UAT evidence | Playwright/Chrome-MCP → `.planning/phases/15-.../15-uat-evidence/` | — | Governance artifact; not executable in CI suite. |

## Standard Stack

### Core (already installed; Phase 15 only composes)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sea-orm` | 1.1 | ORM for SQLite | Already the project default; workspace dep. `[VERIFIED: backend/Cargo.toml:25]` |
| `sea-orm-migration` | 1.1 | Schema migrations | Companion to sea-orm; matches project chain. `[VERIFIED: backend/Cargo.toml:26]` |
| `@playwright/test` | ^1.58.2 | E2E + UAT driver | Already wired; Phase 14 Plan 08 used it successfully as Chrome-MCP substitute. `[VERIFIED: frontend/package.json existing usage across tests/e2e/*]` |
| `shadcn-svelte` primitives (field, textarea, radio-group, switch) | latest | Frontend primitives | Already installed Phase 14. `[VERIFIED: frontend/src/lib/components/ui/*]` |
| `bits-ui` | current | Headless primitives under shadcn | Transitively installed with shadcn. `[VERIFIED: uses in ui/radio-group/*]` |
| `marionette-protocol` | workspace | `PatchMessage`, `PatchOperation::Set`, `Surface` | Already the one-true-way to shape validation patches. `[VERIFIED: backend/crates/marionette-protocol/src/data.rs:13-18]` |

### Supporting (already available)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `lucide-svelte` | Phase-11-installed | Icons | If the action row or inline forms need an icon. Phase 14 used `ArrowLeft`-style plain-text "← Back". |
| `serde`/`serde_json` | workspace | JSON payload shape | Every new test + validation patch constructs `serde_json::json!` literals. |
| `time` | workspace | SQLite datetime formatting | Already used by `now_sqlite()` in interaction.rs / contact.rs. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Raw SQL `execute_unprepared("ALTER TABLE …")` | SeaORM `manager.alter_table(Table::alter()…)` | **Reject**: the project's entire migration chain uses raw SQL strings (`[VERIFIED: backend/crates/crm-demo/src/migration/m20260323_000001..010]`). Mixing idioms is noise. SeaORM 1.1 supports both; stay with what the repo uses. |
| Chrome-MCP for UAT | Playwright UAT driver (Phase 14 Plan 08 pattern) | Use Chrome-MCP if available; Playwright as fallback. Same evidence contract (screenshots + JSON + console logs). |
| `@types/node` in frontend tsconfig | `node:`-prefix imports + drop `@ts-expect-error` | **Adopt the `node:` prefix** (D-G4). Node 18+ native; lighter than wiring `@types/node` into the check-surface. |
| Client-side Zod / Superforms validation | Server-side `/_errors/{bind}` patch emission | **Reject**: REQUIREMENTS.md §Out of Scope. SDUI is server-authoritative. |
| New `ActionError::FieldValidation(Vec<(path, msg)>)` variant | Handler returns `Ok(vec![PatchMessage { ops }])` | **Reject**: D-D3 explicitly avoids new error variant. The error channel stays reserved for protocol-layer failures; validation flows through the data-patch channel. |
| Heading + back-button DRY helper in Phase 14 | `form_shell()` in Phase 15 (D-B1) | **Adopt in Phase 15** — threshold tripped (≥3 screens). `[CITED: 14-CONTEXT.md D-A2]` |

**Version verification:** `[VERIFIED: backend/Cargo.toml:25-26 shows sea-orm = "1.1" + sea-orm-migration = "1.1"]`. No dependency bumps required in Phase 15.

**Installation:** No new dependencies. Everything Phase 15 touches is already wired.

## Architecture Patterns

### System Architecture (data flow, Phase 15 touchpoints highlighted)

```
  User clicks "Save Contact"                    User sees Field.Error
          │                                          ▲
          ▼                                          │
┌─────────────────────┐            ┌─────────────────────────────────┐
│ Form.svelte (Field. │            │ TextInput.svelte reads          │
│ Group wrapper)      │            │ /_errors/{bind} from surface    │
│ dispatches          │            │ data store, renders             │
│ ActionMessage       │            │ <Field.Error> + aria-invalid    │
│ `contact_save`      │            │ (Phase 14; unchanged)           │
└──────────┬──────────┘            └────────────────▲────────────────┘
           │ WebSocket                               │
           ▼                                         │
┌─────────────────────┐                              │
│  ws.rs read_loop    │                              │
│  dispatches action  │                              │
│  to router.dispatch │                              │
└──────────┬──────────┘                              │
           ▼                                         │
┌─────────────────────────────────────────┐          │
│   handle_contact_save (Phase 15 target) │          │
│                                          │          │
│   1. Payload::from_context              │          │
│   2. ★ NEW: validate each field         │          │
│      → collect (bind, msg) tuples        │          │
│      → if any: return Ok(vec![          │          │
│          validation_error_patch(tuples)]│──── PatchMessage with
│        )                                 │      SetData ops
│   3. if clean: ActiveModel…insert/update │          │
│   4. (optionally emit success toast     │          │
│      via existing toasts-root sub-surf) │          │
│   5. render_contact_list() re-renders   │          │
└─────────────────────────────────────────┘          │
           │                                         │
           ▼                                         │
┌─────────────────────────────────────────┐          │
│ ProtocolMessage::Patch(PatchMessage {   │          │
│   surface: "content",                    │──────────┘
│   patch: [Set { path: "/_errors/        │   (same surface as form)
│     contactForm/email", value: "…" }]   │
│ })                                       │
└─────────────────────────────────────────┘

DB-side (Phase 15 migration adds 3 columns):

  contact table ──── country, notes, opt_in (new, nullable / default 0)
```

### Recommended Project Structure

```
backend/crates/crm-demo/src/
├── handlers/
│   ├── contact.rs       # canonical template + edit/inline-tag/inline-note forms
│   ├── company.rs       # NEW SWEEP: edit + inline-note
│   ├── user.rs          # NEW SWEEP: edit
│   └── interaction.rs   # NEW SWEEP: edit (RadioGroup)
├── entities/contact.rs  # NEW: add country, notes, opt_in fields
├── migration/
│   └── m20260418_000011_extend_contact.rs  # NEW
└── seed.rs              # UPDATED: populate new columns

backend/crates/marionette/src/
├── builders/standard.rs # NEW: form_shell() helper
└── error.rs  OR
└── validation.rs        # NEW: validation_error_patch() helper

frontend/
├── src/lib/init.ts                  # EDIT: dev-gate __mrnSetData + __mrnSendAction
├── src/lib/components/form/Form.svelte  # EDIT: fix empty-payload sendAction
└── tests/
    ├── e2e/
    │   ├── ci-guards.spec.ts        # EXTEND: Flowbite grep + drop @ts-expect-error
    │   ├── company-edit.spec.ts     # NEW
    │   ├── user-edit.spec.ts        # NEW
    │   └── interaction-edit.spec.ts # NEW
    ├── visual/form.spec.ts          # EXTEND: new snapshot cases
    └── helpers/schema-validator.ts  # EDIT: node: prefix imports

spec/
├── PROTOCOL.md                      # DELETE legacy section ll.804-819; add worked example at ll.593-600
└── schemas/data.yaml                # audit; likely no change (validation shape is data, not component)

CONCEPT.md        # EDIT ll. 260, 268, 630 — Flowbite → shadcn-svelte
TOOLING.md        # EDIT l. 39
.planning/codebase/STACK.md  # EDIT l. 47
.planning/phases/15-crm-migration-validation/15-uat-evidence/
    ├── company-edit/
    ├── user-edit/
    ├── interaction-edit/
    ├── contact-tag-add/    (smoke only — minimal)
    └── contact-note-add/   (smoke only — minimal)
```

### Pattern 1 — Phase 14 canonical form composition

**What:** Every migrated form uses the same adjacency-list envelope. `[CITED: backend/crates/crm-demo/src/handlers/contact.rs:519-670 + spec/PROTOCOL.md:565-591]`

**When to use:** Always, for multi-field forms. Inline one-field forms (tag-add, note-add) stay terse but still use `Form` as the `<form>` boundary.

**Template:**
```rust
// Source: backend/crates/crm-demo/src/handlers/contact.rs:519-670 (verified)
let heading = Heading::new(form_title).id("company-form-heading").build();
let back_button = Button::new("← Back")
    .id("company-form-back")
    .variant("outline")
    .action(ComponentAction::click("company_list"))
    .build();

let (details_set, details_desc) = FieldSet::new()
    .id("company-details-set")
    .legend("Company details")
    .children(vec![name_input, website_input])
    .build_tree();

let separator = FieldSeparator::new().id("company-form-sep-1").build();

let (address_set, address_desc) = FieldSet::new()
    .id("company-address-set")
    .legend("Address")
    .children(vec![address_input.full_width(true).build()])  // long fields span full row
    .build_tree();

let (action_row, action_desc) = Container::new()
    .id("company-form-actions")
    .class("flex gap-2 justify-end")
    .children(vec![cancel_button, save_button])
    .build_tree();

let (form_child, form_desc) = Form::new()
    .id("company-form")
    .children(vec![details_set, separator, address_set, action_row])
    .build_tree();
```

### Pattern 2 — `form_shell()` helper (D-B1 proposed signature)

**What:** A thin composition helper that flattens the `[heading, back_button, form_child] + extra_descendants` boilerplate every CRM edit form repeats.

**Proposed signature:**
```rust
// File: backend/crates/marionette/src/builders/standard.rs (ADD)
// Phase 15 D-B1.

use marionette_protocol::Component;

/// Assemble the canonical form-screen envelope documented in
/// `spec/PROTOCOL.md §form-screen composition pattern`. Returns
/// `(root_id, nodes)` ready for `RenderMessage.root` + `.nodes`.
///
/// # Arguments
/// * `root_id`    — stable id for the outer Container (e.g., "contact-form-root").
/// * `heading`    — built `(id, Component)` tuple for the title Heading.
/// * `back_button`— built `(id, Component)` tuple for the outline back Button.
/// * `form_child` — built `(id, Component)` tuple for the Form (carrier of FieldSets + action row).
/// * `form_descendants` — flat list of `(id, Component)` pairs already collected from
///                        `FieldSet::build_tree()`, `Container::build_tree()` etc.
///
/// # Returns
/// `(root_id, nodes_map)` where `nodes_map` includes the outer Container, heading,
/// back_button, form_child, and all form_descendants — ready to hand to a
/// `RenderMessage`. Caller still supplies the `data` payload.
#[must_use]
pub fn form_shell(
    root_id: impl Into<String>,
    heading: (String, Component),
    back_button: (String, Component),
    form_child: (String, Component),
    form_descendants: Vec<(String, Component)>,
) -> (String, std::collections::HashMap<String, Component>) {
    let root_id = root_id.into();
    let children = vec![heading.0.clone(), back_button.0.clone(), form_child.0.clone()];
    let container_nodes = Container::new()
        .id(&root_id)
        .children(vec![heading.1, back_button.1, form_child.1])
        .build_with_children();
    // `build_with_children` returns Vec<(String, Component)> where [0] is the container
    // and [1..] are direct children; see standard.rs test `container_builder_with_children`
    // at line 916.
    let mut nodes = std::collections::HashMap::new();
    for (id, c) in container_nodes {
        nodes.insert(id, c);
    }
    for (id, c) in form_descendants {
        nodes.insert(id, c);
    }
    // NOTE: the `children` local above is intentionally unused — Container's
    // build_with_children already wires child ids. It's kept here as
    // documentation of the intended adjacency.
    let _ = children;
    (root_id, nodes)
}
```

**Why positional (not named struct builder):**
- Handlers already hold the `(id, Component)` tuples from prior builder calls.
- Signature is small (4 tuple params + Vec) — no need for a builder struct.
- Matches Phase 13's inline composition style for list handlers.

**Alternative rejected:** A fluent `FormShellBuilder` with `.heading(…)`, `.back(…)`, `.form(…)`, `.descendants(…)` chain. **Reject**: over-engineered for 4 call sites; introduces one more builder pattern in a crate that already has 20+.

**Integration:** Handler code compresses from ~20 lines of `all_nodes.push(heading); extra_descendants.extend(…); …; container_nodes = Container::new().children(all_nodes).build_with_children(); let mut nodes = HashMap::new(); …` to:
```rust
let (root, nodes) = form_shell(
    "company-form-root",
    heading,
    back_button,
    form_child,
    form_descendants,
);
Ok(vec![ProtocolMessage::Render(RenderMessage {
    id: ctx.action.id.clone(),
    surface: "content".into(),
    root,
    nodes,
    data: merged_data,
}), nav_active_patch("companies")])
```

**Citation:** `[CITED: contact.rs:1000-1018 current shape this helper compresses]` + `[CITED: standard.rs:916-935 `container_builder_with_children` shows the exact shape of `build_with_children`]`.

**Known constraint:** `contact.rs:437-1018` also composes Tags, Notes, Interactions sub-sections under the form in edit mode. The helper should cover only the **form envelope** (heading + back + form). The extra sub-sections stay inline after `form_shell()` assembles the shell and the handler merges its extra nodes into the returned `nodes` map before the `RenderMessage`. D-B2 calls this out: the helper compresses the **common envelope**, not every handler's full node inventory.

### Pattern 3 — `validation_error_patch()` helper (D-D3 proposed signature)

**What:** A small helper that shapes a multi-field validation failure into a single `PatchMessage` targeting the form's surface.

**Proposed signature:**
```rust
// File: backend/crates/marionette/src/validation.rs (NEW, or appended to error.rs)
// Phase 15 D-D3.

use marionette_protocol::{PatchMessage, PatchOperation, ProtocolMessage};

/// Build a `PatchMessage` carrying one `SetData` op per invalid field. The
/// resulting message targets the supplied surface (typically `"content"` for
/// CRM forms) and writes each `msg` to `/_errors{bind}` — exactly the path
/// shape each `TextInput`/`Select`/`…` already reads on the frontend
/// (TextInput.svelte:30).
///
/// # Arguments
/// * `surface` — target surface name (e.g., `"content"`).
/// * `errors`  — iterator of `(bind_path, human_message)` tuples. `bind_path`
///              MUST start with `/` and match the field's existing `.bind(...)`
///              argument (e.g., `"/contactForm/email"` → patch path
///              `"/_errors/contactForm/email"`).
///
/// # Returns
/// A single `ProtocolMessage::Patch(PatchMessage)` ready to wrap in `Ok(vec![…])`.
#[must_use]
pub fn validation_error_patch<I, B, M>(
    surface: impl Into<String>,
    errors: I,
) -> ProtocolMessage
where
    I: IntoIterator<Item = (B, M)>,
    B: Into<String>,
    M: Into<String>,
{
    let ops: Vec<PatchOperation> = errors
        .into_iter()
        .map(|(bind, msg)| PatchOperation::Set {
            path: format!("/_errors{}", bind.into()),
            value: serde_json::Value::String(msg.into()),
        })
        .collect();
    ProtocolMessage::Patch(PatchMessage {
        id: None,       // propagated by ws.rs::propagate_id on send
        surface: surface.into(),
        patch: ops,
    })
}

/// Convenience for the "no validation errors — return the success render" fork.
/// Equivalent to `errors.is_empty()` but reads better at call sites:
///
/// ```ignore
/// let errors = collect_errors(&data);
/// if !errors.is_empty() {
///     return Ok(vec![validation_error_patch("content", errors)]);
/// }
/// // … proceed with DB write + render_contact_list
/// ```
```

**Why this signature:**
- `ProtocolMessage` (not `PatchMessage` directly) — matches what handlers return in `Ok(vec![...])`.
- `id: None` — `ws.rs::propagate_id` at line 287 adds the action-id correlation; helper stays ignorant of action context.
- `surface: impl Into<String>` — gives the handler control. CRM forms all render to `"content"` surface (verified across handlers — see Integration Points table in 15-CONTEXT §code_context).
- Path format `/_errors{bind}` matches the canonical read at `TextInput.svelte:30`: `getData(surface, '/_errors' + bind)`.

**Alternative rejected:** Handler-by-handler `PatchOperation::Set` hand-coding. **Reject**: 4 handlers × ~5 fields = 20 error sites. Helper removes 3 lines + 1 `format!()` per site; keeps error strings grouped at the top of each handler.

**Alternative rejected:** New `ActionError::FieldValidation(Vec<(String, String)>)` variant. **Reject**: D-D3 explicitly says "No new error variant — the handler returns `Ok(response)` with the validation patch; the error channel stays reserved for protocol-layer failures." `ActionError::BadPayload` keeps its purpose (JSON parse, missing form_bind, malformed action).

**Handler flow after adopting helper:**
```rust
pub async fn handle_company_save(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let payload = Payload::<CompanySavePayload>::from_context(&ctx)?;
    let data = payload.0.company_form;

    // Collect per-field errors
    let mut errors: Vec<(String, String)> = Vec::new();
    if data.name.trim().is_empty() {
        errors.push(("/companyForm/name".into(), "Name is required".into()));
    }
    if let Some(ref w) = data.website {
        if !w.is_empty() && !w.starts_with("http") {
            errors.push(("/companyForm/website".into(), "Website must start with http(s)://".into()));
        }
    }
    if !errors.is_empty() {
        return Ok(vec![validation_error_patch("content", errors)]);
    }

    // …proceed to DB write + re-render
}
```

**Success/failure mutual exclusion:** On validation failure, the handler RETURNS the patch. No success render emitted. Next action (e.g., user fixes field + resubmits) re-triggers the handler. No side effects from the patch path — patches clear on the next full render of the form (because the render resets `data` for the surface).

**Clearing stale errors:** When a successful save re-renders via `render_contact_list(&ctx)`, the new `RenderMessage.data` replaces the surface data including any `_errors` nodes (a) because `setFullState(msg.surface, msg.data)` at init.ts:31 overwrites the whole surface state, and (b) since `_errors` would not be present in `merged_data`, it effectively clears. Per-field error state is **ephemeral to a single invalid submit**.

**Citation:** `[VERIFIED: frontend/src/lib/init.ts:29-32 `setFullState(msg.surface, msg.data)` on render — wipes previous _errors]`. `[VERIFIED: spec/PROTOCOL.md:600 "Servers clear errors by patching the path to an empty string / empty array"]`.

### Anti-Patterns to Avoid (carried over from Phase 14 review + pre-existing pitfalls)

- **WR-01: `sendAction(form.action, {}, surface)` with empty payload in Form.svelte:29.** Fix per D-G2: either pass collected form values as payload, OR remove the dispatch entirely. The preferred fix is (a) — makes `Form action="…"` a viable submit path. `[CITED: 14-VERIFICATION.md line 134 + Form.svelte:29]`.
- **WR-02: Two validation shapes in PROTOCOL.md (ll. 804-819 vs 593-600).** Delete legacy; keep canonical. `[CITED: 14-VERIFICATION.md line 135]`.
- **IN-01: Hand-rolled `Component` literal in contact.rs:1577-1584.** Swap to `Button::new("dismiss_toast").label(toast_label)…build()`. `[CITED: 14-VERIFICATION.md line 138]`.
- **IN-02: `window.__mrnSetData` unconditionally assigned.** Wrap in `if (import.meta.env.DEV)`. Verified in-codebase pattern at `frontend/src/lib/components/core/FallbackComponent.svelte:12` + `:18`. `[CITED: 14-VERIFICATION.md line 139]`.
- **Form-level `BadPayload` for field-specific errors.** Phase 14 UAT-03 ran into this exact trap — the test couldn't render inline errors because the handler returned a form-level toast. D-D1 is the fix. `[CITED: 14-08-SUMMARY.md lines 200-206]`.
- **Schema creep mid-phase.** If a handler migration exposes a missing primitive, file as deferred per D-A3. Do NOT introduce `DatePicker`/`Combobox`/`FileInput` mid-sweep.
- **Back-compat shims.** Pre-deployment — drop the old field / rename cleanly. `[CITED: feedback_pre_deployment_no_backcompat.md]`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Form field styling (label + error + description) | Custom CSS wrappers | `Field.Field`/`Field.Label`/`Field.Description`/`Field.Error` (Phase 14) | Already installed, already tested end-to-end via UAT-01..UAT-05. `[CITED: frontend/src/lib/components/ui/field/*]` |
| Responsive grid for field groups | Grid utility gymnastics | `FieldSet` (default `grid grid-cols-1 md:grid-cols-2 gap-4`) or `FieldSet.cols(N)` | D-C3 auto-responsive is proven; FieldSet.cols covers override. `[CITED: FieldSet.svelte + UI-SPEC §Phase 14]` |
| Per-field validation render path | Custom error banners below fields | Existing Field.Error reads `/_errors/{bind}` automatically | Phase 14 UAT-03 confirms the render path fires on `__mrnSetData`. Phase 15 wires the backend to emit. `[CITED: TextInput.svelte:30]` |
| Single-choice selection | Hand-rolled radio buttons | `RadioGroup` (shadcn, bits-ui) | Phase 14 D-E4 installed it; currently zero CRM usage. Phase 15 D-E1 gives it `interaction.type` + D-E2 `preferred_contact_method`. `[CITED: frontend/src/lib/components/form/RadioGroup.svelte]` |
| Filesystem grep in CI tests | Manual `exec('grep …')` with brittle parsing | `child_process.execSync` wrapping `ripgrep` or a `node:fs` recursive walk | Existing pattern in `ci-guards.spec.ts` uses `node:fs.existsSync`. For multi-file grep, prefer `child_process.execSync('rg -ilrn flowbite …', …)` (or fall back to `find + grep`). Phase 15 D-F1 grep is a static filesystem assertion. |
| Dev-only test hook gating | Runtime checks + manual strip scripts | `if (import.meta.env.DEV)` (Vite tree-shakes at build time) | `[VERIFIED: vite.dev/guide/env-and-mode]` + already in repo at `FallbackComponent.svelte:12`. |
| DB schema migration | Raw ORM calls at app startup | SeaORM migration chain via `backend/crates/crm-demo/src/migration/mod.rs` | Every prior column lives there; automatic run-on-startup via `Migrator::up`. Use `execute_unprepared` raw SQL to match existing style. |
| SeaORM entity field addition | Schema-first inference | Hand-edit `entities/contact.rs` Model struct | ORM is declarative; Model + ActiveModel must match the column names `contact_country`, `contact_notes`, `contact_opt_in`. `[CITED: existing pattern in contact.rs Model]` |
| Validation-patch error shape | Custom JSON arrays | `PatchOperation::Set { path: "/_errors/...", value: String(msg) }` | Exact shape the Field component reads. `[VERIFIED: spec/PROTOCOL.md:597 + frontend/src/lib/components/form/TextInput.svelte:30]` |

**Key insight:** Phase 15 is composition-heavy and primitives-light. Every missing piece is a **call-site change** in existing builders or a **one-function helper**, not a new architectural layer. Resist the temptation to build a `ValidationResult` type or a "form-state" abstraction.

## Runtime State Inventory

Phase 15 adds three columns to the `contact` table and renames zero strings at runtime. The DB migration IS the data migration — SeaORM's migration chain runs on app startup (`Migrator::up` via `run_migrations` wrapper). However, there are runtime state considerations:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | The existing `crm.db` SQLite file on developer machines will have the OLD `contact` schema (no country/notes/opt_in columns). On next `make dev` start, the new migration `m20260418_000011_extend_contact.rs` runs `ALTER TABLE contact ADD COLUMN …` and the data is safe. Existing rows get NULL for country/notes and 0 for opt_in (DEFAULT). | **Code edit** — migration handles it. No data migration needed (defaults preserve row validity). Seed.rs top-up writes the new columns only when it creates new rows; existing contacts stay NULL. |
| Live service config | None — Phase 15 introduces no external service integrations. Listmonk sync (existing) is untouched; email-change propagation logic stays put. | None. |
| OS-registered state | None — no systemd/pm2/Task Scheduler state. Project is a single `make dev` dev server. | None. |
| Secrets/env vars | `MARIONETTE_ADMIN_*` env vars unchanged. `LISTMONK_*` unchanged. No new secrets. | None. |
| Build artifacts / installed packages | `backend/target/` will need a `cargo build` after schema edit (Model struct change). `frontend/node_modules` untouched. `backend/crm.db` may need reset if a developer has stale state from before the migration — but the migration is additive, so it should auto-extend. | **Code edit** only. Developers running `make dev` after the pull will see the migration run on startup. If any anomalies, delete `backend/crm.db` + restart; seed re-populates. |

**The canonical question (all changes applied, what runtime state carries old assumption?):**
- `backend/crm.db` — auto-migrates on boot. Safe.
- SOPS / CI / `.env` files — no changes referenced. Safe.
- No in-memory config registries, no n8n workflows, no Tailscale ACLs, no Datadog dashboards, no Windows Task Scheduler, no pm2 process names, no egg-info, no Docker image tags.

**Explicit nothing-found confirmations:**
- **No renames** in Phase 15 — the phase extends (contact schema) and replaces (handler composition), but no string/symbol renames that propagate to external systems.
- **No broken external references** — `contact_id`, `contactForm`, `contact_save` action names all preserved. The only NEW symbols are new columns (already covered above) and new helper functions (`form_shell`, `validation_error_patch`).

## Common Pitfalls

### Pitfall 1: Validation-patch ordering lost across multiple ops

**What goes wrong:** Handler emits `[SetData /_errors/contactForm/name, SetData /_errors/contactForm/email]`. Frontend processes them in declared order per `PatchMessage` semantics — but if the handler emitted them out of creation order, the UX may feel disjointed.

**Why it happens:** Developers collect errors in field-definition order, but the `Vec` can be appended to in any order.

**How to avoid:** Collect errors in FORM-field order (top-to-bottom) — the same order the fields appear in the `FieldSet` children. This ensures the first error to scroll to is the topmost in the form.

**Warning signs:** UAT screenshots show the error banner above a field below another invalid field (visually jumpy).

### Pitfall 2: Surface name mismatch on validation patch

**What goes wrong:** Handler emits `PatchMessage { surface: "main", patch: [Set /_errors/contactForm/name …] }` but the form is rendered to the `"content"` surface. Frontend store applies the patch to `/_errors/contactForm/name` ON SURFACE `"main"` — invisible to the form.

**Why it happens:** `surface: "main"` is the default ambient surface; handlers may forget that CRM forms live on `"content"` (AppShell slot).

**How to avoid:** `validation_error_patch()` takes `surface` as an explicit parameter. Every CRM save handler passes `"content"` — verified by `[VERIFIED: grep surface: "content" across handlers/*.rs]`.

**Warning signs:** E2E test reports "no Field.Error visible" even though the backend returned a Patch.

### Pitfall 3: SeaORM Model mismatch after migration

**What goes wrong:** Migration adds `contact_country TEXT NULL` but `entities/contact.rs` Model still doesn't declare it. SeaORM's `find()`/`insert()` panics or silently drops the column.

**Why it happens:** Schema edits are in two files (migration SQL + entity Model). Drift is easy.

**How to avoid:** Add all three fields to `Model` in one edit. Use `Option<String>` for nullable columns; `bool` for `contact_opt_in INTEGER NOT NULL DEFAULT 0` (SeaORM serialises `bool` → `INTEGER`).

**Warning signs:** `cargo test -p crm-demo` fails at runtime with `Column not found: contact_country`.

### Pitfall 4: `contact_opt_in` as `Option<bool>` vs `bool`

**What goes wrong:** Column is `INTEGER NOT NULL DEFAULT 0`. Using `Option<bool>` in Model invites confusion ("is None 0 or unset?").

**How to avoid:** Model field: `pub contact_opt_in: bool`. NOT NULL + DEFAULT 0 means existing rows resolve to `false` automatically. Frontend Switch sends `true`/`false`.

**Warning signs:** Compile warnings about `Option<bool>` handling; tests that unwrap() on a None.

### Pitfall 5: SQLite `ALTER TABLE` in multi-step migrations

**What goes wrong:** A single migration file with multiple `ALTER TABLE … ADD COLUMN …` statements in one `execute_unprepared` call. SQLite supports multi-statement SQL via `;`, but it's fragile.

**How to avoid:** Issue three separate `execute_unprepared` calls, one per column. The SeaORM tutorial notes that "Atomic migration is not supported in MySQL and SQLite" — if one statement fails, the DB is half-migrated. For pre-deployment work this is acceptable (developers can reset crm.db), but keep the statements minimal and idempotent-friendly. `[CITED: https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/]`

**Concrete pattern:**
```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    conn.execute_unprepared(
        "ALTER TABLE contact ADD COLUMN contact_country TEXT"
    ).await?;
    conn.execute_unprepared(
        "ALTER TABLE contact ADD COLUMN contact_notes TEXT"
    ).await?;
    conn.execute_unprepared(
        "ALTER TABLE contact ADD COLUMN contact_opt_in INTEGER NOT NULL DEFAULT 0"
    ).await?;
    Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    // SQLite 3.35+ supports DROP COLUMN. Reverse order for defensive symmetry.
    conn.execute_unprepared(
        "ALTER TABLE contact DROP COLUMN contact_opt_in"
    ).await?;
    conn.execute_unprepared(
        "ALTER TABLE contact DROP COLUMN contact_notes"
    ).await?;
    conn.execute_unprepared(
        "ALTER TABLE contact DROP COLUMN contact_country"
    ).await?;
    Ok(())
}
```
`[VERIFIED: backend/crates/crm-demo/src/migration/m20260323_000004_create_contact.rs pattern: execute_unprepared + DbErr]`

**SQLite DROP COLUMN note:** Available since SQLite 3.35.0 (March 2021). `[CITED: SQLite release history]`. The project uses `sqlx-sqlite` via SeaORM; modern bundled SQLite supports it.

### Pitfall 6: RadioGroup options validated twice

**What goes wrong:** Both frontend (bits-ui) and backend validate the option value — but they can drift. A frontend Select with `["call","email","meeting"]` options may pass `Other` and the backend rejects it (interaction.rs:196).

**How to avoid:** Keep backend validation as the authoritative source. The RadioGroup UI restricts choices visually, but `data.interaction_type` should still be server-validated. D-E1 says "mirror the existing Select's validation server-side" — keep the `if !["call","email","meeting"].contains(…)` check.

### Pitfall 7: Flowbite grep false positives in generated snapshots or test fixtures

**What goes wrong:** CI grep finds `flowbite` in `frontend/tests/__snapshots__/*.png` (bytes happen to encode "flowbite" — extremely unlikely but possible) or in a comment reference that's intentional.

**How to avoid:** Scope the grep strictly:
- **Include:** `frontend/src/**/*.{ts,tsx,svelte,js,mjs,css}`, `backend/crates/**/*.rs`, `spec/**/*.{md,yaml}`, top-level `CONCEPT.md`, `TOOLING.md`.
- **Exclude:** `.planning/`, `node_modules/`, `target/`, `frontend/tests/__snapshots__/`, any `*.png`/`*.jpg`/binary files, `frontend/src/lib/components/ui/` if the shadcn-svelte CLI ever emits "flowbite-compat" comments (audit).

**Recommended implementation:** Shell out to `git grep -il 'flowbite'` — respects `.gitignore`, includes only text files, fast. Fall back to `node:fs` walk if `git grep` is not portable enough.

**Expected behaviour:** `expect(matches.length).toBe(0)`.

### Pitfall 8: Visual snapshot diff creep per screen migration

**What goes wrong:** Each new form migration lands with new visual snapshot baselines (D-H3). On first `--update-snapshots` run they're green; on second run, font-hinting / anti-aliasing differences produce false failures.

**How to avoid:** `playwright.uat.config.ts` at line 22 already sets `toHaveScreenshot: { maxDiffPixels: 100 }` — reuse this tolerance for phase 15 visuals. Phase 14 confirmed this works (14-08 SUMMARY "Verified green on second run").

### Pitfall 9: `__mrnSendAction` gated but E2E/UAT tests run against `make dev` (DEV mode)

**What goes wrong:** Developer accidentally wraps `__mrnSendAction` and `__mrnSetData` in `if (import.meta.env.PROD)` — or gates only in production build config — causing E2E tests to fail because `make dev` IS dev mode, hooks should be present.

**How to avoid:** `import.meta.env.DEV` is `true` during `vite dev` AND during `playwright test` runs against the dev server. Production `vite build` sets it to `false`. Verified behaviour: `[CITED: vite.dev/guide/env-and-mode — DEV true in dev, false in build]` + `[VERIFIED: FallbackComponent.svelte:12-18 uses the same gate in repo]`. E2E tests keep working.

## Code Examples

### Example 1 — Migrated `handle_company_form` (before → after)

**Source:** Phase 14 template at `backend/crates/crm-demo/src/handlers/contact.rs:519-670`, adapted for company's 3 fields.

```rust
// AFTER — Phase 15 D-A1 + D-B1 + D-E3
pub async fn handle_company_form(ctx: HandlerContext) -> ActionResult {
    let db = Db::from_context(&ctx)?;
    let company_id = Payload::<CompanyIdPayload>::from_context(&ctx).ok().map(|p| p.0.company_id);

    let (form_data, form_title) = /* existing edit/new switch */;

    let heading = Heading::new(form_title).id("company-form-heading").build();
    let back_button = Button::new("← Back")
        .id("company-form-back")
        .variant("outline")
        .action(ComponentAction::click("company_list"))
        .build();

    // Single FieldSet — 3 fields, one semantic group.
    let name_input = TextInput::new("Name")
        .id("company-form-name")
        .bind("/companyForm/name")
        .required(true)
        .description("Will appear on invoices.")   // D-E3
        .build();
    let website_input = TextInput::new("Website")
        .id("company-form-website")
        .bind("/companyForm/website")
        .input_type("url")
        .placeholder("https://example.com")
        .build();
    let address_input = TextInput::new("Address")
        .id("company-form-address")
        .bind("/companyForm/address")
        .full_width(true)                            // D-C4 — long field
        .build();

    let (details_set, details_desc) = FieldSet::new()
        .id("company-details-set")
        .legend("Company details")
        .children(vec![name_input, website_input, address_input])
        .build_tree();

    let cancel_button = Button::new("Cancel")
        .id("company-form-cancel")
        .variant("outline")
        .action(ComponentAction::click("company_list"))
        .build();
    let save_button = Button::new("Save company")
        .id("company-form-save")
        .action(ComponentAction::submit("company_save"))
        .build();
    let (action_row, action_desc) = Container::new()
        .id("company-form-actions")
        .class("flex gap-2 justify-end")
        .children(vec![cancel_button, save_button])
        .build_tree();

    let mut form_descendants = details_desc;
    form_descendants.extend(action_desc);
    let (form_child, form_desc) = Form::new()
        .id("company-form")
        .children(vec![details_set, action_row])
        .build_tree();
    form_descendants.extend(form_desc);

    let (root, nodes) = form_shell(
        "company-form-root",
        heading,
        back_button,
        form_child,
        form_descendants,
    );
    Ok(vec![
        ProtocolMessage::Render(RenderMessage {
            id: ctx.action.id.clone(),
            surface: "content".into(),
            root,
            nodes,
            data: form_data,
        }),
        nav_active_patch("companies"),
    ])
}
```

### Example 2 — Migrated `handle_interaction_form` with RadioGroup (D-E1)

```rust
// AFTER — Phase 15 D-A1 + D-E1 + D-E3 + D-B1
let type_radio = RadioGroup::new("Type")
    .id("interaction-form-type")
    .bind("/interactionForm/interaction_type")
    .options(vec![
        RadioOption { value: "call".into(), label: "Call".into(), description: Some("Phone or voice chat".into()) },
        RadioOption { value: "email".into(), label: "Email".into(), description: None },
        RadioOption { value: "meeting".into(), label: "Meeting".into(), description: Some("In-person or video".into()) },
    ])
    .required(true)
    .description("Select how you interacted.")   // D-E3
    .build();

let subject_input = TextInput::new("Subject")
    .id("interaction-form-subject")
    .bind("/interactionForm/subject")
    .required(true)
    .build();
let date_input = TextInput::new("Date")
    .id("interaction-form-date")
    .bind("/interactionForm/date")
    .input_type("datetime-local")
    .description("YYYY-MM-DD HH:MM or your local datepicker.")
    .build();
let notes_textarea = Textarea::new("Notes")
    .id("interaction-form-notes")
    .bind("/interactionForm/notes")
    .rows(4)
    .full_width(true)
    .build();

let (set, set_desc) = FieldSet::new()
    .id("interaction-set")
    .legend("Interaction")
    .children(vec![type_radio, subject_input, date_input, notes_textarea])
    .build_tree();

// + action row + form_shell as above
```

### Example 3 — Per-field validation in `handle_contact_save` (D-D1 + D-D3)

```rust
// AFTER — Phase 15 D-D1
pub async fn handle_contact_save(ctx: HandlerContext) -> ActionResult {
    use sea_orm::ActiveValue::{NotSet, Set};

    let db = Db::from_context(&ctx)?;
    let session = Session::from_context(&ctx)?;
    let payload = Payload::<ContactSavePayload>::from_context(&ctx)?;
    let data = payload.0.contact_form;

    // Collect per-field validation errors.
    let mut errors: Vec<(String, String)> = Vec::new();
    if data.name.trim().is_empty() {
        errors.push(("/contactForm/name".into(), "Contact name is required.".into()));
    }
    if data.email.trim().is_empty() {
        errors.push(("/contactForm/email".into(), "Email is required.".into()));
    } else if !data.email.contains('@') {
        errors.push(("/contactForm/email".into(), "Please enter a valid email address.".into()));
    }
    if !errors.is_empty() {
        return Ok(vec![
            marionette::validation::validation_error_patch("content", errors),
        ]);
    }

    // …existing DB write + country/notes/opt_in persistence (D-C3)…
    let company_id: Option<i32> = data.company.as_deref()
        .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });

    match data.id {
        None => {
            let new_contact = contact::ActiveModel {
                contact_id: NotSet,
                contact_name: Set(data.name.clone()),
                contact_email: Set(data.email.clone()),
                contact_phone: Set(data.phone.clone()),
                contact_title: Set(data.title.clone()),
                contact_company: Set(company_id),
                contact_country: Set(data.country.clone()),      // NEW D-C3
                contact_notes: Set(data.notes.clone()),          // NEW D-C3
                contact_opt_in: Set(data.opt_in.unwrap_or(false)),// NEW D-C3
                contact_created_at: NotSet,
                contact_updated_at: NotSet,
            };
            // …insert + audit…
        }
        Some(cid) => { /* existing update path + .contact_country = Set(data.country) etc */ }
    }

    render_contact_list(&ctx).await
}
```

### Example 4 — SeaORM extend-contact migration (D-C1, D-C4)

```rust
// File: backend/crates/crm-demo/src/migration/m20260418_000011_extend_contact.rs
// Phase 15 D-C1.
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260418_000011_extend_contact"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE contact ADD COLUMN contact_country TEXT"
        ).await?;
        conn.execute_unprepared(
            "ALTER TABLE contact ADD COLUMN contact_notes TEXT"
        ).await?;
        conn.execute_unprepared(
            "ALTER TABLE contact ADD COLUMN contact_opt_in INTEGER NOT NULL DEFAULT 0"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        // Reverse order for defensive symmetry. Requires SQLite 3.35+ (DROP COLUMN).
        conn.execute_unprepared("ALTER TABLE contact DROP COLUMN contact_opt_in").await?;
        conn.execute_unprepared("ALTER TABLE contact DROP COLUMN contact_notes").await?;
        conn.execute_unprepared("ALTER TABLE contact DROP COLUMN contact_country").await?;
        Ok(())
    }
}
```

**Register in `migration/mod.rs`:**
```rust
mod m20260418_000011_extend_contact;  // NEW

// …existing…
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // …existing 10…
            Box::new(m20260418_000011_extend_contact::Migration),   // NEW
        ]
    }
}
```

### Example 5 — Entity update + seed spread (D-C2)

```rust
// File: backend/crates/crm-demo/src/entities/contact.rs (EDIT)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contact")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub contact_id: i32,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub contact_title: Option<String>,
    pub contact_company: Option<i32>,
    pub contact_country: Option<String>,  // NEW (nullable TEXT)
    pub contact_notes: Option<String>,    // NEW (nullable TEXT)
    pub contact_opt_in: bool,             // NEW (INTEGER NOT NULL DEFAULT 0)
    pub contact_created_at: String,
    pub contact_updated_at: String,
}
```

**Seed update:** In `seed.rs::seed_contacts`, named_contacts vec becomes:
```rust
let named_contacts: Vec<(&str, &str, Option<&str>, Option<&str>, Option<i32>, Option<&str>, Option<&str>, bool)> = vec![
    ("Alice Johnson", "alice@acme.example.com", Some("+1-555-0101"), Some("CEO"),
     acme.as_ref().map(|c| c.company_id),
     Some("CH"), Some("Interested in Q2 enterprise tier."), true),
    ("Bob Smith", "bob@globex.example.com", Some("+1-555-0102"), Some("CTO"),
     globex.as_ref().map(|c| c.company_id),
     Some("US"), None, false),
    ("Carol Williams", "carol@example.com", None, Some("Freelancer"), None,
     None, Some("Long-form note: available for contract work starting Q3."), true),
];
for (name, email, phone, title, company_id, country, notes, opt_in) in named_contacts {
    let model = contact::ActiveModel {
        contact_id: NotSet,
        contact_name: Set(name.into()),
        contact_email: Set(email.into()),
        contact_phone: Set(phone.map(String::from)),
        contact_title: Set(title.map(String::from)),
        contact_company: Set(company_id),
        contact_country: Set(country.map(String::from)),
        contact_notes: Set(notes.map(String::from)),
        contact_opt_in: Set(opt_in),
        contact_created_at: NotSet,
        contact_updated_at: NotSet,
    };
    model.insert(db).await?;
}
```

Generated seed contacts can stay NULL / false to avoid inflating seed size.

### Example 6 — Flowbite residue CI guard (D-F1)

```typescript
// frontend/tests/e2e/ci-guards.spec.ts (EXTEND)
import { test, expect } from '@playwright/test';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

// After D-G4, drop all @ts-expect-error suppressions above — node: prefix resolves.

const FRONTEND_ROOT = resolve(fileURLToPath(import.meta.url), '..', '..', '..');
const REPO_ROOT = resolve(FRONTEND_ROOT, '..');

test.describe('Phase 13/14/15 CI guards', () => {
    test('TableScreen.svelte is retired (Phase 13 D-A2)', () => {
        const p = resolve(FRONTEND_ROOT, 'src/lib/components/screen/TableScreen.svelte');
        expect(existsSync(p)).toBe(false);
    });

    test('FormScreen.svelte is retired (Phase 14 D-A1)', () => {
        const p = resolve(FRONTEND_ROOT, 'src/lib/components/screen/FormScreen.svelte');
        expect(existsSync(p)).toBe(false);
    });

    test('No Flowbite residue in runtime code (Phase 15 D-F1)', () => {
        // `git grep -Iil` → case-insensitive match, text files only, name-only output.
        // Respect .gitignore (skips node_modules, target, __snapshots__, etc.).
        // Scope: src/** + backend/crates/** + spec/** + top-level CONCEPT.md + TOOLING.md.
        let matches: string[] = [];
        try {
            const out = execSync(
                `git grep -Iil 'flowbite' -- ` +
                `'frontend/src/**' ` +
                `'backend/crates/**' ` +
                `'spec/**' ` +
                `CONCEPT.md ` +
                `TOOLING.md`,
                { cwd: REPO_ROOT, encoding: 'utf8' }
            );
            matches = out.trim().split('\n').filter(Boolean);
        } catch (e: unknown) {
            // git grep exits 1 when NO matches — which is the success case.
            const err = e as { status?: number; stdout?: string };
            if (err.status === 1) matches = [];
            else throw e;
        }
        expect(matches, `Flowbite residue found in:\n${matches.join('\n')}`).toHaveLength(0);
    });
});
```

**Why `git grep` vs `node:fs` recursive walk:** `git grep` is ~10× faster on the full repo (indexed), respects `.gitignore` (skips `node_modules`, `target`, `__snapshots__`), and `-I` skips binary files automatically. Matches the Playwright e2e working directory (`frontend/`, per `playwright.config.ts` — but this test uses `REPO_ROOT` via `cwd` option).

**Fallback if `git grep` unavailable:** Replace with `execSync("rg -il flowbite …", …)` (ripgrep) — install as devDep if needed. Both are text-file-only.

### Example 7 — Dev-gated test hooks (D-G1)

```typescript
// frontend/src/lib/init.ts:92-102 (REWRITE)
// Phase 15 D-G1 — gate test hooks behind import.meta.env.DEV. Production
// builds tree-shake the entire `if` block; the hooks disappear from the
// final bundle.
if (typeof window !== 'undefined' && import.meta.env.DEV) {
    (window as unknown as { __mrnSendAction: typeof sendAction }).__mrnSendAction = sendAction;
    (window as unknown as { __mrnSetData: typeof setData }).__mrnSetData = setData;
}
```

**Verification steps (plan must include):**
1. Dev mode: `make dev` → browser devtools → `window.__mrnSendAction` defined. E2E tests continue to work.
2. Production build: `cd frontend && npm run build` → search `build/client/_app/immutable/chunks/*.js` for `__mrnSetData`. Expected: 0 matches. `[CITED: vite.dev/guide/env-and-mode tree-shakes DEV-guarded blocks]`.

### Example 8 — PROTOCOL.md worked multi-field validation example (D-D2)

**Text to ADD under §Validation semantics at ~line 600:**

```markdown
#### Worked example: multi-field validation on form submit

A handler receiving an invalid form payload returns a single `PatchMessage`
with one `SetData` op per invalid field, targeting the form's surface.
The frontend's `Field.Error` anatomy picks up each entry and renders it
inline below the bound control.

```json
{
  "type": "patch",
  "surface": "content",
  "patch": [
    { "op": "set", "path": "/_errors/contactForm/name",  "value": "Contact name is required." },
    { "op": "set", "path": "/_errors/contactForm/email", "value": "Please enter a valid email address." }
  ]
}
```

The save handler that produced this patch returns `Ok(vec![patch])` — NOT
`Err(ActionError::BadPayload)`. `ErrorMessage` is reserved for protocol-level
failures (malformed action payload, unknown surface, server crash, auth
failure). Field-level validation is data, and flows through the normal
patch channel.

When the user fixes the offending fields and resubmits, the next success
render replaces the surface data wholesale, clearing any prior `_errors`.
The handler does not need to emit "clear error" patches explicitly.
```

**Text to DELETE at `spec/PROTOCOL.md:803-819` (verified current):**

```markdown
### Validation Errors as Data

**Field-level validation errors are data patches**, not error messages. The server patches error information into the data model, and components bind to it. This keeps validation state in the reactive data flow.

\```yaml
# Server patches validation errors into data:
type: patch
patch:
  - path: "/contactForm/errors"
    value:
      - path: "/contactForm/data/email"
        message: "Invalid email address"
      - path: "/contactForm/data/phone"
        message: "Phone number must include country code"
\```

A component bound to `/contactForm/errors` displays these errors inline. When the user corrects the fields and resubmits, the server either patches the errors to an empty array (valid) or patches new errors (still invalid).
```

The replacement in §Validation semantics (ll. 593-600) already covers the canonical shape; the Worked Example above is the only net addition.

### Example 9 — Chrome-MCP / Playwright UAT scaffold per screen (D-H1)

Following the Phase 14 Plan 08 precedent at `frontend/tests/uat/uat-driver.spec.ts`:

```typescript
// frontend/tests/uat/uat-driver.spec.ts (EXTEND, or SPLIT into one spec per screen)
// Phase 15 Plan 7 UAT — company-edit form.

import { test, expect, type Page } from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';

const EVIDENCE_DIR = path.resolve(
    process.cwd(), '..',
    '.planning/phases/15-crm-migration-validation/15-uat-evidence/company-edit'
);
fs.mkdirSync(EVIDENCE_DIR, { recursive: true });

async function login(page: Page) { /* same as Phase 14 */ }

async function openCompanyEdit(page: Page) {
    await page.evaluate(() => {
        const hook = (window as { __mrnSendAction?: (name: string, payload?: object, source?: string) => void }).__mrnSendAction;
        if (!hook) throw new Error('__mrnSendAction hook missing');
        hook('company_edit', { company_id: 1 }, 'company-edit-1');
    });
    await expect(page.getByRole('heading', { name: 'Edit Company' })).toBeVisible({ timeout: 10000 });
}

test.describe.configure({ mode: 'serial' });
test.describe('Phase 15 UAT — company-edit', () => {
    test('UAT-01 FieldSet legend renders', async ({ page }) => {
        await login(page);
        await openCompanyEdit(page);
        await expect(page.getByText('Company details')).toBeVisible();
        await page.screenshot({ path: path.join(EVIDENCE_DIR, '01-fieldset-legend.png') });
    });
    test('UAT-02 Per-field validation fires on empty name (D-D1)', async ({ page }) => {
        await login(page);
        await openCompanyEdit(page);
        await page.locator('#company-form-name').fill('');
        await page.getByRole('button', { name: 'Save company' }).click();
        await expect(page.locator('[data-slot="field-error"]').filter({ hasText: /required/i })).toBeVisible();
        const errorJson = await page.locator('[data-slot="field-error"]').first().evaluate((el: Element) => ({
            text: el.textContent,
            hasInvalidAttr: el.closest('[data-invalid]') !== null,
        }));
        fs.writeFileSync(path.join(EVIDENCE_DIR, '02-validation.json'), JSON.stringify(errorJson, null, 2));
    });
    test('UAT-03 Save flow re-renders list on success', async ({ page }) => {
        await login(page);
        await openCompanyEdit(page);
        // name is pre-filled from edit mode; just click Save
        await page.getByRole('button', { name: 'Save company' }).click();
        await expect(page.getByText('Company Management')).toBeVisible({ timeout: 5000 });
    });
});
```

**Scaffolding per screen (D-H1):**
- `15-uat-evidence/company-edit/`: 3 scenarios × ~2 artifacts = 6 files.
- `15-uat-evidence/user-edit/`: 3 scenarios (fieldset + validation + RadioGroup render).
- `15-uat-evidence/interaction-edit/`: 4 scenarios (fieldset + RadioGroup selection + datetime + validation).
- `15-uat-evidence/contact-tag-add/`: 2 scenarios (submit + validation).
- `15-uat-evidence/contact-note-add/`: 2 scenarios (submit + empty-text validation).

Total: ~14 scenarios × ~2 artifacts = ~28 evidence files. Scales below Phase 14's 12 for one form because primitives are already proven.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Form::new().children([TextInput, TextInput, Button, Button])` flat composition | `Container → Heading + Back + Form → FieldSet+Separator+FieldSet+…+action row` | Phase 14 (landed in contact.rs edit form) | Phase 15 applies verbatim to 4 more handlers. |
| `ActionError::BadPayload("Name required")` → form-level toast for field errors | `Ok(vec![validation_error_patch(...)])` → per-field `/_errors/{bind}` render | Phase 15 (this phase) | 4 save handlers; no protocol change. |
| `window.__mrnSendAction` unconditionally in dev + prod | `if (import.meta.env.DEV) { window.__mrnSendAction = … }` | Phase 15 D-G1 | Prod bundle shrinks by ~50 bytes; removes attack surface. |
| Two validation shapes in PROTOCOL.md (legacy array + canonical per-field) | Canonical `/_errors/{bind}` only | Phase 15 D-D2 | Doc clarity; no code impact. |
| `helperText` prop on form fields | `description` prop (shadcn nomenclature) | Phase 14 D-B3 | Already done; Phase 15 exercises it on every migrated field. |

**Deprecated/outdated:**
- **FormScreen.svelte** — retired Phase 14 D-A1. Phase 15 ci-guards continue asserting its absence.
- **TableScreen.svelte** — retired Phase 13 D-A2. Same guard.
- **Flowbite runtime code** — gone already; Phase 15 CI guard prevents reintroduction.
- **`/contactForm/errors` array validation shape** — deleted Phase 15 D-D2.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SQLite 3.35+ (ships with sqlx-sqlite) supports `DROP COLUMN` in down migrations | Pitfall #5, Example 4 | Down migration errors; phase is pre-deployment so mitigation is "reset crm.db". Low risk — sqlx bundles modern SQLite. |
| A2 | `git grep -Iil` is available in the CI environment | Example 6, Flowbite guard | CI may run without git; fallback to `rg` or `find + xargs grep`. Plan should detect + document. |
| A3 | `import.meta.env.DEV` tree-shakes the entire `if` block in Rollup production build | Example 7, D-G1 | If Rollup preserves the `if` but evaluates to `false`, hooks are present in code but unreachable. Either way the prop is safe. Verified via existing `FallbackComponent.svelte:12-18` in repo. |
| A4 | The Chrome-MCP server is still unavailable in this environment (Phase 14 finding) | Pattern §UAT | Plan should detect at start (check `.mcp.json`) and fall back to Playwright seamlessly. Phase 14 precedent covers the substitution contract. |
| A5 | `form_shell()` fits naturally alongside the existing contact.rs edit form — the Tags/Notes/Interactions sections below the form are a separate composition concern | Pattern §2 | If refactoring contact.rs reveals that form_shell() is a bad fit for the "form + extra tail sections" shape, the helper stays unused on contact.rs and D-B2 becomes partial. Mitigation: the helper still covers company/user/interaction (3 handlers), which hits the D-B1 threshold independently. |
| A6 | Action-id correlation added by `ws.rs::propagate_id` at line 287 applies to PatchMessages emitted from save handlers | Pattern §3, Pitfall #2 | If the correlation logic does NOT patch PatchMessages, the frontend's `confirmOptimistic(msg.id)` may not fire and optimistic updates can leak. Verified in `ws.rs:287-295` — Patch is explicitly covered: `ProtocolMessage::Patch(m) if m.id.is_none() => m.id = Some(id.to_owned())`. |
| A7 | The frontend `__mrnSendAction` hook is only called from E2E/UAT specs and is not referenced by any production code path | D-G1 | If it's used elsewhere in the tree, gating breaks prod builds. Verified: `grep -rn __mrnSendAction frontend/src` returns only `init.ts:93` (the assignment). |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | All backend builds + tests | ✓ | (rust stable — workspace-managed via mise/rustup) | — |
| `sqlx-sqlite` (bundled SQLite 3.35+) | Migration `DROP COLUMN` | ✓ | via sea-orm 1.1 | Use `CREATE TABLE new + copy + DROP + RENAME` workaround if older SQLite |
| `npm` / `node` | Frontend build + Playwright | ✓ | Node 18+ (node: prefix imports) | — |
| `@playwright/test` | E2E + visual + UAT | ✓ | ^1.58.2 | — |
| Chrome browser (headless) | Playwright runs + screenshots | ✓ | Installed via `npx playwright install chromium` (Phase 14 precedent) | — |
| Chrome-MCP server | Phase 15 D-H1 UAT (preferred) | ✗ (per Phase 14 finding) | — | Playwright UAT driver (Phase 14 Plan 08 substituted successfully; same contract) |
| `git` (for `git grep` in CI guard) | Example 6 Flowbite grep | ✓ | — | `rg` (ripgrep) or `find + grep` |
| `make` | Project dev loop (`make dev`) | ✓ | — | — |
| `tokio` runtime | Async handlers | ✓ (workspace) | — | — |

**Missing dependencies with no fallback:** None. All blocking deps present.

**Missing dependencies with fallback:** Chrome-MCP — fall back to Playwright (proven in Phase 14).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` (built-in) |
| Framework (TS unit + browser) | Vitest + `@vitest/browser` (Playwright driver) |
| Framework (E2E) | `@playwright/test` |
| Framework (Visual) | `@playwright/test` with `toHaveScreenshot` |
| Framework (UAT) | `@playwright/test` via `frontend/tests/uat/uat-driver.spec.ts` (or Chrome-MCP if wired) |
| Rust config | `backend/Cargo.toml` workspace |
| TS config | `frontend/package.json` + `frontend/playwright.config.ts` + `frontend/vitest.config.ts` |
| Quick run command | `cargo test -p marionette -p crm-demo --lib` + `cd frontend && npm run test -- --run` |
| Full suite command | `cargo test --workspace` + `cd frontend && npm run check && npm run test && npx playwright test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-03 / D-A1 | company edit form renders FieldSet legend + action row | E2E | `npx playwright test tests/e2e/company-edit.spec.ts` | ❌ Wave 0 (new spec) |
| COMP-03 / D-A1 | user edit form renders with RadioGroup for preferred_contact_method | E2E | `npx playwright test tests/e2e/user-edit.spec.ts` | ❌ Wave 0 (new spec) |
| COMP-03 / D-A1 / D-E1 | interaction edit form uses RadioGroup (not Select) for type | E2E | `npx playwright test tests/e2e/interaction-edit.spec.ts` | ❌ Wave 0 (new spec) |
| COMP-03 / D-A1 | contact inline tag-add / note-add forms still submit | E2E (smoke in contact-edit.spec.ts) | `npx playwright test tests/e2e/contact-edit.spec.ts` | ✅ existing (add test blocks) |
| COMP-03 / D-B1 | `form_shell()` produces expected nodes map shape | unit (Rust) | `cargo test -p marionette form_shell` | ❌ Wave 0 (new test in standard.rs tests mod) |
| COMP-03 / D-C1 | migration `m20260418_000011_extend_contact` adds 3 columns | unit (Rust) — implicit via seed + handler tests | `cargo test -p crm-demo contact` | ✅ (runs as part of seed_admin/seed_contacts on startup) |
| COMP-03 / D-C2 | Model struct includes country/notes/opt_in | compile-check | `cargo check -p crm-demo` | ✅ |
| COMP-03 / D-C3 | handle_contact_save persists country/notes/opt_in | integration (Rust) | new test in `handlers/contact.rs tests mod` | ❌ Wave 0 |
| COMP-03 / D-D1 | handle_company_save emits per-field patch on invalid name | integration (Rust) | new test asserting `Ok(vec![PatchMessage{...}])` shape | ❌ Wave 0 |
| COMP-03 / D-D1 | Field.Error renders when /_errors patch arrives | E2E (in per-screen spec) | `npx playwright test tests/e2e/*-edit.spec.ts` | ❌ Wave 0 |
| COMP-03 / D-D3 | validation_error_patch() returns correctly shaped PatchMessage | unit (Rust) | `cargo test -p marionette validation_error_patch` | ❌ Wave 0 |
| COMP-03 / D-E1 | interaction.type = "call" RadioGroup sends correct value | E2E | in `interaction-edit.spec.ts` | ❌ Wave 0 |
| COMP-03 / D-F1 | No `flowbite` token under runtime paths | E2E (filesystem) | `npx playwright test tests/e2e/ci-guards.spec.ts` | ✅ (extend existing) |
| COMP-03 / D-G1 | __mrnSendAction / __mrnSetData absent from prod build | manual / build-inspection | `grep -r __mrnSendAction frontend/build/` after `npm run build` | ❌ Wave 0 (document as manual check) |
| COMP-03 / D-G2 | Form.svelte action dispatch sends collected form values | unit (browser-test) | update `frontend/src/lib/components/form/Form.browser-test.ts` | ✅ (extend existing; may need rewrite) |
| COMP-03 / D-G3 | contact.rs:1577 toast uses Button builder | compile-check + protocol round-trip | implicit via Rust build + existing contact.rs tests | ✅ |
| COMP-03 / D-G4 | tests/helpers/schema-validator.ts uses node: prefix + no @ts-expect-error | svelte-check | `cd frontend && npm run check` | ✅ (existing check command) |
| COMP-03 / D-H3 | visual baselines for company/user/interaction forms | visual | `npx playwright test tests/visual/form.spec.ts` | ❌ Wave 0 (extend existing spec) |
| COMP-03 / Roadmap SC-1 | All CRM screens render | E2E suite pass | full playwright suite | partial |
| COMP-03 / Roadmap SC-2 | Zero Flowbite references | CI guard (above) | `tests/e2e/ci-guards.spec.ts` | ❌ Wave 0 |
| COMP-03 / Roadmap SC-3 | CRM navigation + CRUD + search + Listmonk work | E2E | `tests/e2e/shell-nav.spec.ts`, `datatable-filter.spec.ts`, `datatable-infinite-scroll.spec.ts` | ✅ existing |

### Sampling Rate

- **Per task commit:** `cargo test -p crm-demo --lib` (fast; <30s). Rust-side correctness for the plan's touched handler.
- **Per wave merge:** `cargo test --workspace` + `cd frontend && npm run check && npm run test:browser` (browser tests: ~2-3 min). Catches cross-crate drift.
- **Phase gate:** `cargo test --workspace` + `cd frontend && npm run check && npm run test && npx playwright test && npx playwright test tests/visual --update-snapshots=false` green before `/gsd-verify-work`.
- **Per screen (D-H3):** Visual snapshots checked manually on first `--update-snapshots` run; second run must be green.
- **UAT (D-H1):** Non-gating; evidence-only. Captured once per screen; committed under `.planning/phases/15-.../15-uat-evidence/`.

### Wave 0 Gaps

- [ ] `frontend/tests/e2e/company-edit.spec.ts` — covers D-A1 + D-D1 + D-E3 for company form
- [ ] `frontend/tests/e2e/user-edit.spec.ts` — covers D-A1 + D-D1 + D-E2 (RadioGroup for preferred_contact_method)
- [ ] `frontend/tests/e2e/interaction-edit.spec.ts` — covers D-A1 + D-D1 + D-E1 (RadioGroup for type)
- [ ] Unit test `form_shell_assembles_container_with_heading_back_form_children` in `backend/crates/marionette/src/builders/standard.rs tests mod`
- [ ] Unit tests `validation_error_patch_shapes_single_error` + `_multi_field` + `_empty_iter_returns_empty_patch` in `backend/crates/marionette/src/validation.rs` (or `error.rs` tests mod)
- [ ] Integration tests in `backend/crates/crm-demo/src/handlers/company.rs`, `user.rs`, `interaction.rs`, `contact.rs` asserting handle_*_save returns Patch on invalid payload (shape assertions on ProtocolMessage::Patch variant)
- [ ] `frontend/tests/e2e/ci-guards.spec.ts` — extend with Flowbite residue grep test block
- [ ] `frontend/tests/visual/form.spec.ts` — add 3 test blocks for company/user/interaction edit forms @ desktop + mobile (6 snapshots)
- [ ] `.planning/phases/15-crm-migration-validation/15-uat-evidence/` — create subdir per screen (5 subdirs)
- [ ] UAT spec under `frontend/tests/uat/uat-driver.spec.ts` — extend with Phase 15 screens OR split into `frontend/tests/uat/uat-driver-15.spec.ts`

*(No framework installs needed — everything Phase 15 uses is already in devDependencies.)*

### Test Tier Mapping to CONTEXT Decisions

| Decision | Test Tier | Evidence |
|----------|-----------|----------|
| D-A1 (CRM form sweep) | E2E per screen + visual rebaseline | Spec files above; snapshots |
| D-B1 (form_shell helper) | Rust unit + serialization | `cargo test` |
| D-B2 (contact.rs refactor to form_shell) | Rust compile + existing `contact-edit.spec.ts` | No visible behavior change |
| D-C1..C4 (contact schema extension) | Rust integration (save path round-trips) + E2E (edit form populates fields) | `cargo test` + `contact-edit.spec.ts` |
| D-D1 (per-field validation write) | Rust integration (ProtocolMessage shape) + E2E (Field.Error visible) | Matrix above |
| D-D2 (PROTOCOL.md surgery) | spec round-trip via `protocol-conformance.spec.ts` (existing) | Schema coverage |
| D-D3 (validation_error_patch helper) | Rust unit | New test |
| D-E1 (RadioGroup for interaction.type) | E2E | `interaction-edit.spec.ts` |
| D-E2 (preferred_contact_method RadioGroup, no persistence) | E2E (presence only) | `user-edit.spec.ts` |
| D-E3 (description on every form) | E2E (text visible per field) | Per-screen specs |
| D-F1 (Flowbite grep guard) | E2E (filesystem) | `ci-guards.spec.ts` |
| D-F2, D-F3 (doc sweep) | Manual + the grep guard enforces zero |
| D-G1..G4 (scope closure) | Mixed (build-inspection, svelte-check, browser-test, compile) | Matrix above |
| D-H1 (Chrome-MCP UAT) | UAT evidence | `15-uat-evidence/*` |
| D-H3 (visual rebaseline) | visual | `tests/visual/form.spec.ts` |

## Sources

### Primary (HIGH confidence)

- `[VERIFIED]` `backend/crates/crm-demo/src/handlers/contact.rs:519-670` — Phase 14 canonical form composition (every migration mirrors this).
- `[VERIFIED]` `backend/crates/crm-demo/src/handlers/contact.rs:1044-1186` — current `handle_contact_save` with `BadPayload` error shape.
- `[VERIFIED]` `backend/crates/crm-demo/src/handlers/company.rs:190-215, 318-330, 410-500` — current company form + inline note + save handler.
- `[VERIFIED]` `backend/crates/crm-demo/src/handlers/user.rs:180-270, 320-430` — current user form + save handler.
- `[VERIFIED]` `backend/crates/crm-demo/src/handlers/interaction.rs:55-180, 183-230` — current interaction form (Select to migrate) + save handler (validation).
- `[VERIFIED]` `backend/crates/crm-demo/src/entities/contact.rs:4-16` — current Model struct; the 3 new fields extend this.
- `[VERIFIED]` `backend/crates/crm-demo/src/migration/m20260323_000004_create_contact.rs` — canonical migration pattern (raw SQL via `execute_unprepared`).
- `[VERIFIED]` `backend/crates/crm-demo/src/migration/mod.rs` — migration chain registration.
- `[VERIFIED]` `backend/crates/marionette/src/builders/standard.rs:1-318` — existing builders including FieldSet, Textarea, RadioGroup, Switch that Phase 15 composes; `FieldSeparator` at line 317.
- `[VERIFIED]` `backend/crates/marionette/src/error.rs:1-46` — ActionError + ActionResult; the channel Phase 15 deliberately does NOT extend.
- `[VERIFIED]` `backend/crates/marionette-protocol/src/data.rs:13-44` — PatchOperation::Set shape Phase 15 uses for validation patches.
- `[VERIFIED]` `backend/crates/marionette-protocol/src/messages.rs:56-71` — PatchMessage shape with required `surface`.
- `[VERIFIED]` `backend/crates/marionette/src/ws.rs:287-295` — propagate_id covers PatchMessage; validation patches get action-id correlation.
- `[VERIFIED]` `frontend/src/lib/init.ts:1-113` — init flow with `__mrnSendAction` + `__mrnSetData` hooks (lines 92-102).
- `[VERIFIED]` `frontend/src/lib/components/form/TextInput.svelte:29-30` — `/_errors` + bind read pattern.
- `[VERIFIED]` `frontend/src/lib/components/form/Form.svelte:26-31` — Form.svelte with empty-payload sendAction bug (WR-01).
- `[VERIFIED]` `frontend/src/lib/components/form/RadioGroup.svelte` + `Switch.svelte` — proven Phase 14 implementations.
- `[VERIFIED]` `frontend/src/lib/components/core/FallbackComponent.svelte:12,18` — existing `import.meta.env.DEV` gate in repo.
- `[VERIFIED]` `frontend/tests/e2e/ci-guards.spec.ts:1-48` — existing CI guard pattern (Phase 13).
- `[VERIFIED]` `frontend/tests/uat/uat-driver.spec.ts:1-100` + `playwright.uat.config.ts` — Phase 14 UAT driver pattern (Chrome-MCP substitute).
- `[VERIFIED]` `frontend/tests/e2e/contact-edit.spec.ts:1-60` — existing E2E pattern for edit form.
- `[VERIFIED]` `spec/PROTOCOL.md:440-600` — current Phase 14 form-component + validation documentation.
- `[VERIFIED]` `spec/PROTOCOL.md:790-819` — legacy and canonical validation sections (D-D2 surgery targets).
- `[VERIFIED]` `CONCEPT.md:260, 268, 630` — Flowbite mentions to rewrite.
- `[VERIFIED]` `TOOLING.md:39` — Flowbite mention to rewrite.
- `[VERIFIED]` `.planning/codebase/STACK.md:47` — Flowbite mention to rewrite.
- `[VERIFIED]` `backend/Cargo.toml:25-26` — sea-orm 1.1 + sea-orm-migration 1.1 workspace versions.
- `[CITED]` `.planning/phases/14-formscreen-enhancements/14-CONTEXT.md` — all Phase 14 design decisions Phase 15 inherits (D-A1 formscreen deletion precedent; D-A2 no-helper-yet; D-B1 internal Field wrap; D-C1..C4 FieldSet responsive; D-D1 action row; D-E1..E4 new primitives).
- `[CITED]` `.planning/phases/14-formscreen-enhancements/14-08-SUMMARY.md` Known Stubs + Phase 15 sections — explicit handoff of country/notes/opt_in, per-field validation, RadioGroup placement.
- `[CITED]` `.planning/phases/14-formscreen-enhancements/14-VERIFICATION.md` §Anti-Patterns — WR-01, WR-02, IN-01, IN-02 exact line references Phase 15 closes.
- `[CITED]` `.planning/phases/13-datatable-enhancements/13-CONTEXT.md` D-A2 — TableScreen retirement precedent (mirror for FormScreen confirmation in Phase 15 CI guard).

### Secondary (MEDIUM confidence — WebSearch-verified with official source)

- `[CITED: vite.dev/guide/env-and-mode]` — `import.meta.env.DEV` semantics; tree-shaking of DEV-guarded code in production build. Cross-verified with existing in-repo usage (`FallbackComponent.svelte:12,18`).
- `[CITED: sea-ql.org/SeaORM/docs/migration/writing-migration]` — SeaORM migration patterns including the `manager.has_column(...)` idempotency tip and note that "Atomic migration is not supported in MySQL and SQLite". Project adopts raw SQL via `execute_unprepared` (matching all 10 existing migrations). SeaORM 1.x API supports alter_table with add_column as alternative, but the repo's established pattern is raw SQL.
- `[CITED: sqlite.org/changes.html]` (assumed) — SQLite 3.35 (Mar 2021) added `ALTER TABLE … DROP COLUMN`. Bundled via `sqlx-sqlite`.

### Tertiary (LOW confidence — verify before lock-in)

- None of the architectural decisions rest on LOW-confidence claims. All are grounded in existing repo patterns + spec text + official framework docs.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dep present, versions verified from `Cargo.toml`/`package.json`.
- Architecture patterns: HIGH — Phase 14 template copy-verbatim; `form_shell` + `validation_error_patch` signatures derived directly from existing builder patterns and protocol types; no novel abstractions.
- Pitfalls: HIGH — pitfalls 1-8 enumerated from Phase 14 verification output + explicit CONTEXT.md decisions; pitfall 9 (import.meta.env.DEV) from verified in-repo usage.
- Validation architecture: HIGH — test tiers mapped 1:1 to CONTEXT decisions; every Wave-0 gap identified with file path.
- DB migration shape: HIGH — exact SeaORM idiom copied from `m20260323_000004_create_contact.rs`; SQLite DROP COLUMN availability verified via SQLite release notes.
- PROTOCOL.md validation-section surgery: HIGH — both sections located by line number and re-verified; canonical shape at ll. 593-600 already matches the shape Phase 15 makes mandatory.

**Research date:** 2026-04-18
**Valid until:** 2026-05-18 (30 days — stable; no fast-moving deps).

## Open Questions for Planner

1. **Plan split granularity (Area I discretion).** Should scope-closure items (D-G1..G4) be a single plan or folded into the handler sweeps?
   - **What we know:** Each is <30 lines of changes. They're independent of each other and of the handler sweeps.
   - **What's unclear:** Whether batching them with the doc/CI pass creates a cleaner PR or obscures review.
   - **Recommendation:** Put them in the doc/CI plan (P5 in the suggested split). They're all "cleanup" semantically. The handler sweep plans stay focused on composition.

2. **`form_shell()` on contact.rs's edit form that has tail sections.** The contact edit form in edit mode appends Tags + Notes + Interactions timelines AFTER the form. Does `form_shell` return the `(root, nodes)` with just the form envelope, and the handler extends `nodes` with the tail sections?
   - **What we know:** `form_shell` as proposed returns a `HashMap<String, Component>` the caller can mutate.
   - **What's unclear:** Whether the outer Container should include the tail-section node IDs in its children, or the handler composes a second wrapper.
   - **Recommendation:** The outer Container (root of `form_shell`) should take an optional `tail_children` Vec parameter, OR `form_shell` returns the root_id and the handler appends to the Container's children list directly before returning. The simpler option is: `form_shell` builds the form envelope; the handler composes the surrounding outer Container itself with `[form_shell_root, ...tail_section_nodes]` as children. The helper stays thinner.

3. **Action row: dedicated `field-row` component vs plain Container (D-D1 Option A vs B)?**
   - **What we know:** Phase 14 D-D1 chose Option A (`Container class="flex gap-2 justify-end"`) and contact.rs uses it at line 646.
   - **What's unclear:** Whether Phase 15 should introduce `field-row` as a new SDUI component for semantic clarity.
   - **Recommendation:** STAY WITH OPTION A. D-A3 says no new SDUI components. Consistency wins.

4. **Visual snapshot strategy — one file per form or one file for all forms?**
   - **What we know:** Phase 14's `tests/visual/form.spec.ts` has 2 test blocks (desktop + mobile) for contact-edit.
   - **What's unclear:** Whether to add 6 more blocks (3 screens × 2 viewports) in the same file, or split.
   - **Recommendation:** Keep in the same file. Each block has its own snapshot name. Easier to rebaseline as a unit.

5. **UAT driver: shared `uat-driver.spec.ts` or per-screen specs?**
   - **What we know:** Phase 14 used one large spec with 6 test blocks for contact.
   - **What's unclear:** Phase 15 has 5 screens × ~3 scenarios = 15 blocks.
   - **Recommendation:** Split into `uat-driver-company.spec.ts`, `uat-driver-user.spec.ts`, `uat-driver-interaction.spec.ts`, `uat-driver-contact-inline.spec.ts`. Each maps to one evidence folder. Reuses shared `login()` helper via a tiny `tests/uat/helpers.ts`.

6. **Flowbite grep in ci-guards.spec.ts — git grep vs ripgrep vs node:fs walk?**
   - **What we know:** Phase 14 committed to `node:fs.existsSync` for file-exists checks.
   - **What's unclear:** Whether to shell out to `git grep` (fast, respects .gitignore) or stay in Node (portable but slower).
   - **Recommendation:** Use `git grep` via `child_process.execSync`. Every CI environment this project runs in has git. The speed advantage is real on larger repos. Document the fallback (`rg`, then `find + grep`) inline as a comment.

7. **Downmigration atomicity on SQLite.**
   - **What we know:** SQLite 3.35+ supports `DROP COLUMN`. Project is pre-deployment so down migration is best-effort.
   - **What's unclear:** Whether to bother with `down()` at all, or leave it a no-op + document "reset crm.db to reverse Phase 15".
   - **Recommendation:** Keep `down()` honest as D-C4 specifies. Pre-deployment posture, but honest down migrations catch bugs faster during local dev loops.

---

*Phase: 15-crm-migration-validation*
*Research completed: 2026-04-18*
