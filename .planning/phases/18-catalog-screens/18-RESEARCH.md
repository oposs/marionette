# Phase 18: Catalog Screens - Research

**Researched:** 2026-04-23
**Domain:** SDUI catalog-screen composition in a Rust/Axum + Svelte 5 SDUI framework
**Confidence:** HIGH (codebase evidence; all 10 open questions answered from source)

## Summary

Phase 18 ships five catalog screens in the existing `gallery-demo` crate that exhaustively showcase each component family's visual surface. Architecture is settled: each catalog fn is a `#[gallery_demo(key = "catalog-<family>")]` pure-fn returning `Vec<Node>` that composes raw Marionette builders. Wave work splits cleanly into one pre-flight Wave (framework polish — Button variant/size/loading/icon wiring, blur-action wiring on 4 inputs, Tailwind safelist, optional Form-polish pass) and five independent catalog waves, one per family.

The open questions surface **three framework gaps** that the planner must decide on:

1. **Button builder gap** — The Rust `Button` struct has `variant` + `size` fields but the frontend `Button.svelte` reads `props.color`, `props.outline`, and hardcodes size to `default` or `icon`. Also `loading` and `icon` are implemented in the Svelte wrapper but absent in the Rust builder. CAT-01's "every variant × size × state" is not satisfiable without framework-level wiring. (**HIGH risk** — pre-CAT-01 framework polish required.)
2. **Blur-action wiring asymmetry** — `TextInput` and `Textarea` already fire a blur action when `action.type === 'blur'`. The other four inputs (`Select`, `Checkbox`, `Switch`, `RadioGroup`) do NOT. D-3-B requires blur for all six. Planner must either (a) wire blur into the four missing Svelte components, or (b) redesign CAT-02's four interactive flows to fire on change instead of blur.
3. **Phase 12 node-tree-op mapping conflates two error mechanisms** — The existing error display path is **data-store-driven**: `{op: "set", path: "/_errors/{bind}", value: "message"}`. The frontend form components already react to this data path and render `<Field.Error>`. D-3-C's "set-children / set-node / delete-node" node-tree-op mapping describes a *different and unused* pathway. Both work, but the planner must pick one and treat the other as deferred or split across the six inputs intentionally for didactic purposes (which is D-3-C's stated intent).

**Primary recommendation:**

- Wave 0: Framework polish (Button fields + blur wiring + Tailwind safelist + fixtures.rs + optional Form-polish).
- Waves 1–5: CAT-01 through CAT-05, one per plan, each < 1 day. CAT-02 and CAT-04 are heavier (interactive handlers + feedback flow); CAT-01, CAT-03, CAT-05 are mostly composition.
- Entire phase: 6 plans.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Catalog screen composition (Vec<Node> builder calls) | Rust builders (`marionette` crate) | — | Pure-fn contract per DEMO-02; catalog fns live in `gallery-demo/src/catalog/` |
| Catalog screen registration (auto-discovery) | `marionette::gallery` registry | Linkme | Same pathway as leaf demos; no new machinery |
| Catalog routing (nav click → render) | `gallery-demo/src/handlers/show.rs` | — | Reuses `gallery-show` verbatim (D-1-D); new match arms in `seed_for_key` |
| Per-input blur validation handlers (CAT-02) | `gallery-demo/src/handlers/*.rs` | — | 6 new handlers under `gallery-demo/catalog-forms/validate-<input>` |
| Error rendering (per-field) | Frontend form components | `/_errors/{bind}` data store | Existing mechanism — Phase 15 `validation_error_patch` |
| DataTable virtualization (CAT-03) | Frontend `DataTable.svelte` | TanStack virtual-core | Engages once total_rows exceeds viewport; 500 rows proves it |
| DataTable server pagination (CAT-03) | `gallery-demo/src/handlers/fetch_rows.rs` | `fixtures::synthetic_rows` | New source-dispatch arm for `source = "catalog-synthetic-rows"` |
| Feedback interactions (CAT-04) | Existing gallery-demo handlers | — | Trigger-open buttons reuse `gallery-demo/modal-open` / `confirm-open` / `toast-fire` |
| Typography & tokens rendering (CAT-05) | Pure static render | Container.class + Tailwind | No interactive state — Container + inline-styled swatches |

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-1-A** Catalog screens register via `#[gallery_demo]` inside the gallery-demo crate. Each catalog fn lives in `backend/crates/gallery-demo/src/catalog/<family>.rs` with `#[marionette_macros::gallery_demo(key = "catalog-<family>", name = "Catalog: <Family>")]`.
- **D-1-B** Nav stays flat alphabetical; grouping remains deferred.
- **D-1-C** Demo keys: `catalog-buttons`, `catalog-forms`, `catalog-data-table`, `catalog-feedback`, `catalog-typography`.
- **D-1-D** Catalog screens fully reuse the `gallery-show` handler; static seed data per screen lives in `seed_for_key`. Interactive flows register dedicated actions under `gallery-demo/*` namespace — e.g., `gallery-demo/catalog-forms/validate-<input>`. No parallel routing action.
- **D-2-A** Catalog screens coexist with leaf demos, unchanged.
- **D-2-B** Catalog fns compose fresh via direct builder calls; zero calls into leaf `gallery_demo()` fns.
- **D-2-C** Phase 18 does not touch any marionette leaf `gallery_demo()` fns. Only possible marionette-side touch: small Form-component polish inspired by formsnap (D-3-E). W-06 ErrorDisplay `message` dead-state deferred.
- **D-2-D** Files organized as `gallery-demo/src/catalog/<family>.rs` with `mod.rs` declaring them.
- **D-3-A** One live-validation story per input type — six total (TextInput email, Select required, Checkbox must-agree, Switch must-agree, Radio required, Textarea min-length).
- **D-3-B** Validation fires on **blur** (field loses focus).
- **D-3-C** Patches exercise all three Phase 12 component-tree ops across the six inputs — TextInput: `set-children` on FieldGroup; Select: `delete-node` on error node; Checkbox: `set-node` on per-field error slot; Switch: `set-node` on per-field error slot; Radio: `set-children` on FieldGroup; Textarea: `delete-node` on error node.
- **D-3-D** Matrix layout mobile-first — per-input Cards with responsive inner grid `grid-cols-1 sm:grid-cols-2 lg:grid-cols-5`. Mobile must work.
- **D-3-E** Formsnap is a DESIGN REFERENCE only, not a dependency. No client-side validation.
- **D-4-A** Row shape: `{ id: u64, name: String, email: String, status: Status, score: i32, joined_at: NaiveDate }` with `Status = { Active, Inactive, Pending }`.
- **D-4-B** ColumnKinds exercised: `id` (Number), `name` (Text), `email` (Text), `status` (Badge), `score` (Number right-aligned), `joined_at` (Date) + trailing Actions column (DropdownMenu Edit/Delete/Duplicate). Column visibility toggle with 1–2 columns initially hidden.
- **D-4-C** Row generator at `backend/crates/gallery-demo/src/fixtures.rs`: `pub fn synthetic_rows(n: usize) -> Vec<Row>` deterministic (seeded RNG). CAT-03 uses `n = 500`; Phase 19 EXER-03 uses `n = 10_000`. Phase 17's `seed_table_rows()` stays untouched.
- **D-4-D** Rows delivered via virtualized `fetch-rows` pagination. Initial `gallery-show` seeds page 1 (50 rows); sentinel triggers `fetch-rows` handler which slices generator output by offset + limit. Source dispatch arm for `source = "catalog-synthetic-rows"`.

### Claude's Discretion

- **CAT-01 Buttons & Actions**: per-family Card + responsive inner grid layout (same as CAT-02 D-3-D) applied to Button's variant × size × state matrix. 5 variants × 3 sizes × 4 states = 60 combinations. Icon-only variant uses any lucide icon. Loading-state variant exercises the button's spinner affordance (if it exists; researcher confirms).
- **CAT-04 Feedback**: side-by-side triggers per feedback surface — toast, confirm, modal, plus empty/loading/error placeholder mini-Card examples. Does NOT fix W-06.
- **CAT-05 Typography & tokens**: three sections as separate Cards — text scale (every Heading level + Text variants), lucide-svelte icon catalog (14 icons minimum from the current registry, displayed `grid-cols-4 sm:grid-cols-6 lg:grid-cols-8`), OKLCH swatches (27 tokens from `:root` in `app.css`).

### Deferred Ideas (OUT OF SCOPE)

- W-06 ErrorDisplay `message` field dead-state fix (Phase 18 polish or follow-up plan).
- Unifying Phase 17's `seed_table_rows()` with the new shared generator.
- Adopting formsnap as a dependency (REJECTED — design reference only).
- Leaf-demo bind-path drift fixes discovered during catalog construction.
- Grouping metadata on `DemoEntry` (Phase 17 §deferred).
- Dynamic lucide icon search / full library scan.
- Dark-theme preview pane for CAT-05 swatches (Phase 20's job).
- Tooltip / popover triggers in catalog screens (not in builder surface).
- GALLERY-LINT CI enforcement (v1.3+).
- Tabs / Accordion as layout primitive in catalog screens.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CAT-01 | Buttons & Actions screen renders every Button variant × size × state | §Framework Gap 1 (Button wiring) + §Standard Stack + §Code Examples §CAT-01 |
| CAT-02 | Forms screen — every input × every state + live validation patch-demo via node patching | §Framework Gap 2 (blur wiring) + §Framework Gap 3 (op mapping) + §Code Examples §CAT-02 + §Pitfall 4 + §Pitfall 5 |
| CAT-03 | DataTable filter + virtualized infinite scroll + column vis + per-ColumnKind with ≥500 rows | §Standard Stack + §Code Examples §CAT-03 + §Pitfall 8 + §fixtures.rs design |
| CAT-04 | Feedback screen — toast, confirm, modal, empty/loading/error placeholders | §Code Examples §CAT-04 + §Pitfall 7 |
| CAT-05 | Typography & tokens — text scale + lucide icon catalog + OKLCH swatches | §Code Examples §CAT-05 + §Standard Stack |

## Project Constraints (from CLAUDE.md & global memory)

- **Home dir is enormous** — never use `find /home/oetiker`. Use `cargo metadata`, `Glob`, or targeted paths.
- **No hand-rolled UI** (global feedback `feedback_no_handrolling_ui.md`) — use shadcn `Card`, `DropdownMenu`, `Badge`, Tailwind responsive grid classes, lucide icons. Do not invent new UI primitives.
- **Chrome MCP for UAT** (global `feedback_use_chrome_for_uat.md`) — every CAT-01 through CAT-05 screen is UAT-verified via Chrome MCP at desktop + mobile widths; not handed off to user as a walkthrough.
- **shadcn-svelte tooling** — use `shadcnSvelteListTool` / `shadcnSvelteGetTool` or WebFetch. Do NOT call `shadcnSvelteSearchTool` (hangs).
- **No back-compat shims** — pre-deployment; fix root causes.
- **Options need reasoning** — every gray area gets pros/cons/rationale; check framework recipes before inventing custom designs.

## Standard Stack

### Core (Rust workspace)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `marionette` | path dep, `features = ["gallery"]` | Builder primitives + DemoEntry + registry | Established by Phase 16 + 17 — verified in `Cargo.toml:9` |
| `marionette-protocol` | path dep | `Component`, `ComponentAction`, `PatchOperation`, `ProtocolMessage` | Verified in `backend/crates/marionette-protocol/src/` |
| `marionette-macros` | path dep | `#[gallery_demo]`, `#[derive(ComponentBuilder)]`, `#[action]`, `#[requires]` | Verified in `backend/crates/marionette-macros/src/lib.rs` |
| `chrono` | 0.4 (workspace) | `NaiveDate` for synthetic row `joined_at` | Already in workspace (`backend/Cargo.toml:29`); add to gallery-demo Cargo.toml |
| `serde_json` | 1 (workspace) | Seed-value construction | Already used throughout gallery-demo |

**Installation (diff to gallery-demo/Cargo.toml):**

```toml
[dependencies]
# existing...
chrono.workspace = true  # <-- add for NaiveDate in fixtures
```

No `rand` crate needed — `fixtures::synthetic_rows` uses a seeded LCG by hand (deterministic, ~10 LOC, zero dep weight). **HIGH confidence** — verified `rand` is not in the workspace today.

### Supporting (Frontend)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@lucide/svelte` | 1.8.0 (verified `frontend/package.json:19`) | CAT-05 icon catalog | 7278 icon files = ~1800 icons available via `@lucide/svelte/icons/<kebab>` |
| shadcn `Card` | vendored (`frontend/src/lib/components/ui/card/`) | Outer container per D-3-D | Already available in codebase |
| shadcn `Badge` | vendored (`frontend/src/lib/components/ui/badge/`) | ColumnKind::Badge rendering (used by DataTable.svelte) | Already wired |
| shadcn `Skeleton` | vendored (`frontend/src/lib/components/ui/skeleton/`) | CAT-04 "loading" placeholder | Already available |
| Tailwind | 4.2.0 | Responsive grid classes, utility styling | Already in use |

### Existing Rust Builders (all catalog needs)

| Builder | Type string | Props (abridged) | File |
|---------|-------------|------------------|------|
| `Button` | `button` | label, variant, size, disabled, bind, action | `marionette/src/builders/button.rs` |
| `TextInput` | `text-input` | label, placeholder, required, input_type, disabled, description, full_width | `.../text_input.rs` |
| `Select` | `select` | label, options, required, placeholder, disabled, description, full_width | `.../select.rs` |
| `Checkbox` | `checkbox` | label, disabled, description, full_width | `.../checkbox.rs` |
| `Switch` | `switch` | label, disabled, description, full_width | `.../switch.rs` |
| `RadioGroup` | `radio-group` | label, options (Vec<RadioOption>), required, disabled, description, full_width | `.../radio_group.rs` |
| `Textarea` | `textarea` | label, placeholder, rows, required, disabled, description, full_width | `.../textarea.rs` |
| `Form` | `form` | submit_label | `.../form.rs` |
| `FieldSet` | `field-set` | legend, description, cols | `.../field_set.rs` |
| `FieldSeparator` | `field-separator` | (none — structural) | `.../field_separator.rs` |
| `DataTable` | `data-table` | columns (Vec<TableColumn>), page_size, total_rows, filters (Vec<Filter>), row_id_key, source | `.../data_table.rs` |
| `ConfirmDialog` | `confirm-dialog` | title, message, confirm_label, cancel_label, cancel_action, destructive | `.../confirm_dialog.rs` |
| `Container` | `container` | class (+ frontend reads `card`, `padding`, `class`) | `.../container.rs` |
| `Grid` | `grid` | cols, gap | `.../grid.rs` |
| `Heading` | `heading` | text, level (1–6) | `.../heading.rs` |
| `Text` | `text` | text | `.../text.rs` |
| `Spinner` | `spinner` | size (sm/md/lg) | `.../spinner.rs` |
| `ErrorDisplay` | `error-display` | message (currently dead — bind-driven on frontend) | `.../error_display.rs` |

### Framework Gaps Requiring Pre-CAT Work

| Gap | Current state | Minimum fix | Blocks | Notes |
|-----|--------------|-------------|--------|-------|
| **Gap 1 — Button wiring** | Rust has `variant: Option<String>` + `size: Option<String>` but `Button.svelte` reads `props.color` + `props.outline` + ignores size (except `icon` derivation from `!label && icon`). No `loading` or `icon` fields in the Rust builder. | Add `loading: Option<bool>`, `icon: Option<String>`, `aria_label: Option<String>` to Rust `Button`; rewrite `Button.svelte` to read `variant` directly (mapping to shadcn variant enum: default/destructive/outline/ghost/link/secondary) and `size` directly (default/xs/sm/lg/icon/icon-xs/icon-sm/icon-lg). Drop the `color`/`outline` derivation. | CAT-01 (all 60 matrix cells) | **HIGH risk — pre-deployment posture means no back-compat; all leaf Button demos must also be rechecked.** No CRM callers use color/outline-as-derivation because CRM is Rust-builder-driven; grep confirms no `.color(` or `.outline(` usages in Rust. |
| **Gap 2 — Blur wiring on 4 inputs** | `TextInput.svelte` + `Textarea.svelte` fire `sendAction(action.name, {value}, action.target)` when `action?.type === 'blur'` and the blur event fires on the underlying input. `SelectInput.svelte`, `Checkbox.svelte`, `Switch.svelte`, `RadioGroup.svelte` do NOT wire any blur-action dispatch. | Add parallel `handleBlur` + wiring to the 4 missing Svelte components. For Select: hook `onOpenChange(false)` as the blur signal (Select has no DOM blur in the closed state — use popover close as the semantic equivalent). For Checkbox/Switch/Radio: hook their underlying shadcn primitive's `onblur` handler if exposed, else attach a `<div onfocusout>` wrapper. | CAT-02 blur-validate flows for Select, Checkbox, Switch, Radio | **MEDIUM risk** — planner MUST investigate bits-ui's component API for each; the blur event may not bubble out of shadcn primitives the same way a native `<input>` does. Alternative if blur is hard: switch CAT-02 to on-change validation for these 4 (Checkbox/Switch are boolean-change, Select/Radio are value-change). |
| **Gap 3 — Tailwind safelist** | `frontend/src/app.css:7` safelist: `grid-cols-1..6 md:grid-cols-1..6`. **No `sm:` or `lg:` variants.** | Extend safelist to include `sm:grid-cols-1..6 lg:grid-cols-1..8` (CAT-05 icon grid uses `lg:grid-cols-8`). Or: since Tailwind v4 also scans source files, any `.rs` file that emits these classes should be scanned — but the `@source inline(...)` directive is the bulletproof path. | CAT-02 matrix (D-3-D `grid-cols-1 sm:grid-cols-2 lg:grid-cols-5`), CAT-05 icon grid | **LOW risk** — trivial 1-line edit. |
| **Gap 4 — Form/Field polish (optional per D-3-E)** | Current `<Field.Field>` anatomy already implements per-field error slot (`<Field.Error>`), description (`<Field.Description>`), and label (`<Field.Label>`). Field wrappers use a mount-time UUID fallback id. No Svelte context for auto-wiring `aria-describedby` between Description/Error. | **Option A (RECOMMEND — skip):** Current shadcn Field anatomy already covers formsnap's composition concerns via markup conventions. Improvements would be cosmetic. **Option B (opt-in):** Introduce a `<Field.Provider>` Svelte context that exposes `fieldId`, `descriptionId`, `errorId` to descendants so `aria-describedby` stitches automatically. Affects all 7 form Svelte components. | CAT-02 quality bar | **LOW risk** — recommend punting to v1.3. CAT-02's requirement (`FieldSet` + `FieldSeparator` + live validation) is satisfied by the current markup. |

## Architecture Patterns

### System Architecture Diagram

```
                                ┌─────────────────────────────────────────┐
                                │  Nav click in gallery's sidebar         │
                                │  (payload: { key: "catalog-<family>" }) │
                                └──────────────────┬──────────────────────┘
                                                   │ sendAction('gallery-show', ...)
                                                   ▼
      ┌──────────────────────────────────────────────────────────────────┐
      │  handle_gallery_show (gallery-demo/src/handlers/show.rs)         │
      │  1. Look up entry by key in registered_demos()                   │
      │  2. Invoke entry.render() → Vec<Node>                            │
      │  3. Call seed_for_key(key) → initial JSON state                  │
      │  4. Emit Render{surface=content, root, nodes_map, data}          │
      └───────┬──────────────────────────────────────────┬───────────────┘
              │                                          │
              │ (render path)                            │ (seed path)
              │                                          ▼
              │                            Surface data store (/demo/...)
              │                                          │
              ▼                                          │
      ┌─────────────────────────────┐                   │
      │  catalog_<family>()         │                   │
      │  (pure fn,                  │                   │
      │   returns Vec<Node>)        │                   │
      │                             │                   │
      │  builds outer Card-stack +  │                   │
      │  inner responsive grid via  │                   │
      │  Container+class / builders │                   │
      └──────────┬──────────────────┘                   │
                 │                                      │
                 ▼                                      │
      ┌─────────────────────────────────┐               │
      │  Frontend NodeRenderer          │◀──────────────┘
      │  resolves component_type → *.svelte via registry/defaults.ts
      │  reads bind values from surface store
      │  renders shadcn primitives     │
      └──────────┬─────────────────────┘
                 │
                 │ (user interaction — blur, click, scroll)
                 ▼
      ┌──────────────────────────────────────────────────────────────────┐
      │  Interactive actions under gallery-demo/* namespace              │
      │  • gallery-demo/catalog-forms/validate-<input>  (CAT-02 blur)    │
      │  • gallery-demo/modal-open / confirm-open / toast-fire  (CAT-04) │
      │  • fetch-rows with source="catalog-synthetic-rows"  (CAT-03)     │
      └──────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
backend/crates/gallery-demo/
├── src/
│   ├── main.rs                       (unchanged)
│   ├── lib.rs                        (ADD: pub mod catalog; pub mod fixtures;)
│   ├── home.rs                       (unchanged)
│   ├── state.rs                      (unchanged)
│   ├── fixtures.rs                   NEW (CONTEXT §D-4-C)
│   ├── catalog/
│   │   ├── mod.rs                    NEW (pub mod buttons; forms; data_table; feedback; typography;)
│   │   ├── buttons.rs                NEW (CAT-01)
│   │   ├── forms.rs                  NEW (CAT-02)
│   │   ├── data_table.rs             NEW (CAT-03)
│   │   ├── feedback.rs               NEW (CAT-04)
│   │   └── typography.rs             NEW (CAT-05)
│   └── handlers/
│       ├── mod.rs                    (+6 action registrations for CAT-02)
│       ├── show.rs                   (+5 seed_for_key arms for catalog-*)
│       ├── fetch_rows.rs             (+source-dispatch arm for catalog-synthetic-rows)
│       └── catalog_forms.rs          NEW (6 validate-<input> handlers)
```

### Pattern 1: Catalog fn skeleton (applies to all 5)

**What:** Every catalog fn is a pure `fn() -> Vec<Node>` annotated with `#[cfg(feature = "gallery")]` and `#[marionette_macros::gallery_demo(key = "catalog-<family>", name = "Catalog: <Family>")]`.

**Example (source: adapted from `backend/crates/marionette/src/builders/form.rs` Phase 17 Plan 17-04 composite pattern):**

```rust
// backend/crates/gallery-demo/src/catalog/buttons.rs
#![allow(clippy::too_many_lines)]

use marionette::builders::{Button, Container, Heading};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-buttons", name = "Catalog: Buttons")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // Per-variant Card with inner responsive grid over (size × state).
    // Outer: stack of 5 Cards (one per variant).
    // Inner: grid-cols-1 sm:grid-cols-3 lg:grid-cols-4 — 3 sizes × 4 states.
    let variants = ["default", "destructive", "outline", "ghost", "link"];
    let cards: Vec<Node> = variants
        .iter()
        .flat_map(|v| build_variant_card(v))
        .collect();
    let title = Heading::new("Buttons & Actions — full matrix")
        .id("catalog-buttons-title")
        .level(1)
        .build();
    // ... assemble all Cards + title into a Container root
    Container::new()
        .id("catalog-buttons-root")
        .class("flex flex-col gap-6 p-6")
        .children({
            let mut kids = vec![title];
            kids.extend(cards);
            kids
        })
        .build_with_children()
}
```

Notes:
- Gallery-demo crate has `marionette = { features = ["gallery"] }` — the `#[cfg(feature = "gallery")]` gate lets the fn compile. Consistent with Phase 17 leaf contract.
- `#[must_use]` is required by the macro's signature check (per Phase 16 hand-off + `GALLERY-DEMOS.md`).
- Direct builder calls (no nesting of leaf `gallery_demo()` — that's D-2-B).

### Pattern 2: Card-stack + responsive inner grid (D-3-D)

**What:** Each catalog screen uses an outer Container stacking per-family Cards (one Card per variant/family); each Card has a responsive Tailwind grid inside that breaks 1-col (mobile) → N-col (desktop).

**Source:** Container has a `class: Option<String>` prop that appends to the wrapper div's class list (verified in `frontend/src/lib/components/layout/Container.svelte:38`). Tailwind applies whatever you pass. Card styling via Tailwind utilities: `rounded-lg border bg-card text-card-foreground shadow-sm p-6`.

**Example (inner grid):**

```rust
// Inside build_variant_card(), an inner Container with responsive grid:
let inner = Container::new()
    .id(format!("catalog-buttons-{}-grid", variant))
    .class("grid grid-cols-1 sm:grid-cols-3 lg:grid-cols-4 gap-3")
    .children(matrix_cells)  // 3 sizes × 4 states = 12 cells for this variant
    .build_with_children();
```

**Anti-pattern:** Do NOT use the Marionette `Grid` builder with `cols: Option<u8>` for responsive breakpoints — `Grid.svelte` uses `inline-style "grid-template-columns: repeat(N, 1fr)"` (verified `Grid.svelte:38`), which gives a **fixed** column count at all widths. The Tailwind `grid-cols-*` responsive path lives in Container's `class` prop.

### Pattern 3: Container-as-Card via class prop (D-3-D)

**What:** There is NO Marionette `Card` builder. Container has a `card: boolean` prop in the Svelte component that wraps content in `<Card.Root>` (verified `Container.svelte:33`), but it also applies `max-w-md` and centers vertically — wrong for catalog layout.

**Workaround (RECOMMENDED):** Skip `card: true` and emulate a Card with Container + Tailwind class:

```rust
Container::new()
    .id(format!("catalog-buttons-card-{}", variant))
    .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
    .children(vec![variant_title, inner_grid])
    .build_with_children()
```

This matches the shadcn Card visual appearance without the centering/width constraints of `Container.svelte`'s `card=true` branch. `bg-card` and `text-card-foreground` are OKLCH tokens already in `:root`.

**Alternative:** Introduce a Rust `Card` builder (new struct with `#[component(type = "card")]`) that maps to shadcn Card primitives on the frontend. **Cost:** +1 new builder + +1 frontend component + +1 registry entry + +1 gallery_demo sibling (for DEMO-01 convention). **Benefit:** Cleaner API. **Recommendation:** punt to v1.3 unless CAT-04 specifically needs it for placeholder mini-Cards.

### Pattern 4: Responsive Container class pattern (Tailwind)

**What:** Safelist the breakpoint prefixes so Tailwind's JIT actually compiles them.

**Fix in `frontend/src/app.css:7`:**

```css
@source inline("grid-cols-1 grid-cols-2 grid-cols-3 grid-cols-4 grid-cols-5 grid-cols-6 grid-cols-7 grid-cols-8 md:grid-cols-1 md:grid-cols-2 md:grid-cols-3 md:grid-cols-4 md:grid-cols-5 md:grid-cols-6 sm:grid-cols-1 sm:grid-cols-2 sm:grid-cols-3 sm:grid-cols-4 sm:grid-cols-5 sm:grid-cols-6 lg:grid-cols-1 lg:grid-cols-2 lg:grid-cols-3 lg:grid-cols-4 lg:grid-cols-5 lg:grid-cols-6 lg:grid-cols-7 lg:grid-cols-8");
```

Plus `rounded-lg border bg-card text-card-foreground shadow-sm` classes (all used by existing shadcn components) will be picked up automatically via content scanning. **Only the dynamic `sm:grid-cols-N`/`lg:grid-cols-N` classes** need safelisting because they are emitted from Rust and never appear in a Svelte source file that Tailwind's content scanner reads.

### Pattern 5: Deterministic fixtures generator (D-4-C)

**What:** A pure function that returns `n` deterministic synthetic rows using a seeded LCG; zero external-crate cost.

**Example (`backend/crates/gallery-demo/src/fixtures.rs`):**

```rust
//! Shared synthetic-row generator for CAT-03 (n=500) and Phase 19 EXER-03 (n=10_000).
//!
//! Deterministic: same `n` always yields same rows (seeded LCG). No dependency on
//! `rand` — keeps gallery-demo crate-weight minimal.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status { Active, Inactive, Pending }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub status: Status,
    pub score: i32,
    pub joined_at: NaiveDate,
}

/// Generate `n` deterministic synthetic rows. Same `n` → same rows.
#[must_use]
pub fn synthetic_rows(n: usize) -> Vec<Row> {
    // Linear-congruential generator (Numerical Recipes constants).
    // Not cryptographic — purely for stable pseudo-random sequence.
    let mut state: u64 = 0x1234_5678_9ABC_DEF0;
    let mut rng = || {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        state
    };
    let first_names = ["Alice", "Bob", "Carol", "Dan", "Eva", "Frank", "Gina", "Henry",
                       "Iris", "Jack", "Kara", "Leo", "Maya", "Noah", "Olive", "Paul"];
    let last_names  = ["Baker", "Chen", "Davis", "Evans", "Frost", "Gomez", "Hale", "Iqbal"];
    let statuses    = [Status::Active, Status::Inactive, Status::Pending];
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");

    (1..=n as u64)
        .map(|id| {
            let f = first_names[(rng() as usize) % first_names.len()];
            let l = last_names[(rng() as usize) % last_names.len()];
            let st = statuses[(rng() as usize) % statuses.len()].clone();
            let sc = (rng() % 1000) as i32;
            let days = (rng() % 700) as i64;  // ~2 years spread
            let joined = base_date + chrono::Duration::days(days);
            Row {
                id,
                name: format!("{f} {l}"),
                email: format!("{}.{}@example.com", f.to_lowercase(), l.to_lowercase()),
                status: st,
                score: sc,
                joined_at: joined,
            }
        })
        .collect()
}
```

**Tests to add:** `synthetic_rows(500).len() == 500`; `synthetic_rows(500)[0] == synthetic_rows(500)[0]` (stability); `synthetic_rows(500)[0].id == 1`.

### Pattern 6: fetch-rows source dispatch extension (D-4-D)

**Current `fetch_rows.rs` (verified `gallery-demo/src/handlers/fetch_rows.rs`):** no source dispatch; returns 5 hardcoded rows ignoring payload. Must be extended.

**Minimum diff:**

```rust
#[derive(serde::Deserialize)]
struct FetchRowsPayload {
    source: String,
    #[serde(default)]
    offset: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 { 50 }

pub async fn handle_demo_fetch_rows(ctx: HandlerContext) -> ActionResult {
    let payload: FetchRowsPayload = serde_json::from_value(
        ctx.action.payload.clone().unwrap_or_default(),
    ).map_err(|e| ActionError::BadPayload(format!("fetch-rows payload invalid: {e}")))?;

    let (path, rows) = match payload.source.as_str() {
        "demo-rows" => ("/demo/data-table/rows", five_hardcoded_rows()),
        "catalog-synthetic-rows" => {
            let start = payload.offset as usize;
            let end = (start + payload.limit as usize).min(500);
            let rows = crate::fixtures::synthetic_rows(500);
            let slice = rows.get(start..end).unwrap_or(&[]).to_vec();
            let json_rows = slice.into_iter()
                .map(|r| serde_json::to_value(r).expect("Row serialization"))
                .collect();
            ("/demo/catalog-data-table/rows", json_rows)
        }
        other => return Err(ActionError::BadPayload(format!(
            "unknown fetch-rows source: {other}"
        ))),
    };

    let ops: Vec<PatchOperation> = rows.iter()
        .filter_map(|row| {
            let id = row.get("id")?.as_u64()?;
            Some(PatchOperation::Set {
                path: format!("{path}/{id}"),
                value: row.clone(),
            })
        })
        .collect();

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}
```

**Pattern matches CRM's `fetch_rows.rs`** (verified lines 120-131) — dispatch via `payload.source.as_str()`.

### Pattern 7: CAT-02 live-validation handler (D-3-B + D-3-C + Gap 3)

**Two distinct mechanisms — pick one per input per D-3-C:**

#### 7a — Existing data-store error path (RECOMMENDED for all 6 inputs)

Uses `validation_error_patch` from `marionette::validation`. The frontend form components (`TextInput.svelte:30`, `Textarea.svelte:30`, `SelectInput.svelte:32`, `Checkbox.svelte:28`, `Switch.svelte:26`, `RadioGroup.svelte:29`) **all already** read `getData(surface, '/_errors' + bind)` and render `<Field.Error>` when non-empty. Add a single data op per blur:

```rust
// Error case: write "Enter a valid email" to /_errors/demo/catalog-forms/text-value
pub async fn validate_text_input(ctx: HandlerContext) -> ActionResult {
    let value = ctx.action.payload.clone()
        .and_then(|p| p.get("value").cloned())
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let error_msg = if value.contains('@') && value.contains('.') { None }
                    else { Some("Enter a valid email address.") };

    let op = match error_msg {
        Some(m) => PatchOperation::Set {
            path: "/_errors/demo/catalog-forms/text-value".into(),
            value: serde_json::Value::String(m.into()),
        },
        None => PatchOperation::Set {
            path: "/_errors/demo/catalog-forms/text-value".into(),
            value: serde_json::Value::String(String::new()),  // empty = cleared
        },
    };
    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: vec![op],
    })])
}
```

#### 7b — Node-tree ops (D-3-C's specified approach)

D-3-C mandates `set-children` / `delete-node` / `set-node` rotated across the 6 inputs. This is a **different mechanism** than 7a — it mutates the component tree rather than the data store. The catalog screen would have to set up per-input "error slot" containers as siblings of each input (initially empty Containers), then mutate those slot containers on blur.

**The catch:** the frontend form components' error rendering is **bound to the data store** (`/_errors/{bind}`), NOT to tree children of the input. A node-tree op that inserts an `ErrorDisplay` node into a sibling slot WILL render, but it will render the `ErrorDisplay` component — NOT the shadcn `<Field.Error>` slot inside the input's own `<Field.Field>` wrapper. Visually they differ.

**Recommendation to the planner:** Keep D-3-C's didactic intent but be explicit about the two error-rendering pathways:

| Input | Op | Error slot | Rendered as |
|-------|------|-----------|-------------|
| TextInput | set-children on FieldGroup | Add/remove a sibling `ErrorDisplay` node | `ErrorDisplay.svelte` |
| Select | delete-node on ErrorDisplay node | Start with ErrorDisplay in tree; delete on valid | `ErrorDisplay.svelte` |
| Checkbox | set-node on per-field error slot | Swap empty Container ↔ Text/Error component at the slot node id | Component swap |
| Switch | set-node on per-field error slot | Same as Checkbox | Component swap |
| Radio | set-children on FieldGroup | Add/remove a sibling `ErrorDisplay` | `ErrorDisplay.svelte` |
| Textarea | delete-node on ErrorDisplay | Same as Select | `ErrorDisplay.svelte` |

This mapping DOES exercise all three Phase 12 ops (verified in `backend/crates/marionette-protocol/src/data.rs:13-44`). The `<Field.Error>` data-store path stays unused for these 6 demo inputs; the catalog is showing off a DIFFERENT node-patching pathway that's equally valid but less commonly used in production. **Call this out in the catalog screen's explainer Text so the viewer understands what they're seeing.**

### Pattern 8: Catalog keys and bind paths (D-1-C + Phase 17 D-D2)

```
catalog-buttons             /demo/catalog-buttons/*          (no bind — mostly static)
catalog-forms               /demo/catalog-forms/*            (6 input binds + 6 error slots)
catalog-data-table          /demo/catalog-data-table/rows    (collection — fetch-rows target)
catalog-feedback            /demo/catalog-feedback/*         (no bind — triggers only)
catalog-typography          /demo/catalog-typography/*       (no bind — static)
```

Actions:
- `gallery-demo/catalog-forms/validate-text-input`
- `gallery-demo/catalog-forms/validate-select`
- `gallery-demo/catalog-forms/validate-checkbox`
- `gallery-demo/catalog-forms/validate-switch`
- `gallery-demo/catalog-forms/validate-radio`
- `gallery-demo/catalog-forms/validate-textarea`
- Plus existing: `gallery-demo/toast-fire`, `gallery-demo/modal-open`, `gallery-demo/confirm-open`, `gallery-demo/confirm-accept`, `gallery-demo/confirm-reject`, `fetch-rows` (for CAT-04 and CAT-03).

### Anti-Patterns to Avoid

- **DO NOT** extend leaf `gallery_demo()` fns (D-2-B, D-2-C).
- **DO NOT** introduce a new "catalog-show" routing action (D-1-D says reuse `gallery-show`).
- **DO NOT** use the Marionette `Grid` builder for responsive breakpoint grids — it uses inline `grid-template-columns: repeat(N, 1fr)` (fixed at all breakpoints). Use Container with `class="grid grid-cols-1 sm:grid-cols-X lg:grid-cols-Y"`.
- **DO NOT** use the `card: true` Container prop (limits width to `max-w-md` and centers vertically). Emulate a card with Container + Tailwind classes.
- **DO NOT** bypass the seed-alignment contract: every bind path has a matching `seed_for_key` arm (Phase 17 Plan 17-05 + 17-06 taught this the hard way — G-05).
- **DO NOT** try to client-side validate (D-3-E — explicitly REJECTED).
- **DO NOT** introduce a Card builder in Phase 18 unless CAT-04 forces it — Container + class covers the visual need.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| "Card container" visual | Custom Card SDUI component | Container + `class="rounded-lg border bg-card text-card-foreground shadow-sm p-6"` | Matches shadcn Card visual; zero new primitives; OKLCH tokens already defined |
| Responsive grid layout | Custom breakpoint resolver | Container + `class="grid grid-cols-1 sm:grid-cols-N lg:grid-cols-M"` (safelisted) | Tailwind JIT does the math; Grid builder is fixed-col by design |
| Deterministic PRNG for 500 rows | `rand` + `rand_chacha` | ~10 LOC LCG in `fixtures.rs` | Zero new crate dep; deterministic across runs (critical for tests); `rand` isn't in workspace today |
| Virtualization engine | Custom scroll-observer | Existing `createRuneVirtualizer` in `frontend/src/lib/utils/virtualizer.svelte.ts` | Already wired inside `DataTable.svelte`; CAT-03 only needs row volume ≥500 to exercise it |
| Blur-event handling on form inputs | Custom focus-tracking store | Native `onblur`/`onfocusout` DOM events bridged via `sendAction('action-name', {value}, target)` | TextInput/Textarea already do this; extend same pattern to 4 remaining Svelte components |
| Per-field error rendering | Custom error-collector | Existing `<Field.Error>` in `frontend/src/lib/components/ui/field/field-error.svelte` (or ErrorDisplay component) | Two pathways already exist; pick one per input per D-3-C |
| Icon rendering | Custom SVG loader | `@lucide/svelte/icons/<kebab-name>` via existing `frontend/src/lib/registry/icons.ts` | 14 already registered; extending to a catalog set is a 1-file edit |
| OKLCH swatch | Custom color-picker widget | Inline-styled `<div>` with `background: oklch(...)` | Static tokens in `app.css:9-37`; pure display |

**Key insight:** 95% of Phase 18 is composition over existing primitives. The only framework additions are (1) Button wiring gaps (variant/size/loading/icon fields), (2) optional blur-action wiring on 4 form components, and (3) Tailwind safelist extension.

## Runtime State Inventory

Phase 18 is **purely additive** (no rename, no refactor). No stored data, live service config, OS-registered state, secrets, or build artifacts are affected by the new catalog screens. The one existing runtime state item that would shift if we unified the table-row generator — Phase 17's `seed_table_rows()` — is explicitly preserved untouched per D-4-C. **Nothing found in any category — verified by grep for file touches in CONTEXT.md §code_context.**

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compile | ✓ | Workspace edition 2024 | — |
| `chrono` | fixtures.rs NaiveDate | ✓ (workspace) | 0.4 | Could use `String` ISO dates instead |
| `@lucide/svelte` | CAT-05 icons | ✓ (frontend/node_modules) | 1.8.0 | — |
| shadcn Card primitives | Visual reference | ✓ (`frontend/src/lib/components/ui/card/`) | vendored | — |
| shadcn Skeleton | CAT-04 loading placeholder | ✓ (`frontend/src/lib/components/ui/skeleton/`) | vendored | — |
| Chrome MCP | UAT | Assumed available per global feedback | — | User-walk handoff if unavailable (but per global: MUST use Chrome MCP) |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** `chrono` is redundant if we serialize dates as ISO strings directly — but it's already in the workspace, so no reason to avoid it.

## Open Questions (RESOLVED)

### Q1 — Blur-action wiring (D-3-B)

**Answer: HIGH-confidence partial.** `TextInput.svelte` and `Textarea.svelte` already emit a blur action when `action?.type === 'blur'` (verified lines 45-56 of each). The other 4 components (`SelectInput.svelte`, `Checkbox.svelte`, `Switch.svelte`, `RadioGroup.svelte`) do NOT. The Rust `.action(ComponentAction)` method accepts any `ComponentAction`; a `ComponentAction` with `r#type: "blur"` is constructible by hand today (no builder helper).

**Minimum diff:**
- **Rust side:** add `ComponentAction::blur(name)` constructor to `marionette-protocol/src/component.rs` for symmetry (or skip — hand-constructing is fine).
- **Svelte side (4 files):** add a `handleBlur` handler parallel to TextInput's. For `Checkbox.svelte`/`Switch.svelte` — attach `onblur` to the shadcn primitive element if it propagates (verify); else wrap in `<div onfocusout={handleBlur}>`. For `SelectInput.svelte` — hook `onOpenChange(false)` (semantic blur is popover close). For `RadioGroup.svelte` — wrap RadioGroup in `<div onfocusout={handleBlur}>`.

**Risk:** MEDIUM. Svelte's event propagation through bits-ui primitives may surface gotchas. Plan a 30-min spike with Chrome MCP per component.

### Q2 — Phase 12 op assignment validity (D-3-C)

**Answer: HIGH confidence.** All three Phase 12 node-tree ops are supported by the frontend surface store (verified `frontend/src/lib/store/data.svelte.ts:82-110`):
- `set` (data op)
- `set-node` (replaces/creates component at id)
- `delete-node` (removes from adjacency)
- `set-children` (replaces children array)
- `insert-child`
- `remove-child`

**BUT**: the existing form-input Svelte components read errors from `/_errors/{bind}` in the **data store**, not from tree children. D-3-C's node-tree-op mapping is valid **only** when the error is rendered by a SEPARATE `ErrorDisplay` node that is a sibling of the input — NOT via the input's own `<Field.Error>` slot (which needs a data op).

**Two implementation options for CAT-02 (planner picks):**

| Option | How | Trade-off |
|--------|-----|-----------|
| **A: Data op for all 6** | Each validate-<input> handler emits a single `Set{path:"/_errors{bind}", value: msg}` (or empty string to clear). Existing `<Field.Error>` renders. | Uniform, tested mechanism, but doesn't exercise node-tree ops as D-3-C intends |
| **B: Node-tree ops per D-3-C spec** | Catalog tree has a sibling `ErrorDisplay` or empty Container for each input; handlers emit `set-children` / `delete-node` / `set-node` per input. | Matches D-3-C's didactic intent. Requires pre-mounting error slots for `set-node` targets (Checkbox/Switch). Visually different from production (error appears as sibling, not in Field.Error slot). |

**Recommendation:** Option B, but with an explainer Text above the CAT-02 matrix saying "This screen exercises Marionette's Phase 12 node-tree patching ops. The visible error surface (ErrorDisplay) is rendered as a sibling node to each input — a different mechanism than the in-field Field.Error slot used by production form handlers. Both are supported." This honors D-3-C and exercises the framework rather than mirroring the data-op pathway that production uses.

**Pre-mount requirement for `set-node` (Checkbox, Switch):** The error-slot node MUST exist in the tree before the first `set-node` op, else `set-node` is interpreted as CREATE (per frontend docs line 92-93). The catalog render places empty Containers at the slot IDs, ready to be swapped for an ErrorDisplay on first invalid blur and back to empty Container on valid blur.

### Q3 — Tailwind safelist (`sm:grid-cols-*` / `lg:grid-cols-*`)

**Answer: HIGH confidence.** Current safelist in `frontend/src/app.css:7`:

```
@source inline("grid-cols-1 grid-cols-2 grid-cols-3 grid-cols-4 grid-cols-5 grid-cols-6 md:grid-cols-1 md:grid-cols-2 md:grid-cols-3 md:grid-cols-4 md:grid-cols-5 md:grid-cols-6");
```

**`sm:` and `lg:` prefixes are NOT included.** CAT-02 D-3-D requires `sm:grid-cols-2 lg:grid-cols-5`. CAT-05 icon grid likely wants `sm:grid-cols-6 lg:grid-cols-8`. CAT-01 inner grid over (size × state) might use `sm:grid-cols-3 lg:grid-cols-4`.

**Required edit in Wave 0:** extend the `@source inline(...)` directive. See §Pattern 4 above for the verbatim extended string.

### Q4 — DataTable fetch-rows source dispatch (D-4-D)

**Answer: HIGH confidence.** Current `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` (verified lines 13-40) ignores the `source` payload entirely; returns 5 hardcoded rows. No dispatch arm.

**Minimum diff:** parse `FetchRowsPayload { source, offset, limit }` (mirroring `crm-demo/src/handlers/fetch_rows.rs:38-52`), match on `source.as_str()`, keep `"demo-rows"` as the existing 5-row arm, add `"catalog-synthetic-rows"` arm delegating to `fixtures::synthetic_rows(500)` with offset/limit slicing. Full diff shape in §Pattern 6.

Note the current gallery handler emits to `/demo/data-table/rows/{id}` hardcoded; the extended handler must emit to `/demo/catalog-data-table/rows/{id}` for the new source. **The path prefix must match the catalog screen's `DataTable::new().bind("/demo/catalog-data-table/rows")` — otherwise fetched rows land in the wrong collection.**

### Q5 — Button "loading" state

**Answer: HIGH confidence.** The Rust `Button` struct (`backend/crates/marionette/src/builders/button.rs:11-19`) has `label`, `variant`, `size`, `disabled` — **NO `loading` field, NO `icon` field, NO `ariaLabel` field.** The Svelte `Button.svelte` (verified `frontend/src/lib/components/form/Button.svelte:30-31`) IMPLEMENTS both: `isIconOnly = !props.label && !!props.icon` and `isLoading = !!props.loading` (shows `<Loader2 class="size-4 animate-spin" />`).

So the loading-state affordance exists at the Svelte layer but **is unreachable from a Rust `Button` builder** today. CRM never invokes loading state because no CRM Rust handler calls `Button::new(...).loading(true)` (no such method exists).

**Minimum fix (framework addition):**
1. Add 3 optional fields to Rust `Button` struct: `loading: Option<bool>`, `icon: Option<String>`, `aria_label: Option<String>`.
2. Also fix variant pass-through: frontend reads `props.color === 'red'` and `props.outline` — rewrite `Button.svelte:24-28` to read `variant` directly. Map:
   - `variant = "default"` / unset → shadcn `default`
   - `variant = "destructive"` → shadcn `destructive`
   - `variant = "outline"` → shadcn `outline`
   - `variant = "ghost"` → shadcn `ghost`
   - `variant = "link"` → shadcn `link`
   - `variant = "secondary"` → shadcn `secondary` (bonus)
3. Similarly fix size pass-through: frontend line 59 hardcodes `size={isIconOnly ? 'icon' : 'default'}`. Rewrite to honor `props.size` when set, falling back to the icon-only derivation when absent.

**This is a Wave 0 framework-polish task**, not a CAT-01-specific task. Risk: **HIGH** — leaf `Button` demo in `backend/crates/marionette/src/builders/button.rs:26-42` uses `.variant("destructive")` which currently works by coincidence (sets `props.variant = "destructive"` which frontend ignores; frontend renders default color); after Wave 0 Button polish, this demo should actually render destructive. Re-UAT the leaf demo after Wave 0.

### Q6 — Formsnap composition anatomy

**Answer: MEDIUM confidence (WebFetch returned summary).** Formsnap's composition pattern relies on:
1. `<Field name="...">` establishes a naming context via Svelte context.
2. `<Control>` snippet receives pre-wired props (`id`, `aria-describedby`, `aria-invalid`) via context.
3. `<FieldErrors />` renders per-field errors scoped to its parent `<Field>`.
4. `useFormField()` context hook exposes field state + constraints to descendants.

**What Marionette's form components already have:**
- `<Field.Field>` from shadcn renders the container (verified `frontend/src/lib/components/ui/field/field.svelte`).
- `<Field.Label>`, `<Field.Description>`, `<Field.Error>` render the labeled parts.
- Each form Svelte component generates a `fieldId` (mount-time UUID fallback, verified `TextInput.svelte:26`) and passes it to `<Field.Label for={fieldId}>` + `id={fieldId}` on the input.
- `aria-invalid={hasError || undefined}` is wired per-component.

**What's missing vs. formsnap:**
- No Svelte context for descendants to pick up `fieldId` automatically. Each form Svelte component re-derives it locally. (Fine — simpler.)
- No automatic `aria-describedby` stitching between Description/Error and the input.

**Recommendation:** **SKIP** a Form-polish wave in Phase 18. The current `<Field.Field>` shadcn anatomy already gives 80% of formsnap's composition benefit; the remaining 20% (auto-stitched aria-describedby, context-driven field id) is minor accessibility polish that's out of scope for a "show the existing surface" catalog. If the planner disagrees, the scope is:
- Add a `<FieldContext>` Svelte context provider in the builder composition chain.
- Modify 7 form Svelte components to consume context when available.
- Estimate: 1 plan, ~2 days.

**Gate:** the planner may promote this to Wave 0 if CAT-02 UAT surfaces accessibility concerns. Otherwise punt to v1.3.

### Q7 — shadcn Card usage in the codebase

**Answer: HIGH confidence.** shadcn Card primitives ARE vendored (`frontend/src/lib/components/ui/card/` contains `card.svelte`, `card-header.svelte`, etc. — verified by `ls`). The `Container.svelte` component reads `props.card` as a boolean and wraps in `<Card.Root>` when `true` (verified line 33-46) — **but adds `max-w-md` + centers vertically**, wrong for full-width catalog layouts.

**No Rust `Card` builder exists.** No SDUI `'card': Card` registry entry exists in `frontend/src/lib/registry/defaults.ts` (verified). So catalog screens cannot emit a `card` component type directly.

**Resolution:** Use Container + Tailwind classes (§Pattern 3 above) to visually emulate a Card. If a Card builder is desired later, it's a framework addition (new Rust struct + new Svelte component + new registry entry + new gallery_demo sibling) — out of Phase 18 scope.

### Q8 — Lucide icon registry expansion path

**Answer: HIGH confidence.** `frontend/src/lib/registry/icons.ts` currently registers 14 icons (verified). The `@lucide/svelte` package ships ~1800 icons (7278 dist files / 4 per icon = ~1800; verified via `ls node_modules/@lucide/svelte/dist/icons | wc -l`). Import pattern: `import X from '@lucide/svelte/icons/<kebab-name>';`.

**Expansion path for CAT-05:**
- **Minimum (locked per CONTEXT)**: ship with the existing 14 icons.
- **Discretion upgrade**: register ~50-100 common icons covering core UI intents (actions, navigation, status, I/O) — a single `icons.ts` edit. No framework change.
- **Far future**: dynamic full-library scan with fuzzy search — v1.3+ only (CONTEXT §deferred).

**Recommendation:** ship with 14 icons for Phase 18 per the locked minimum. Nothing prevents expanding later.

### Q9 — Phase 13 DataTable filter bar + column visibility

**Answer: HIGH confidence.** Rust surface (verified `backend/crates/marionette/src/builders/data_table.rs`):
- `DataTable::new(columns: Vec<TableColumn>)` — struct with `page_size`, `total_rows`, `filters`, `row_id_key`, `source`.
- `.filter(Filter::text("id").label("Label").placeholder("..."))` — hand-written append setter (line 212-222).
- `Filter` enum variants: `Text { id, label, placeholder, span }`, `Select { id, label, options, span }`, `DateRange { id, label, span }` — serialized as tagged union keyed by `kind`.
- `TableColumn::new(key, label).kind(ColumnKind::Badge).hidden_default(true)` — chainable.
- `ColumnKind`: `Text`, `Badge`, `Actions`, `Date`, `Number` (verified enum at line 67-79).

Frontend surface (verified `frontend/src/lib/components/table/DataTable.svelte`):
- Filter bar auto-renders from `props.filters` (lines 376-428).
- Column visibility via DropdownMenu (lines 431-448); columns with `hidden_default: true` start hidden, user toggles via the dropdown.
- `Actions` column expects each row to have `actions: [{label, action}, ...]` inside the row data — verified `DataTableActions.svelte` component.

**CAT-03 columns (per D-4-B):**

```rust
let columns = vec![
    TableColumn::new("id", "ID").kind(ColumnKind::Number),
    TableColumn::new("name", "Name"),  // Text is default
    TableColumn::new("email", "Email"),
    TableColumn::new("status", "Status").kind(ColumnKind::Badge).hidden_default(true),  // start hidden
    TableColumn::new("score", "Score").kind(ColumnKind::Number),
    TableColumn::new("joined_at", "Joined").kind(ColumnKind::Date),
    TableColumn::new("actions", "").kind(ColumnKind::Actions).hidden_default(true),  // start hidden
];

let filters = vec![
    Filter::text("name-search").label("Name").placeholder("Filter by name..."),
    Filter::select("status-filter").label("Status").options(vec![
        SelectOption { value: "active".into(), label: "Active".into() },
        SelectOption { value: "inactive".into(), label: "Inactive".into() },
        SelectOption { value: "pending".into(), label: "Pending".into() },
    ]),
    Filter::date_range("joined-range").label("Joined"),
];
```

**Important:** the frontend filter bar is PURELY LOCAL (verified `DataTable.svelte:123` comment "NOT /bind round-trip per D-C4"). Filters don't trigger server round-trips for pagination. The virtualized fetch-rows sentinel fires unconditionally; filters act client-side. CAT-03 will exercise both independently.

**Actions column rows:** each row in the `catalog-synthetic-rows` output needs an `actions: [{...}]` array for the `ColumnKind::Actions` column. Add to `fixtures::Row` as `actions: Vec<ActionItem>` OR append actions in the fetch_rows handler before emitting:

```rust
// In fetch_rows handler, per row:
let mut row_json = serde_json::to_value(&row)?;
row_json["actions"] = serde_json::json!([
    {"label": "Edit",      "action": {"type": "click", "name": "gallery-demo/noop"}},
    {"label": "Delete",    "action": {"type": "click", "name": "gallery-demo/noop"}},
    {"label": "Duplicate", "action": {"type": "click", "name": "gallery-demo/noop"}},
]);
```

### Q10 — Virtualization engagement threshold

**Answer: MEDIUM confidence.** The TanStack virtualizer engages on every render (verified `frontend/src/lib/utils/virtualizer.svelte.ts` uses the headless `Virtualizer` from `@tanstack/virtual-core`); it always computes only the visible window. The benefit is visible **whenever total_rows exceeds the viewport capacity** — at the default `estimateSize: () => 40` and a 400px scroll container (verified `DataTable.svelte:461`), that's ~10 rows visible at once.

**500 rows is plenty to prove virtualization works** — you can scroll, see ~10 rows at a time, and the IntersectionObserver sentinel triggers `fetch-rows` dispatches page-by-page (page_size default 50). With 500 rows → 10 fetch-rows dispatches to saturate the collection. Phase 19 EXER-03 (10k rows) gives 200 dispatches and exercises performance baselines.

**Row-height rendering:** Default `estimateSize` is 40px (verified virtualizer.svelte.ts line 27). If the CAT-03 rows are visually taller (multi-line name/email) they'll render at the actual height but the sentinel logic uses the estimate for the initial offset calculations — minor visual thrash at first scroll is acceptable.

**Recommendation:** CAT-03 ships with `page_size: Some(50)`, `total_rows: Some(500)` (so the sentinel idles after saturation). UAT at desktop AND mobile viewport widths — the 400px scroll container height is viewport-independent but row rendering responsiveness matters.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | [ASSUMED] Chrome MCP is available for UAT (global feedback says to use it) | §Wave Structure §UAT | Planner falls back to user-handoff UAT (against global policy) |
| A2 | [ASSUMED] `onblur` attaches cleanly to shadcn Checkbox/Switch elements via the wrapper element | Gap 2 | Spike during Wave 0; fall back to `<div onfocusout>` wrapper |
| A3 | [ASSUMED] `onOpenChange(false)` on bits-ui Select is a semantically acceptable blur equivalent | Gap 2, Q1 | Spike during Wave 0; may need to attach onblur to trigger element instead |
| A4 | [ASSUMED] Adding `loading`/`icon`/`aria_label` fields to `Button` struct doesn't require new macros (existing `#[derive(ComponentBuilder)]` handles optional fields) | Gap 1, Q5 | Verified by reading `marionette-macros/src/component_builder.rs:170-206` — optional setters generated from `#[builder(optional)]` |
| A5 | [ASSUMED] Node-tree ops on pre-mounted error slot Containers work without re-Render (set-node mutates in place) | Q2, Pattern 7b | Verified by frontend `data.svelte.ts:92-93` comment — set-node creates or replaces at id |
| A6 | [ASSUMED] `chrono` NaiveDate serializes as ISO string via serde | Pattern 5 | Standard chrono+serde behavior; confirmed by Cargo.toml feature `features = ["serde"]` |
| A7 | [ASSUMED] Tailwind v4 `@source inline("...")` directive behavior works for `sm:` and `lg:` prefixed classes identically to `md:` | Gap 3, Pattern 4 | Standard Tailwind v4 JIT; 100% reliable. |

## Common Pitfalls

### Pitfall 1: Seed-alignment contract (Phase 17 G-05 regression)

**What goes wrong:** A bind path in a catalog component doesn't have a matching `seed_for_key` arm — the field renders as empty/disabled/invisible because `getData()` returns `undefined`/`""`/`false` and the frontend's guards hide the component.

**Why it happens:** Phase 17 taught this through 3 bugs. Catalog screens will have ~10+ new binds each; each one MUST be seeded in `seed_for_key`.

**How to avoid:** Co-locate the bind path and seed in a single pass. For every `.bind("/demo/catalog-forms/X")` call, immediately add the corresponding entry to `seed_for_key("catalog-forms")`. The initial seed should be an **explicit empty/default value** (empty string, `false`, `null` — NOT `undefined`/missing).

**Warning signs:** A field renders without value on first visit, no error but silent empty. Chrome MCP UAT: click into the screen, verify EVERY field has its default value visible.

### Pitfall 2: `grid-cols-N` class compilation

**What goes wrong:** Tailwind JIT doesn't see `sm:grid-cols-2` or `lg:grid-cols-5` emitted from Rust — those classes never appear in Svelte source files and aren't in the safelist. Result: browser shows 1-column on all breakpoints regardless of code intent.

**How to avoid:** Extend the `@source inline(...)` directive in `app.css:7` (Wave 0 task). See §Pattern 4.

**Warning signs:** Mobile and desktop render identically at breakpoints. DevTools shows the class present on the element but no CSS rule applied.

### Pitfall 3: Marionette `Grid` builder is fixed-column

**What goes wrong:** Planner uses `Grid::new().cols(5)` expecting responsive behavior; ships a layout that's 5-col at 320px phone widths, unreadable.

**Why:** Grid.svelte (verified line 38-40) sets inline `grid-template-columns: repeat(N, 1fr)` — CSS wins over any Tailwind `grid-cols-*` class. There's no breakpoint logic.

**How to avoid:** Use Container with `class="grid grid-cols-1 sm:grid-cols-N lg:grid-cols-M"`, not the `Grid` builder, when responsiveness matters. Grid's still fine for fixed-column layouts (Home tiles on desktop).

### Pitfall 4: Phase 12 node-tree ops need pre-mounted slots

**What goes wrong:** CAT-02's Checkbox/Switch use `set-node` to swap the error-slot content. If the slot node ID doesn't exist in the tree before the first `set-node`, the op creates the component at that id but doesn't wire it into any parent's children. Result: component exists in the adjacency list but is orphaned (garbage-collected by the D-A8 gc pass per `data.svelte.ts:109`).

**How to avoid:** The initial catalog render for CAT-02 MUST include empty `Container` nodes at each slot id (`catalog-forms-checkbox-error-slot`, `catalog-forms-switch-error-slot`) as children of the Card. First `set-node` replaces the component AT that id, still a child of the Card.

**Warning signs:** `set-node` op applied but error never visually appears.

### Pitfall 5: Node-tree ops vs. data-ops for errors (D-3-C)

**What goes wrong:** Planner mixes the two error-rendering pathways. Some input has both a data-store `/_errors/{bind}` entry AND a sibling ErrorDisplay from a node-tree op — two errors rendered for one failure.

**How to avoid:** For CAT-02, pick Option A (data-ops only) or Option B (node-tree ops only per D-3-C) — not both. §Pattern 7 discusses the trade-off. RECOMMEND Option B with an explainer for didactic value. If Option B is chosen, ensure the six handlers do NOT also emit `Set{path:"/_errors/{bind}", ...}` — only node-tree ops.

### Pitfall 6: Leaf demos may regress after Button polish (Gap 1)

**What goes wrong:** `backend/crates/marionette/src/builders/button.rs:26-42` gallery_demo uses `.variant("destructive")` — currently no-op because the frontend reads `props.color`, not `props.variant`. After Wave 0 Button polish, destructive will actually render red. That's correct behavior but a visible change.

**How to avoid:** After Wave 0, re-UAT ALL 19 leaf demos in the gallery (not just the new catalog screens). Expect: `button` leaf demo now renders destructive visually; all others unchanged.

**Warning signs:** Phase 18 smoke tests pass but Phase 17's in-progress UAT snapshots diverge.

### Pitfall 7: Toast surface ≠ content surface

**What goes wrong:** CAT-04's toast trigger is inside the content surface, but the toast itself lives in the `toasts` surface. A handler that emits a Patch to `content` instead of `toasts` will silently drop the toast.

**How to avoid:** Verify `surface: "toasts"` in every toast-firing handler. Reuse the existing `gallery-demo/toast-fire` handler (verified `gallery-demo/src/handlers/toast.rs:11-35`) — it correctly targets `"toasts"` and uses `InsertChild` into `"toasts-root"`.

### Pitfall 8: Column visibility dropdown uses column.id as label

**What goes wrong:** Frontend's visibility dropdown (verified `DataTable.svelte:441-446`) shows `{column.id}` (the key) as the label, not `column.label`. For `joined_at`, users see "joined_at" in the dropdown — ugly.

**Why:** Line 441 uses `column.id` which is the TanStack `accessorKey` / column key, not the `header` label.

**Workaround:** Use prettier column keys (`joined` instead of `joined_at`) OR accept the limitation for now (don't fix in Phase 18). The `actions` column's empty label (`""`) will render as just whitespace in the dropdown — annoying but tolerable.

### Pitfall 9: sendAction `action.target` is undefined for gallery-demo

**What goes wrong:** `sendAction(name, payload, action.target)` — if `action.target` is undefined, the server side receives no target routing. Gallery demos set `target: None` (default), so `handleBlur` sends `sendAction(..., undefined)`. This is fine for gallery-demo where all handlers are routed by action name alone.

**How to avoid:** Nothing to avoid in CAT-02; noted for planner awareness. The payload shape that backend receives for blur: `{ value: "<field-value>" }` (per `TextInput.svelte:50-54`).

### Pitfall 10: Mutex on `GalleryState.demo_values`

**What goes wrong:** Multiple rapid blur actions hitting the backend concurrently could deadlock if handlers take write locks on `demo_values`. Current `GalleryState` is `Arc<RwLock<HashMap<String, Value>>>` (verified `gallery-demo/src/state.rs:23`).

**How to avoid:** CAT-02's validate handlers should NOT write to `demo_values` — they only emit PatchMessage. No state mutation needed. The `/_errors/{bind}` path is in the FRONTEND surface store, not the backend AppState.

## Code Examples

### CAT-01 Buttons & Actions skeleton

```rust
// backend/crates/gallery-demo/src/catalog/buttons.rs
//! CAT-01 Catalog: Buttons & Actions — every variant × size × state.

use marionette::builders::{Button, Container, Heading, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-buttons", name = "Catalog: Buttons")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Buttons & Actions").id("catalog-buttons-title").level(1).build();
    let intro = Text::new(
        "Every Button variant × size × state visible at once. \
         Mobile: stacks vertically. Desktop: 4-column grid inside each variant Card.",
    ).id("catalog-buttons-intro").build();

    let variants = ["default", "destructive", "outline", "ghost", "link"];
    let sizes    = ["sm", "default", "lg"];
    // states: normal, disabled, loading, icon-only

    let cards: Vec<Node> = variants.iter().flat_map(|variant| {
        let legend = Heading::new(format!("variant = {variant}"))
            .id(format!("catalog-buttons-{variant}-legend")).level(3).build();

        let cells: Vec<Node> = sizes.iter().flat_map(|size| {
            // 4 states per (variant, size) = 4 cells in the inner grid row.
            vec![
                Button::new(format!("{variant}/{size}"))
                    .id(format!("cb-{variant}-{size}-normal"))
                    .variant(*variant).size(*size)
                    .action(ComponentAction::submit("gallery-demo/noop"))
                    .build(),
                Button::new(format!("{variant}/{size}"))
                    .id(format!("cb-{variant}-{size}-disabled"))
                    .variant(*variant).size(*size).disabled(true)
                    .build(),
                Button::new(format!("{variant}/{size}"))
                    .id(format!("cb-{variant}-{size}-loading"))
                    .variant(*variant).size(*size).loading(true)  // <-- requires Gap 1 fix
                    .build(),
                // Icon-only — no label, just icon.
                Button::new("")
                    .id(format!("cb-{variant}-{size}-icon"))
                    .variant(*variant).size(*size)
                    .icon("plus")  // <-- requires Gap 1 fix
                    .aria_label(format!("{variant} {size} icon button"))
                    .build(),
            ]
        }).collect();

        let grid = Container::new()
            .id(format!("catalog-buttons-{variant}-grid"))
            .class("grid grid-cols-1 sm:grid-cols-4 lg:grid-cols-4 gap-3")
            .children(cells).build_with_children();

        let card = Container::new()
            .id(format!("catalog-buttons-{variant}-card"))
            .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
            .children(vec![legend, grid[0].clone()])  // grid is Vec<Node>; index 0 is root
            .build_with_children();

        let mut out = vec![card[0].clone()];  // Card root
        out.extend(card.into_iter().skip(1)); // Card descendants
        out.extend(grid.into_iter().skip(1)); // Grid descendants (all 12 button nodes)
        out
    }).collect();

    let root = Container::new()
        .id("catalog-buttons-root")
        .class("flex flex-col gap-6 p-6")
        .children({
            let mut k = vec![title, intro];
            // Add the 5 card roots as children; their descendants are already in `cards`.
            k.extend(cards.iter().filter(|(id, _)| id.ends_with("-card")).cloned());
            k
        })
        .build_with_children();

    let mut all = root;
    // cards already includes ALL node tuples (card roots + card descendants + grid descendants),
    // but those not matching -card suffix are not yet in `root`'s children list — add them:
    all.extend(cards.into_iter().filter(|(id, _)| !id.ends_with("-card")));
    all
}
```

**Note:** the above pseudo-code simplifies the tree-flattening. Actual implementation will use the `Vec<Node>` composition pattern from `backend/crates/marionette/src/builders/form.rs:35-48`.

### CAT-02 Forms skeleton (key excerpt)

```rust
// backend/crates/gallery-demo/src/catalog/forms.rs — CAT-02
// Outer: 6 Cards, one per input type, stacked.
// Per Card: 5 state variants (normal / disabled / error / focused / with-description) in a
//           grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 grid, PLUS an interactive validation row.

use marionette::builders::{Container, TextInput, Select, SelectOption, Checkbox, Switch, Textarea, RadioGroup, RadioOption, FieldSet, FieldSeparator, Heading, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-forms", name = "Catalog: Forms")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let mut all: Vec<Node> = Vec::new();
    let title = Heading::new("Forms").id("catalog-forms-title").level(1).build();
    let intro = Text::new("Every input × every state, plus live validation via Phase 12 node patching.")
        .id("catalog-forms-intro").build();
    all.push(title);
    all.push(intro);

    // -- Card 1: TextInput (5 states × variant) + interactive validation row --
    let text_states: Vec<Node> = vec![
        TextInput::new("Normal").bind("/demo/catalog-forms/text-normal").build(),
        TextInput::new("Disabled").disabled(true).bind("/demo/catalog-forms/text-disabled").build(),
        TextInput::new("With error").bind("/demo/catalog-forms/text-with-error").build(),
        TextInput::new("Focused (click me)").bind("/demo/catalog-forms/text-focused").build(),
        TextInput::new("With description")
            .description("Helper line below.").bind("/demo/catalog-forms/text-desc").build(),
    ];
    let text_states_grid = Container::new()
        .id("catalog-forms-text-states-grid")
        .class("grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3")
        .children(text_states).build_with_children();
    // Interactive validation row with blur-action wired:
    let mut blur = ComponentAction::submit("gallery-demo/catalog-forms/validate-text-input");
    blur.r#type = "blur".into();
    let text_interactive = TextInput::new("Type an email then tab out")
        .id("catalog-forms-text-interactive")
        .bind("/demo/catalog-forms/text-value")
        .description("Invalid input turns red on blur; correcting clears the error via node patch.")
        .action(blur).build();
    // Pre-mounted ErrorDisplay slot (for D-3-C set-children pattern):
    let text_error_slot = Container::new()
        .id("catalog-forms-text-error-slot")  // empty container; swapped on first blur
        .build();
    // Card body:
    let text_card_heading = Heading::new("TextInput")
        .id("catalog-forms-text-heading").level(2).build();
    let text_card = Container::new()
        .id("catalog-forms-text-card")
        .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
        .children(vec![
            text_card_heading,
            text_states_grid[0].clone(),      // root of states-grid
            text_interactive,                  // interactive field
            text_error_slot,                   // pre-mounted empty slot
        ]).build_with_children();
    all.extend(text_card);
    all.extend(text_states_grid.into_iter().skip(1));  // states-grid descendants

    // -- Card 2 through 6 (Select, Checkbox, Switch, Radio, Textarea) follow the same shape --
    // ... similar block for each input type (see patterns above) ...

    // Root Container:
    let root = Container::new()
        .id("catalog-forms-root")
        .class("flex flex-col gap-6 p-6")
        .children(vec![
            title, intro,
            // +6 card roots
        ]).build_with_children();
    // ... proper tree flattening ...
    all
}
```

### CAT-02 validate-text-input handler (Pattern 7b — node-tree ops)

```rust
// backend/crates/gallery-demo/src/handlers/catalog_forms.rs
use marionette::builders::{Container, ErrorDisplay};
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;

const TEXT_SLOT_ID: &str = "catalog-forms-text-error-slot";
const TEXT_CARD_ID: &str = "catalog-forms-text-card";

pub async fn handle_validate_text_input(ctx: HandlerContext) -> ActionResult {
    let value = ctx.action.payload.clone()
        .and_then(|p| p.get("value").cloned())
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default();
    let is_valid = value.contains('@') && value.contains('.');

    // D-3-C: TextInput exercises set-children op.
    // Valid → set children to [input]. Invalid → set children to [input, error_display].
    let (_err_id, err_component) = ErrorDisplay::new("text-err")
        .id("catalog-forms-text-error")
        .bind("/_errors/demo/catalog-forms/text-value")  // bind to data store so message shows
        .build();

    // Err details: render an ErrorDisplay with the message.
    let ops: Vec<PatchOperation> = if is_valid {
        vec![
            // set-children on the card: children list does NOT include the error display.
            // Need to know the card's children ids — they are [heading, states_grid, interactive_input, slot].
            // Keep the slot as an empty container (same as catalog render).
            PatchOperation::SetChildren {
                id: TEXT_CARD_ID.into(),
                children: vec![
                    "catalog-forms-text-heading".into(),
                    "catalog-forms-text-states-grid".into(),
                    "catalog-forms-text-interactive".into(),
                    TEXT_SLOT_ID.into(),  // back to empty slot
                ],
            },
            // Also clear the error message in the data store:
            PatchOperation::Set {
                path: "/_errors/demo/catalog-forms/text-value".into(),
                value: serde_json::Value::String(String::new()),
            },
        ]
    } else {
        vec![
            // Create the ErrorDisplay node:
            PatchOperation::SetNode {
                id: "catalog-forms-text-error".into(),
                component: err_component,
            },
            // set-children on the card: children now include the error display.
            PatchOperation::SetChildren {
                id: TEXT_CARD_ID.into(),
                children: vec![
                    "catalog-forms-text-heading".into(),
                    "catalog-forms-text-states-grid".into(),
                    "catalog-forms-text-interactive".into(),
                    "catalog-forms-text-error".into(),  // error replaces slot
                ],
            },
            // Write the error message to the /_errors/... path:
            PatchOperation::Set {
                path: "/_errors/demo/catalog-forms/text-value".into(),
                value: serde_json::Value::String("Enter a valid email address.".into()),
            },
        ]
    };

    Ok(vec![ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    })])
}
```

**Similar handlers for Select (delete-node), Checkbox (set-node), Switch (set-node), Radio (set-children), Textarea (delete-node).**

### CAT-03 DataTable catalog

```rust
// backend/crates/gallery-demo/src/catalog/data_table.rs
use marionette::builders::{Container, DataTable, Heading, Text};
use marionette::builders::data_table::{TableColumn, ColumnKind, Filter};
use marionette::builders::select::SelectOption;
use marionette::gallery::Node;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-data-table", name = "Catalog: Data Table")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Data Table").id("catalog-data-table-title").level(1).build();
    let intro = Text::new(
        "Filter bar, virtualized infinite scroll, column visibility, per-ColumnKind rendering. \
         Scroll to bottom to trigger fetch-rows pagination (500 synthetic rows total).",
    ).id("catalog-data-table-intro").build();

    let columns = vec![
        TableColumn::new("id", "ID").kind(ColumnKind::Number),
        TableColumn::new("name", "Name"),
        TableColumn::new("email", "Email"),
        TableColumn::new("status", "Status").kind(ColumnKind::Badge).hidden_default(true),
        TableColumn::new("score", "Score").kind(ColumnKind::Number),
        TableColumn::new("joined_at", "Joined").kind(ColumnKind::Date),
        TableColumn::new("actions", "").kind(ColumnKind::Actions).hidden_default(true),
    ];

    let (table_id, table_comp) = DataTable::new(columns)
        .id("catalog-data-table-main")
        .source("catalog-synthetic-rows")
        .bind("/demo/catalog-data-table/rows")
        .row_id_key("id")
        .page_size(50u32)
        .total_rows(500u64)
        .filter(Filter::text("name-search").label("Name").placeholder("Filter by name..."))
        .filter(Filter::select("status-filter").label("Status").options(vec![
            SelectOption { value: "active".into(), label: "Active".into() },
            SelectOption { value: "inactive".into(), label: "Inactive".into() },
            SelectOption { value: "pending".into(), label: "Pending".into() },
        ]))
        .filter(Filter::date_range("joined-range").label("Joined"))
        .build();

    Container::new()
        .id("catalog-data-table-root")
        .class("flex flex-col gap-6 p-6")
        .children(vec![title, intro, (table_id, table_comp)])
        .build_with_children()
}
```

### CAT-04 Feedback catalog

```rust
// backend/crates/gallery-demo/src/catalog/feedback.rs
use marionette::builders::{Button, Container, ErrorDisplay, Heading, Spinner, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-feedback", name = "Catalog: Feedback")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    let title = Heading::new("Feedback").id("catalog-feedback-title").level(1).build();

    // Triggers card (toast, modal, confirm)
    let toast_btn = Button::new("Fire toast")
        .id("cf-toast-btn").action(ComponentAction::click("gallery-demo/toast-fire")).build();
    let modal_btn = Button::new("Open modal")
        .id("cf-modal-btn").action(ComponentAction::click("gallery-demo/modal-open")).build();
    let confirm_btn = Button::new("Open confirm dialog")
        .id("cf-confirm-btn").action(ComponentAction::click("gallery-demo/confirm-open")).build();

    let triggers_grid = Container::new()
        .id("cf-triggers-grid")
        .class("grid grid-cols-1 sm:grid-cols-3 gap-3")
        .children(vec![toast_btn, modal_btn, confirm_btn]).build_with_children();
    let triggers_card = Container::new()
        .id("cf-triggers-card")
        .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
        .children(vec![
            Heading::new("Trigger surfaces").id("cf-triggers-heading").level(2).build(),
            triggers_grid[0].clone(),
        ]).build_with_children();

    // Placeholders card (empty, loading, error side-by-side)
    let empty_placeholder = Container::new()
        .id("cf-empty-placeholder")
        .class("rounded-md border-2 border-dashed p-8 flex items-center justify-center text-muted-foreground")
        .children(vec![Text::new("No data yet.").id("cf-empty-text").build()])
        .build_with_children();
    let loading_placeholder = Container::new()
        .id("cf-loading-placeholder")
        .class("rounded-md border p-8 flex items-center justify-center gap-3")
        .children(vec![
            Spinner::new().size("md").id("cf-loading-spinner").build(),
            Text::new("Loading...").id("cf-loading-text").build(),
        ]).build_with_children();
    // ErrorDisplay — bind-driven, seed provides the errors array
    let error_placeholder = ErrorDisplay::new("err")
        .id("cf-error-placeholder").bind("/demo/catalog-feedback/errors").build();

    let placeholders_grid = Container::new()
        .id("cf-placeholders-grid")
        .class("grid grid-cols-1 sm:grid-cols-3 gap-3")
        .children(vec![empty_placeholder[0].clone(), loading_placeholder[0].clone(), error_placeholder])
        .build_with_children();
    let placeholders_card = Container::new()
        .id("cf-placeholders-card")
        .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
        .children(vec![
            Heading::new("Placeholder states").id("cf-placeholders-heading").level(2).build(),
            placeholders_grid[0].clone(),
        ]).build_with_children();

    // Root assembly:
    let mut all: Vec<Node> = Vec::new();
    all.push(title);
    all.extend(triggers_card.clone());
    all.extend(placeholders_card.clone());
    all.extend(triggers_grid.into_iter().skip(1));
    all.extend(empty_placeholder.into_iter().skip(1));
    all.extend(loading_placeholder.into_iter().skip(1));
    all.extend(placeholders_grid.into_iter().skip(1));
    // Container root holding the two cards + title:
    let root = Container::new()
        .id("catalog-feedback-root")
        .class("flex flex-col gap-6 p-6")
        .children(vec![
            title.clone(), triggers_card[0].clone(), placeholders_card[0].clone(),
        ]).build_with_children();
    // ... proper tree flattening (see Pattern 1)
    root  // simplified; real code merges
}
```

### CAT-05 Typography & tokens catalog

```rust
// backend/crates/gallery-demo/src/catalog/typography.rs
use marionette::builders::{Container, Heading, Text};
use marionette::gallery::Node;

#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-typography", name = "Catalog: Typography")]
#[must_use]
pub fn gallery_demo() -> Vec<Node> {
    // Card 1: Text scale (Heading levels 1-6 + Text)
    let headings: Vec<Node> = (1..=6u8).map(|lvl| {
        Heading::new(format!("Heading level {lvl}"))
            .id(format!("ct-h{lvl}")).level(lvl).build()
    }).collect();
    // ... + Text samples

    // Card 2: Lucide icon catalog — 14 registered icons from icons.ts
    // Strategy: render Buttons with icon prop (icon-only size); each shows one lucide icon + label below.
    let icon_names = ["plus", "chevron-up", "chevron-down", "alert-circle", "x", "menu",
                      "arrow-left", "search", "filter", "pencil", "trash", "check",
                      "loader", "circle-help"];
    let icons: Vec<Node> = icon_names.iter().map(|n| {
        // Use Button with icon-only size as icon-render affordance (requires Gap 1 fix).
        // Each icon gets wrapped in a Container with the icon Button + text label.
        // ...
        Text::new(*n).id(format!("ct-icon-label-{n}")).build()  // fallback — just label
    }).collect();

    // Card 3: OKLCH swatches — 27 tokens from app.css:9-37
    let tokens = [
        ("--background", "oklch(1 0 0)"),
        ("--foreground", "oklch(0.141 0.005 285.823)"),
        ("--primary", "oklch(0.21 0.006 285.885)"),
        // ... all 27 — or read from app.css at build time
    ];
    let swatches: Vec<Node> = tokens.iter().map(|(name, color)| {
        // Render a Container with inline style: background: <color>; plus a Text label below.
        // Requires either (a) a new Swatch builder (out of scope), OR (b) Container with class+inline style.
        // Marionette Container has `class: Option<String>` — use Tailwind utility + emit inline-style
        // via a custom wrapper. Workaround: use Container class to set w-16 h-16 rounded border,
        // and rely on Tailwind's native bg-* variables if the token is in @theme inline (which it is).
        // Example: a swatch for --primary uses class="bg-primary" — Tailwind translates via the theme tokens.
        Container::new()
            .id(format!("ct-swatch-{}", name.trim_start_matches("--")))
            .class(format!("w-16 h-16 rounded border bg-{}", name.trim_start_matches("--")))
            .build_with_children()[0].clone()
    }).collect();

    // Assemble into 3 Cards + root...
    vec![]  // simplified
}
```

**IMPORTANT CAT-05 NOTE:** Tailwind's theme-integration converts `bg-primary` → `background: var(--color-primary)` because `@theme inline { --color-primary: var(--primary); ...}` is set in `app.css:68-99`. So `class="bg-primary"` works for tokens that have a `--color-<name>` mapping. The `--radius` and `--radius-sm/md/lg/xl` tokens aren't colors — render them differently (text sample showing border radius).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|-------------|------------------|--------------|--------|
| `Modal::new(...)` wrapper in Rust | Compositional popups (empty Container to close; any SDUI tree into `modal` sub-surface) | Phase 17 Plan 17-08 | Modal struct deleted; `GALLERY-DEMOS.md §Popup composition` is authoritative |
| Hand-maintained SideNav menu list | `registered_demos()` iterator builds nav at runtime | Phase 17 | Catalog screens auto-appear in nav via `#[gallery_demo]` |
| Form-level `BadPayload` errors | Per-field `validation_error_patch` via data op `/_errors/{bind}` | Phase 15 Plan 04 | Frontend reads `getData(surface, '/_errors' + bind)` and renders in `<Field.Error>` |
| `DemoEntry.render: fn() -> Node` | `fn() -> Vec<Node>` (flat adjacency list) | Phase 16.5 / 17 Plan 17-01 | All composite demos carry descendants properly |

**Deprecated/outdated:**
- `props.color` / `props.outline` on Rust Button — vestigial; Gap 1 fixes this.
- Standalone `Modal` builder — deleted in Plan 17-08.
- Hand-rolled SideNav menu — replaced by `registered_demos()` iteration.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: `cargo test` (unit + integration) with trybuild fixtures for macro tests. Frontend: `vitest` + `vitest-browser-playwright` for `.browser-test.ts` files. |
| Config file | Rust: workspace `Cargo.toml` + per-crate `Cargo.toml`; Frontend: `vitest.config.ts` (in frontend/) |
| Quick run command | `cargo test -p gallery-demo` (backend) / `cd frontend && pnpm test` (frontend) |
| Full suite command | `cargo test --workspace` + `cd frontend && pnpm test` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CAT-01 | Buttons catalog renders full matrix without panic | Integration + UAT | `cargo test -p gallery-demo catalog_buttons_renders` (Wave 0) + Chrome MCP | ❌ Wave 0 |
| CAT-01 | Button variant/size/loading/icon fields pass through to frontend | Rust unit | `cargo test -p marionette button_serializes_loading_icon` (Wave 0) | ❌ Wave 0 |
| CAT-02 | Forms catalog renders 6 cards without panic | Integration | `cargo test -p gallery-demo catalog_forms_renders` | ❌ Wave 0 |
| CAT-02 | validate-text-input handler emits SetChildren on invalid | Rust unit | `cargo test -p gallery-demo validate_text_input_emits_set_children` | ❌ Wave 0 |
| CAT-02 | validate-select handler emits DeleteNode on valid | Rust unit | `cargo test -p gallery-demo validate_select_emits_delete_node` | ❌ Wave 0 |
| CAT-02 | validate-checkbox handler emits SetNode | Rust unit | `cargo test -p gallery-demo validate_checkbox_emits_set_node` | ❌ Wave 0 |
| CAT-02 | Blur-action wiring fires on SelectInput/Checkbox/Switch/RadioGroup Svelte | Frontend browser-test | `frontend/src/lib/components/form/*.browser-test.ts` (extend existing) | ❌ Wave 0 |
| CAT-02 | Live validation end-to-end (type invalid → blur → error; correct → blur → error clears) | UAT | Chrome MCP walkthrough | ❌ per-wave |
| CAT-03 | fetch-rows source="catalog-synthetic-rows" dispatches correctly | Rust unit | `cargo test -p gallery-demo fetch_rows_catalog_source` | ❌ Wave 0 |
| CAT-03 | synthetic_rows(500).len() == 500 and deterministic | Rust unit | `cargo test -p gallery-demo fixtures::tests` | ❌ Wave 0 |
| CAT-03 | Virtualization scrolls through all 500 rows | UAT | Chrome MCP scroll-to-bottom walkthrough | ❌ per-wave |
| CAT-04 | Feedback catalog renders without panic | Integration | `cargo test -p gallery-demo catalog_feedback_renders` | ❌ Wave 0 |
| CAT-04 | Each trigger button dispatches correct action name | UAT | Chrome MCP | ❌ per-wave |
| CAT-05 | Typography catalog renders without panic | Integration | `cargo test -p gallery-demo catalog_typography_renders` | ❌ Wave 0 |
| CAT-05 | 27 swatches visible at desktop width | UAT | Chrome MCP | ❌ per-wave |
| Shared | All 5 catalog keys registered in `registered_demos()` | Integration | `cargo test -p gallery-demo catalog_keys_registered` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p gallery-demo` + `cargo clippy -p gallery-demo -- -D warnings`
- **Per wave merge:** `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` + `cd frontend && pnpm test`
- **Phase gate:** Full suite green + Chrome MCP UAT walk of all 5 catalog screens at desktop + mobile widths before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `backend/crates/gallery-demo/tests/catalog_registry.rs` — asserts 5 catalog-* keys present in `registered_demos()`
- [ ] `backend/crates/gallery-demo/src/fixtures.rs` with `#[cfg(test)] mod tests` for determinism + length
- [ ] Integration test per catalog (render without panic): add to `gallery-demo/tests/catalog_*.rs`
- [ ] Unit tests per validate-<input> handler in `gallery-demo/src/handlers/catalog_forms.rs`
- [ ] Extend 4 frontend browser-tests with blur-action assertions (`SelectInput.browser-test.ts`, `Checkbox.browser-test.ts`, `Switch.browser-test.ts`, `RadioGroup.browser-test.ts`)
- [ ] Extend `backend/crates/marionette/src/builders/button.rs` unit tests with loading/icon/aria_label field coverage

## Security Domain

Gallery-demo has no auth, no DB, no user-supplied input beyond form fields that stay in-memory. **Security surface is minimal**; the relevant ASVS items:

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Gallery has no auth (CONTEXT §Out of Scope) |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | No per-source auth in gallery fetch-rows (gallery is dev-only) |
| V5 Input Validation | yes | payload.source whitelist in fetch_rows; payload.value in validate-* handlers used only to compute error message (not reflected back untrusted) |
| V6 Cryptography | no | No crypto operations |
| V7 Error Handling | partial | ActionError::BadPayload for unknown source; toast error messages are server-authored (no injection vector) |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unvalidated `payload.source` in fetch-rows allowing arbitrary path dispatch | Tampering | Whitelist match; error on unknown source (mirrors CRM's `fetch_rows.rs` guard) |
| User-supplied blur-action payload reflected into `/_errors/{bind}` | XSS (if reflected without escape) | Server-authored error messages only; never interpolate `payload.value` into the message |
| DoS via huge `limit` in fetch-rows | DoS | Cap limit at `MIN(50, payload.limit)` for catalog-synthetic-rows (mirrors CRM's `MAX_LIMIT = 100`) |
| Frontend over-fetching via repeated sentinel triggers | DoS (client-side) | Existing `fetching` guard in `DataTable.svelte:336` prevents concurrent dispatches |

**All are LOW risk** — gallery-demo is a dev harness (no prod exposure) and every input is in-memory with no side effects.

## Wave Structure

**Total plans: 6** (1 framework-polish + 5 catalog screens). Fine granularity per config.

### Plan 18-01: Framework Polish (Wave 0 — BLOCKING all CATs)

**Scope:**
1. Extend Tailwind safelist in `frontend/src/app.css:7` with `sm:grid-cols-*` + `lg:grid-cols-*` variants (2-min edit).
2. Add `loading: Option<bool>`, `icon: Option<String>`, `aria_label: Option<String>` to Rust `Button` struct.
3. Rewrite `Button.svelte` to honor `props.variant` (6 shadcn variants) and `props.size` (6 shadcn sizes) directly; remove `color`/`outline` derivation; honor `props.loading` and `props.icon` (already partially present).
4. Add blur-action wiring to 4 Svelte form components: `SelectInput.svelte`, `Checkbox.svelte`, `Switch.svelte`, `RadioGroup.svelte` (mirror TextInput's `handleBlur` pattern).
5. Add `backend/crates/gallery-demo/src/fixtures.rs` with `Row`, `Status`, `synthetic_rows(n)` + tests.
6. Extend `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` with source dispatch + `catalog-synthetic-rows` arm.
7. Register 6 new `gallery-demo/catalog-forms/validate-*` actions in `handlers/mod.rs` (empty stubs for now — filled in Plan 18-03).
8. Wave 0 UAT checkpoint: run gallery-demo locally, navigate to existing leaf Button demo; verify destructive renders red (new behavior), verify loading-state visible when `loading(true)` passed. Verify Checkbox/Switch/Select/Radio blur-action spike.

**Risks:**
- Gap 1 Button polish affects the existing Phase 17 leaf `button` demo (destructive will render red; previously was gray-by-default). Expected regression for the better.
- Gap 2 blur wiring may hit bits-ui primitive quirks (spike required during this wave).

**Success:** `cargo test --workspace` + `cd frontend && pnpm test` green. Manual inspection of leaf Button demo shows all 6 shadcn variants renderable. Chrome MCP: all 19 Phase 17 leaf demos still pass (regression).

### Plan 18-02: CAT-01 Buttons & Actions

**Scope:**
1. `backend/crates/gallery-demo/src/catalog/mod.rs` + `buttons.rs`.
2. `catalog/mod.rs` added to `lib.rs` + `main.rs` `mod catalog;`.
3. `seed_for_key("catalog-buttons")` arm in `handlers/show.rs` returning empty seed (no binds on this screen).
4. Integration test: render returns non-empty `Vec<Node>` containing 5 variant Cards × 12 cells.
5. Chrome MCP UAT at desktop + mobile widths.

**Dependencies:** Plan 18-01 (Button fields wired).

### Plan 18-03: CAT-02 Forms

**Scope:**
1. `catalog/forms.rs` — 6 Cards with state grids + interactive validation rows + pre-mounted error slots.
2. `handlers/catalog_forms.rs` — 6 validate-<input> handlers emitting per-D-3-C node-tree ops.
3. `seed_for_key("catalog-forms")` arm — seeds 12+ bind paths (6 interactive + 6 state-demo binds per input type) + empty `/_errors/{bind}` defaults.
4. Unit tests per handler (verify op shape).
5. Chrome MCP UAT: type invalid email → blur → red error appears; correct → blur → error clears.

**Dependencies:** Plan 18-01 (blur wiring + Tailwind safelist).

### Plan 18-04: CAT-03 Data Table

**Scope:**
1. `catalog/data_table.rs` — 7 columns (3 ColumnKinds + Actions + plain Text) + 3 filters.
2. `seed_for_key("catalog-data-table")` arm — seeds the first 50 rows via `fixtures::synthetic_rows(50)` into `/demo/catalog-data-table/rows/{id}`.
3. Extend `fetch_rows` handler to append `actions: [...]` to each row before emission.
4. Chrome MCP UAT: scroll to bottom (fetch-rows fires), toggle column visibility, use filters.

**Dependencies:** Plan 18-01 (fixtures.rs, fetch_rows source dispatch).

### Plan 18-05: CAT-04 Feedback

**Scope:**
1. `catalog/feedback.rs` — triggers Card (toast/modal/confirm) + placeholders Card (empty/loading/error).
2. `seed_for_key("catalog-feedback")` arm — seeds `/demo/catalog-feedback/errors: [{message: "Sample error", path: null}]`.
3. Chrome MCP UAT: click each trigger → surface renders correctly.

**Dependencies:** Plan 18-01 (safelist).

### Plan 18-06: CAT-05 Typography & Tokens

**Scope:**
1. `catalog/typography.rs` — 3 Cards (text scale, icons, swatches).
2. `seed_for_key("catalog-typography")` arm — returns empty seed.
3. GALLERY-DEMOS.md coverage matrix: add 5 rows for catalog-* keys.
4. Chrome MCP UAT: visual inspection at desktop + mobile.

**Dependencies:** Plan 18-01 (Button fields for icon-only rendering; Tailwind safelist for `lg:grid-cols-8`).

### Parallelization

After Plan 18-01 ships, Plans 18-02 through 18-06 are INDEPENDENT (disjoint files, no shared state). They can parallelize if the team has bandwidth. Recommend serial execution per GSD config for smoother Chrome MCP UAT flow.

## Sources

### Primary (HIGH confidence)
- `backend/crates/marionette/src/builders/*.rs` (button, text_input, select, checkbox, switch, radio_group, textarea, form, field_set, data_table, confirm_dialog, container, grid, heading, text, spinner, error_display, modal) — current Rust builder surfaces, verified by reading source.
- `frontend/src/lib/components/form/*.svelte` (Button, TextInput, Textarea, SelectInput, Checkbox, Switch, RadioGroup) — current Svelte component surfaces, verified by reading source.
- `frontend/src/lib/components/table/DataTable.svelte` — virtualization, filter bar, column visibility, fetch-rows sentinel.
- `frontend/src/lib/store/data.svelte.ts:60-118` — patch operation dispatch, verified.
- `frontend/src/lib/components/ui/card/` + `ui/button/button.svelte:6-35` + `ui/field/field-error.svelte` — shadcn primitives and variant definitions, verified.
- `frontend/src/app.css:1-99` — Tailwind safelist and OKLCH tokens, verified line-by-line.
- `backend/crates/marionette-protocol/src/data.rs:1-44` — PatchOperation enum, verified.
- `backend/crates/marionette/src/validation.rs:58-76` — existing `validation_error_patch` helper, verified.
- `backend/crates/crm-demo/src/handlers/fetch_rows.rs:93-158` — source-dispatch pattern, verified.
- `backend/crates/gallery-demo/src/handlers/*.rs` — current gallery handler surface, verified.
- `backend/crates/marionette/GALLERY-DEMOS.md` — author contract, verified.
- `backend/crates/marionette-macros/src/component_builder.rs:170-206` — optional setter/action/bind generation, verified.
- `.planning/phases/18-catalog-screens/18-CONTEXT.md` — locked decisions.
- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-CONTEXT.md` — Phase 17 locked decisions.

### Secondary (MEDIUM confidence)
- https://formsnap.dev — WebFetch summary (see Q6). Composition anatomy inferred from WebFetch response.
- https://tailwindcss.com/docs/grid-column — Tailwind v4 responsive grid class behavior (standard knowledge).

### Tertiary (LOW confidence)
- Lucide icon count = ~1800 — computed from `ls node_modules/@lucide/svelte/dist/icons | wc -l = 7278 / 4 = ~1820`. Approximation may be off by a few dozen; exact count is not material for Phase 18.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every recommended library/builder is already in the codebase and cited by source line.
- Architecture: HIGH — patterns are all verified existing patterns; Wave structure follows established Phase 17 decomposition.
- Pitfalls: HIGH — the 10 open questions are answered from source code; the three framework gaps (Button, blur wiring, Tailwind safelist) are discovery-level findings, not speculative.
- Op mapping (D-3-C): MEDIUM — the specified node-tree op approach works but diverges from the currently-used data-op approach; planner must consciously pick one.

**Research date:** 2026-04-23
**Valid until:** 2026-05-23 (30 days; stable subsystem — catalog composition on top of locked Phase 16/17 framework).
