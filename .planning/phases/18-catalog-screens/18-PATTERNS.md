# Phase 18: Catalog Screens - Pattern Map

**Mapped:** 2026-04-23
**Files analyzed:** 18 (10 new, 8 modified)
**Analogs found:** 18 / 18

## Scope Summary

Phase 18 is almost pure Rust composition in `backend/crates/gallery-demo/`. Every new file has a strong in-repo analog:

- Catalog fns mirror the existing `#[gallery_demo]` leaves in `backend/crates/marionette/src/builders/` (same macro, same `Vec<Node>` return, same composition recipe).
- New handlers mirror existing `gallery-demo/src/handlers/*.rs` files (same `HandlerContext` signature, same `ActionResult` return, same `PatchMessage` / `RenderMessage` shape).
- The shared fixtures generator mirrors the payload-parsing + seeded-rows idiom from `crm-demo/src/handlers/fetch_rows.rs`.
- Frontend edits (Button variant/size rewrite, blur wiring on 4 form components, Tailwind safelist) are isolated to existing files that already use the patterns being extended.

No new frontend component is created; no new registry entry, no new routing action, no new builder struct (D-2-C). Framework Gap 1 (Button props) adds optional fields to an existing Rust struct — identical shape to what every other builder already uses.

---

## File Classification

### New files

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `backend/crates/gallery-demo/src/catalog/mod.rs` | module declaration | static | `backend/crates/marionette/src/builders/mod.rs` | exact |
| `backend/crates/gallery-demo/src/catalog/buttons.rs` | gallery-demo fn (CAT-01) | pure fn → `Vec<Node>` | `backend/crates/marionette/src/builders/button.rs` `gallery_demo()` | exact (role + data-flow) |
| `backend/crates/gallery-demo/src/catalog/forms.rs` | gallery-demo fn (CAT-02) | pure fn → `Vec<Node>` | `backend/crates/marionette/src/builders/form.rs` `gallery_demo()` + `backend/crates/marionette/src/builders/field_set.rs` `gallery_demo()` | exact (composite pattern) |
| `backend/crates/gallery-demo/src/catalog/data_table.rs` | gallery-demo fn (CAT-03) | pure fn → `Vec<Node>` | `backend/crates/marionette/src/builders/data_table.rs` `gallery_demo()` | exact (role + component) |
| `backend/crates/gallery-demo/src/catalog/feedback.rs` | gallery-demo fn (CAT-04) | pure fn → `Vec<Node>` | `backend/crates/marionette/src/builders/confirm_dialog.rs` `gallery_demo()` + `backend/crates/gallery-demo/src/handlers/modal.rs` | exact (composition) + role-match (render) |
| `backend/crates/gallery-demo/src/catalog/typography.rs` | gallery-demo fn (CAT-05) | pure fn → `Vec<Node>` | `backend/crates/marionette/src/builders/heading.rs` `gallery_demo()` + `backend/crates/gallery-demo/src/home.rs` `build_home_page()` | role-match (pure render) |
| `backend/crates/gallery-demo/src/fixtures.rs` | shared utility module | pure fn → `Vec<Row>` | `backend/crates/gallery-demo/src/handlers/show.rs` `seed_table_rows()` (row shape) + `backend/crates/crm-demo/src/handlers/fetch_rows.rs` (dispatch pattern) | role-match |
| `backend/crates/gallery-demo/src/handlers/catalog_forms.rs` | action handler (6 blur-validate fns) | request → `PatchMessage` | `backend/crates/gallery-demo/src/handlers/toast.rs` (PatchMessage shape) + `backend/crates/marionette/src/validation.rs::validation_error_patch` (payload) + `backend/crates/crm-demo/src/handlers/contact.rs` lines 1095-1113 (validate-then-patch) | exact (data-op path A) / role-match (node-tree path B) |

### Modified files

| Modified File | Role | Data Flow | Pattern Source | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `backend/crates/gallery-demo/src/lib.rs` | module declarations | static | existing `pub mod handlers; pub mod home; pub mod state;` in same file | exact |
| `backend/crates/gallery-demo/src/handlers/show.rs` | extend `seed_for_key` match | request → `RenderMessage` | existing match arms in same file (lines 57-98) | exact |
| `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` | add source-dispatch arm | request → `PatchMessage` | `backend/crates/crm-demo/src/handlers/fetch_rows.rs` lines 100-158 | exact |
| `backend/crates/gallery-demo/src/handlers/mod.rs` | action registrations | router wiring | existing `register_gallery_actions()` in same file (lines 22-41) | exact |
| `backend/crates/marionette/GALLERY-DEMOS.md` | docs (coverage matrix rows) | documentation | existing `marionette` section of same file | exact |
| `backend/crates/marionette/src/builders/button.rs` | add `loading` / `icon` / `aria_label` optional fields | builder struct | existing `Button` struct + `text_input.rs` `description`/`full_width` optional-fields precedent | exact |
| `frontend/src/lib/components/form/Button.svelte` | rewrite variant/size props pass-through | svelte component | shadcn `Button` API (vendored under `frontend/src/lib/components/ui/button`) + current `Button.svelte` structure | exact |
| `frontend/src/lib/components/form/{SelectInput,Checkbox,Switch,RadioGroup}.svelte` | add `handleBlur` + `action?.type === 'blur'` dispatch | event → `sendAction` | `frontend/src/lib/components/form/TextInput.svelte` lines 45-56 (exact `handleBlur` to copy) | exact |
| `frontend/src/app.css` | extend `@source inline(...)` safelist | css | existing `@source inline(...)` on line 7 | exact |

---

## Pattern Assignments

### `backend/crates/gallery-demo/src/catalog/mod.rs` (module declaration)

**Analog:** `backend/crates/marionette/src/builders/mod.rs` lines 8-35.

**Pattern to copy (mod.rs shape):**
```rust
// One `pub mod <name>;` per file in the directory, alphabetized or grouped.
// No glob re-exports to avoid `gallery_demo` fn name collision between files.
pub mod buttons;
pub mod data_table;
pub mod feedback;
pub mod forms;
pub mod typography;
```

**Note:** `backend/crates/marionette/src/builders/mod.rs` lines 1-7 carries a comment explaining WHY glob re-exports are avoided — every file has a `gallery_demo` fn, causing ambiguity. This applies verbatim to `catalog/mod.rs`; copy the `#![allow(ambiguous_glob_reexports)]` only if re-exports are needed (they are not — catalog fns are accessed via the linkme registry, not by path).

---

### `backend/crates/gallery-demo/src/catalog/buttons.rs` (CAT-01)

**Analog:** `backend/crates/marionette/src/builders/button.rs` lines 23-42 (the in-tree `gallery_demo()` sibling for the `button` key).

**Imports pattern to copy** (from `marionette/src/builders/button.rs` plus composition needs):
```rust
use marionette::builders::{Button, Container, Heading, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;
```

**Gallery-demo attribute + signature pattern** (from `marionette/src/builders/button.rs` lines 22-26):
```rust
#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-buttons", name = "Catalog: Buttons")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    // body returns Vec<Node>
}
```

**Note:** `marionette` leaves use `key = "button"` (one positional arg); catalog fns use `key = "..."` + `name = "..."` (RESEARCH.md Pattern 1). Both shapes are supported by the `#[gallery_demo]` macro.

**Composition tail pattern to copy** (from `marionette/src/builders/button.rs` lines 38-41):
```rust
crate::builders::container::Container::new()
    .id("demo-<key>-root")
    .children(vec![/* children */])
    .build_with_children()
```

Catalog fns use `marionette::builders::Container` directly (gallery-demo is a consumer, not a sub-module of marionette). Keep the `.id("catalog-buttons-root")` + `.class("flex flex-col gap-6 p-6")` (UI-SPEC §Per-Screen Anatomy) + `.children(vec![title, intro, ...cards])` + `.build_with_children()` shape.

**Variant/size/state matrix loop pattern** (from RESEARCH.md Pattern 1 + §Code Examples CAT-01 lines 870-902):
```rust
let variants = ["default", "destructive", "outline", "ghost", "link"];
let sizes    = ["sm", "default", "lg"];
let cards: Vec<Node> = variants.iter().flat_map(|variant| {
    let legend = Heading::new(format!("variant = {variant}"))
        .id(format!("catalog-buttons-{variant}-legend")).level(3).build();
    let cells: Vec<Node> = sizes.iter().flat_map(|size| {
        vec![
            Button::new(format!("{variant}/{size}"))
                .id(format!("cb-{variant}-{size}-normal"))
                .variant(*variant).size(*size)
                .action(ComponentAction::submit("gallery-demo/noop"))
                .build(),
            // ... disabled, loading, icon states
        ]
    }).collect();
    // Card = Container with inner grid class
    Container::new()
        .id(format!("catalog-buttons-card-{variant}"))
        .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
        .children({ let mut kids = vec![legend]; kids.extend(cells); kids })
        .build_with_children()
}).collect();
```

**Container-as-Card idiom** (RESEARCH.md Pattern 3 — DO NOT use `Container::card(true)`, which adds `max-w-md` and vertical centering per `frontend/src/lib/components/layout/Container.svelte:36-41`). Use Tailwind classes directly on `Container::class(...)`:
```
rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4
```

**Inner grid class** (UI-SPEC §CAT-01 line 312):
```
grid grid-cols-1 sm:grid-cols-4 lg:grid-cols-4 gap-3
```

Only a `Container::new().class("grid …")` wrapper produces responsive columns; `marionette::builders::Grid` is FIXED-column (inline `grid-template-columns` on the svelte side — verified `frontend/src/lib/components/layout/Grid.svelte:38`) and must NOT be used here. RESEARCH.md Pitfall 3 is the locked rationale.

---

### `backend/crates/gallery-demo/src/catalog/forms.rs` (CAT-02)

**Analog:** `backend/crates/marionette/src/builders/field_set.rs` lines 41-64 (composite pattern — mixes multiple form-input builders into one Container root).

**Secondary analog:** `backend/crates/marionette/src/builders/form.rs` lines 18-48 (`build_tree()` usage for composite nesting — returns `(root_tuple, descendants)`).

**Imports pattern to copy** (superset of all six input demos + Container/Heading/Text + FieldSeparator + ErrorDisplay):
```rust
use marionette::builders::{
    Checkbox, Container, ErrorDisplay, FieldSeparator, Heading, RadioGroup, Select,
    Switch, Text, TextInput, Textarea,
};
use marionette::builders::radio_group::RadioOption;
use marionette::builders::select::SelectOption;
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;
```

(Note: `SelectOption` + `RadioOption` are NOT re-exported at `marionette::builders::*` — they live under their respective sub-modules per `backend/crates/marionette/src/builders/mod.rs` line 1-7 comment about ambiguous glob re-exports. Import them from the module path.)

**Per-Card composition pattern** (copy from `field_set.rs` — one wrapping Container per input type; each Card has: heading + state-matrix grid + FieldSeparator + interactive field + error-slot Container):
```rust
// ---- TextInput Card ----
let text_heading = Heading::new("TextInput").id("catalog-forms-text-heading").level(2).build();
let text_state_grid = Container::new()
    .id("catalog-forms-text-state-grid")
    .class("grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3")
    .children(vec![
        TextInput::new("Normal").bind("/demo/catalog-forms/text-normal").build(),
        TextInput::new("Disabled").disabled(true).bind("/demo/catalog-forms/text-disabled").build(),
        TextInput::new("With error").bind("/demo/catalog-forms/text-with-error").build(),
        TextInput::new("Focused (click me)").bind("/demo/catalog-forms/text-focused").build(),
        TextInput::new("With description")
            .description("Helper text rendered below via Field.Description.")
            .bind("/demo/catalog-forms/text-desc").build(),
    ])
    .build_with_children();
let text_sep = FieldSeparator::new().id("catalog-forms-text-sep").build();
let text_interactive = TextInput::new("Email (type then tab out)")
    .description("Invalid → red border on blur. Correct → error clears via set-children + delete-node patch.")
    .bind("/demo/catalog-forms/text-value")
    .action(blur_action("gallery-demo/catalog-forms/validate-text-input"))
    .build();
let text_error_slot = Container::new().id("catalog-forms-text-error-slot").build();
let text_card = Container::new()
    .id("catalog-forms-text-card")
    .class("rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4")
    .children(vec![text_heading, text_state_grid, text_sep, text_interactive, text_error_slot])
    .build_with_children();
```

**Blur-action helper** (RESEARCH.md §Q1 — `ComponentAction` with `type: "blur"` is hand-constructed; no `.blur()` helper exists today):
```rust
fn blur_action(name: &str) -> ComponentAction {
    // Parallel shape to ComponentAction::click(...) / ::submit(...),
    // but with type="blur" which TextInput.svelte and Textarea.svelte match
    // against (frontend/src/lib/components/form/TextInput.svelte:48).
    ComponentAction {
        r#type: "blur".into(),
        name: Some(name.into()),
        target: None,
        payload: None,
        optimistic: None,
        extra: Default::default(),
    }
}
```

**Error-slot pre-mount requirement** (RESEARCH.md Pitfall 4 + §Q2): every input's error-slot Container MUST be rendered at initial mount with its stable id. The node-tree ops (`set-node`, `set-children`, `delete-node`) target these ids. If the slot is missing on first render, `set-node` creates an orphan (RESEARCH.md Pitfall 4).

**Seed-alignment contract** (RESEARCH.md §CAT-02 Code Examples lines 384-431; Pitfall 1): every `.bind("/demo/catalog-forms/<path>")` call in this file MUST have a matching entry in the `seed_for_key("catalog-forms")` arm of `handlers/show.rs`. The UI-SPEC §CAT-02 locks the complete seed table (36 paths). Implementer copies the table verbatim into the `seed_for_key` arm.

---

### `backend/crates/gallery-demo/src/catalog/data_table.rs` (CAT-03)

**Analog:** `backend/crates/marionette/src/builders/data_table.rs` lines 228-255 (the in-tree `gallery_demo()` sibling for the `data-table` key — builds a `DataTable` component with columns + source + bind).

**Imports pattern to copy** (lines 1-8 of `data_table.rs`):
```rust
use marionette::builders::data_table::{ColumnKind, DataTable, Filter, TableColumn};
use marionette::builders::select::SelectOption;
use marionette::builders::{Container, Heading, Text};
use marionette::gallery::Node;
```

**DataTable composition pattern to copy** (from the leaf demo lines 230-255, extended with filters + all `ColumnKind` variants per UI-SPEC §CAT-03):
```rust
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
    .id("catalog-data-table-root")
    .source("catalog-synthetic-rows")
    .bind("/demo/catalog-data-table/rows")
    .row_id_key("id")
    .page_size(50u32)
    .total_rows(500u64)
    .filter(Filter::text("name-search").label("Name").placeholder("Filter by name…"))
    .filter(Filter::select("status-filter", vec![
        SelectOption { value: "active".into(),   label: "Active".into() },
        SelectOption { value: "inactive".into(), label: "Inactive".into() },
        SelectOption { value: "pending".into(),  label: "Pending".into() },
    ]).label("Status"))
    .filter(Filter::date_range("joined-range").label("Joined"))
    .build();
```

Key contract points inherited from the leaf demo:
- **`.bind("/demo/catalog-data-table/rows")` is MANDATORY.** Missing bind ⇒ DataTable.svelte reads `getData(surface, bind)` as `{}`, renders zero rows (G-03 lesson; verified at `marionette/src/builders/data_table.rs:240-243` comment).
- **`.source("catalog-synthetic-rows")`** MUST match the new source-dispatch arm added to `handlers/fetch_rows.rs`.
- **`.row_id_key("id")`** matches the `id` field on the `Row` struct in `fixtures.rs`.

**`.filter(...)` chaining pattern** uses the hand-written `DataTableBuilder::filter()` append-setter at `data_table.rs:212-222` (derived setters replace; `.filter()` appends).

**Wrapper Container pattern** (title + intro above the DataTable; standard catalog-root shape from UI-SPEC §CAT-03 lines 438-457):
```rust
let title = Heading::new("Data Table").id("catalog-data-table-title").level(1).build();
let intro = Text::new("Filter bar, virtualized infinite scroll…").id("catalog-data-table-intro").build();
Container::new()
    .id("catalog-data-table-container")
    .class("flex flex-col gap-6 p-6")
    .children(vec![title, intro, (table_id, table_comp)])
    .build_with_children()
```

---

### `backend/crates/gallery-demo/src/catalog/feedback.rs` (CAT-04)

**Analog:** `backend/crates/marionette/src/builders/confirm_dialog.rs` lines 42-66 (trigger Button + explainer Text composed in a Container) and `backend/crates/gallery-demo/src/handlers/modal.rs` (what the triggers open — no new handlers needed; reuse `gallery-demo/modal-open`, `gallery-demo/confirm-open`, `gallery-demo/toast-fire`).

**Imports pattern to copy:**
```rust
use marionette::builders::{Button, Container, ErrorDisplay, Heading, Spinner, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;
```

**Trigger Button pattern** (from `confirm_dialog.rs` lines 51-54):
```rust
let toast_trigger = Button::new("Fire toast")
    .id("catalog-feedback-toast-trigger")
    .action(ComponentAction::click("gallery-demo/toast-fire"))
    .build();
let modal_trigger = Button::new("Open modal")
    .id("catalog-feedback-modal-trigger")
    .action(ComponentAction::click("gallery-demo/modal-open"))
    .build();
let confirm_trigger = Button::new("Open confirm dialog")
    .id("catalog-feedback-confirm-trigger")
    .action(ComponentAction::click("gallery-demo/confirm-open"))
    .build();
```

**Placeholder mini-Card pattern** (UI-SPEC §CAT-04 lines 525-539; emulate three side-by-side `Container`s with specialized classes):
```rust
let empty_placeholder = Container::new()
    .id("catalog-feedback-empty")
    .class("rounded-md border-2 border-dashed p-8 flex flex-col items-center justify-center gap-2 text-center text-muted-foreground")
    .children(vec![
        Heading::new("No data yet").id("catalog-feedback-empty-h").level(4).build(),
        Text::new("Start by adding your first item — empty states should always tell users what to do next.")
            .id("catalog-feedback-empty-body").build(),
    ])
    .build_with_children();

let loading_placeholder = Container::new()
    .id("catalog-feedback-loading")
    .class("rounded-md border p-8 flex flex-col items-center justify-center gap-3")
    .children(vec![
        Spinner::new().size("md").id("catalog-feedback-loading-spinner").build(),
        Text::new("Loading…").id("catalog-feedback-loading-label").build(),
    ])
    .build_with_children();

let error_placeholder = ErrorDisplay::new("errors")  // positional label; visible errors come from bind
    .id("catalog-feedback-error")
    .bind("/demo/catalog-feedback/errors")
    .build();
```

ErrorDisplay contract inherited from `marionette/src/builders/error_display.rs` lines 18-42: the positional `message` arg is dead; errors render from `bind`. Seed the errors array at `/demo/catalog-feedback/errors` in the new `seed_for_key("catalog-feedback")` arm (see `show.rs` pattern assignment below).

---

### `backend/crates/gallery-demo/src/catalog/typography.rs` (CAT-05)

**Analog A (type-scale Card):** `backend/crates/marionette/src/builders/heading.rs` lines 19-31 — existing `gallery_demo()` composing H1/H2/H3 inside a Container.

**Analog B (pure static render Container composition):** `backend/crates/gallery-demo/src/home.rs` `build_home_page()` lines 22-72 — composes Heading + Text + tile Grid → returns `(root_id, HashMap, data)`. Catalog fns return `Vec<Node>` instead (DemoEntry contract), but the composition shape matches.

**Imports pattern to copy** (heading.rs lines 1-5 + container/text/button for icon tiles):
```rust
use marionette::builders::{Button, Container, Heading, Text};
use marionette::gallery::Node;
use marionette_protocol::ComponentAction;
```

**Type-scale Card pattern** (from `heading.rs` lines 22-31, extended to all 6 levels + two Text variants per UI-SPEC §CAT-05 lines 555-566):
```rust
let type_scale_heading = Heading::new("Type scale").id("catalog-typo-type-heading").level(2).build();
let h1 = Heading::new("Heading 1 — sample").id("catalog-typo-h1").level(1).build();
let h2 = Heading::new("Heading 2 — sample").id("catalog-typo-h2").level(2).build();
let h3 = Heading::new("Heading 3 — sample").id("catalog-typo-h3").level(3).build();
let h4 = Heading::new("Heading 4 — sample").id("catalog-typo-h4").level(4).build();
let h5 = Heading::new("Heading 5 — sample").id("catalog-typo-h5").level(5).build();
let h6 = Heading::new("Heading 6 — sample").id("catalog-typo-h6").level(6).build();
let body = Text::new("Body text. The quick brown fox jumps over the lazy dog.")
    .id("catalog-typo-body").build();
// (A `Text` with `.class("text-xs text-muted-foreground")` is NOT a supported
// builder field — Text has no class prop. Workaround: wrap in Container with class.)
```

**Icon catalog grid pattern** (UI-SPEC §CAT-05 line 592 anatomy):
```rust
// One "cell" per icon — Container with flex layout; the icon is rendered by
// a Button(size="icon", variant="outline") carrying `icon = "<kebab>"`, then
// a Text label below. The 14 icons are enumerated from the locked list in
// UI-SPEC §CAT-05 line 594.
let icons = [
    "plus", "chevron-up", "chevron-down", "alert-circle", "x", "menu",
    "arrow-left", "search", "filter", "pencil", "trash", "check", "loader",
    "circle-help",
];
let icon_cells: Vec<Node> = icons.iter().map(|name| {
    let btn = Button::new("")
        .id(format!("catalog-typo-icon-{name}"))
        .variant("outline").size("icon")
        .icon(*name)                               // ← requires Gap 1 fix
        .aria_label(format!("{name} icon"))        // ← requires Gap 1 fix
        .build();
    let label = Text::new(*name).id(format!("catalog-typo-icon-label-{name}")).build();
    Container::new()
        .id(format!("catalog-typo-icon-cell-{name}"))
        .class("flex flex-col items-center gap-1 p-2 rounded border")
        .children(vec![btn, label])
        .build_with_children()
}).flatten().collect();  // flatten the per-cell Vec<Node> returned by build_with_children
```

**Swatch grid pattern** (UI-SPEC §CAT-05 line 597; pure-display `Container` with fill-class from the token table on lines 604-630):
```rust
// Each swatch: outer Container (cell) + inner Container (color box with bg-<token>)
// + Text label underneath.
let tokens = [
    "background", "foreground", "card", "card-foreground", "popover",
    "popover-foreground", "primary", "primary-foreground", "secondary",
    "secondary-foreground", "muted", "muted-foreground", "accent",
    "accent-foreground", "destructive", "border", "input", "ring",
    "sidebar", "sidebar-foreground", "sidebar-primary",
    "sidebar-primary-foreground", "sidebar-accent", "sidebar-accent-foreground",
    "sidebar-border", "sidebar-ring",
];
// For each token, emit:
//   Container(class=f"w-full h-16 rounded-md border bg-{token}")
//   Text(class=... via wrapper Container; Text builder has no class prop — use Container wrap)
```

**Static-only note:** CAT-05 has zero interactive actions. No `bind`, no `action`, no `seed_for_key` state (the match arm for `catalog-typography` returns empty `{}`).

---

### `backend/crates/gallery-demo/src/fixtures.rs` (shared row generator)

**Analog A (row shape):** `backend/crates/gallery-demo/src/handlers/show.rs` `seed_table_rows()` lines 102-115 — object-map keyed by stringified id, five fields per row. New `Row` struct is the strongly-typed Rust equivalent.

**Analog B (generation pattern):** `backend/crates/crm-demo/src/handlers/fetch_rows.rs` lines 37-51 (`FetchRowsPayload` deserialization) + the per-source fetch fns lower in that file (deterministic row emission based on offset + limit).

**Imports pattern** (RESEARCH.md Pattern 5):
```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
```

**Struct + enum definitions verbatim from RESEARCH.md Pattern 5** (§RESEARCH lines 349-361):
```rust
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
```

**LCG deterministic generator** (copy verbatim from RESEARCH.md Pattern 5 lines 362-397). Pure-fn, zero external dep (no `rand` crate), same `n` always yields same rows.

**Tests pattern to copy** (from `marionette/src/builders/button.rs` lines 44-83 — `#[cfg(test)] mod tests` at the bottom of the file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn generator_length() { assert_eq!(synthetic_rows(500).len(), 500); }
    #[test] fn generator_stability() { assert_eq!(synthetic_rows(500)[0].id, synthetic_rows(500)[0].id); }
    #[test] fn generator_id_starts_at_one() { assert_eq!(synthetic_rows(10)[0].id, 1); }
}
```

**Cargo.toml diff** (RESEARCH.md §Standard Stack line 114): add `chrono.workspace = true` to `gallery-demo/Cargo.toml` `[dependencies]` block, alphabetically-sorted between `axum` and `marionette`. The workspace already exposes `chrono = "0.4"` with the `serde` feature (confirmed by `backend/Cargo.toml:29` per research); no extra feature gate needed.

---

### `backend/crates/gallery-demo/src/handlers/catalog_forms.rs` (6 blur-validate handlers)

**Analog A (handler signature + PatchMessage shape):** `backend/crates/gallery-demo/src/handlers/toast.rs` lines 11-36 — identical `handle_<name>(ctx: HandlerContext) -> ActionResult` shape, `PatchOperation::Set` construction, `PatchMessage` wrapper.

**Analog B (validation-error bind-path pattern):** `backend/crates/marionette/src/validation.rs` `validation_error_patch()` lines 58-76 — the canonical `/_errors{bind}` path shape the frontend Field.Error reads.

**Analog C (validate-then-patch pattern):** `backend/crates/crm-demo/src/handlers/contact.rs` lines 1093-1114 — input check → `Vec<(path, message)>` → `validation_error_patch("content", errors)` → return.

**Imports pattern** (combined from all three analogs):
```rust
use marionette::error::ActionResult;
use marionette::extractors::HandlerContext;
use marionette_protocol::data::PatchOperation;
use marionette_protocol::messages::PatchMessage;
use marionette_protocol::ProtocolMessage;
```

**Payload extraction pattern** (toast.rs line 40-46 for the simpler `get-by-key` shape; RESEARCH.md Pattern 7a lines 472-477 for the `value` field specifically):
```rust
let value = ctx.action.payload.clone()
    .and_then(|p| p.get("value").cloned())
    .and_then(|v| v.as_str().map(String::from))
    .unwrap_or_default();
```

**Option A — data-op path** (RESEARCH.md §Pattern 7a lines 466-495, RECOMMENDED fallback). Returns a single `PatchOperation::Set` writing to `/_errors/demo/catalog-forms/<input>-value`:
```rust
let op = PatchOperation::Set {
    path: "/_errors/demo/catalog-forms/text-value".into(),
    value: if is_valid {
        serde_json::Value::String(String::new())  // empty clears
    } else {
        serde_json::Value::String("Enter a valid email address.".into())
    },
};
Ok(vec![ProtocolMessage::Patch(PatchMessage {
    id: ctx.action.id.clone(),
    surface: "content".into(),
    patch: vec![op],
})])
```

**Option B — node-tree op path** (RESEARCH.md §Pattern 7b + UI-SPEC §CAT-02 lines 371-382, LOCKED: planner picks Option B per D-3-C didactic intent). Per input, emits a specific mix of `PatchOperation::SetNode`, `PatchOperation::SetChildren`, `PatchOperation::DeleteNode` against the pre-mounted error-slot id:
```rust
// Example: TextInput invalid → set-node the slot to an ErrorDisplay component
//          + set-children to add it to the card's children list.
let (error_id, error_comp) = ErrorDisplay::new("error")
    .id("catalog-forms-text-error")
    .bind("/_errors/demo/catalog-forms/text-value")  // data-store read for message
    .build();

let ops = vec![
    // Write the message to data store so ErrorDisplay.svelte can read it
    PatchOperation::Set {
        path: "/_errors/demo/catalog-forms/text-value".into(),
        value: serde_json::json!([{ "message": "Enter a valid email address." }]),
    },
    // Mount the error component at the slot id
    PatchOperation::SetNode {
        id: "catalog-forms-text-error-slot".into(),
        component: error_comp,
    },
    // Example set-children op — swap the card's children list so the error
    // appears in the right order. Concrete op per input per UI-SPEC §CAT-02 table.
];

Ok(vec![ProtocolMessage::Patch(PatchMessage {
    id: ctx.action.id.clone(),
    surface: "content".into(),
    patch: ops,
})])
```

**Locked per-input op mix** (UI-SPEC §CAT-02 lines 371-382; copy verbatim into the six handlers):

| Handler fn | Invalid-blur ops | Valid-blur ops |
|-----------|------------------|----------------|
| `validate_text_input` | `set-node` + `set-children` + data `set` | `set-children` (remove) + data `set` empty |
| `validate_select` | `set-node` + `set-children` | `delete-node` + data `set` empty |
| `validate_checkbox` | `set-node` (replace empty slot) + data `set` | `set-node` (replace slot back) + data `set` empty |
| `validate_switch` | `set-node` + data `set` | `set-node` (revert) + data `set` empty |
| `validate_radio` | `set-node` + `set-children` + data `set` | `set-children` (remove) + data `set` empty |
| `validate_textarea` | `set-node` + `set-children` | `delete-node` + data `set` empty |

**Anti-pattern warning** (RESEARCH.md Pitfall 10): these handlers MUST NOT write to `GalleryState.demo_values`. The `/_errors/{bind}` path is the FRONTEND surface store, not the backend AppState. Do not acquire the `Arc<RwLock<...>>` lock.

---

### `backend/crates/gallery-demo/src/handlers/show.rs` (MODIFY — extend `seed_for_key`)

**Pattern source:** existing match arms in the same file (lines 57-98). Copy the shape exactly — one arm per new catalog key, each returning a `serde_json::json!({ "demo": { "catalog-<family>": { ... } } })` value.

**Arms to add** (per UI-SPEC §CAT-02 seed table + RESEARCH.md Pattern 8):

```rust
"catalog-buttons" => serde_json::json!({}),   // no state — pure display
"catalog-forms" => serde_json::json!({
    "demo": { "catalog-forms": {
        "text-normal": "",
        "text-disabled": "Cannot edit",
        "text-with-error": "bad-input",
        "text-focused": "",
        "text-desc": "",
        "text-value": "",
        // … 30 more paths per UI-SPEC §CAT-02 lines 384-429
    }},
    "_errors": { "demo": { "catalog-forms": {
        "text-with-error": "Enter a valid email address.",
        "select-with-error": "Please make a selection.",
        "checkbox-with-error": "You must agree to continue.",
        "switch-with-error": "Notifications must be enabled.",
        "radio-with-error": "Please pick one option.",
        "textarea-with-error": "Bio must be at least 20 characters.",
    }}},
}),
"catalog-data-table" => serde_json::json!({
    "demo": { "catalog-data-table": {
        "rows": fixtures_rows_as_object_map(50),  // first page
    }},
}),
"catalog-feedback" => serde_json::json!({
    "demo": { "catalog-feedback": {
        "errors": [{ "message": "Sample error: failed to load resource. Retry or check your connection.", "path": null }],
    }},
}),
"catalog-typography" => serde_json::json!({}),  // no state
```

**Helper idiom** (for `catalog-data-table` initial page): convert `fixtures::synthetic_rows(50)` into the object-map shape the frontend expects per the existing `seed_table_rows()` lines 102-115 contract. Extract a local helper; do NOT call `seed_table_rows()` (D-4-C locks it as untouched).

**Seed-alignment hard contract:** every `.bind("/demo/catalog-<family>/<path>")` in the catalog fns MUST have a matching entry here. This is the G-05 lesson from Phase 17 (RESEARCH.md Pitfall 1).

---

### `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` (MODIFY — add source-dispatch)

**Analog:** `backend/crates/crm-demo/src/handlers/fetch_rows.rs` lines 37-51 (`FetchRowsPayload` struct) + lines 100-158 (parse + dispatch + emit).

**Minimum diff** (RESEARCH.md Pattern 6 lines 408-458):
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
                .map(|r| {
                    let mut v = serde_json::to_value(&r).expect("Row serialization");
                    // Append Actions-column data per UI-SPEC §CAT-03 (RESEARCH.md Q9)
                    v["actions"] = serde_json::json!([
                        {"label": "Edit",      "action": {"type": "click", "name": "gallery-demo/noop"}},
                        {"label": "Delete",    "action": {"type": "click", "name": "gallery-demo/noop"}},
                        {"label": "Duplicate", "action": {"type": "click", "name": "gallery-demo/noop"}},
                    ]);
                    v
                })
                .collect::<Vec<_>>();
            ("/demo/catalog-data-table/rows", json_rows)
        }
        other => return Err(ActionError::BadPayload(format!("unknown fetch-rows source: {other}"))),
    };
    // … one `PatchOperation::Set { path: format!("{path}/{id}"), value: row }` per row
}
```

**Path prefix must match `DataTable::bind(...)`** (RESEARCH.md §Q4 line 633): catalog screen uses `.bind("/demo/catalog-data-table/rows")` → handler emits to `/demo/catalog-data-table/rows/{id}`.

---

### `backend/crates/gallery-demo/src/handlers/mod.rs` (MODIFY — register actions)

**Pattern source:** existing `register_gallery_actions()` at lines 22-41 of the same file. Each new action is a new `.action(name, box_handler(fn), AuthRequirement::None)` chain-call in the same style.

**Six new registrations** (RESEARCH.md Pattern 8):
```rust
.action("gallery-demo/catalog-forms/validate-text-input",
    box_handler(catalog_forms::validate_text_input), AuthRequirement::None)
.action("gallery-demo/catalog-forms/validate-select",
    box_handler(catalog_forms::validate_select), AuthRequirement::None)
.action("gallery-demo/catalog-forms/validate-checkbox",
    box_handler(catalog_forms::validate_checkbox), AuthRequirement::None)
.action("gallery-demo/catalog-forms/validate-switch",
    box_handler(catalog_forms::validate_switch), AuthRequirement::None)
.action("gallery-demo/catalog-forms/validate-radio",
    box_handler(catalog_forms::validate_radio), AuthRequirement::None)
.action("gallery-demo/catalog-forms/validate-textarea",
    box_handler(catalog_forms::validate_textarea), AuthRequirement::None)
```

**`pub mod catalog_forms;`** declaration added in the module-list block at lines 10-16 of the same file.

---

### `backend/crates/gallery-demo/src/lib.rs` (MODIFY — module declarations)

**Pattern source:** existing `pub mod handlers; pub mod home; pub mod state;` at lines 25-27 of the same file. Add `pub mod catalog;` and `pub mod fixtures;` in the same block.

---

### `backend/crates/marionette/src/builders/button.rs` (MODIFY — Gap 1 Button struct)

**Pattern source:** optional-fields idiom already used extensively in the same file and every other builder (e.g., `text_input.rs` lines 11-29 shows `description` + `full_width` added as additive optional fields).

**Diff to the struct** (RESEARCH.md §Q5 + Gap 1):
```rust
#[derive(ComponentBuilder)]
#[component(type = "button")]
pub struct Button {
    pub label: String,
    #[builder(optional)]
    pub variant: Option<String>,
    #[builder(optional)]
    pub size: Option<String>,
    #[builder(optional)]
    pub disabled: Option<bool>,
    // ---- NEW (Phase 18 Gap 1) ----
    #[builder(optional)]
    pub loading: Option<bool>,
    #[builder(optional)]
    pub icon: Option<String>,
    #[builder(optional)]
    pub aria_label: Option<String>,
}
```

The `#[derive(ComponentBuilder)]` macro auto-generates `.loading(...)`, `.icon(...)`, `.aria_label(...)` setters for each new optional field (RESEARCH.md Assumption A4 — verified against `marionette-macros/src/component_builder.rs:170-206`).

**Leaf demo regression risk** (RESEARCH.md Pitfall 6): after this change, the existing `gallery_demo()` at lines 26-42 will render "Destructive" as an actually-red button (currently rendered as default because `Button.svelte` reads `props.color` + `props.outline`, not `props.variant`). That is CORRECT behavior and the leaf demo needs a re-UAT snapshot.

---

### `frontend/src/lib/components/form/Button.svelte` (MODIFY — read variant/size directly)

**Pattern source:** the shadcn `Button` API (vendored at `frontend/src/lib/components/ui/button/index.ts`) accepts `variant: "default" | "destructive" | "outline" | "ghost" | "link" | "secondary"` and `size: "default" | "sm" | "lg" | "icon"`. Current `Button.svelte` lines 24-31 DERIVE variant from `props.color` / `props.outline` — a historical quirk. Rewrite to pass `variant` and `size` through directly.

**Diff shape** (RESEARCH.md §Q5 lines 643-651):
```svelte
<script lang="ts">
  // ... existing imports ...
  let variant = $derived(
    (props.variant as string | undefined) ?? 'default'
  );
  let size = $derived(
    (props.size as string | undefined) ??
    (isIconOnly ? 'icon' : 'default')
  );
  // Drop the color/outline derivation entirely (no caller uses `.color(` in Rust,
  // verified by grep in RESEARCH.md Gap 1 row).
</script>

<ShadcnButton
  {variant}
  {size}
  disabled={isLoading || (props.disabled as boolean)}
  onclick={handleClick}
  class={props.icon && props.label ? 'gap-2' : ''}
  aria-label={isIconOnly ? (props.aria_label as string) ?? (props.label as string) ?? (props.icon as string) : undefined}
>
  ...
</ShadcnButton>
```

**Prop name note:** Svelte reads from the JSON the backend sent; the Rust field `aria_label: Option<String>` serializes to key `"aria_label"` (snake_case, like `full_width` in `TextInput.svelte:61`). The Svelte side reads `props.aria_label` (not `props.ariaLabel` — which was the old hand-coded key, used only by Phase 12 pre-Gap-1 code).

---

### `frontend/src/lib/components/form/{SelectInput,Checkbox,Switch,RadioGroup}.svelte` (MODIFY — add blur dispatch)

**Analog:** `frontend/src/lib/components/form/TextInput.svelte` lines 45-56 — the exact `handleBlur` to copy + the `onblur={handleBlur}` wiring on the input element.

**Pattern to copy verbatim** (TextInput.svelte):
```svelte
function handleBlur() {
    if (bind) {
        clearDirty(bind, (op) => setData(surface, op.path, op.value));
        if (action?.type === 'blur') {
            sendAction(
                action.name ?? action.type,
                { value: getData(surface, bind!) },
                action.target
            );
        }
    }
}
```

**Per-file wire-up differences** (RESEARCH.md §Q1):

| Component | Element for onblur | Notes |
|-----------|-------------------|-------|
| `TextInput.svelte` | `<Input onblur={handleBlur}>` (line 77) | ALREADY DONE — reference only |
| `Textarea.svelte` | `<ShadcnTextarea onblur={handleBlur}>` (line 77) | ALREADY DONE — reference only |
| `SelectInput.svelte` | hook into `handleOpenChange(open: false)` (line 57-67) | "Blur" semantic = popover close; no DOM blur on closed Select |
| `Checkbox.svelte` | attach `onblur={handleBlur}` to `<ShadcnCheckbox>` (line 44) OR wrap `<Field.Field>` in `<div onfocusout={handleBlur}>` if onblur doesn't propagate | Researcher Assumption A2 — spike in Chrome MCP |
| `Switch.svelte` | attach `onblur={handleBlur}` to `<Switch>` (line 45) OR `<div onfocusout>` wrapper | Same as Checkbox |
| `RadioGroup.svelte` | wrap `<RadioGroup>` in `<div onfocusout={handleBlur}>` | bits-ui RadioGroup doesn't expose onblur |

**Value extraction differs per component:** TextInput uses `getData(surface, bind!)` (string). Checkbox/Switch read `checked` boolean; SelectInput + RadioGroup read `value` string. Keep the `{ value: ... }` payload shape identical (matches backend handler expectation per RESEARCH.md Pitfall 9).

---

### `frontend/src/app.css` (MODIFY — extend safelist)

**Pattern source:** existing `@source inline(...)` directive at line 7 of the same file.

**Diff:** replace the existing `@source inline("grid-cols-1 … md:grid-cols-6")` with RESEARCH.md Pattern 4 verbatim string (lines 328-330):
```css
@source inline("grid-cols-1 grid-cols-2 grid-cols-3 grid-cols-4 grid-cols-5 grid-cols-6 grid-cols-7 grid-cols-8 md:grid-cols-1 md:grid-cols-2 md:grid-cols-3 md:grid-cols-4 md:grid-cols-5 md:grid-cols-6 sm:grid-cols-1 sm:grid-cols-2 sm:grid-cols-3 sm:grid-cols-4 sm:grid-cols-5 sm:grid-cols-6 lg:grid-cols-1 lg:grid-cols-2 lg:grid-cols-3 lg:grid-cols-4 lg:grid-cols-5 lg:grid-cols-6 lg:grid-cols-7 lg:grid-cols-8");
```

Tailwind v4 JIT scans Svelte templates for class literals but dynamic classes emitted from Rust (via `Container::class(...)`) never appear in a scanned source file; the safelist is the only path (RESEARCH.md Pitfall 2).

---

### `backend/crates/marionette/GALLERY-DEMOS.md` (MODIFY — coverage matrix rows)

**Pattern source:** existing coverage matrix rows in the same doc (one row per currently-registered demo key). Append five new rows:
- `catalog-buttons` — yes
- `catalog-forms` — yes
- `catalog-data-table` — yes
- `catalog-feedback` — yes
- `catalog-typography` — yes

Add a §Catalog-Screens section explaining these are app-level showcases (not framework demos) and linking the `gallery-demo/src/catalog/<family>.rs` file convention. Do NOT touch the §Contract or §Skip list sections.

---

## Shared Patterns

### Pattern: catalog fn contract (applies to ALL 5 catalog fns)

**Source:** `backend/crates/marionette/src/builders/*.rs` — every file with a `gallery_demo()` sibling follows this shape (19 analogs).

```rust
#[cfg(feature = "gallery")]
#[marionette_macros::gallery_demo(key = "catalog-<family>", name = "Catalog: <Family>")]
#[must_use]
pub fn gallery_demo() -> Vec<crate::gallery::Node> {
    // Pure — zero args, zero generics, no async, no I/O.
    // Must return Vec<Node> (Phase 16 D-Z1 signature after the 16.5 refactor).
    // The macro enforces the #[must_use] and the parameterless signature.
}
```

**Pure-fn invariants** (inherited from Phase 17 §D-Z1):
- No `async` — catalog fns are synchronous.
- No `fn(&self)` — free fn at module root.
- No `?` operator — infallible. Panic is acceptable for programmer-error (e.g. invalid id), but prefer `.expect("stable reason")`.
- First entry in the returned `Vec<Node>` is the ROOT tuple (`(root_id, root_component)`); handler uses `nodes_vec[0].0` as the render root.

### Pattern: Container-as-Card (applies to CAT-01, CAT-02, CAT-04 per UI-SPEC §D-3-D)

**Source:** UI-SPEC §Spacing Scale line 47 — LOCKED class string:
```
rounded-lg border bg-card text-card-foreground shadow-sm p-6 flex flex-col gap-4
```

**Why not `Container::card(true)`:** `frontend/src/lib/components/layout/Container.svelte:36-41` applies `max-w-md` + vertical centering on the `card=true` branch — wrong for full-width catalog layouts (RESEARCH.md §Q7 + Pattern 3).

**Why not a new `Card` Rust builder:** new Rust struct + new Svelte component + new registry entry + new `gallery_demo` sibling. Out of scope for Phase 18 (RESEARCH.md §Q7 final recommendation).

### Pattern: outer catalog-root Container (applies to ALL 5 catalog fns)

**Source:** UI-SPEC §Spacing Scale line 46 — LOCKED class string:
```
flex flex-col gap-6 p-6
```

Composition:
```rust
Container::new()
    .id("catalog-<family>-root")
    .class("flex flex-col gap-6 p-6")
    .children(vec![title_h1, intro_text, card1, card2, ...])
    .build_with_children()
```

### Pattern: inner responsive grid (applies to CAT-01, CAT-02, CAT-04, CAT-05)

**Source:** UI-SPEC §Spacing Scale line 48 + §Responsive Breakpoints line 650:

| Screen | Inner grid class |
|--------|------------------|
| CAT-01 | `grid grid-cols-1 sm:grid-cols-4 lg:grid-cols-4 gap-3` |
| CAT-02 | `grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3` |
| CAT-04 triggers | `grid grid-cols-1 sm:grid-cols-3 gap-3` |
| CAT-04 placeholders | `grid grid-cols-1 sm:grid-cols-3 gap-3` |
| CAT-05 icon grid | `grid grid-cols-4 sm:grid-cols-6 lg:grid-cols-8 gap-3` |
| CAT-05 swatch grid | `grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 gap-3` |

All classes MUST be present in the `app.css` safelist (Pattern above).

### Pattern: seed-alignment (applies to all catalog fns that read bind)

**Source:** `backend/crates/gallery-demo/src/handlers/show.rs` lines 56-99 `seed_for_key(...)` — plus Phase 17 Plan 17-05/17-06 G-05 regression lessons.

Rule: every `.bind("/demo/<key>/<path>")` in a catalog fn MUST have a matching entry in the `seed_for_key("<key>")` arm. Default value is explicit (`""`, `false`, `null`), never missing. A missing seed ⇒ frontend's `getData()` returns `undefined` ⇒ guarded components render empty silently.

### Pattern: `gallery-demo/*` action namespace (applies to all new actions)

**Source:** Phase 17 §D-C4 + `handlers/mod.rs` existing registrations (lines 31-40).

- Blur-validate actions: `gallery-demo/catalog-forms/validate-<input>`
- Trigger-open actions: reuse existing `gallery-demo/modal-open`, `gallery-demo/confirm-open`, `gallery-demo/toast-fire` (no new actions for CAT-04).
- fetch-rows: reuse existing `fetch-rows` (generic; extended with new source arm).
- noop: reuse existing `gallery-demo/noop` (fires a toast naming the source; already wired).

### Pattern: `Vec<Node>` assembly with composite nesting

**Source:** `backend/crates/marionette/src/builders/form.rs` lines 24-48 (the canonical composite-nesting idiom — reuse sub-demos as subtrees).

```rust
let (outer_root, outer_desc) = OuterBuilder::new()
    .id("outer-id")
    .children(vec![child_root_tuple_1, child_root_tuple_2])
    .build_tree();

let mut all = vec![outer_root];
all.extend(subtree_1_desc.into_iter().skip(1));
all.extend(subtree_2_desc.into_iter().skip(1));
all.extend(outer_desc);
all
```

Catalog fns that compose many per-Card children (CAT-01 with 60 cells, CAT-02 with six complex Cards, CAT-05 with ~47 static cells) use this flatten-to-Vec<Node> pattern at the outermost level.

---

## No Analog Found

None — every file in Phase 18 has a strong in-repo analog. The three framework gaps (Button variant/size, blur wiring on 4 components, Tailwind safelist) each have an analog INSIDE the file being modified: existing optional-field pattern in `text_input.rs` for Button; existing `handleBlur` in `TextInput.svelte:45-56` for the 4 form components; existing `@source inline(...)` at `app.css:7` for the safelist extension.

---

## Metadata

**Analog search scope:**
- `backend/crates/gallery-demo/src/` (all existing handlers + home.rs + state.rs)
- `backend/crates/marionette/src/builders/` (all 22 per-component files, specifically their `gallery_demo()` siblings and builder struct definitions)
- `backend/crates/marionette/src/validation.rs` (`validation_error_patch` helper)
- `backend/crates/crm-demo/src/handlers/fetch_rows.rs` + `contact.rs` (source-dispatch + validate-then-patch)
- `frontend/src/lib/components/form/*.svelte` (blur wiring reference + Button structure)
- `frontend/src/lib/components/layout/{Container,Heading,Grid}.svelte` (render contracts for `class` + `level` + `cols`)
- `frontend/src/lib/registry/icons.ts` (14 registered icons list)
- `frontend/src/app.css` (safelist directive location + OKLCH token declarations)

**Files scanned (read):** 20
**Pattern extraction date:** 2026-04-23
**Phase:** 18 — catalog-screens
