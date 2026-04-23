# Phase 18: Catalog Screens - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 18 ships **five curated catalog screens** that go beyond Phase 17's minimal leaf `gallery_demo()` siblings (2–3 representative instances per component) and exhaustively showcase the full variant surface for each component family:

1. **CAT-01 Buttons & Actions** — every Button variant × size × state (default/destructive/outline/ghost/link × sm/md/lg × normal/disabled/loading/icon-only) visible on one page.
2. **CAT-02 Forms** — every input type (text/select/checkbox/switch/radio/textarea) across every state (normal/disabled/error/focused/with-description), grouped with `FieldSet` + `FieldSeparator`, plus a **live validation patch-demo** exercising Phase 12 component-tree node patching.
3. **CAT-03 DataTable** — filter bar, virtualized infinite scroll, column visibility, and per-`ColumnKind` rendering seeded with ≥500 synthetic rows so virtualization actually engages.
4. **CAT-04 Feedback** — toast dispatch, confirm dialog flow, modal surface, and empty/loading/error placeholder states side-by-side and individually triggerable.
5. **CAT-05 Typography & tokens** — full text scale + lucide-svelte icon catalog + OKLCH swatches for every semantic token in `app.css`.

**What this phase is NOT:**

- **NOT extending leaf `gallery_demo()` fns.** Phase 17 leaves stay minimal (2–3 instances); catalog screens are purely additive nav entries that compose fresh via direct builder calls. Zero changes to `marionette/src/builders/*.rs` leaf fns.
- **NOT a framework-level change to DemoEntry or the macro.** No `group` field addition, no new attribute args, no change to iteration semantics. Phase 17 locked those.
- **NOT a new action-routing mechanism.** Catalog screens ride on the existing `gallery-show` handler exactly like leaves; interactive flows (form blur-validate, confirm open, toast fire) reuse the existing `gallery-demo/*` namespace with per-catalog-screen action names where needed.
- **NOT client-side form validation.** Formsnap is studied as a design reference for composition patterns only. Server-driven validation via Phase 12 node patching stays the model. (See `<decisions>` §CAT-02 Forms.)
- **NOT the W-06 ErrorDisplay `message` field fix.** Deferred to a follow-up plan; Phase 18 is purely additive and does not touch leaf demos or marionette builders.
- **NOT Phase 19 Exerciser screens or Phase 20 Live Token Editor.** Catalog (CAT-*) only.
- **NOT a shared seed helper unification.** Phase 17's `seed_table_rows()` in `gallery-demo/src/handlers/show.rs` is untouched; the new shared `fixtures.rs` generator is for CAT-03 + Phase 19 EXER-03 only.

</domain>

<decisions>
## Implementation Decisions

### Area 1 — Registration + nav placement

- **D-1-A: Catalog screens register via `#[gallery_demo]` inside the gallery-demo crate.** Each catalog fn lives in `backend/crates/gallery-demo/src/catalog/<family>.rs` annotated with `#[marionette_macros::gallery_demo(key = "catalog-<family>", name = "Catalog: <Family>")]`. Auto-discovery picks them up via the existing linkme-backed `registered_demos()` iterator. Pros: consistent mental model with leaves, reuses all Phase 16/17 machinery, new catalog screens need zero main.rs changes. Registration mechanism matches the leaf contract verbatim — pure-fn, explicit key, `Vec<Node>` return.

- **D-1-B: Nav stays flat alphabetical — grouping remains deferred.** At ~25 total v1.2 entries (5 catalog + 19 leaves + Home), flat alphabetical still reads. Phase 17 §deferred's `group: Option<&'static str>` addition is NOT opened in Phase 18. Phase 19/20 may revisit if EXER/THEME bloats nav further. Display-name prefix `Catalog: <Family>` clusters entries visually.

- **D-1-C: Demo keys use `catalog-<family>` prefix.** Exact keys: `catalog-buttons`, `catalog-forms`, `catalog-data-table`, `catalog-feedback`, `catalog-typography`. Alphabetically groups the 5 entries contiguously early in nav. Deep-link URL hashes use `#catalog-buttons` etc. Pros: predictable, semantic, works with flat sort.

- **D-1-D: Catalog screens fully reuse the `gallery-show` handler.** Handler lookup + render path is identical to leaves. Static seed data per screen lives in the existing `seed_for_key` match arm in `handlers/show.rs` (extended with catalog keys). Interactive flows (CAT-02 blur-validate, CAT-04 trigger-open) register their own dedicated actions under the `gallery-demo/*` namespace — e.g., `gallery-demo/catalog-forms/validate-<input>`. No new routing action; no parallel `catalog-show`.

### Area 2 — Catalog ↔ leaf demo relationship

- **D-2-A: Catalog screens coexist with leaf demos, unchanged.** Leaves stay at 2–3 instances per GALLERY-DEMOS.md coverage matrix (Phase 17 contract). Catalog screens add 5 new nav entries; leaf entries are untouched. Two distinct purposes: `button` leaf = "does the builder work?", `catalog-buttons` = "full variant × size × state story".

- **D-2-B: Catalog fns compose fresh via direct builder calls.** Zero calls into leaf `gallery_demo()` from catalog fns. Each catalog fn builds the full matrix from raw builder invocations (`Button::new(...).variant(...).size(...).build()` in loops or explicit enumerations). Pros: aligns with Phase 17 §domain ("catalog screens compose via direct builder calls, not via extended `gallery_demo()` fns"), keeps leaf contract stable.

- **D-2-C: Phase 18 does not touch any marionette leaf `gallery_demo()` fns.** All Phase 18 code lives in `backend/crates/gallery-demo/`. The only marionette-side touch is a potential small Form-component polish pass **inspired by formsnap** (see D-3-E below) — researcher decides whether that lands in Phase 18 or a separate plan. The W-06 ErrorDisplay `message` field dead-state is deferred.

- **D-2-D: Files organized as `gallery-demo/src/catalog/<family>.rs`.** New sub-module with one file per catalog screen: `buttons.rs`, `forms.rs`, `data_table.rs`, `feedback.rs`, `typography.rs`, plus `catalog/mod.rs` declaring them. Scales to Phase 19's `exerciser/` sub-module pattern.

### Area 3 — CAT-02 Forms live validation patch design

- **D-3-A: One live-validation story per input type — six total.** TextInput, Select, Checkbox, Switch, Radio, Textarea each get their own server-driven validation round-trip inside the catalog screen. Each input has a representative rule (TextInput: email format; Select: required non-empty; Checkbox/Switch: must-agree; Radio: required choice; Textarea: min-length). Maximally exercises the framework surface rather than a single email field.

- **D-3-B: Validation fires on **blur** (field loses focus).** Single-pass demo: type invalid → tab out → error appears via node patch; type correction → tab out → error clears via node patch. Responsive trigger. **Open question for researcher:** the current builders (`TextInput`, `SelectInput`, `Checkbox`, `Switch`, `RadioGroup`, `Textarea`) may not all emit a blur action. Researcher must determine whether (a) existing change-dispatch semantics cover "leave field", or (b) a new `.on_blur(ComponentAction)` method needs to be added to the affected builders + wired in the Svelte components. This directly affects the Phase 18 plan count.

- **D-3-C: Patches exercise all three Phase 12 component-tree ops across the six inputs.** Each input type's validation handler emits a different node-tree operation to showcase the full protocol surface:

  | Input     | Op             | Shape                                                                     |
  |-----------|----------------|---------------------------------------------------------------------------|
  | TextInput | `set-children` | FieldGroup children list swapped between `[input]` and `[input, error]`   |
  | Select    | `delete-node`  | Error node targeted directly by stable id                                 |
  | Checkbox  | `set-node`     | Dedicated per-field error node content swapped (empty Container ↔ error) |
  | Switch    | `set-node`     | Same pattern as Checkbox                                                  |
  | Radio     | `set-children` | FieldGroup children list swap (mirrors TextInput approach)                |
  | Textarea  | `delete-node`  | Mirrors Select pattern                                                    |

  Rotation keeps each op exercised twice for robustness; reader sees all three ops in one screen. **Researcher should validate this mapping** against what the current frontend surface store actually accepts per Phase 12 protocol semantics before the planner locks it.

- **D-3-D: Matrix layout is mobile-first — per-input Cards with responsive inner grid.** Outer layout: one shadcn `Card` per input type (6 Cards stacked vertically everywhere — naturally mobile-friendly). Inside each Card: a responsive Tailwind grid for the 5 state variants — `grid-cols-1 sm:grid-cols-2 lg:grid-cols-5`. On phone: state variants stack. On tablet: 2 columns. On desktop: 5 columns (one per state). The live-validate interactive section sits inside the same Card below the state grid. **Mobile must work** — this is a hard constraint.

- **D-3-E: Formsnap is a DESIGN REFERENCE, not a dependency.** Client-side schema validation would corrupt marionette's server-driven model and is explicitly rejected. However, formsnap's composition patterns (per-field error slot, `<Form.Field>` auto-wiring of name/id/aria, `<Form.FieldErrors>` anatomy, field-level description binding) are considered solid UX + accessibility work and may inspire improvements to marionette's Form/Field builders. **Researcher task:** study formsnap's composition anatomy at https://formsnap.dev and evaluate whether small improvements to the Rust `Form` / `FieldSet` / per-input Field wrapping (and the matching Svelte components) should bundle into Phase 18 — ideally **before** CAT-02 implementation so the catalog showcases the improved surface. If scope is meaningful, planner may carve out a dedicated Form-polish plan before the CAT-02 plan.

### Area 4 — CAT-03 DataTable synthetic data + shared generator

- **D-4-A: Row shape is generic synthetic.** Columns: `{ id: u64, name: String, email: String, status: Status, score: i32, joined_at: NaiveDate }` where `Status` is an enum `{ Active, Inactive, Pending }`. Domain-neutral — not tied to CRM's Contact shape. Exercises every `ColumnKind` naturally.

- **D-4-B: All available ColumnKinds exercised.** Column layout: `id` (Number), `name` (Text), `email` (Text), `status` (Badge with per-variant color), `score` (Number, right-aligned), `joined_at` (Date formatted), plus a trailing Actions column (DropdownMenu with Edit/Delete/Duplicate items firing noop actions → toast). This satisfies CAT-03's "per-`ColumnKind` rendering" criterion exhaustively. Column visibility toggle is demoed prominently with one or two columns initially hidden to encourage user interaction.

- **D-4-C: Row generator lives in a new shared module `gallery-demo/src/fixtures.rs` with param-driven row count.** Exports `pub fn synthetic_rows(n: usize) -> Vec<Row>` returning deterministic rows (seeded RNG so the same inputs always yield the same data — important for test stability). CAT-03 calls with `n = 500`; Phase 19 EXER-03 will call with `n = 10_000`. Single source of truth for data shape + generation across both phases. Phase 17's existing `seed_table_rows()` helper in `handlers/show.rs` (5 rows, object-map shape) stays **untouched** — unifying is deferred.

- **D-4-D: Rows deliver via virtualized `fetch-rows` pagination, not one-shot seed.** Matches Phase 13's server-driven infinite-scroll machinery exactly. Initial `gallery-show` Render seeds the first page (50 rows); subsequent scrolls dispatch the existing `fetch-rows` generic handler (registered in `handlers/mod.rs`) which slices the generator's output and returns the next page via standard DataTable data patches. This is the *reason* CAT-03 exists — to exercise Phase 13's virtualization end-to-end at meaningful scale. The `fetch-rows` handler in `handlers/fetch_rows.rs` may need a new source-dispatch arm matching `source = "catalog-synthetic-rows"` (or similar) that delegates to `fixtures::synthetic_rows(...)` with offset + limit.

### Claude's Discretion (CAT-01, CAT-04, CAT-05 not explicitly discussed)

These three catalog screens follow the standard approaches established by Areas 1–4 above; planner and researcher choose implementation details consistent with the established patterns.

- **CAT-01 Buttons & Actions:** Same per-family Card + responsive inner grid layout pattern as CAT-02 (D-3-D) applied to Button's variant × size × state matrix. 5 variants × 3 sizes × 4 states = 60 combinations. Expected layout: outer stack of per-variant Cards (or per-size Cards — planner chooses), inner responsive grid over the remaining two axes. Icon-only variant uses any lucide icon from the registered set. Loading-state variant exercises the button's spinner affordance (if one exists; researcher confirms — otherwise this becomes a small builder addition).

- **CAT-04 Feedback screen:** Side-by-side triggers + individual render of each feedback surface — toast (Fire-toast button + notes on queue behavior), confirm dialog (open trigger + structured ConfirmDialog per Phase 17 17-05 contract), modal (open trigger + compositional popup content per Phase 17 17-08 recipe in GALLERY-DEMOS.md §Popup composition), empty/loading/error placeholder states as static mini-Card examples (no trigger needed for placeholders). Layout: the same Card-stack + responsive inner grid pattern. The feedback screen does NOT fix W-06 — that remains deferred.

- **CAT-05 Typography & tokens:** Three sections stacked vertically as separate Cards:
  1. **Text scale** — every `Heading` level (1–6 if all supported) + Text size variants in one Card with visible typography samples.
  2. **Lucide icon catalog** — renders the 14 icons currently registered in `frontend/src/lib/registry/icons.ts` (plus any added in a future registry expansion). Simple responsive grid (`grid-cols-4 sm:grid-cols-6 lg:grid-cols-8`) with icon name label beneath each. Registering more icons is a gallery-side enhancement opportunity but is Claude's discretion — minimum bar is the currently-registered set.
  3. **OKLCH swatches** — every `--*` token from `frontend/src/app.css :root` rendered as a labeled swatch. 27 tokens in light theme. Grouped visually by family (base/sidebar/radius) or rendered flat — planner's call. Dark-theme preview is a stretch goal (Phase 20's job proper).

### Folded Todos

None — STATE.md §Pending Todos was empty.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-level intent

- `.planning/ROADMAP.md` §Phase 18 (lines 209–220) — goal (5 catalog screens composing built-ins into curated showcases), depends-on (Phase 17), 5 success criteria, UI hint: yes.
- `.planning/ROADMAP.md` §Progress (line 250–252) — Phase 18 and 19 are parallelizable after Phase 17; Phase 20 naturally follows 18 for stable catalog to iterate against.
- `.planning/REQUIREMENTS.md` §Catalog screens (lines 37–42) — CAT-01 through CAT-05 with explicit coverage requirements including the "live validation patch-demo" language for CAT-02 and "≥500 synthetic rows" for CAT-03.
- `.planning/REQUIREMENTS.md` §Out of Scope (lines 63–71) — third-party-component demos out; framework composition machinery for composite demos out; client-side schema validation not explicitly listed but conflicts with protocol model (D-3-E grounds the formsnap rejection).
- `.planning/PROJECT.md` §Key Decisions — "Pure fn() -> Node demo contract", "Gallery app as second demo alongside CRM". Both apply to catalog fns. Phase 18's catalog fns also satisfy the pure-fn contract verbatim.

### Phase 17 hand-off (LOCKED — do not re-litigate)

- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-CONTEXT.md` — full Phase 17 decision set. Especially:
  - **§domain** — "Phase 18 catalog screens compose via direct builder calls, not via extended `gallery_demo()` fns". D-2-B here is the direct continuation.
  - **§D-A1** — leaf hybrid density (2–3 representative instances per leaf). Catalog screens do NOT retroactively expand this contract.
  - **§D-C1** — flat alphabetical nav, grouping deferred. D-1-B here keeps that contract.
  - **§D-C3** — single `gallery-show` action routes every nav click. D-1-D here keeps that contract.
  - **§D-C4** — `gallery-demo/*` action namespace. Catalog-specific interactive actions follow the convention: `gallery-demo/catalog-<family>/<verb>`.
  - **§D-D2** — `/demo/{key}/...` bind-path convention. Catalog screens use `/demo/catalog-<family>/...` for their state.
  - **§D-Z1** — `DemoEntry.render: fn() -> Vec<Node>` signature (post-Phase 16.5 refactor). Catalog fns return `Vec<Node>` like leaves.
- `backend/crates/marionette/GALLERY-DEMOS.md` — the permanent author-facing demo contract. Catalog fns MUST follow the pure-fn rules (§Contract), use explicit `key = "catalog-<family>"` (D-1-C), and respect the bind-path convention (§Bind-path convention). The coverage matrix gets 5 new rows for catalog keys.
- `.planning/STATE.md` §Phase 18 hand-off + §Blockers/Concerns:
  - **W-06 ErrorDisplay `message` field** — deferred per D-2-C; NOT a Phase 18 deliverable.
  - **Toast global-overlay refactor** — deferred v1.3+.
  - **AppShell nestability blocker** — Phase 19 EXER-01 owns; not Phase 18.
- `.planning/PROJECT.md` §Key Decisions (v1.2 Phase 17 entries) — Popups are compositional (no `Modal::new` wrapper), ConfirmDialog is structured, demo bind-path alignment is a hard contract. All apply to CAT-04 Feedback screen directly.

### Phase 12 / 13 / 14 foundation (LOCKED — consumed verbatim)

- `.planning/milestones/v1.1-phases/12-protocol-node-patching-appshell/12-CONTEXT.md` — Phase 12 component-tree patch ops (`set-node`, `delete-node`, `set-children`). D-3-C assigns all three to the 6 CAT-02 inputs. Researcher validates op/input mapping against the frontend surface store's reactivity contract.
- `.planning/milestones/v1.1-phases/13-datatable-enhancements/` — DataTable filter bar + virtualized infinite scroll + column visibility + `fetch-rows` handler. D-4-D consumes this machinery directly.
- `.planning/milestones/v1.1-phases/14-formscreen-enhancements/14-CONTEXT.md` — Field anatomy, FieldSet grouping, Textarea/RadioGroup/Switch, description/full-width field flags. D-3-D composes the CAT-02 matrix on top of this surface. D-3-E potentially extends it.

### CAT-02 design reference (NOT a dependency)

- `https://formsnap.dev` — formsnap's composition anatomy. Studied for inspiration (per-field error slot, `<Form.Field>` auto-wiring, `<Form.FieldErrors>` pattern, `useFormField()` context) per D-3-E. Explicitly NOT adopted as a dependency. Researcher writes up what patterns could improve marionette's Form/Field builders without pulling client-side validation into the model.
- Phase 15 Plan 15-02's `validation_error_patch()` helper — not a reference for the *mechanism* of CAT-02's patches (D-3-C picks its own op mix), but a reference for *how error patches have been wired in the codebase before*.

### Frontend-side surfaces (CAT-05)

- `frontend/src/app.css` — 27 semantic tokens in `:root` (and mirrored in `.dark`). D-CAT-05 renders swatches for every one.
- `frontend/src/lib/registry/icons.ts` — 14 currently-registered lucide icons. D-CAT-05 minimum bar is this set; registry expansion is Claude's discretion for the planner.

### Code the phase touches

- **New files:**
  - `backend/crates/gallery-demo/src/catalog/mod.rs` — module declaration.
  - `backend/crates/gallery-demo/src/catalog/buttons.rs` — CAT-01 catalog fn.
  - `backend/crates/gallery-demo/src/catalog/forms.rs` — CAT-02 catalog fn + 6 on-blur validation handlers (under `gallery-demo/catalog-forms/validate-<input>` action names).
  - `backend/crates/gallery-demo/src/catalog/data_table.rs` — CAT-03 catalog fn; references the shared `fixtures.rs` generator.
  - `backend/crates/gallery-demo/src/catalog/feedback.rs` — CAT-04 catalog fn.
  - `backend/crates/gallery-demo/src/catalog/typography.rs` — CAT-05 catalog fn.
  - `backend/crates/gallery-demo/src/fixtures.rs` — shared `synthetic_rows(n: usize) -> Vec<Row>` generator (D-4-C).
- **Modified files:**
  - `backend/crates/gallery-demo/src/lib.rs` — declare new `catalog` + `fixtures` modules.
  - `backend/crates/gallery-demo/src/handlers/show.rs` — extend `seed_for_key` with 5 new match arms for `catalog-*` keys. CAT-03's arm calls `fixtures::synthetic_rows(50)` for the initial page.
  - `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` — add source-dispatch arm for CAT-03 (`source = "catalog-synthetic-rows"` or similar) that paginates the shared generator by offset + limit.
  - `backend/crates/gallery-demo/src/handlers/mod.rs` — register the 6 new `gallery-demo/catalog-forms/validate-<input>` actions plus any catalog-specific action the planner adds (e.g., feedback screen's trigger-open variants).
  - `backend/crates/marionette/GALLERY-DEMOS.md` — append 5 rows to the coverage matrix (`catalog-buttons`, `catalog-forms`, etc. all marked `yes`); add a Catalog-Screens section noting these are app-level showcases (not framework demos) and link the file-layout convention (§D-2-D). NO changes to the pure-fn contract, skip list, or recipe sections.
- **Potentially modified (D-3-E — researcher decides):**
  - `backend/crates/marionette/src/builders/text_input.rs`, `select.rs`, `checkbox.rs`, `switch.rs`, `radio_group.rs`, `textarea.rs` — if blur-action wiring is needed (D-3-B open question), add `.on_blur(ComponentAction)` methods.
  - Corresponding `frontend/src/lib/components/form/*.svelte` files — emit blur events to the dispatch layer.
  - Form/Field composition improvements inspired by formsnap — researcher drafts the scope.

### External library docs

- https://docs.rs/linkme/latest/linkme/ — consumed via Phase 16 API; no new usage in Phase 18.
- https://www.shadcn-svelte.com/docs/components/card — shadcn `Card` is the outer container for D-3-D and CAT-01/04/05.
- https://tailwindcss.com/docs/grid-column — responsive `grid-cols-*` breakpoint classes (used in D-3-D and implied for CAT-01/04/05).
- https://formsnap.dev — D-3-E design reference (NOT a dependency).

### Codebase intel

- `.planning/codebase/STRUCTURE.md` — `backend/crates/` layout. No structural change in Phase 18 (gallery-demo sub-module addition is in-crate).
- `.planning/codebase/CONVENTIONS.md` — `#![warn(clippy::pedantic)]`, snake_case modules, PascalCase builders. Applies to all new files.
- `.planning/codebase/TESTING.md` — UAT via Chrome MCP. Every catalog screen walked end-to-end through Chrome MCP, not a handed-off checklist (see feedback memory below).

### User preferences (global memory)

- `feedback_pre_deployment_no_backcompat.md` — **applies**: no back-compat shims. Catalog screens are fresh code; no deprecation aliases needed.
- `feedback_options_need_reasoning.md` — **applied throughout this discussion** (every gray area question presented reasoned options with pros/cons; framework-recipe-first check done for shadcn Card and Tailwind responsive grids).
- `feedback_no_handrolling_ui.md` — **applies**: use shadcn `Card`, shadcn `Accordion` (if planner picks it for CAT-05 instead of flat scrolling), Tailwind responsive grid classes, lucide icons from registry. Do not invent new UI primitives for catalog screens.
- `feedback_use_chrome_for_uat.md` — **applies hard**: every CAT-01 through CAT-05 screen is UAT-verified via Chrome MCP (click into each nav entry, verify all states render on desktop width, resize to mobile width, verify responsive reflow, trigger each interactive flow). NOT a walkthrough handed to the user.
- `feedback_shadcn_svelte_search_broken.md` — **applies**: use `shadcnSvelteListTool` / `shadcnSvelteGetTool` or WebFetch for shadcn docs; do not call `shadcnSvelteSearchTool` (hangs).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`backend/crates/gallery-demo/src/handlers/show.rs`** — existing `handle_gallery_show` + `seed_for_key` match dispatch. CAT-* keys slot in as new match arms; no handler signature change. `seed_table_rows()` 5-row helper stays for the `data-table` leaf demo; catalog uses the new shared generator.
- **`backend/crates/gallery-demo/src/handlers/fetch_rows.rs`** — Phase 13 + Phase 17 paginated row handler. Already wired; extend with a catalog source arm that calls `fixtures::synthetic_rows(offset..offset+limit)`.
- **`backend/crates/gallery-demo/src/handlers/mod.rs`** — `register_gallery_actions(ActionRouter) -> ActionRouter` is the single wiring point for new catalog-specific actions. Add 6+ new action lines per D-3-B.
- **`backend/crates/marionette/src/builders/`** — every builder needed for catalog composition exists (Button, TextInput, SelectInput, Checkbox, Switch, RadioGroup, Textarea, FieldSet, FieldSeparator, Form, DataTable, Modal {compositional via raw trees}, ConfirmDialog, Heading, Text, Grid, Container, ErrorDisplay). Phase 17's per-component-file refactor is in place. No new primitives needed unless D-3-E surfaces Form-polish additions.
- **`backend/crates/marionette/src/gallery.rs`** — `registered_demos()` iterator. Unchanged; catalog fns flow through it transparently.
- **shadcn `Card` component** — reused for D-3-D (and implied for CAT-01/04/05). Already available in `frontend/src/lib/components/ui/card` (Phase 10 + 11).
- **Tailwind responsive grid classes** — already in the codebase's allowed class set (see `frontend/src/app.css` safelist for `grid-cols-1` through `grid-cols-6` and their `md:` variants). If `sm:` or `lg:` variants with `grid-cols-5` aren't safelisted yet, a small safelist extension is needed — researcher confirms.

### Established Patterns

- **Pure-fn `gallery_demo()` contract** — catalog fns follow it verbatim (zero args, zero generics, no async, no I/O, `Vec<Node>` return, `#[cfg(feature = "gallery")]` gate). Macro enforces syntactically.
- **`/demo/{key}/...` bind-path convention** — catalog screens use `/demo/catalog-<family>/...` for their state paths. CAT-02 forms bind to `/demo/catalog-forms/text-value`, `/demo/catalog-forms/select-value`, etc.
- **`gallery-demo/*` action namespace** — catalog-specific actions follow the convention: `gallery-demo/catalog-forms/validate-text-input`, `gallery-demo/catalog-forms/validate-select`, etc.
- **Seed alignment as hard contract** — every bind path MUST have a matching `seed_for_key` arm writing to the same path. Phase 17 17-05/06 UAT discovered silent-failure from mismatches. Researcher + planner enforce.
- **Chrome MCP UAT** — the canonical verification path. Phase 18 UAT walks every catalog screen at multiple viewport widths.

### Integration Points

- **`backend/crates/gallery-demo/src/lib.rs`** — declare new `catalog` + `fixtures` modules (and re-export if needed for integration tests).
- **`backend/crates/gallery-demo/src/handlers/show.rs`** — extend `seed_for_key` with 5 new `catalog-*` match arms.
- **`backend/crates/gallery-demo/src/handlers/fetch_rows.rs`** — extend source-dispatch for the catalog synthetic-row source.
- **`backend/crates/gallery-demo/src/handlers/mod.rs`** — register 6+ new catalog-specific actions.
- **`backend/crates/marionette/GALLERY-DEMOS.md`** — extend coverage matrix with 5 catalog keys; add a catalog-screens section explaining they are app-level screens (not framework demos) and linking to the gallery-demo file convention.

</code_context>

<specifics>
## Specific Ideas

- **"Formsnap inspires, doesn't dominate."** User instruction (verbatim, 2026-04-23): "I don't want to corrupt our model in the sense that we do client side validation, what I like about this form component is that they also thought a lot about how to compose forms ... so maybe instead of using it directly is that we should take it as inspiration for improving our form component". The researcher studies formsnap's composition anatomy (how `<Form.Field>` wires name/id/aria, per-field error slot vs separate ErrorDisplay, `<Form.FieldErrors>` rendering) and the planner evaluates a small marionette Form/Field polish pass in Phase 18 — ideally before CAT-02 — without dragging client-side schema validation into the server-driven model.

- **"Mobile must work."** User callout on CAT-02 matrix layout (and transitively all catalog screens). D-3-D responsive grid is the answer — Tailwind breakpoint classes (`grid-cols-1 sm:grid-cols-2 lg:grid-cols-5`) let inner state-variant grids stack on phones. Chrome MCP UAT walks every screen at desktop + mobile widths.

- **"Exercise the framework, not mirror CRM."** User callout on CAT-02 patch design (verbatim 2026-04-23): "you keep mentioning the crm ... this is NOT about crm but about exercising and demoing marionette". D-3-C rotates all three Phase 12 ops across the 6 inputs to teach the full protocol surface — not because CRM does it that way, but because the catalog exists to showcase what marionette's protocol enables.

- **"One live-validation story per input type — maximally exercise the framework."** User chose the most exhaustive option for CAT-02's live validation. All 6 input types demonstrate a full blur → validate → patch round-trip. Forces the researcher to resolve the blur-action wiring open question (D-3-B) for all affected builders, not just TextInput.

- **"Catalog is additive, not replacing."** User locked the coexist-unchanged model for leaf ↔ catalog (D-2-A). Leaves keep their intentional minimalism; catalog adds the full matrix. Two distinct purposes.

- **"Fully reuse `gallery-show` — no parallel routing action."** D-1-D. The Phase 17 investment in auto-discovery + single-action nav routing is preserved verbatim. Catalog screens don't carve out their own routing path.

- **"Shared generator now, serves Phase 19 without rework."** D-4-C's `fixtures.rs` is extracted upfront (at 500 rows for CAT-03) so Phase 19 EXER-03 inherits directly at 10k rows without Phase 19 having to refactor. Deterministic row generation (seeded RNG) keeps test stability.

- **"Virtualized fetch-rows pagination — actually exercise Phase 13."** D-4-D. A one-shot seed of 500 rows would defeat the purpose of CAT-03; the catalog needs to exercise Phase 13's server-driven infinite-scroll code path at meaningful scale.

</specifics>

<deferred>
## Deferred Ideas

- **W-06 ErrorDisplay `message` field dead-state fix** — The Rust `ErrorDisplay` builder accepts a `message` positional arg but the Svelte `ErrorDisplay.svelte` reads errors only from `bind`. Fix options (remove the field, or wire it as bind-fallback when `getData(surface, bind)` is empty) are deferred out of Phase 18 per D-2-C. Candidate for a dedicated follow-up plan or Phase 19 polish pass.

- **Unifying Phase 17's `seed_table_rows()` with the new shared generator** — Phase 17's 5-row object-map helper in `handlers/show.rs` stays untouched (D-4-C, supports D-2-C "purely additive"). A future refactor could unify the `data-table` leaf demo + catalog + exerciser on the same generator.

- **Adopting formsnap as a dependency** — explicitly rejected per D-3-E and user instruction. Preserves marionette's server-driven validation model. Stays as design reference only.

- **Leaf-demo bind-path drift fixes discovered during catalog construction** — any mismatch between builder `.bind(...)` paths and `seed_for_key` arms for leaf demos stays deferred. Phase 18 is purely additive (D-2-C).

- **Grouping metadata on `DemoEntry`** — Phase 17 §deferred. Not opened in Phase 18 (D-1-B). Re-open for Phase 19 or 20 if nav bloats past readable.

- **Dynamic lucide icon search / full library scan** — CAT-05 ships with the 14 currently-registered icons at minimum. Expanding the registry is a gallery-side enhancement; dynamic full-library search (with fuzzy matching) is v1.3+.

- **Dark-theme preview pane for CAT-05 swatches** — Phase 20's Live Token Editor is the proper home for theme preview; CAT-05 ships light-theme swatches with dark-theme as stretch goal (Claude's discretion).

- **Tooltip / popover triggers in catalog screens** — shadcn-svelte has Tooltip + Popover primitives. Neither is currently in marionette's builder surface; adding them is out of Phase 18 scope.

- **GALLERY-LINT CI enforcement** — v1.3+ per REQUIREMENTS.md. Phase 18 coverage additions are documented in GALLERY-DEMOS.md, not CI-enforced.

- **Tabs / Accordion as layout primitive** — CAT-02 layout picked responsive Cards (D-3-D). Accordion could slot in for CAT-05 (icon catalog expandable by category) as Claude's discretion; Tabs are not currently in marionette's builder surface and adding them is out of scope.

- **Reviewed Todos (not folded)** — none. STATE.md §Pending Todos was empty.

</deferred>

---

*Phase: 18-catalog-screens*
*Context gathered: 2026-04-23*
