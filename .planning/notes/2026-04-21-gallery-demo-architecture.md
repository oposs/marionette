---
date: "2026-04-21 17:14"
promoted: false
topic: gallery-demo-app
---

# Gallery Demo App — Architecture (exploration output, 2026-04-21)

Captured from `/gsd-explore` session. Serves as the design reference for downstream
`/gsd-new-milestone` / `/gsd-plan-phase` work on v1.2.

## Problem

The `crm-demo` app is not ideal for iterating on marionette's look-and-feel. It is
an opinionated business surface — auth, db, domain models, seeded contacts — which
gets in the way when the goal is "tweak a CSS token and see the ripple across every
widget." A second demo app, purpose-built as a **visual-iteration harness** and as
an **SDUI-frontend exerciser**, is warranted.

## Purpose (two jobs, held simultaneously)

1. **Visual-iteration harness** — fast loop for theme/token/component polish work.
2. **Frontend SDUI exerciser** — surfaces capability edges (nested shells, rapid
   node-patching, pathological scale) that a clean business app never hits.

The second job legitimises deliberately weird demos (nested AppShell, 10k-row
tables) alongside the clean catalog shots.

## Structural decisions

### New workspace crate — `backend/crates/gallery-demo/`
- 5th member alongside `marionette-protocol`, `marionette-macros`, `marionette`,
  `crm-demo`.
- Runs on its own port; the existing frontend is a generic SDUI consumer, so nothing
  changes on the frontend build side.

### Thin backend
- **No auth, no database, no migrations.** In-memory `Arc<RwLock<_>>` state only.
- Rationale: the stated goal is fast-boot zero-friction iteration. If a widget only
  looks right against "real" data, that's a widget problem, not a gallery problem.
- Stateful fixtures that exist (seeded DataTable rows, form drafts) live in handlers,
  not in the framework crate.

### Auto-discoverable component demos — option C (committed)

- New proc macro in `marionette-macros`: `#[gallery_demo]` attribute.
- Each built-in in `backend/crates/marionette/src/builders/<component>.rs` grows a
  sibling `pub fn gallery_demo() -> Node`, attributed with `#[gallery_demo]`.
- Registration backbone: `inventory` or `linkme` distributed slice — the gallery
  binary iterates the slice at startup. No central registry Vec to keep in sync.
- Gated behind a `gallery` cargo feature on the `marionette` crate (default OFF).
  Production consumers of `marionette` do not compile demo code. The
  `gallery-demo` binary enables the feature in its `Cargo.toml`.
- Rationale for picking C over B (manual one-line registry): we're in this for the
  long haul; the ongoing cost of forgetting to register a new component is worth
  paying a one-time macro/inventory investment to eliminate.

### Demo contract (enforced by convention, possibly by lint later)

- **Pure `fn() -> Node`.** No state, no I/O, no hidden fixtures.
- **Composite demos are nested function calls.** `FormScreen::gallery_demo()`
  internally calls `TextInput::gallery_demo()`, `SelectInput::gallery_demo()`,
  `Checkbox::gallery_demo()`, etc. No framework-level composition machinery needed.
- Live/stateful fixtures (DataTable with 120 seeded rows, form validation state)
  live in the gallery binary's handlers, NOT in the builder sibling.
- **Nested AppShell** follows naturally: `AppShell::gallery_demo()` embeds itself in
  its own content slot. Nestability stops being a stunt and becomes a property of
  the demo contract.

## Content shape (v1 candidate — subject to phase-planning refinement)

### Catalog screens (clean showcases — visual-iteration work)

- **Home / TOC** — grid of links, AppShell chrome on display
- **Buttons & actions** — every variant/size/state, icon buttons, destructive, loading
- **Forms** — every input type × every state, FieldSet grouping, validation patch demo
- **DataTable** — filter bar, virtualization, column visibility, large seeded dataset
- **Feedback** — toasts, confirm dialog, modal surface, empty/loading/error states
- **Typography & tokens** — text hierarchy, icon catalog, color swatches

### Exerciser screens (frontend robustness stress tests)

- **Nested AppShell** — proves (or exposes gaps in) shell composition
- **Rapid patching** — node patches firing at 500ms intervals, focus-preservation sanity
- **Pathological scale** — 10k-row table, 80-field form

### Theme tools (force multiplier)

- **Live `--token` editor** — pickers/sliders for `--primary`, `--radius`,
  `--sidebar-*`, etc. Backend sends a small panel; frontend applies via
  `document.documentElement.style.setProperty()`. Single biggest
  "improve look and feel" accelerator. Scope risk — may be deferred past v1 if phase
  budget gets tight (see seed: `gallery-live-token-editor`).

## Likely phase decomposition (v1.2 candidate milestone)

- **Phase A — Framework hooks.** `#[gallery_demo]` proc macro in
  `marionette-macros`, `inventory`/`linkme` plumbing, `gallery` cargo feature,
  registry iteration API in `marionette` crate. No component demos yet — just the
  rails.
- **Phase B — Colocate demos for existing built-ins + `gallery-demo` skeleton.**
  Add `gallery_demo()` siblings for ~20 built-in components. Scaffold the
  `gallery-demo` crate: main.rs, handlers, AppShell-based navigation listing every
  registered component.
- **Phase C — Catalog screens with composite demos.** Buttons, Forms, DataTable,
  Feedback, Typography.
- **Phase D — Exerciser screens.** Nested AppShell, Rapid patching, Pathological
  scale.
- **Phase E — Live token editor** (if still in v1 scope).

## Open questions carried forward (for `/gsd-discuss-phase` or `/gsd-new-milestone`)

- **Crate naming.** `gallery-demo` (mirrors `crm-demo`) vs `marionette-gallery`
  (implies framework-level shipping doc) vs `showcase-demo`. Preference leans
  `gallery-demo` for pattern consistency.
- **Registration library.** `inventory` (widely used, no proc-macro assistance)
  vs `linkme` (more explicit, mac-friendlier). Decide during Phase A.
- **Enforcement.** Is "every new component must have a `gallery_demo()`" a hard
  rule enforced by CI lint, or aspirational convention? Phase A seems too early;
  could be a Phase F or a rolling seed.
- **Nestability risk.** AppShell (Phase 12) uses shadcn's Sidebar with provider
  context, CSS tokens, mobile sheet, keyboard shortcut. Nesting may require non-
  trivial fixes. Exerciser phase (D) would be the place this surfaces.
