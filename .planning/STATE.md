---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Gallery Demo App + Auto-Discoverable Component Demos
status: defining-requirements
stopped_at: Milestone v1.2 opened — defining requirements
last_updated: "2026-04-21T17:14:00.000Z"
last_activity: 2026-04-21 -- v1.1 closed, v1.2 opened via /gsd-new-milestone
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-21)

**Core value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI
**Current focus:** v1.2 — gallery-demo + auto-discoverable component demos (defining requirements)

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-04-21 — Milestone v1.2 started via /gsd-new-milestone

Progress: v1.2 just opened — phase decomposition pending roadmapper run.

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

See also: `.planning/notes/2026-04-21-gallery-demo-architecture.md` (full design) and `.planning/seeds/gallery-live-token-editor.md` (scope-flexible theme-editor seed).

### Pending Todos

None carried over from v1.1.

### Blockers/Concerns

- **AppShell nestability unknown** — Phase 12's AppShell uses shadcn SidebarProvider context, `--sidebar-*` CSS tokens, mobile sheet behaviour, and a keyboard shortcut. These may or may not compose cleanly when an outer shell hosts an inner shell. v1.2's exerciser phase is the place this will surface; may require non-trivial fixes.
- **Registration library selection** — `inventory` vs `linkme` decision deferred to Phase A scoping. `inventory` is widely used; `linkme` gives more explicit control. Pick during `/gsd-plan-phase`.
- **Enforcement policy** — whether "every new built-in must ship a `gallery_demo()`" becomes a CI lint (hard rule) or aspirational convention is a downstream decision.
- Pre-existing concerns carried from v1.1 (unchanged): 5 popup browser-test failures, some clippy pedantic warnings in crm-demo from toolchain drift.

## Session Continuity

Last session: 2026-04-21T17:14:00.000Z
Stopped at: Milestone v1.2 opened; `/gsd-new-milestone` still running — requirements + roadmap steps ahead
Resume: continue `/gsd-new-milestone` workflow at "Research Decision" step (step 8)
