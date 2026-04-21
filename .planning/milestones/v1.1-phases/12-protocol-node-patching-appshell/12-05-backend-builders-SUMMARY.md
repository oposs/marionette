---
phase: 12
plan: 05
subsystem: backend-builders
tags: [backend, rust, builders, app-shell, surface-mount]
requirements: [SHELL-04]
dependency_graph:
  requires: [12-01, 12-02]
  provides:
    - "SurfaceMount derived builder (component type 'surface-mount', required name: String)"
    - "AppShell hand-written builder with six named slots and flat-list build_with_children"
  affects:
    - backend/crates/marionette/src/builders/standard.rs
    - backend/crates/marionette/src/builders/app_shell.rs
tech_stack:
  added: []
  patterns:
    - "#[derive(ComponentBuilder)] for leaf components with a single required prop"
    - "Hand-written builder pattern: Entry-type::new() -> Builder, chainable slot methods taking (id, Component) tuples, props store slot IDs, build_with_children flattens into Vec<(id, Component)>"
    - "Targeted #[allow(clippy::new_ret_no_self)] on hand-written Entry::new() to match ComponentBuilder derive convention"
key_files:
  created: []
  modified:
    - backend/crates/marionette/src/builders/standard.rs
    - backend/crates/marionette/src/builders/app_shell.rs
decisions:
  - "SurfaceMount is a plain derived builder — no hand-written code needed (D-B2)"
  - "AppShell slot methods accept pre-built (String, Component) tuples; slot IDs land in props under sidebarNodeId/headerNodeId/footerNodeId/mainNodeId/popupsNodeId/toastsNodeId (D-B3)"
  - "build_with_children returns a flat Vec containing shell first, then slot roots in canonical order, then descendants — ready for direct insertion into RenderMessage.nodes (D-B4)"
  - "Absent slots are omitted from both props and the flat list — no placeholder entries"
metrics:
  duration: "~5 min wall-clock for Task 2 (Task 1 was a separate earlier run)"
  completed: "2026-04-10"
---

# Phase 12 Plan 05: Backend Builders Summary

One-liner: SurfaceMount derived builder plus hand-written AppShell builder with six named slots and a flat-list build_with_children that feeds RenderMessage.nodes.

## Scope

Two builders added to the `marionette` crate to support Part B (AppShell) of Phase 12:

1. **SurfaceMount** — leaf component (`type = "surface-mount"`) with a single required `name: String` prop. Added as a `#[derive(ComponentBuilder)]` struct to `builders/standard.rs`. On the frontend it will render a recursive `<Surface name={props.name}/>` mount, composing sub-surfaces (content / modal / toasts) into the shell surface.

2. **AppShell** — hand-written builder in `builders/app_shell.rs` (replacing the Plan 01 scaffold). Cannot be derived because slot setters have custom semantics (they store node IDs in props instead of emitting children array entries). It exposes six slot methods (`sidebar`, `header`, `footer`, `main`, `popups`, `toasts`), each taking a pre-built `(String, Component)` tuple; an `.id()` override for the shell's own UUID; a `.with_descendants(Vec)` sink for sub-tree nodes harvested from sub-builders' `build_tree()` calls; and a `.build_with_children()` that returns a flat `Vec<(String, Component)>` in the canonical order `[shell, sidebar, header, footer, main, popups, toasts, ...descendants]` with missing slots skipped.

## Tasks

### Task 1 — SurfaceMount derived builder (commit b7b3df8, prior run)

- Added `SurfaceMount` struct with required `name: String` prop to `backend/crates/marionette/src/builders/standard.rs` under a new `// -- Sub-surface mounting --` section between the Navigation and Form component blocks.
- The `#[derive(ComponentBuilder)] #[component(type = "surface-mount")]` macro invocation generates `SurfaceMount::new(name)`, all standard builder methods (`.id`, `.bind`, `.visible`, `.action`, `.children`, `.build`, `.build_with_children`, `.build_tree`) via the same expansion used by all other leaf components in the file.
- Added two inline tests:
  - `surface_mount_builder` — asserts default id prefix, `type`, `props.name`, absence of children/bind/action.
  - `surface_mount_builder_custom_id` — asserts `.id("shell-modal-mount")` overrides the generated UUID exactly.
- Renamed the existing `all_18_standard_types` coverage test to `all_19_standard_types`, added `SurfaceMount::new("x").build().1.r#type` to the `types` Vec, and inserted `"surface-mount"` into `expected` at the matching index. The test enforces lexical ordering between the two vectors.
- Updated the module doc header from "18 protocol component types" to "19 protocol component types".
- Fixed a pre-existing `clippy::doc_markdown` lint on `app_shell.rs` (the Plan 01 scaffold) by putting backticks around `AppShell` in its doc comment — paved the way for Task 2's clippy-clean rewrite.

### Task 2 — AppShell hand-written builder (commit 45b3be4, this run)

- Replaced the entire Plan 01 scaffold contents of `backend/crates/marionette/src/builders/app_shell.rs` with the real implementation.
- Public surface:
  - `pub struct AppShell;` — zero-sized entry point whose only method is `new()`.
  - `pub struct AppShellBuilder` (`#[derive(Default)]`) — holds six `Option<(String, Component)>` slot fields, a `Vec<(String, Component)>` descendants field, and an optional `id: Option<String>`.
  - Slot methods `sidebar / header / footer / main / popups / toasts`, each `#[must_use]` and taking `slot: (String, Component)`.
  - `with_descendants(Vec<(String, Component)>) -> Self` — append additional nodes (order preserved).
  - `id(impl Into<String>) -> Self` — override the generated UUID with a stable id.
  - `build(self) -> (String, Component)` — writes slot IDs into props under `sidebarNodeId / headerNodeId / footerNodeId / mainNodeId / popupsNodeId / toastsNodeId`, sets `children: None` (slots live in props, not as positional children), and fabricates the shell id via `format!("app-shell-{}", Uuid::new_v4())` if not overridden.
  - `build_with_children(mut self) -> Vec<(String, Component)>` — iterates slot Options in canonical order, taking each present slot into both a freshly-built props map (preserving id→*NodeId mapping) and a `slot_roots` vec, drains descendants via `std::mem::take`, then constructs the shell component directly and emits `[shell, ...slot_roots, ...descendants]`. Missing slots are correctly omitted from both props and the flat list; the index-drift pitfall from the earlier draft is avoided by populating props and slot_roots in lock-step during the same drain pass instead of zipping drained slots against a fixed key array.
- Five inline tests:
  1. `app_shell_build_with_all_slots_populates_props` — all six slots set, asserts every `*NodeId` prop equals its slot id and `children` is None.
  2. `app_shell_build_without_slots_yields_empty_props` — no slots, asserts `props` is an empty JSON object (not null / not missing).
  3. `app_shell_build_with_children_flattens_all_nodes` — only sidebar, header, main set plus a descendant vec; asserts flat list length = 5, order is `[shell, sidebar, header, main, descendant]`, shell props contain exactly the three populated `*NodeId` keys (absent keys are explicitly asserted missing), and the main slot's component type is `surface-mount`.
  4. `app_shell_generates_uuid_id_when_not_set` — default id starts with `"app-shell-"` and is strictly longer than the prefix (i.e. the UUID was actually appended).
  5. `app_shell_with_sidenav_build_tree_pattern` — canonical handler usage: call `SideNav::build_tree()` to get `(root, descendants)`, pass them separately via `.sidebar(root).with_descendants(descendants)`, assert flat list is `[shell, sidenav_root, nav_item_heading]`.

## Verification

Plan-scoped verification (the authoritative acceptance criteria for this plan):

- `cargo test -p marionette builders::app_shell::tests` — **5/5 passed** (all AppShell tests).
- `cargo test -p marionette builders::standard::tests::surface_mount_builder` and `surface_mount_builder_custom_id` — **passed** (from Task 1).
- `cargo test -p marionette builders::standard::tests::all_19_standard_types` — **passed** (renamed test from Task 1).
- `cargo test -p marionette` — **33 unit + 6 messages-style + 3 macro + 5 ws integration = 47 tests passing, 0 failing, 1 ignored (pre-existing doctest)**.
- `cargo clippy -p marionette -- -D warnings` — **clean** after the targeted `#[allow(clippy::new_ret_no_self)]` on `AppShell::new()`.

Grep-based acceptance (from PLAN.md `<acceptance_criteria>`): all succeed — `pub struct AppShellBuilder`, all six slot methods (`sidebar`/`header`/`footer`/`main`/`popups`/`toasts`), `build_with_children`, and `sidebarNodeId` are all present in `app_shell.rs`; `struct SurfaceMount`, `component(type = "surface-mount")`, `pub name: String` are all present in `standard.rs`.

## Deviations from Plan

### Rule 3 (auto-fix blocking issue) — clippy::new_ret_no_self

- **Found during:** Task 2 — first run of `cargo clippy -p marionette -- -D warnings` after writing `app_shell.rs`.
- **Issue:** Clippy's `new_ret_no_self` (suspicious group, on by default, promoted to error under `-D warnings`) flags `impl AppShell { pub fn new() -> AppShellBuilder }` because methods named `new` "usually" return `Self`. The derived builders emit the same `Type::new() -> TypeBuilder` shape, but macro-generated spans are exempt from this lint so the 19 standard builders are silently accepted.
- **Fix:** Added a targeted `#[allow(clippy::new_ret_no_self)]` to `AppShell::new()` only, with a 3-line doc-comment rationale explaining the derive-builder convention. No broader `#![allow]` at crate or module level. This is the minimum change that keeps the API shape consistent with the other 19 component types — renaming the constructor to `builder()` would have broken the `Type::new().field(...)` convention everyone uses in handler code. No alternative restructuring (e.g., unifying `AppShell` and `AppShellBuilder` into a single struct) would preserve the plan's explicit `pub struct AppShellBuilder` acceptance criterion.
- **Files modified:** `backend/crates/marionette/src/builders/app_shell.rs` (the `allow` attribute is scoped to the single method).
- **Commit:** `45b3be4` (Task 2 commit).

### Draft iteration (no deviation, noted for transparency)

An initial draft of `build_with_children` attempted to drain slot options first and then call `build(self)` on the already-drained builder, which would have produced empty props. I recognised this in the same edit pass, rewrote the function to populate props and `slot_roots` in lock-step during a single drain (see the committed version), and avoided cloning `Component` values entirely. Only the committed version reached clippy/tests.

## Pre-existing Out-of-Scope Issues (Deferred)

`cargo clippy --workspace -- -D warnings` fails with 76 errors, **all inside `crates/crm-demo`** (entities/*, handlers/*, main.rs). These are pre-existing lints (too-many-lines, same-prefix fields, missing doc backticks, map-unwrap-or-else, auto-deref, collapsible-if, etc.) unrelated to Plan 12-05's changes — the `marionette` crate itself is clippy-clean. Per the SCOPE BOUNDARY rule, crm-demo clippy errors are not fixed here; they should be tracked by a future cleanup plan or the next plan that touches crm-demo source (Plan 12-07 `crm-integration` is a likely owner since it already modifies crm-demo handlers). Not added to `deferred-items.md` because the existing file already tracks Phase 12 deferred work and this issue is not newly introduced by this plan.

## Known Stubs

None. AppShell and SurfaceMount both have complete, wired implementations with exhaustive inline tests. `build()` and `build_with_children()` are fully implemented; no `todo!()`, no empty placeholder branches, no hardcoded values leaking to callers.

## Self-Check: PASSED

- [x] `backend/crates/marionette/src/builders/app_shell.rs` present (verified by Read + grep during execution).
- [x] `backend/crates/marionette/src/builders/standard.rs` contains `SurfaceMount` and `all_19_standard_types` (verified in git log of commit b7b3df8 and in Task 1's own test run, which is re-exercised as part of Task 2's `cargo test -p marionette` green run).
- [x] Commit `b7b3df8` (Task 1) exists: `git log --oneline | grep b7b3df8` → `b7b3df8 feat(12-05): add SurfaceMount derived builder` — confirmed.
- [x] Commit `45b3be4` (Task 2) exists: `git log --oneline | grep 45b3be4` → `45b3be4 feat(12-05): add hand-written AppShell builder` — confirmed (HEAD).
- [x] `cargo test -p marionette` green (33 unit + 14 integration/macro/ws = 47 tests pass).
- [x] `cargo clippy -p marionette -- -D warnings` green.
- [x] All six slot method names match D-B3 exactly (sidebar, header, footer, main, popups, toasts).
- [x] Slot ID prop keys match D-B3 exactly (sidebarNodeId, headerNodeId, footerNodeId, mainNodeId, popupsNodeId, toastsNodeId).
- [x] `build_with_children` returns a flat `Vec<(String, Component)>` (D-B4).
- [x] AppShell is exported from `builders/mod.rs` via the pre-existing `pub use app_shell::*;` line (unchanged in this plan — no mod.rs edit needed, confirmed by reading the file).
