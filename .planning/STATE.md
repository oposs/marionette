---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Gallery Demo App + Auto-Discoverable Component Demos
status: Phase 17 shipped; ready to start Phase 18
stopped_at: Phase 18 context gathered
last_updated: "2026-04-23T12:09:40.628Z"
last_activity: 2026-04-22 — Phase 17 UAT gap closure complete (20/20 nav entries render; all 8 surfaced gaps closed)
progress:
  total_phases: 11
  completed_phases: 2
  total_plans: 12
  completed_plans: 12
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-21)

**Core value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI
**Current focus:** Phase 18 (Catalog Screens) — Phase 17 just shipped 2026-04-22

## Current Position

Phase: 17 COMPLETE; next: Phase 18 (Catalog Screens — CAT-01 through CAT-05)
Plan: 8 of 8 complete
Status: Phase 17 shipped; ready to start Phase 18
Last activity: 2026-04-22 — Phase 17 UAT gap closure complete (20/20 nav entries render; all 8 surfaced gaps closed)

Progress: [██████████] 100% (within Phase 17; v1.2 milestone has Phases 18/19/20 still ahead)

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

**v1.2 Phase 17 plan-level metrics (complete):**

| Plan | Duration | Tasks | Files | Outcome |
|------|----------|-------|-------|---------|
| 17-08 | 3h 15min | 6 | 5 | G-08 stranded Modal builder cleanup; struct deleted, demo preserved; GALLERY-DEMOS.md §Popup composition added |
| 17-07 | ~30min | 6 (Tasks 1-2 already done by orchestrator UAT walk; Tasks 3-6 docs-only finalization) | 5 (VERIFICATION.md + STATE.md + ROADMAP.md + REQUIREMENTS.md + 17-07-SUMMARY.md) | Phase 17 close-out: full 20-nav-entry Chrome MCP re-UAT confirmed; VERIFICATION.md status flipped to verified; SC #5 PASS; all 4 phase requirements (CRATE-01/02, DEMO-01/02) validated |

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
- [v1.2 Phase 17-07]: Phase 17 closed via Chrome MCP re-UAT walk. All 20 nav entries render (regression-pass for the 11 previously-passing demos + new pass for all 7 originally-failing demos + G-08 confirmed resolved). All 5 Phase 17 SCs pass; all 4 phase requirements (CRATE-01, CRATE-02, DEMO-01, DEMO-02) validated. SC #5 (the long-failing "every nav entry produces a screen, not an error surface") finally passes after gap-closure plans 17-05/06/08.

### Phase 17 close-out (2026-04-22)

**Phase 17 (Gallery Crate Skeleton + Colocated Built-in Demos) shipped** after 4 original plans (17-01 through 17-04) + 4 gap-closure plans (17-05 through 17-08). Chrome MCP UAT confirmed 20/20 demos pass SC #5, all 7 UAT-discovered original gaps closed (G-01..G-07), G-08 architectural debt resolved, 3/3 interactive flows (Modal, ConfirmDialog, Toast) work end-to-end.

**Key Phase 17 lessons carried into Phase 18/19:**

1. **Modal sub-surface contract** — The `modal` sub-surface tree's root MUST be a plain content component (Container, Heading, Text, ConfirmDialog, etc.), NEVER a `type: "modal"` component (that type used to map to ModalSurface.svelte in the frontend registry — rendering it into the modal sub-surface caused infinite recursion; now removed). Documented inline in `handlers/modal.rs` and the layout-root mount in `+layout.svelte`. Compositional pattern documented in `GALLERY-DEMOS.md` §Popup composition.

2. **AppShell nestability status** — Nesting an AppShell inside an outer AppShell's content sub-surface causes shadcn-svelte Sidebar.Provider context collision (inner Sidebar.Root renders at the same viewport position as outer, visually replacing it). Phase 17 ships a static structural preview in AppShell::gallery_demo; **Phase 19 EXER-01 owns the real nestability investigation** (see §Blockers/Concerns). No framework change made in Phase 17.

3. **ModalSurface close semantics** — With no ClearSurface protocol message, the close-sentinel is an empty Container rendered into the modal sub-surface. `ModalSurface.svelte`'s `isOpen` check discriminates empty-Container-at-root as closed. Frontend change was minimal (single `isEmptyContainer` helper fn). Documented inline.

4. **DataTable row-patch shape** — `fetch-rows` handlers MUST emit a SINGLE `Set` op with the full rows array (or per-row Sets keyed by id, matching CRM's pattern), NOT a single Set carrying an array when the frontend expects an object-map. CRM's `handle_fetch_rows` is the reference pattern; Phase 17's `seed_table_rows` now mirrors the object-map keyed by stringified id.

5. **Demo-bind / seed alignment** — Demos with `.bind("/demo/{key}/{leaf}")` MUST have a matching `seed_for_key` arm in `show.rs` that writes to THE SAME PATH. Static analysis alone cannot catch typos; Chrome MCP UAT was essential. Phase 18/19 demo authors must cross-check their bind paths against show.rs seeds before submitting.

6. **Compositional popups, not primitives** — The `Modal` builder struct was deleted in Plan 17-08. Popups are now compositional: emit any SDUI tree (Container with Form, TextInput, Button, …) into the `modal` sub-surface and ModalSurface.svelte (layout-root singleton) wraps in `<Dialog.Root>`/`<Dialog.Content>` automatically. ConfirmDialog remains as the canonical structured accept-cancel variant.

### Phase 18 hand-off

Phase 18 (Catalog Screens — CAT-01 through CAT-05) can start. Depends on Phase 17. See ROADMAP.md §Phase 18 for goals. No blockers carried forward from Phase 17.

**Deferred work explicitly NOT a Phase 17 blocker (carry into v1.3+ or Phase 19):**

- Toast global-overlay refactor (toasts render inline in AppShell today; user's "same for toasts I guess" architectural hint captured for a future popup-unification plan).
- Pre-existing crm-demo clippy::pedantic drift from toolchain drift (documented in `deferred-items.md`; out of scope for Phase 17).
- W-06 ErrorDisplay `message` field dead-state (Phase 18 CAT-04 polish).
- Pre-existing frontend ESLint baseline (~68 problems; v1.3+ cleanup).
- AppShell nestability blocker — confirmed real in Phase 17; **Phase 19 EXER-01 owns the resolution** (likely needs scoped Sidebar.Provider context in shadcn-svelte or a scoped-surface-name framework extension).

### Phase 17 hand-off (from Phase 16)

Phase 16 ships the `#[gallery_demo]` macro with a D-C1 convention: when no `key = "..."` arg is provided, the registry key defaults to the annotated fn's ident. Phase 17's DEMO-01 convention names every built-in's demo fn `gallery_demo()` (pure-fn sibling in each builder file), which would mass-collide at runtime since every default-derived key becomes `"gallery_demo"`. **Phase 17 planners must therefore use explicit `key = "..."` overrides on every `#[gallery_demo]` annotation in `backend/crates/marionette/src/builders/`.** The natural choice is to match each builder's `#[component(type = "…")]` string (e.g. `#[gallery_demo(key = "button")]` on Button, `#[gallery_demo(key = "text-input")]` on TextInput). Phase 18/19 catalog + exerciser screens use distinct fn idents and may skip the override.

Cross-reference: `.planning/phases/16-framework-hooks/16-CONTEXT.md` §D-C1 (lines 48-49), §specifics "Every Phase 17 annotation will use `key = \"…\"` explicitly" (line 180).

Also note: `gallery-smoke` landed in Phase 16 as a permanent workspace member (automated regression guard for the registry + macro + FRAME-03 symbol test). Phase 17's `gallery-demo` binary therefore becomes the 6th workspace crate, not the 5th as REQUIREMENTS.md §CRATE-01 currently states — adjust wording or accept the ordinal shift.

See also: `.planning/notes/2026-04-21-gallery-demo-architecture.md` (full design) and `.planning/seeds/gallery-live-token-editor.md` (scope-flexible theme-editor seed).

### Pending Todos

None carried over from v1.1.

### Blockers/Concerns

- **AppShell nestability blocker (confirmed 2026-04-22; ownership handed to Phase 19 EXER-01)** — Phase 17 Plan 17-06 Task 1 confirmed the blocker is real (shadcn `<Sidebar.Provider>` context collision: inner Sidebar.Root renders at the same viewport position as outer, visually replacing the outer 20-entry nav with the inner Dashboard/Reports/Settings nav). `AppShell::gallery_demo` ships a static structural preview workaround (Plain Container + 5 labeled slot-boxes built from Container + Heading + Text — no nested AppShell builder). **Phase 19 EXER-01 owns the resolution** — likely requires either (a) scoped Sidebar.Provider context in shadcn-svelte, or (b) a scoped-surface-name framework extension. Phase 17 closure does NOT depend on this blocker.
- ✅ **Registration library selection (resolved 2026-04-21):** `linkme` chosen over `inventory` per Phase 16 CONTEXT.md D-A1 — type-safe `#[distributed_slice]`, zero runtime cost, explicit mental model. Logged in PROJECT.md Key Decisions. Implementation: `.planning/phases/16-framework-hooks/16-01-PLAN.md`; stable iteration order is owned by `marionette::gallery::registered_demos()` via sort-at-iteration-time, not delegated to linkme.
- **Enforcement policy** — whether "every new built-in must ship a `gallery_demo()`" becomes a CI lint (hard rule) or aspirational convention is a downstream decision (tracked in v1.3+ as GALLERY-LINT).
- **Phase 20 scope risk** — THEME-01 is explicitly scope-flexible per seed `gallery-live-token-editor`; if Phases 16–19 overrun, Phase 20 is the natural deferral target.
- Pre-existing concerns carried from v1.1: ~97 clippy pedantic warnings in crm-demo from toolchain drift (documented in Phase 17 deferred-items.md); ~68 pre-existing frontend ESLint baseline (stash-revert-confirmed 2026-04-22). Popup browser-test failures incidentally auto-fixed by 17-05 commit `7c2f29f` (ConfirmDialog browser tests rewritten around current markup: now 5/5 passing).
- ✅ **G-08 stranded Modal builder primitive (resolved 2026-04-22 via Plan 17-08)** — `marionette::builders::Modal` struct deleted; modal `gallery_demo()` sibling preserved as the doc-stub host so the modal nav entry still renders; `pub use modal::*;` removed from mod.rs + standard.rs; smoke test renamed `all_19_standard_types` → `all_18_standard_types` with the `"modal"` row + expected entry both removed; GALLERY-DEMOS.md gained a `## Popup composition` section with the canonical form-in-popup recipe; `handle_modal_open` comment refreshed (no more stale `Modal::new` antipattern callout). See `17-08-SUMMARY.md`.
- **Toast global-overlay refactor deferred** — User noted "same for toasts I guess" during 17-05 architectural escalation. Not in 17-05 scope; inline-in-AppShell toasts still work. Candidate for Phase 19 EXER-01 or a v1.3+ popup-unification plan.
- **W-06 ErrorDisplay `message` field dead-state (new 2026-04-22 via Plan 17-06)** — The Rust `ErrorDisplay` builder has a `message` positional arg (`new(message)`) but the frontend `ErrorDisplay.svelte` reads errors ONLY from `bind`. Phase 18 CAT-04 polish should either remove the field or wire it as a bind-fallback when `getData(surface, bind)` is empty.

## Session Continuity

Last session: --stopped-at
Stopped at: Phase 18 context gathered
Resume: `/gsd-discuss-phase 18` or `/gsd-plan-phase 18` (Phase 18 — Catalog Screens — CAT-01 through CAT-05; depends on Phase 17 which is now complete)

**Planned Phase:** 18 (Catalog Screens) — TBD plans — pending plan creation
