---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Gallery Demo App + Auto-Discoverable Component Demos
status: executing
stopped_at: Plan 17-05 complete (5 of 7 Phase 17 gaps closed; 17-06/17-07/17-08 pending)
last_updated: "2026-04-22T19:00:00.000Z"
last_activity: 2026-04-22 -- Plan 17-05 complete (G-01/G-03/G-04/G-06/G-07 closed via Chrome MCP UAT)
progress:
  total_phases: 11
  completed_phases: 1
  total_plans: 15
  completed_plans: 9
  percent: 60
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-21)

**Core value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI
**Current focus:** Phase 17 — gallery-crate-skeleton-colocated-built-in-demos

## Current Position

Phase: 17 (gallery-crate-skeleton-colocated-built-in-demos) — EXECUTING
Plan: 5 of 7 complete (17-06 / 17-07 / 17-08 pending; 17-05 closed G-01/G-03/G-04/G-06/G-07)
Status: Executing Phase 17
Last activity: 2026-04-22 -- Plan 17-05 complete (Chrome MCP UAT passed 5 targeted gaps + 5 regression spot-checks)

Progress: v1.2 scoped into 5 phases (16–20). Phase 16 delivers the `#[gallery_demo]` proc macro, registry iteration API, and `gallery` cargo feature gate — the rails everything else rides on.

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

### Phase 17 hand-off (from Phase 16)

Phase 16 ships the `#[gallery_demo]` macro with a D-C1 convention: when no `key = "..."` arg is provided, the registry key defaults to the annotated fn's ident. Phase 17's DEMO-01 convention names every built-in's demo fn `gallery_demo()` (pure-fn sibling in each builder file), which would mass-collide at runtime since every default-derived key becomes `"gallery_demo"`. **Phase 17 planners must therefore use explicit `key = "..."` overrides on every `#[gallery_demo]` annotation in `backend/crates/marionette/src/builders/`.** The natural choice is to match each builder's `#[component(type = "…")]` string (e.g. `#[gallery_demo(key = "button")]` on Button, `#[gallery_demo(key = "text-input")]` on TextInput). Phase 18/19 catalog + exerciser screens use distinct fn idents and may skip the override.

Cross-reference: `.planning/phases/16-framework-hooks/16-CONTEXT.md` §D-C1 (lines 48-49), §specifics "Every Phase 17 annotation will use `key = \"…\"` explicitly" (line 180).

Also note: `gallery-smoke` landed in Phase 16 as a permanent workspace member (automated regression guard for the registry + macro + FRAME-03 symbol test). Phase 17's `gallery-demo` binary therefore becomes the 6th workspace crate, not the 5th as REQUIREMENTS.md §CRATE-01 currently states — adjust wording or accept the ordinal shift.

See also: `.planning/notes/2026-04-21-gallery-demo-architecture.md` (full design) and `.planning/seeds/gallery-live-token-editor.md` (scope-flexible theme-editor seed).

### Pending Todos

None carried over from v1.1.

### Blockers/Concerns

- **AppShell nestability unknown** — Phase 12's AppShell uses shadcn SidebarProvider context, `--sidebar-*` CSS tokens, mobile sheet behaviour, and a keyboard shortcut. These may or may not compose cleanly when an outer shell hosts an inner shell. Phase 19 (exerciser, EXER-01) is the place this will surface; may require non-trivial fixes or a deferred-item note.
- ✅ **Registration library selection (resolved 2026-04-21):** `linkme` chosen over `inventory` per Phase 16 CONTEXT.md D-A1 — type-safe `#[distributed_slice]`, zero runtime cost, explicit mental model. Logged in PROJECT.md Key Decisions. Implementation: `.planning/phases/16-framework-hooks/16-01-PLAN.md`; stable iteration order is owned by `marionette::gallery::registered_demos()` via sort-at-iteration-time, not delegated to linkme.
- **Enforcement policy** — whether "every new built-in must ship a `gallery_demo()`" becomes a CI lint (hard rule) or aspirational convention is a downstream decision (tracked in v1.3+ as GALLERY-LINT).
- **Phase 20 scope risk** — THEME-01 is explicitly scope-flexible per seed `gallery-live-token-editor`; if Phases 16–19 overrun, Phase 20 is the natural deferral target.
- Pre-existing concerns carried from v1.1: ~97 clippy pedantic warnings in crm-demo from toolchain drift (documented in Phase 17 deferred-items.md); ~68 pre-existing frontend ESLint baseline (stash-revert-confirmed 2026-04-22). Popup browser-test failures incidentally auto-fixed by 17-05 commit `7c2f29f` (ConfirmDialog browser tests rewritten around current markup: now 5/5 passing).
- **G-08 stranded Modal builder primitive** — Created by 17-05's architectural popup-global fix (`a55f055`). `marionette::builders::Modal` struct + `gallery_demo()` sibling are now dead code. Scheduled for cleanup in `17-08-PLAN.md` (wave 2, autonomous).
- **Toast global-overlay refactor deferred** — User noted "same for toasts I guess" during 17-05 architectural escalation. Not in 17-05 scope; inline-in-AppShell toasts still work. Candidate for Phase 19 EXER-01 or a v1.3+ popup-unification plan.

## Session Continuity

Last session: 2026-04-22T19:00:00.000Z
Stopped at: Plan 17-05 complete (5 of 7 Phase 17 gaps closed; 17-06/17-07/17-08 pending)
Resume: execute `17-06-PLAN.md` (wave 1 sibling — G-02 AppShell nested-sidebar fix + G-05 5 empty demo bodies), then `17-08-PLAN.md` (wave 2 autonomous — G-08 stranded Modal builder cleanup), then `17-07-PLAN.md` (full 20-demo Chrome MCP re-UAT + phase close)

**Planned Phase:** 17 (Gallery Crate Skeleton + Colocated Built-in Demos (gap closure)) — 7 plans — 5 complete, 3 pending (17-06, 17-07, 17-08)
