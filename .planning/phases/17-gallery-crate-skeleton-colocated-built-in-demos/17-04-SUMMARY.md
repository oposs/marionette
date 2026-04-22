---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 04
subsystem: gallery-sdui
tags: [gallery, demos, linkme, feature-gate, sweep, docs]

requires:
  - marionette::gallery::registered_demos (Phase 16 FRAME-01)
  - marionette_macros::gallery_demo macro (Phase 16 FRAME-02, Plan 17-01 Vec<Node> refactor)
  - per-component file layout (Plan 17-02)

provides:
  - 19 `gallery_demo()` siblings across marionette's builder files (DEMO-01)
  - GALLERY-DEMOS.md authoring contract (DEMO-02)
  - gallery::builtin_coverage_tests coverage regression test
  - REQUIREMENTS.md §CRATE-01 ordinal reconciliation

affects:
  - marionette/src/builders/{button,text_input,select,checkbox,grid,heading,text,form,textarea,radio_group,switch,field_set,data_table,modal,toast,confirm_dialog,spinner,error_display,app_shell}.rs
  - marionette/src/builders/{mod,standard}.rs (ambiguous_glob_reexports allow)
  - marionette/src/lib.rs (extern crate self as marionette)
  - marionette/src/gallery.rs (builtin_coverage_tests module)
  - backend/crates/marionette/GALLERY-DEMOS.md (new)
  - .planning/REQUIREMENTS.md (CRATE-01 rewording)

tech-stack:
  added: []
  patterns:
    - "linkme-backed demo registration via #[gallery_demo(key = \"...\")]"
    - "Composite demo nesting via `.into_iter().skip(1)` flattening"
    - "Feature-gated coverage test guards both IN_SCOPE_KEYS presence and SKIPPED_KEYS absence"

key-files:
  created:
    - backend/crates/marionette/GALLERY-DEMOS.md
  modified:
    - backend/crates/marionette/src/lib.rs
    - backend/crates/marionette/src/gallery.rs
    - backend/crates/marionette/src/builders/mod.rs
    - backend/crates/marionette/src/builders/standard.rs
    - backend/crates/marionette/src/builders/button.rs
    - backend/crates/marionette/src/builders/text_input.rs
    - backend/crates/marionette/src/builders/select.rs
    - backend/crates/marionette/src/builders/checkbox.rs
    - backend/crates/marionette/src/builders/grid.rs
    - backend/crates/marionette/src/builders/heading.rs
    - backend/crates/marionette/src/builders/text.rs
    - backend/crates/marionette/src/builders/textarea.rs
    - backend/crates/marionette/src/builders/radio_group.rs
    - backend/crates/marionette/src/builders/switch.rs
    - backend/crates/marionette/src/builders/spinner.rs
    - backend/crates/marionette/src/builders/error_display.rs
    - backend/crates/marionette/src/builders/form.rs
    - backend/crates/marionette/src/builders/field_set.rs
    - backend/crates/marionette/src/builders/data_table.rs
    - backend/crates/marionette/src/builders/modal.rs
    - backend/crates/marionette/src/builders/toast.rs
    - backend/crates/marionette/src/builders/confirm_dialog.rs
    - backend/crates/marionette/src/builders/app_shell.rs
    - .planning/REQUIREMENTS.md

decisions:
  - "Add `extern crate self as marionette;` to lib.rs to make the gallery_demo macro's `::marionette::…` absolute paths resolve inside the marionette crate itself (the macro was designed for external consumers like gallery-smoke)."
  - "Apply module-level `#![allow(ambiguous_glob_reexports)]` in builders/mod.rs and builders/standard.rs: every in-scope builder module re-exports a `gallery_demo` fn, so the glob re-exports collide on that ident. Callers access gallery_demo via explicit module paths (`crate::builders::<module>::gallery_demo`) — never via the parent namespace."
  - "Grid demo carries a local `#[allow(clippy::many_single_char_names)]` because a/b/c/d/e/f names intentionally mirror the rendered Heading labels (spreadsheet-cell style)."
  - "DataTable demo returns a single-tuple Vec (no descendants) since DataTable is a leaf Component with columns encoded in props."
  - "Modal/Toast/ConfirmDialog demos ship trigger-Button + explainer-Text patterns (D-A4); the popup itself renders out-of-band via the gallery-demo crate's handlers (Plan 17-03 territory)."

metrics:
  duration: "~45 min"
  tasks-completed: 5/6
  completed-date: 2026-04-22
---

# Phase 17 Plan 04: Colocated built-in demos + GALLERY-DEMOS.md + coverage test (Tasks 1-5 complete, Task 6 pending orchestrator)

**One-liner:** 19 `#[gallery_demo]` siblings across marionette's builders, a feature-gated coverage test, a 200-line authoring contract doc, and the CRATE-01 ordinal fix — Task 6's Chrome MCP UAT is handed back to the orchestrator.

## What shipped

### Tasks 1-3: 19 `gallery_demo()` siblings added (DEMO-01)

**12 leaf demos** (Task 1, commit 57ccded): Button, TextInput, Select, Checkbox, Grid, Heading, Text, Textarea, RadioGroup, Switch, Spinner, ErrorDisplay. Each stacks 2-3 representative instances inside a Container per D-A1. Interactive leaves (Button, Checkbox, Switch) fire `gallery-demo/noop` actions; binding leaves (TextInput, Select, Textarea, RadioGroup, Switch, Checkbox) use `/demo/{key}/...` paths per D-D2.

**6 composite demos** (Task 2, commit a6f5222): Form, FieldSet, DataTable, Modal, Toast, ConfirmDialog.
- **Form** + **FieldSet** nest `text_input::gallery_demo()` + `select::gallery_demo()` inline, flattening descendants with `.into_iter().skip(1)` after feeding the children's root tuple into `Form::new().children(...).build_tree()`.
- **Modal**, **Toast**, **ConfirmDialog** follow D-A4: trigger Button + explainer Text. The popup/toast themselves render out-of-band via Plan 17-03's handlers.
- **DataTable** demo carries 4 columns (id, name, email, created with ColumnKind::Date), source="demo-rows".

**1 hand-designed AppShell demo** (Task 3, commit 9c18ed3) per D-A2: SideNav with 3 NavItems (Dashboard, Reports, Settings) + Heading + Container-wrapped Text. Phase 19 EXER-01 will exercise outer+inner nesting explicitly.

### Task 3: `builtin_coverage_tests` module (commit 9c18ed3)

Added `#[cfg(all(test, feature = "gallery"))] mod builtin_coverage_tests` to `marionette/src/gallery.rs`:

| Test                         | Asserts                                                                     |
|------------------------------|-----------------------------------------------------------------------------|
| `all_in_scope_keys_present`  | All 19 IN_SCOPE_KEYS appear in `registered_demos()`; force-links each module |
| `skipped_keys_not_present`   | None of container/side-nav/nav-item/nav-group/surface-mount/field-separator appear |

The "force-link every builder module" pattern prevents test-binary DCE from dropping a builder module before its linkme static registers — without it, a silent regression could slip through.

### Task 4: GALLERY-DEMOS.md + REQUIREMENTS.md reconciliation (commit 93c613a)

`backend/crates/marionette/GALLERY-DEMOS.md` (200 lines, 7 top-level sections):
1. Contract (feature gate, explicit key, Vec<Node>, pure fn, existing methods only)
2. Bind-path convention (`/demo/{key}/...`)
3. Action namespace table (6 `gallery-demo/*` + 2 frontend-hardcoded `close-modal`, `dismiss-toast`)
4. Skip list + rationale (6 structural components demoed transitively)
5. Composite-nesting rule (D-A1) + AppShell exception (D-A2)
6. Coverage matrix (19 yes + 6 skip)
7. Recipe — 5-step guide for adding new built-ins

REQUIREMENTS.md §CRATE-01 rewording: "5th Cargo workspace entry" → "6th Cargo workspace entry (the 5th slot is occupied by `gallery-smoke`, a permanent test-fixture crate landed in Phase 16)". Single-line cosmetic edit per RESEARCH.md Open Q #4.

### Task 5: Workspace verification sweep (commit a25d203)

Full verification executed:

| Gate                                                                 | Result |
|----------------------------------------------------------------------|--------|
| `cargo build -p marionette` (no gallery feature; FRAME-03 guard)     | PASS   |
| `cargo build -p marionette --features gallery`                       | PASS   |
| `cargo test --workspace --exclude crm-demo --features gallery`       | PASS (100+ tests across 5 crates) |
| `cargo clippy -p marionette --features gallery -- -D warnings`       | PASS   |
| `cargo clippy --workspace --exclude crm-demo --features gallery -- -D warnings` | PASS |
| `cargo build -p marionette --features gallery --release`             | PASS   |

Exactly 2 coverage tests (`all_in_scope_keys_present` + `skipped_keys_not_present`) pass alongside Phase 16's 4 gallery tests — total 6 gallery-module tests.

### Task 6: Chrome MCP UAT (pending orchestrator)

Per the orchestrator's instructions, Task 6 (Chrome MCP UAT — human-verify checkpoint) is handed back. The gallery-demo binary (Plan 17-03) does not exist in this worktree, so the UAT must wait for Plan 17-03 to merge before driving Chrome MCP through the 20-demo nav walk.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `#[gallery_demo]` macro emits `::marionette::…` absolute paths that don't resolve inside the marionette crate**

- **Found during:** Task 1 — first `cargo build -p marionette --features gallery` failed with E0433 "could not find `marionette` in the list of imported crates" for every gallery_demo annotation site.
- **Issue:** The macro was designed for external consumers (like gallery-smoke) and hardcodes `::marionette::gallery::__linkme` absolute paths. When applied from within the marionette crate itself, those paths don't resolve because the crate doesn't alias itself.
- **Fix:** Added `extern crate self as marionette;` to `backend/crates/marionette/src/lib.rs`. This is the standard Rust pattern for making a crate's own absolute path work inside itself (`$crate` paths would be cleaner, but the macro is already written and `extern crate self` is the minimal-blast-radius fix).
- **Files modified:** `backend/crates/marionette/src/lib.rs`
- **Commit:** 57ccded (folded into Task 1)

**2. [Rule 3 - Blocking] Ambiguous glob re-exports on `gallery_demo` ident across builder modules**

- **Found during:** Task 1 — `cargo build -p marionette --features gallery` produced 2 `ambiguous_glob_reexports` warnings because every in-scope builder module now defines a public `gallery_demo` fn, and `builders/mod.rs` + `builders/standard.rs` both do `pub use <module>::*` across all of them.
- **Issue:** Rust's glob re-export lint flags the collision; callers reaching for `marionette::builders::gallery_demo` would get an undefined resolution.
- **Fix:** Applied module-level `#![allow(ambiguous_glob_reexports)]` in both `builders/mod.rs` and `builders/standard.rs`. Callers access demo fns via their explicit module path (`crate::builders::button::gallery_demo`) — the parent `builders` namespace intentionally does not resolve `gallery_demo`. Documented the rationale inline in both files.
- **Files modified:** `backend/crates/marionette/src/builders/mod.rs`, `backend/crates/marionette/src/builders/standard.rs`
- **Commit:** 57ccded (folded into Task 1)

**3. [Rule 1 - Clippy] `clippy::many_single_char_names` on grid::gallery_demo**

- **Found during:** Task 5 Step 4 — `cargo clippy -p marionette --features gallery -- -D warnings` flagged the 6 bindings (a/b/c/d/e/f) in grid.rs.
- **Issue:** Clippy pedantic flags >=4 single-char names in one scope.
- **Fix:** Attached a local `#[allow(clippy::many_single_char_names)]` attribute on the `gallery_demo` fn. The names intentionally mirror the rendered Heading labels ("A".."F", spreadsheet-cell convention); renaming to `cell_a`/`cell_b`/... would obscure the visual intent. Documented the rationale in an inline comment.
- **Files modified:** `backend/crates/marionette/src/builders/grid.rs`
- **Commit:** a25d203

**4. [Rule 2 - Correctness] DataTable builder API mismatch with plan's example**

- **Found during:** Task 2 — plan's example used `TableColumn::new(key, label, ColumnKind::Text)` (3 args) and `DataTable::new().source(...).columns(...)`.
- **Issue:** Actual APIs are `TableColumn::new(key, label)` returning Self (then `.kind(ColumnKind::Date)` to set kind), and `DataTable::new(columns: Vec<TableColumn>)` taking positional columns. Also there's no plain text `.build()` → `Vec<Node>` — DataTable is a leaf, so `vec![(id, component)]` is the correct flat return.
- **Fix:** Adjusted the demo body to use the real API — 4 columns via `TableColumn::new(...).kind(...)`, `DataTable::new(columns).source("demo-rows").row_id_key("id").page_size(10u32).build()`, and `vec![(id, component)]` return.
- **Files modified:** `backend/crates/marionette/src/builders/data_table.rs`
- **Commit:** a6f5222 (folded into Task 2)

All auto-fixes land as clean root-cause edits (no deprecation shims, no legacy fallbacks — per pre-deployment no-backcompat posture). No architectural decisions were escalated to Rule 4.

## Evidence pointers

### Key-collision prevention (T-17.04-01 threat mitigation)

```
$ grep -rn '#\[gallery_demo' backend/crates/marionette/src/builders/ | grep -v 'key =' | wc -l
0

$ grep -rn 'pub fn gallery_demo() -> Vec<crate::gallery::Node>' backend/crates/marionette/src/builders/ | wc -l
19
```

Every `#[gallery_demo]` carries an explicit `key = "..."`. 19 sibling fns across the 19 in-scope builder files.

### Composite-nesting flattening (Form example)

```rust
// form.rs — Phase 17 Plan 04 Task 2
let text_input_nodes = crate::builders::text_input::gallery_demo();  // Vec<Node>: [container, ti_a, ti_b, ti_c]
let select_nodes     = crate::builders::select::gallery_demo();      // Vec<Node>: [container, sel_a, sel_b]

let (form_root, form_desc) = Form::new()
    .id("demo-form-root")
    .children(vec![
        text_input_nodes[0].clone(),  // text_input's Container root
        select_nodes[0].clone(),      // select's Container root
        submit,                       // demo-form-submit Button
    ])
    .build_tree();

let mut all = vec![form_root];
all.extend(text_input_nodes.into_iter().skip(1));  // preserve text_input's descendants
all.extend(select_nodes.into_iter().skip(1));      // preserve select's descendants
all.extend(form_desc);
all
```

### AppShell hand-design (D-A2)

Per the plan's D-A2 rationale, `app_shell.rs::gallery_demo()` does NOT auto-nest other gallery_demo() calls. Instead it hand-picks:
- **Sidebar:** SideNav with 3 NavItems (Dashboard, Reports, Settings) routed to `/demo/app-shell/{dashboard,reports,settings}`
- **Header:** Heading("Demo App")
- **Main:** Container wrapping explanatory Text that flags Phase 19 EXER-01 as the nested-shell exerciser

The choice keeps the AppShell demo surfacing a curated "this is how you'd really build it" shell rather than a combinatorially-arbitrary auto-nest.

### Coverage matrix ↔ builtin_coverage_tests

`builtin_coverage_tests::IN_SCOPE_KEYS` (20 entries in the test but only 19 from builders — nothing for `smoke`; the smoke-check test key comes from gallery-smoke, not marionette's builders) exactly mirrors the GALLERY-DEMOS.md coverage matrix's "yes" rows. Both lists MUST stay in sync — the recipe in GALLERY-DEMOS.md Step 5 documents the requirement.

The skip list (6 items) appears in three places consistently:
1. CONTEXT.md §D-B2 (decision record)
2. GALLERY-DEMOS.md's Skip list + rationale section (author-facing doc)
3. `gallery.rs::builtin_coverage_tests::SKIPPED_KEYS` (CI guard)

## Threat Flags

None introduced. Plan 17-04's surface is purely compile-time registration + one doc file + a lint-like test; the `gallery-show` action handler (Plan 17-03) is the runtime surface that consumes `registered_demos()`.

## Known Stubs

None. The AppShell demo's NavItems target `/demo/app-shell/{dashboard,reports,settings}` paths — these are routed by the gallery-demo crate's `handle_gallery_show` handler (Plan 17-03) and seeded as needed. No UI placeholders; every rendered component is wired to its own bind/action where applicable.

## Self-Check: PASSED

Files verified present:
- `backend/crates/marionette/GALLERY-DEMOS.md` — FOUND (200 lines)
- `backend/crates/marionette/src/gallery.rs` — FOUND with `mod builtin_coverage_tests`
- 19 builder files with gallery_demo siblings — FOUND (12 leaf + 6 composite + 1 app_shell)

Commits verified present (via `git log --oneline`):
- 57ccded — FOUND (Task 1: 12 leaf demos)
- a6f5222 — FOUND (Task 2: 6 composite demos)
- 9c18ed3 — FOUND (Task 3: AppShell + builtin_coverage_tests)
- 93c613a — FOUND (Task 4: GALLERY-DEMOS.md + REQUIREMENTS.md reconciliation)
- a25d203 — FOUND (Task 5: clippy many_single_char_names fix)

Verification commands run clean:
- `cargo build -p marionette` → exit 0
- `cargo build -p marionette --features gallery` → exit 0
- `cargo test --workspace --exclude crm-demo --features gallery --lib` → all passing
- `cargo clippy --workspace --exclude crm-demo --features gallery -- -D warnings` → exit 0
- `cargo build -p marionette --features gallery --release` → exit 0

## Task 6 — Checkpoint handoff to orchestrator

Task 6 (Chrome MCP UAT walking every nav entry in the running gallery) is **not executed** in this plan. The orchestrator must drive Chrome MCP through the gallery-demo app produced by Plan 17-03 once that plan lands and merges. See the CHECKPOINT REACHED message returned to the orchestrator for the walkthrough script.
