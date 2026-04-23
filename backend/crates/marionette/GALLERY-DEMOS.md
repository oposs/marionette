# Gallery Demos — Authoring Contract

**Purpose:** This document is the permanent author-facing contract for the
`#[gallery_demo]` ecosystem. Every built-in component in `marionette` ships
a pure-fn sibling named `gallery_demo()` that the `gallery-demo` binary
auto-discovers and surfaces in its nav. This doc explains how to write one
correctly.

---

## Contract

Every `gallery_demo()` sibling MUST:

1. Be annotated `#[cfg(feature = "gallery")]` so default marionette builds
   compile zero demo symbols (FRAME-03).
2. Be annotated `#[marionette_macros::gallery_demo(key = "<type-string>")]`
   with an EXPLICIT `key` argument. The key MUST match the builder's
   `#[component(type = "...")]` string verbatim (e.g. `"text-input"`,
   `"data-table"`, `"confirm-dialog"`). Without an explicit key, the macro
   would default to the fn ident `"gallery_demo"` — collision across every
   builder file.
3. Be a `#[must_use] pub fn gallery_demo() -> Vec<crate::gallery::Node>` —
   a flat vector where index 0 is the root tuple `(String, Component)` and
   remaining entries are descendants.
4. Be **pure**: zero args, zero generics, zero where-clause, zero async,
   zero I/O, zero external state. The `#[gallery_demo]` macro enforces
   these syntactically.
5. Use existing builder methods only — no hand-rolled components, no new
   primitives. Composition reuses the builder vocabulary.

## Bind-path convention

Demo-level state paths follow `/demo/{key}/...` where `{key}` is the
annotation key. Examples:

- `/demo/text-input/value` — the demo TextInput's value
- `/demo/select/value` — the demo Select's value
- `/demo/form/email` + `/demo/form/name` — the demo Form's bound fields
- `/demo/data-table/rows/{id}` — DataTable demo rows delivered by the
  gallery-demo crate's `fetch-rows` handler

The gallery-demo crate's `handle_gallery_show` seeds matching defaults
when the demo is visited.

## Action namespace

Demo-fired actions use the `gallery-demo/*` namespace:

| Action name                   | Trigger                                 | Handler                                        |
|-------------------------------|-----------------------------------------|------------------------------------------------|
| `gallery-demo/noop`           | Leaf-demo Buttons, interactive leaves   | Enqueues a toast naming the source action      |
| `gallery-demo/modal-open`     | Modal demo trigger                      | Renders Modal into `modal` sub-surface         |
| `gallery-demo/confirm-open`   | ConfirmDialog demo trigger              | Renders ConfirmDialog into `modal` sub-surface |
| `gallery-demo/confirm-accept` | ConfirmDialog Accept                    | Clears `modal` + enqueues "accepted" toast     |
| `gallery-demo/confirm-reject` | ConfirmDialog Reject                    | Clears `modal` + enqueues "rejected" toast     |
| `gallery-demo/toast-fire`     | Toast demo Fire-toast Button            | Enqueues a demo toast                          |

Two action names live OUTSIDE the `gallery-demo/*` namespace because the
frontend hardcodes them:

| Action name       | Frontend hardcode site                                         |
|-------------------|----------------------------------------------------------------|
| `close-modal`     | `frontend/src/lib/components/popup/ModalSurface.svelte`        |
| `dismiss-toast`   | `frontend/src/lib/components/feedback/ToastSurface.svelte`     |

## Skip list + rationale

Some ComponentBuilder structs have no `gallery_demo()` sibling by design
(CONTEXT.md §D-B2):

| Struct            | Skipped because                                                                             | Demoed transitively via       |
|-------------------|---------------------------------------------------------------------------------------------|-------------------------------|
| `Container`       | Empty Container renders nothing; "wrap some Text" is indistinguishable from the Text demo.  | Every composite that wraps content |
| `SideNav`         | Standalone outside an AppShell Sidebar context looks contextually wrong.                    | `AppShell::gallery_demo`      |
| `NavItem`         | Single nav entry; only meaningful inside a SideNav.                                         | `AppShell::gallery_demo`      |
| `NavGroup`        | Nav subgroup; only meaningful inside a SideNav.                                             | `AppShell::gallery_demo`      |
| `SurfaceMount`    | Mount point with no visual; demoed wherever AppShell hosts it.                              | `AppShell::gallery_demo`      |
| `FieldSeparator`  | Divider inside a FieldSet; only meaningful inside a FieldSet.                               | `FieldSet::gallery_demo`      |

Note: `TableColumn`, `ColumnKind`, `Filter`, and related DataTable support
types are not ComponentBuilder structs — they're props types. They're
excluded by nature, not by the skip list.

## Composite-nesting rule (D-A1)

Composite demos SHOULD nest other `gallery_demo()` calls where the leaf-demo
shape fits. Examples:

- `Form::gallery_demo()` calls `crate::builders::text_input::gallery_demo()` +
  `crate::builders::select::gallery_demo()` inline, takes the root tuple of
  each, and feeds them into `Form::new().children(...)`.
- `FieldSet::gallery_demo()` does the same.

The nesting unpacks like:

```rust
let text_input_nodes = crate::builders::text_input::gallery_demo();   // Vec<Node>
// text_input_nodes[0] is the Container root; rest are its descendants.
let form_children = vec![text_input_nodes[0].clone(), /* ... */];
let (form_root, form_desc) = Form::new().children(form_children).build_tree();

let mut all = vec![form_root];
all.extend(text_input_nodes.into_iter().skip(1));   // preserve descendants
all.extend(form_desc);
all
```

### AppShell exception (D-A2)

`AppShell::gallery_demo()` is hand-designed, NOT auto-nested. Too many
reasonable content combinations (which demo to put in sidebar? header?
main?) to pick automatically. The AppShell demo ships a curated "this is
how you'd really build it" showcase with 3 hand-picked NavItems, a specific
header, and a specific main-content block.

Other composites (Form, FieldSet, ConfirmDialog body) follow the standard
nesting rule. Modal, Toast, and ConfirmDialog use a "trigger-button +
explainer" pattern (D-A4) rather than nesting, because their interactive
semantics (opening a popup, firing a toast) don't compose naturally with
inline content.

## Coverage matrix

| Key              | Status   | Content shape                                            |
|------------------|----------|----------------------------------------------------------|
| button           | yes      | 3 Buttons (default, disabled, destructive) in Container  |
| text-input       | yes      | 3 TextInputs (default, disabled, with-description)       |
| select           | yes      | 2 Selects with fruit options                             |
| checkbox         | yes      | 3 Checkboxes (unchecked, with description, disabled)     |
| grid             | yes      | 2×3 Grid of Heading placeholders                         |
| heading          | yes      | 3 Headings (levels 1/2/3)                                |
| text             | yes      | 3 Text blocks (short, paragraph, technical)              |
| form             | yes      | Form nesting text-input + select + Submit Button         |
| textarea         | yes      | 2 Textareas (default, with description)                  |
| radio-group      | yes      | 1 RadioGroup with 3 options                              |
| switch           | yes      | 2 Switches                                               |
| field-set        | yes      | FieldSet nesting text-input + select                     |
| data-table       | yes      | DataTable with 4 columns + "demo-rows" source            |
| modal            | yes      | "Open modal" trigger Button + explainer Text             |
| toast            | yes      | "Fire toast" Button + label Heading                      |
| confirm-dialog   | yes      | "Open confirm" trigger Button + explainer Text           |
| spinner          | yes      | 3 Spinners (sm, md, lg)                                  |
| error-display    | yes      | 2 ErrorDisplays                                          |
| app-shell        | yes      | Hand-designed (D-A2): SideNav + Heading + main Container |
| container        | **skip** | See skip list                                            |
| side-nav         | **skip** | See skip list                                            |
| nav-item         | **skip** | See skip list                                            |
| nav-group        | **skip** | See skip list                                            |
| surface-mount    | **skip** | See skip list                                            |
| field-separator  | **skip** | See skip list                                            |
| catalog-buttons       | Catalog: Buttons       | yes | Plan 18-04 |
| catalog-forms         | Catalog: Forms         | yes | Plan 18-05 |
| catalog-data-table    | Catalog: Data Table    | yes | Plan 18-06 |
| catalog-feedback      | Catalog: Feedback      | yes | Plan 18-07 |
| catalog-typography    | Catalog: Typography    | yes | Plan 18-08 |

Coverage is documented here, not CI-enforced (CONTEXT.md §D-B4). A
GALLERY-LINT CI rule is deferred to v1.3+ per REQUIREMENTS.md §v1.3+.

Automated sanity: `marionette/src/gallery.rs`'s `builtin_coverage_tests`
module asserts this list matches the registered set (the 19 in-scope keys
must all be present; the 6 skipped keys must all be absent).

## Recipe — adding a new built-in component

When you add a new `ComponentBuilder` struct to `marionette`, ship a
`gallery_demo` sibling in the same commit:

1. **Add the ComponentBuilder struct** in its own file
   `backend/crates/marionette/src/builders/<snake_case>.rs` (per Phase 17
   D-B3 — one file per builder).

2. **Add the `gallery_demo()` sibling at the bottom of the same file:**

   ```rust
   #[cfg(feature = "gallery")]
   #[marionette_macros::gallery_demo(key = "<type-string>")]
   #[must_use]
   pub fn gallery_demo() -> Vec<crate::gallery::Node> {
       // 2-3 representative instances in a Container for leaves,
       // OR a nested composite with other gallery_demo() calls for composites.
   }
   ```

   The `key` MUST match the struct's `#[component(type = "<type-string>")]`
   verbatim.

3. **Update `backend/crates/marionette/src/builders/mod.rs`** to declare
   the new module: `pub mod <snake_case>;` + `pub use <snake_case>::*;`.

4. **Update this file's coverage matrix** — add a row for the new key.

5. **Update `marionette/src/gallery.rs`'s `builtin_coverage_tests::IN_SCOPE_KEYS`**
   constant with the new key (or add to `SKIPPED_KEYS` if it's a structural
   piece with no standalone demo value) and force-link the new module in
   `all_in_scope_keys_present`.

Rebuild the gallery: `cargo run -p gallery-demo`. The new component's
demo appears in the sidebar nav — alphabetically sorted — without
touching the gallery-demo binary.

---

## Popup composition

The popup toolbox is **compositional, not primitive-based**. Phase 17 moved
`ModalSurface.svelte` to a layout-root singleton mount (Plan 17-05) — it is
always live, reads the `modal` sub-surface tree, and wraps whatever you
render there in a shadcn `<Dialog.Root>` / `<Dialog.Content>` overlay.

**To open a popup with arbitrary content**, render any SDUI tree (Container,
Form, TextInput, Button, Heading, Text, …) to the `modal` sub-surface. The
`<Dialog.Root>` open/close state is driven by ModalSurface's `isOpen`
derivation: an empty `Container` root = closed; any non-empty tree = open.

**To close a popup**, render an empty `Container` (id convention:
`modal-empty`) into the `modal` sub-surface — ModalSurface observes the
empty tree and dismisses the dialog. The frontend also dispatches a
`close-modal` action when the user clicks the X button or backdrop; the
gallery-demo's `handle_modal_close` handler implements that by rendering
`modal-empty`.

There is **no `Modal::new(...)` wrapper** — the old `Modal` builder was
removed in Phase 17 Plan 17-08 after the popups-global refactor made it
dead code. Handler authors compose the popup body directly.

**Canonical "form in popup" recipe:**

```rust
pub async fn open_contact_form(ctx: HandlerContext) -> ActionResult {
    use marionette::builders::{Button, Container, Form, Heading, TextInput};
    use marionette_protocol::{Component, ComponentAction};
    use marionette_protocol::messages::RenderMessage;

    let name = TextInput::new("Name")
        .id("contact-name")
        .bind("/contact/name")
        .build();
    let email = TextInput::new("Email")
        .id("contact-email")
        .bind("/contact/email")
        .build();
    let form = Form::new()
        .id("contact-form")
        .children(vec![name, email])
        .build();

    let cancel = Button::new("Cancel")
        .id("contact-cancel")
        .variant("outline")
        .action(ComponentAction::click("close-modal"))
        .build();
    let save = Button::new("Save")
        .id("contact-save")
        .action(ComponentAction::click("save-contact"))
        .build();
    let button_row = Container::new()
        .id("contact-button-row")
        .children(vec![cancel, save])
        .build();

    let heading = Heading::new("New contact")
        .id("contact-heading")
        .build();
    let dialog_nodes = Container::new()
        .id("contact-popup-root")
        .children(vec![heading, form, button_row])
        .build_with_children();

    let mut nodes: std::collections::HashMap<String, Component> =
        std::collections::HashMap::new();
    for (id, c) in dialog_nodes {
        nodes.insert(id, c);
    }

    Ok(vec![marionette_protocol::ProtocolMessage::Render(RenderMessage {
        id: ctx.action.id.clone(),
        surface: "modal".into(),
        root: "contact-popup-root".into(),
        nodes,
        data: serde_json::json!({}),
    })])
}
```

The Save handler does its side-effects (DB write, toast enqueue, etc.) and
also renders `modal-empty` into the `modal` sub-surface to dismiss the
dialog.

**Note on builder APIs used above:**

- `TextInput::new(label)` takes the label as its positional argument;
  there is no separate `.label(...)` method. See
  `backend/crates/marionette/src/builders/text_input.rs`.
- `Button::new(label)` takes the label positional; variant defaults to
  shadcn's "default" when `.variant(...)` is omitted.
- `Container::new()` is argument-free; use `.children(vec![...])` to
  populate children and `.build_with_children()` to return the full
  adjacency-list vec.
- `Form::new()` is argument-free; the Submit label is optional via
  `.submit_label(...)`.

**When to use `ConfirmDialog` instead:** if the popup is a simple two-
choice accept/cancel prompt (title + message + two buttons), reach for the
`ConfirmDialog` primitive — it encodes that shape directly via the
structured `confirm_label` / `cancel_label` / `cancel_action` /
`destructive` contract (Plan 17-05). For anything more complex (form,
multi-step, custom layout), compose from raw nodes as above.

See also:
- `backend/crates/gallery-demo/src/handlers/modal.rs` — working example of
  `handle_modal_open` / `handle_modal_close` using this pattern.
- `backend/crates/marionette/src/builders/confirm_dialog.rs` — structured
  ConfirmDialog primitive (remains after G-08 cleanup).
- `frontend/src/lib/components/popup/ModalSurface.svelte` — the
  layout-root singleton that drives the Dialog overlay.

---

## Catalog Screens

Catalog screens are app-level showcases distinct from framework-level leaf demos. They live in
`backend/crates/gallery-demo/src/catalog/<family>.rs` (not under `marionette/src/builders/`) and
compose their content by calling builder constructors directly — never by invoking a leaf
`gallery_demo()` fn (per Phase 18 CONTEXT.md §D-2-B).

**Registration convention:** explicit `key = "catalog-<family>"` + `name = "Catalog: <Family>"` on
every `#[gallery_demo]` annotation. This keeps catalog entries visually clustered in the alphabetical
nav and keeps them clearly separable from leaf demos.

**File layout:**

```
backend/crates/gallery-demo/src/catalog/
├── mod.rs          -- declares the 5 per-family modules
├── buttons.rs      -- CAT-01 (every variant × size × state)
├── forms.rs        -- CAT-02 (every input × state, live validation patch-demo)
├── data_table.rs   -- CAT-03 (filter bar + virtualized scroll + column visibility)
├── feedback.rs     -- CAT-04 (toast / modal / confirm triggers + placeholder states)
└── typography.rs   -- CAT-05 (type scale + icon catalog + OKLCH swatches)
```

**When to add new catalog screens vs extend a leaf demo:**

- **Leaf demo** (in `marionette/src/builders/*.rs`): 2–3 representative instances of a single component
  so the gallery proves the builder works at all.
- **Catalog screen** (in `gallery-demo/src/catalog/*.rs`): exhaustive variant × state matrix of a
  component family — the "full visual surface" view.

Catalog screens do NOT replace leaf demos; they coexist (Phase 18 §D-2-A).

---

*Last updated: Phase 18 Plan 18-08 (2026-04) — Catalog Screens section added after CAT-05 ships.*
