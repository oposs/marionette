---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Gallery Demo App + Auto-Discoverable Component Demos
status: executing
stopped_at: Plan 17-08 complete (G-08 stranded Modal builder cleanup; modal nav entry preserved; GALLERY-DEMOS.md §Popup composition added; 7/8 Phase 17 plans complete; 17-07 full re-UAT remains pending)
last_updated: "2026-04-22T20:51:00.299Z"
last_activity: 2026-04-22
progress:
  total_phases: 11
  completed_phases: 1
  total_plans: 12
  completed_plans: 11
  percent: 92
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-21)

**Core value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI
**Current focus:** Phase 17 — gallery-crate-skeleton-colocated-built-in-demos

## Current Position

Phase: 17 (gallery-crate-skeleton-colocated-built-in-demos) — EXECUTING
Plan: 7 of 8 complete (only 17-07 pending; 17-08 just closed G-08 stranded Modal builder cleanup)
Status: Ready to execute
Last activity: 2026-04-22

Progress: [█████████░] 92%

## Performance Metrics

**Velocity (through v1.1):**

- Total plans completed: 70 (v1.0 32 + v1.1 38)
- v1.0 duration: ~3 months wall-clock
- v1.1 duration: ~10 days wall-clock (2026-04-08 to 2026-04-18)
- Phase 13 duration: ~3 hours wall-clock (discuss → research → plan → revision → execute → verify → UAT)

**By Phase (v1.0 + v1.1, archived):**

| Phase | Plans | Notes |
|-------|-------|-------|
| v1.0 (1–9) | 32 | MVP — protocol, frontend lib, backend toolkit, CRM core + features + listmonk |
| 10 | 3 | Foundation: shadcn init + Flowbite removal |
| 11 | 5 | Leaf component migration |
| 12 | 8 | Protocol node-patch + AppShell |
| 13 | 7 | DataTable (filter/virtualize/column viz) |
| 14 | 8 | FormScreen primitives |
| 15 | 7 | CRM migration + validation |

**v1.2 Phase 17 plan-level metrics (in-progress):**

| Plan | Duration | Tasks | Files | Outcome |
|------|----------|-------|-------|---------|
| 17-08 | 3h 15min | 6 | 5 | G-08 stranded Modal builder cleanup; struct deleted, demo preserved; GALLERY-DEMOS.md §Popup composition added |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.

Recent decisions affecting v1.2:

- [v1.2]: Gallery app is a second demo alongside the CRM (separate workspace crate `backend/crates/gallery-demo/`, thin backend, no auth, no DB) — dedicated surface for visual iteration and frontend capability exercise
- [v1.2]: Auto-discoverable demos via `#[gallery_demo]` proc macro + `inventory`/`linkme` distributed slice; gated behind `gallery` cargo feature on `marionette` crate so production consumers do not compile demo code
- [v1.2]: Demo contract is a pure `fn() -> Node`; composite demos are nested function calls; stateful fixtures live in the gallery binary, not the framework crate
- [v1.2]: Nested AppShell is an intentional capability exerciser, not a stunt — exposes whether shadcn Sidebar provider-context and mobile-sheet behaviour compose under nesting
- [v1.2]: Phase 17 bundles gallery-crate skeleton (CRATE-01/02) with colocated built-in demos (DEMO-01/02) because CRATE-02's auto-nav is only verifiable once DEMO-01's sweep has landed
- [v1.2 Phase 16]: `linkme` chosen over `inventory` as the gallery-demo registry backbone — type-safe `#[distributed_slice]`, zero runtime cost, explicit mental model. Stable iteration order owned by marionette (sort at iteration time), not delegated to linkme. Logged in PROJECT.md Key Decisions.
- [v1.2 Phase 17-05]: Popups are layout-root singletons. `ModalSurface.svelte` is mounted in `frontend/src/routes/+layout.svelte` as a sibling of the main Surface, independent of AppShell. Registry entry `'modal': ModalSurface` retired. User instruction (verbatim, 2026-04-22): "By default popups should work independent of any other component being displayed (AppShell included). If we ever need area-constrained popups, that would be a separate extension."
- [v1.2 Phase 17-05]: ConfirmDialog contract is structured (confirm_label / cancel_label / cancel_action / destructive), not child-based. `ConfirmDialog.svelte` renders its own shadcn Buttons; handlers emit a single structured node instead of orphan Accept/Reject children.
- [v1.2 Phase 17-05]: Modal sub-surface "closed" state is an empty Container (id="modal-empty"). `ModalSurface.isOpen` returns false when the tree root is a Container with no children; backend handlers use this sentinel to close modals.
- [v1.2 Phase 17-06]: AppShell demos use the structural-preview pattern, NOT nested AppShell invocation. When a frontend component relies on a viewport-anchored context provider (e.g. shadcn `<Sidebar.Provider>`), its `gallery_demo()` renders a static representation built from plain Container + Heading + Text — nesting a second AppShell inside the outer gallery causes Sidebar.Provider context collision (G-02, confirmed 2026-04-22). Phase 19 EXER-01 is the designated surface for true nested-shell composition.
- [v1.2 Phase 17-06]: Demo bind-path alignment is a hard contract. Every `/demo/<key>/<slot>` path a demo binds MUST have a matching `seed_for_key` arm writing the same path; a mismatch falls through as unseeded data (empty string / undefined / empty array) and the frontend's guards (`{#if errors.length > 0}`, `checked=false`, etc.) silently hide the component. Surfaced in G-05 via 3 of 5 demos (error-display, switch, textarea); radio-group + field-set seeds were already correctly aligned.
- [v1.2 Phase 17-08]: Stranded Modal builder primitive deleted (G-08 closure). The Modal struct is gone but the modal gallery_demo() sibling is preserved as a doc-stub host so the modal nav entry still renders. Popups are officially compositional — handler authors emit any SDUI tree into the modal sub-surface; ModalSurface.svelte (layout-root singleton, Plan 17-05) wraps in Dialog.Root automatically. ConfirmDialog remains as the structured accept-cancel variant. GALLERY-DEMOS.md gained a §Popup composition section with the canonical form-in-popup recipe.

### Phase 17 hand-off (from Phase 16)

Phase 16 ships the `#[gallery_demo]` macro with a D-C1 convention: when no `key = "..."` arg is provided, the registry key defaults to the annotated fn's ident. Phase 17's DEMO-01 convention names every built-in's demo fn `gallery_demo()` (pure-fn sibling in each builder file), which would mass-collide at runtime since every default-derived key becomes `"gallery_demo"`. **Phase 17 planners must therefore use explicit `key = "..."` overrides on every `#[gallery_demo]` annotation in `backend/crates/marionette/src/builders/`.** The natural choice is to match each builder's `#[component(type = "…")]` string (e.g. `#[gallery_demo(key = "button")]` on Button, `#[gallery_demo(key = "text-input")]` on TextInput). Phase 18/19 catalog + exerciser screens use distinct fn idents and may skip the override.

Cross-reference: `.planning/phases/16-framework-hooks/16-CONTEXT.md` §D-C1 (lines 48-49), §specifics "Every Phase 17 annotation will use `key = \"…\"` explicitly" (line 180).

Also note: `gallery-smoke` landed in Phase 16 as a permanent workspace member (automated regression guard for the registry + macro + FRAME-03 symbol test). Phase 17's `gallery-demo` binary therefore becomes the 6th workspace crate, not the 5th as REQUIREMENTS.md §CRATE-01 currently states — adjust wording or accept the ordinal shift.

See also: `.planning/notes/2026-04-21-gallery-demo-architecture.md` (full design) and `.planning/seeds/gallery-live-token-editor.md` (scope-flexible theme-editor seed).

### Pending Todos

None carried over from v1.1.

### Blockers/Concerns

- **AppShell nestability unknown (partial confirmation 2026-04-22)** — Phase 12's AppShell uses shadcn SidebarProvider context, `--sidebar-*` CSS tokens, mobile sheet behaviour, and a keyboard shortcut. Phase 17-06 G-02 confirmed the Sidebar.Provider context collides under nesting (the inner Sidebar.Root renders at the same viewport position as the outer, visually replacing the outer nav). Plan 17-06 worked around this by rewriting `AppShell::gallery_demo()` as a structural-preview (no nested AppShell builder). Full diagnosis + fix/defer decision for true nested shells remains Phase 19 EXER-01's scope.
- ✅ **Registration library selection (resolved 2026-04-21):** `linkme` chosen over `inventory` per Phase 16 CONTEXT.md D-A1 — type-safe `#[distributed_slice]`, zero runtime cost, explicit mental model. Logged in PROJECT.md Key Decisions. Implementation: `.planning/phases/16-framework-hooks/16-01-PLAN.md`; stable iteration order is owned by `marionette::gallery::registered_demos()` via sort-at-iteration-time, not delegated to linkme.
- **Enforcement policy** — whether "every new built-in must ship a `gallery_demo()`" becomes a CI lint (hard rule) or aspirational convention is a downstream decision (tracked in v1.3+ as GALLERY-LINT).
- **Phase 20 scope risk** — THEME-01 is explicitly scope-flexible per seed `gallery-live-token-editor`; if Phases 16–19 overrun, Phase 20 is the natural deferral target.
- Pre-existing concerns carried from v1.1: ~97 clippy pedantic warnings in crm-demo from toolchain drift (documented in Phase 17 deferred-items.md); ~68 pre-existing frontend ESLint baseline (stash-revert-confirmed 2026-04-22). Popup browser-test failures incidentally auto-fixed by 17-05 commit `7c2f29f` (ConfirmDialog browser tests rewritten around current markup: now 5/5 passing).
- ✅ **G-08 stranded Modal builder primitive (resolved 2026-04-22 via Plan 17-08)** — `marionette::builders::Modal` struct deleted; modal `gallery_demo()` sibling preserved as the doc-stub host so the modal nav entry still renders; `pub use modal::*;` removed from mod.rs + standard.rs; smoke test renamed `all_19_standard_types` → `all_18_standard_types` with the `"modal"` row + expected entry both removed; GALLERY-DEMOS.md gained a `## Popup composition` section with the canonical form-in-popup recipe; `handle_modal_open` comment refreshed (no more stale `Modal::new` antipattern callout). See `17-08-SUMMARY.md`.
- **Toast global-overlay refactor deferred** — User noted "same for toasts I guess" during 17-05 architectural escalation. Not in 17-05 scope; inline-in-AppShell toasts still work. Candidate for Phase 19 EXER-01 or a v1.3+ popup-unification plan.
- **W-06 ErrorDisplay `message` field dead-state (new 2026-04-22 via Plan 17-06)** — The Rust `ErrorDisplay` builder has a `message` positional arg (`new(message)`) but the frontend `ErrorDisplay.svelte` reads errors ONLY from `bind`. Phase 18 CAT-04 polish should either remove the field or wire it as a bind-fallback when `getData(surface, bind)` is empty.

## Session Continuity

Last session: 2026-04-22T20:51:00.291Z
Stopped at: Plan 17-08 complete (G-08 stranded Modal builder cleanup; modal nav entry preserved; GALLERY-DEMOS.md §Popup composition added; 7/8 Phase 17 plans complete; 17-07 full re-UAT remains pending)
Resume: execute `17-07-PLAN.md` (full 20-demo Chrome MCP re-UAT + phase close — the only remaining Phase 17 plan)

**Planned Phase:** 17 (Gallery Crate Skeleton + Colocated Built-in Demos (gap closure)) — 8 plans — 7 complete, 1 pending (17-07)
