---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 05
subsystem: gallery-sdui
tags: [gallery, gap-closure, sdui, modal, confirm-dialog, data-table, uat, chrome-mcp]

requires:
  - gallery-demo crate (Plan 17-03)
  - 19 built-in gallery_demo() siblings (Plan 17-04)
  - marionette::builders::ConfirmDialog + ModalSurface (Phase 11 CR-02)
  - ComponentBuilder snake_case derive contract

provides:
  - Working Modal demo (no tab lockup; true Dialog overlay)
  - Working DataTable demo (5 synthetic rows visible)
  - Working ConfirmDialog demo (Accept/Reject labels, close + toast)
  - Home-page UX polish (small muted footer text, no LoadingSkeleton grey bars)
  - Popups-global architectural pattern (ModalSurface mounted at layout root)
  - Extended ConfirmDialog contract (confirm_label / cancel_label / cancel_action / destructive)

affects:
  - Plan 17-06 (still runs — owns G-02 + G-05 demo-body fixes)
  - Plan 17-07 (full Phase 17 re-UAT after 17-06 + 17-08 land)
  - Plan 17-08 (stranded Modal builder cleanup — spawned from G-01's architectural fix)

tech-stack:
  added: []
  patterns:
    - "Empty-Container close-sentinel: modal sub-surface tree root = Container with no children ⇒ Dialog closed"
    - "Layout-root popup mounts: ModalSurface lives in +layout.svelte, independent of AppShell"
    - "Structured ConfirmDialog contract: confirm_label / cancel_label / cancel_action / destructive props instead of orphan children"
    - "snake_case-first props read (camelCase fallback) on frontend to match ComponentBuilder derive output"

key-files:
  created: []
  modified:
    - backend/crates/gallery-demo/src/handlers/navigate.rs
    - backend/crates/gallery-demo/src/handlers/modal.rs
    - backend/crates/gallery-demo/src/handlers/show.rs
    - backend/crates/gallery-demo/src/handlers/confirm.rs
    - backend/crates/marionette/src/builders/data_table.rs
    - backend/crates/marionette/src/builders/confirm_dialog.rs
    - frontend/src/lib/components/popup/ModalSurface.svelte
    - frontend/src/lib/components/popup/ConfirmDialog.svelte
    - frontend/src/lib/components/popup/ConfirmDialog.browser-test.ts
    - frontend/src/lib/registry/defaults.ts
    - frontend/src/routes/+layout.svelte
    - .planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/deferred-items.md

key-decisions:
  - "Popups live at the layout root, independent of any shell. ModalSurface is no longer an SDUI-dispatched registry entry; it is mounted as a sibling of the main Surface in +layout.svelte. Author instruction verbatim (2026-04-22): 'By default popups should work independent of any other component being displayed (AppShell included). If we ever need area-constrained popups, that would be a separate extension.'"
  - "Empty-Container close-sentinel replaces the 'tree !== undefined' heuristic. ModalSurface.isOpen now returns false when the modal sub-surface root is a Container with no children (the canonical `modal-empty` sentinel already emitted by handle_modal_close + handle_confirm_accept/reject + the navigate seed)."
  - "ConfirmDialog contract is structured, not child-based. The backend ConfirmDialog struct gained optional confirm_label / cancel_label / cancel_action / destructive fields; handle_confirm_open emits a single structured node and drops orphan Accept/Reject children. Matches the precedent set by DataTable's snake_case props."
  - "DataTable demo bind shape: the demo fn now calls .bind('/demo/data-table/rows') and seed_for_key('data-table') emits an object-map keyed by stringified id, aligning with DataTable.svelte's Object.entries iteration and CRM's per-row-Set pattern."
  - "Toast global-overlay refactor deferred. The user noted 'same for toasts I guess,' but unifying ToastSurface store-vs-tree is out of scope here. Regression spot-check confirms inline-in-AppShell toasts still work."

requirements-completed: [SC-17-05]

metrics:
  duration: "~3h (wall clock across multiple UAT + corrective passes)"
  tasks-completed: 6/6
  completed-date: 2026-04-22
---

# Phase 17 Plan 05: Gap closure — Modal / ConfirmDialog / DataTable / Home footer / sub-surface seeds

**Popup system relocated to a layout-root singleton, ConfirmDialog upgraded to a structured contract, DataTable demo rewired to the correct bind + object-map seed, footer downsized to muted Text, and modal sub-surface seeded — five UAT-surfaced Phase 17 gaps (G-01 / G-03 / G-04 / G-06 / G-07) closed and verified via Chrome MCP.**

## Performance

- **Duration:** ~3h wall clock (UAT pass 1 → pass 2 corrective → Chrome MCP re-walk)
- **Completed:** 2026-04-22
- **Tasks:** 6/6 (5 implementation + 1 human-verify checkpoint)
- **Files modified:** 11 (7 backend + 4 frontend) + 1 tracking file
- **Commits:** 11 (5 original implementation + 6 corrective/architectural/docs)

## Gap Closure Map

| Gap   | Task | Root cause (one line)                                                                                                                 | Fix (one line)                                                                                                                | Chrome MCP verification |
|-------|------|---------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|-------------------------|
| G-01  | 2 + arch | `handle_modal_open` rendered `type: "modal"` into the modal sub-surface → ModalSurface-inside-ModalSurface infinite recursion         | Emit a plain Container body; mount ModalSurface at layout root (not via SDUI registry)                                         | Dialog overlay opens with title/body/X/backdrop; tab does not hang; click X closes cleanly |
| G-03  | 3    | DataTable demo missing `.bind(...)` + `seed_for_key("data-table")` emitted an array when the frontend iterates an object-map          | Add `.bind("/demo/data-table/rows")` + rewrite seed to object-map keyed by stringified id                                     | 5 rows (Alice, Bob, Carol, Dan, Eva) + headers + Columns toggle visible |
| G-04  | 4 + corrective | `ModalSurface.isOpen = tree !== undefined` never flipped back to false; ConfirmDialog.svelte ignored orphan Accept/Reject children    | Treat empty Container as closed; extend ConfirmDialog struct with `confirm_label`/`cancel_label`/`cancel_action`; rewire handler | "Reject" (outline) + "Accept" (primary) labels render; each closes dialog + emits matching toast ("Confirm accepted"/"Confirm rejected") |
| G-06  | 1    | Footer used `Heading::new(...)` → `<h2 class="text-xl font-semibold">` overrode the footer wrapper's `text-xs text-muted-foreground`  | Replace both invocations with `Text::new(...)`                                                                                | Footer renders as small muted text: "Marionette Gallery · v1.2" + "connected" |
| G-07  | 1    | Modal sub-surface unseeded; `SurfaceMount("modal")` rendered a LoadingSkeleton grey bar                                               | Seed modal with empty Container (`id="modal-empty"`); mirrors CRM's toasts seed pattern                                        | No grey skeleton bars below footer on Home |

## Commits (11 total)

Ordered chronologically (original implementation → architectural corrections → G-04 corrective pass):

1. `154b7ff` — **fix(17-05): footer Text + modal seed in navigate handler (G-06, G-07)** — Task 1 original: Heading→Text on footer_version/footer_status; add 4th Render for `modal` sub-surface with empty Container seed.
2. `0ab3bea` — **fix(17-05): handle_modal_open emits Container body (G-01 pass 1)** — Task 2 original: replace `Modal::new(...)` with plain Container carrying the Heading+Text body; add `#[allow(clippy::too_many_lines)]` on handle_navigate (Rule 3 blocking).
3. `ad44991` — **fix(17-05): DataTable demo `.bind(...)` + object-map seed shape (G-03)** — Task 3: add `.bind("/demo/data-table/rows")` in the builder fn; rewrite `seed_table_rows()` to object-map keyed by stringified id; update coverage test.
4. `4994831` — **fix(17-05): ModalSurface.isOpen treats empty Container as closed (G-04 pass 1)** — Task 4: add `isEmptyContainer` helper; gate isOpen on tree-presence AND non-empty root; document CRM regression guard (single SurfaceMount match, no render target).
5. `a55f055` — **fix(17-05): mount ModalSurface globally in layout (G-01 architectural)** — Architectural correction: mount `<ModalSurface />` as sibling of `<Surface name="main" />` in `+layout.svelte`; drop `'modal': ModalSurface` from registry.
6. `3d6b19f` — **fix(17-05): remove modal mount from AppShell popups slot (G-01 architectural)** — Companion: drop `.popups(modal_mount)` from handle_navigate; keep the `modal-empty` seed for the layout-root ModalSurface's first-paint.
7. `61acb5c` — **docs(17-05): record pre-existing frontend lint baseline** — Document the 67+1 pre-existing lint problems in deferred-items.md (baseline-confirmed via stash-revert).
8. `7c2f29f` — **feat(17-05): ConfirmDialog supports cancelAction + label props** — Frontend: add `cancel_action` prop; read confirm_label/cancel_label/cancel_action in snake_case first (camelCase fallback); rewrite browser tests (5/5 pass).
9. `e03990e` — **feat(17-05): backend ConfirmDialog struct gains label + cancel_action fields** — Backend: extend struct with confirm_label/cancel_label/cancel_action/destructive; add serialization tests.
10. `1f2d24d` — **style(17-05): backtick snake_case/camelCase identifiers in ConfirmDialog doc** — Clippy doc_markdown fix for the module-level doc comment introduced in commit #9.
11. `8611971` — **fix(17-05): confirm-open uses structured ConfirmDialog, drops orphan children (G-04 root-cause fix)** — Handler alignment: handle_confirm_open emits a single structured ConfirmDialog with labels + distinct accept/reject actions; orphan Accept/Reject children removed.

Also landed alongside this plan (separate commit, 17-08 scaffolding): `93e1311` — **docs(17): add G-08 gap + 17-08-PLAN for stranded Modal builder cleanup** — NOT a 17-05 commit; documents the Modal-builder dead-code debt created by the G-01 architectural fix.

## Files Modified

### Backend

- `backend/crates/gallery-demo/src/handlers/navigate.rs` — Footer Text, modal sub-surface seed, drop `.popups(modal_mount)` after architectural fix, add `#[allow(clippy::too_many_lines)]`.
- `backend/crates/gallery-demo/src/handlers/modal.rs` — `handle_modal_open` emits a plain Container body (not `Modal::new(...)`).
- `backend/crates/gallery-demo/src/handlers/show.rs` — `seed_table_rows()` rewritten to object-map keyed by stringified id; coverage test updated.
- `backend/crates/gallery-demo/src/handlers/confirm.rs` — `handle_confirm_open` emits structured `ConfirmDialog::new(...).confirm_label(...).cancel_label(...).cancel_action(...).action(...)` instead of orphan children.
- `backend/crates/marionette/src/builders/data_table.rs` — `gallery_demo()` now calls `.bind("/demo/data-table/rows")`.
- `backend/crates/marionette/src/builders/confirm_dialog.rs` — Struct extended with `confirm_label`, `cancel_label`, `cancel_action`, `destructive`; module doc updated; new serialization tests.

### Frontend

- `frontend/src/routes/+layout.svelte` — Mount `<ModalSurface />` as sibling of `<Surface name="main" />` at the layout root.
- `frontend/src/lib/registry/defaults.ts` — Remove `'modal': ModalSurface` registry entry + unused import.
- `frontend/src/lib/components/popup/ModalSurface.svelte` — Add `isEmptyContainer()` helper; gate `isOpen` on tree-presence AND non-empty root; remove unused `rootProps` derivation.
- `frontend/src/lib/components/popup/ConfirmDialog.svelte` — Add `cancelAction` prop; read `confirm_label`/`cancel_label`/`cancel_action` snake-case first, camelCase fallback.
- `frontend/src/lib/components/popup/ConfirmDialog.browser-test.ts` — Rewrite tests around the actual markup (role="dialog" / `h2.text-lg`), add new cancel_action dispatch test; 5/5 pass.

### Tracking

- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/deferred-items.md` — Document 3 pre-existing baselines (crm-demo clippy::pedantic drift, frontend ESLint baseline, ConfirmDialog browser-test failures).

## Decisions Made

### D-A: Popups are layout-root singletons (architectural)

**Decision:** ModalSurface is no longer an SDUI-dispatched registry entry; it is mounted statically in `frontend/src/routes/+layout.svelte` as a sibling of the main Surface.

**User instruction (verbatim, 2026-04-22):** *"By default popups should work independent of any other component being displayed (AppShell included). If we ever need area-constrained popups, that would be a separate extension. Same for toasts I guess."*

**Rationale:** The original G-01 Task 2 fix (Container body instead of `Modal::new(...)`) stopped the recursion lockup but still left the modal body rendering inline below the footer — the AppShell's popups slot used plain NodeRenderer, never ModalSurface, and the registry-driven ModalSurface invocation was deliberately removed (it was the recursion trigger). Making ModalSurface a layout-root singleton decouples popup chrome from any shell structure, matching shadcn-svelte's own Dialog portal semantics.

**Consequence:** The `Modal` builder primitive (which emitted `type: "modal"` nodes) is now dead code — tracked as G-08 in `17-08-PLAN.md` (separate plan, wave 2, autonomous).

**Deferred:** The toast overlay equivalent refactor is out of scope for 17-05. The user's "same for toasts" note is logged; current inline-in-AppShell toasts still work per the regression spot-check.

### D-B: ConfirmDialog contract is structured, not child-based

**Decision:** Extend the backend `ConfirmDialog` struct with optional `confirm_label` / `cancel_label` / `cancel_action` / `destructive` fields; drop the pattern of emitting orphan Accept/Reject Button children.

**Rationale:** `ConfirmDialog.svelte` renders its own `<ShadcnButton>` instances in the footer and never consults a `children` snippet. The previous handler's `.children(vec![accept_btn, reject_btn])` was silently ignored — Accept fired `sendAction(undefined, ...)` (top-level action was also undefined) and Cancel fired the hardcoded `'close-modal'`, producing no Accept / Reject toasts. The structured contract aligns with DataTable's snake_case props precedent (`page_size`) and gives handler authors explicit typed fields.

**Rationale for snake_case-first read on the frontend:** ComponentBuilder derive emits snake_case keys. The previous camelCase reads only matched hand-written legacy call sites. New code paths use snake_case; camelCase fallback preserves any latent legacy callers until v1.3+ cleanup.

### D-C: DataTable bind shape matches CRM's per-row-Set pattern

**Decision:** Demo fn calls `.bind("/demo/data-table/rows")`; `seed_for_key("data-table")` emits an object-map keyed by stringified id (not an array).

**Rationale:** `DataTable.svelte:113-119` reads `getData(surface, bind)` and iterates via `Object.entries(rawData)`. The previous `seed_table_rows() -> serde_json::json!([...])` emitted an array which does not match that contract; the missing `.bind(...)` meant rawData was always `{}` regardless of seed shape. Matching CRM's proven pattern (`crm-demo/src/handlers/fetch_rows.rs:136-149` — one `PatchOperation::Set` per row keyed by id) keeps the two demos' runtime behavior identical and the fetch_rows handler is preserved unchanged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `clippy::too_many_lines` on `handle_navigate` after Task 1**

- **Found during:** Task 2 gate run.
- **Issue:** Adding the modal-seed block (Task 1) nudged `handle_navigate` over clippy's default `too_many_lines` threshold.
- **Fix:** Attach `#[allow(clippy::too_many_lines)]` to `handle_navigate`. The function is a top-level shell assembler that must stitch header + sidebar + footer + 4 Renders inline; CRM's `crm-demo/src/main.rs::handle_navigate` uses the same allow for the same reason.
- **Files modified:** `backend/crates/gallery-demo/src/handlers/navigate.rs`.
- **Committed in:** `0ab3bea` (Task 2 commit).

**2. [Rule 4 → user decision] Popup architectural refactor (G-01 escalation)**

- **Found during:** Task 6 Chrome MCP UAT pass 1, after Task 2's surgical Container-body fix.
- **Issue:** Task 2 stopped the recursion lockup but left the modal body rendering inline below the footer — no Dialog.Root, no backdrop, no close button. Root cause diagnosis deepened: AppShell rendered the popups slot via plain NodeRenderer; ModalSurface was never reached because its only invocation path (the `'modal': ModalSurface` registry entry) was deliberately no longer produced by the handler.
- **Rule applied:** Rule 4 (architectural change). Escalated to the user, who gave the verbatim "popups should work independent of AppShell" decision.
- **Fix:** Mount `<ModalSurface />` at `frontend/src/routes/+layout.svelte`; remove the registry entry; drop `.popups(modal_mount)` from `handle_navigate`; keep the `modal-empty` seed for first-paint.
- **Files modified:** `frontend/src/routes/+layout.svelte`, `frontend/src/lib/registry/defaults.ts`, `backend/crates/gallery-demo/src/handlers/navigate.rs`.
- **Committed in:** `a55f055` + `3d6b19f`.
- **Downstream:** Creates G-08 (stranded Modal builder primitive). Plan `17-08-PLAN.md` scheduled to clean it up, tracked in roadmap Phase 17.

**3. [Rule 1 - Bug] ConfirmDialog orphan-children rendering contract mismatch (G-04 root-cause pass)**

- **Found during:** Task 6 Chrome MCP UAT pass 2, after Task 4's `isEmptyContainer` close semantics fix.
- **Issue:** Dialog dismissed correctly, but clicking "Confirm" fired `sendAction(undefined)` (no-op, no toast) and clicking "Cancel" closed the dialog without firing `confirm-reject` (hardcoded `'close-modal'`, no toast). Root cause: `handle_confirm_open` emitted Accept/Reject Buttons as orphan children; `ConfirmDialog.svelte` ignores children entirely and renders its own buttons.
- **Fix (Option A, selected at user checkpoint):** Extend `ConfirmDialog` struct with `confirm_label` / `cancel_label` / `cancel_action` / `destructive`; teach `ConfirmDialog.svelte` to read them (snake-case first, camelCase fallback); rewrite `handle_confirm_open` to emit a single structured node. Preserves `ConfirmDialog.svelte` as the sole source of truth for dialog layout.
- **Files modified:** `backend/crates/marionette/src/builders/confirm_dialog.rs`, `backend/crates/gallery-demo/src/handlers/confirm.rs`, `frontend/src/lib/components/popup/ConfirmDialog.svelte`, `frontend/src/lib/components/popup/ConfirmDialog.browser-test.ts`.
- **Committed in:** `7c2f29f` + `e03990e` + `1f2d24d` + `8611971`.

**4. [Rule 1 - Clippy] `clippy::doc_markdown` on new ConfirmDialog module doc**

- **Found during:** Workspace clippy gate after commit `e03990e`.
- **Issue:** The new module-level doc comment referenced `snake_case`, `camelCase`, `ConfirmDialog.svelte`, `ComponentBuilder`, `DataTable.page_size` without backticks; `-D warnings` gate failed.
- **Fix:** Wrap the identifiers in backticks. No behavioral change.
- **Files modified:** `backend/crates/marionette/src/builders/confirm_dialog.rs`.
- **Committed in:** `1f2d24d`.

### Out-of-scope discoveries (logged to deferred-items.md, not fixed)

- **Pre-existing crm-demo clippy::pedantic failures (~97)** — toolchain drift, STATE.md-tracked. Per-plan clippy on touched crates passes.
- **Pre-existing frontend ESLint baseline (67 errors + 1 warning)** — Stash-revert baseline confirmed on 2026-04-22; none of the lint errors touch 17-05's modified files.
- **Pre-existing ConfirmDialog.browser-test.ts failures (4)** — Carried from v1.1 "5 popup browser-test failures" STATE blocker. **Auto-fixed incidentally** by commit `7c2f29f`'s test rewrite (the old tests queried `[data-slot="dialog-title"]` selectors that were removed in Phase 11 CR-02). All 5 ConfirmDialog browser tests now pass.

---

**Total deviations:** 4 auto-fixed (1 Rule 3 clippy block, 1 Rule 4 user-escalated architectural, 2 Rule 1 root-cause fixes) + 1 Rule 4 downstream dead-code (G-08) deferred to 17-08.
**Impact on plan:** All auto-fixes landed as clean root-cause edits with no migration shims (pre-deployment posture). The Rule 4 architectural fix expanded 17-05's scope to touch `+layout.svelte` and `defaults.ts`, documented and locked in via user instruction. No scope creep beyond the 5 original gaps + their newly-discovered root causes.

## Issues Encountered

- **Chrome MCP UAT pass 1 → pass 2 recursion:** Original Task 2 surgical fix (Container body in `handle_modal_open`) and original Task 4 surgical fix (`isEmptyContainer` helper) stopped the visible failure symptoms but left the actual user-facing behavior broken (no overlay, no toasts). UAT pass 2 caught both, triggering the architectural escalation (D-A) and the structured-contract refactor (D-B). Final Chrome MCP walk on 2026-04-22 confirmed all 5 targets passed + 5 regression spot-checks passed.
- **Pre-existing ConfirmDialog browser-test failures** auto-fixed as a side-effect of the test rewrite — previously 4/4 failing, now 5/5 passing (added a new cancel_action dispatch test).

## UAT Evidence

**Chrome MCP walk on 2026-04-22 against `make gallery-dev` on :3002:**

Targeted gap confirmations:
- **G-06** ✅ Home footer renders as small muted text (`text-xs text-muted-foreground`): "Marionette Gallery · v1.2" + "connected"
- **G-07** ✅ No grey skeleton bars on Home page below footer
- **G-01** ✅ Modal demo — "Open modal" produces a proper shadcn `<Dialog.Root>` overlay: fixed centered card, title "Example modal", body "Clicking X or the backdrop dismisses this dialog.", X close button, blurred backdrop. Tab does not hang. Click X closes cleanly.
- **G-03** ✅ DataTable — 5 rows render (Alice Baker, Bob Chen, Carol Davis, Dan Evans, Eva Frost) + headers (ID, Name, Email, Created) + Columns toggle button
- **G-04** ✅ ConfirmDialog — labels now show "Reject" (outline) + "Accept" (primary). Accept → dialog closes + "Confirm accepted" toast visible at bottom. Reject → dialog closes + "Confirm rejected" toast visible at bottom. Both flows clean.

Regression spot-checks (all pass):
- Home (20-tile grid) ✅
- Button (Primary + Disabled + Destructive) ✅
- Form (3 TextInputs + 2 Selects + Submit) ✅
- Toast fire (inline in AppShell toasts slot — "Demo toast from gallery-demo/toast-fire") ✅

## CRM Regression Guard (per plan §W-04)

```
$ grep -rn '"modal"' backend/crates/crm-demo/src/
backend/crates/crm-demo/src/main.rs:239:    let modal_mount = SurfaceMount::new("modal")
```

Exactly one match — the AppShell `SurfaceMount::new("modal")` mount declaration, NOT a Render destination. CRM has no handler that targets `surface: "modal"`, so neither the empty-Container close-sentinel semantics (`ModalSurface.svelte`) nor the layout-root ModalSurface mount can affect CRM runtime behavior.

## Threat Flags

None. The only new trust-boundary-adjacent surface is the `cancel_action` field on `ConfirmDialog`, which is an action-name string identical in treatment to existing action strings (the ActionRouter dispatches by string key, same path as every other action).

T-17.05-02 (DoS via handle_modal_open recursion) is mitigated by the architectural change — no SDUI path can now produce a `type: "modal"` node in the modal sub-surface. The `Modal` builder primitive is dead (G-08 cleanup in 17-08).

## Known Stubs

None in 17-05. The Modal demo body (heading + text + `Example modal`) is a deliberate minimum-surface example; Phase 18 CAT-04 (Feedback screen) will layer form-in-popup compositional examples.

## Deferred / Tracked Separately

- **G-08 stranded Modal builder primitive** — Created by 17-05's architectural fix (`a55f055`). `marionette::builders::Modal` struct + `gallery_demo()` sibling + `mod.rs` re-exports still exist but emit dead nodes (`type: "modal"` no longer has a registry handler). Tracked in `17-08-PLAN.md` (wave 2, autonomous). Documented as Phase 17 VERIFICATION.md §G-08.
- **Toast global-overlay refactor** — User's "same for toasts" note (2026-04-22). Out of scope for 17-05; current inline-in-AppShell toasts still work (regression spot-check confirms). Candidate for Phase 19 EXER-01 or a dedicated v1.3+ popup-unification plan.
- **Pre-existing crm-demo clippy::pedantic drift (~97 errors)** — STATE.md-tracked toolchain-drift baseline; requires a separate cleanup plan or toolchain-pin pass.
- **Pre-existing frontend ESLint baseline (67 errors + 1 warning)** — Confirmed pre-existing on 2026-04-22. None touch 17-05's modified files.

See `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/deferred-items.md` for baseline details.

## Next Plan Readiness

- **Plan 17-06 (Wave 1 sibling, gap-closure: G-02 + G-05)** — Independent of 17-05's changes. Touches `backend/crates/marionette/src/builders/{app_shell,error_display,field_set,radio_group,switch,textarea}.rs` demo bodies. Can execute now on the shared main working tree.
- **Plan 17-07 (full Phase 17 re-UAT)** — Waits for 17-06 + 17-08 to land.
- **Plan 17-08 (G-08 Modal builder cleanup)** — Wave 2, autonomous; waits for 17-06 if 17-08 references shared files (it should not — G-08 touches only `marionette/src/builders/modal.rs`, `builders/mod.rs`, `builders/standard.rs`, `GALLERY-DEMOS.md`, and a handler comment).

**Note to Plan 17-07 executor:** Plan 17-05 closed 5 of 7 gaps (G-01/G-03/G-04/G-06/G-07). Run the full 20-demo Chrome MCP re-UAT after Plan 17-06 closes G-02 + G-05 and Plan 17-08 cleans up G-08's stranded Modal primitive.

## Self-Check: PASSED

Files verified present:
- `backend/crates/gallery-demo/src/handlers/navigate.rs` — FOUND
- `backend/crates/gallery-demo/src/handlers/modal.rs` — FOUND
- `backend/crates/gallery-demo/src/handlers/show.rs` — FOUND
- `backend/crates/gallery-demo/src/handlers/confirm.rs` — FOUND
- `backend/crates/marionette/src/builders/data_table.rs` — FOUND
- `backend/crates/marionette/src/builders/confirm_dialog.rs` — FOUND
- `frontend/src/routes/+layout.svelte` — FOUND
- `frontend/src/lib/registry/defaults.ts` — FOUND
- `frontend/src/lib/components/popup/ModalSurface.svelte` — FOUND
- `frontend/src/lib/components/popup/ConfirmDialog.svelte` — FOUND
- `frontend/src/lib/components/popup/ConfirmDialog.browser-test.ts` — FOUND
- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/deferred-items.md` — FOUND

Commits verified present (via `git log --oneline`):
- `154b7ff` (Task 1 — footer Text + modal seed) — FOUND
- `0ab3bea` (Task 2 — handle_modal_open Container body) — FOUND
- `ad44991` (Task 3 — DataTable .bind + object-map seed) — FOUND
- `4994831` (Task 4 — ModalSurface.isEmptyContainer) — FOUND
- `a55f055` (architectural — ModalSurface layout-root mount) — FOUND
- `3d6b19f` (architectural — drop .popups from AppShell) — FOUND
- `61acb5c` (baseline — frontend lint deferred) — FOUND
- `7c2f29f` (G-04 corrective — ConfirmDialog cancelAction + labels) — FOUND
- `e03990e` (G-04 corrective — ConfirmDialog struct fields) — FOUND
- `1f2d24d` (clippy doc_markdown) — FOUND
- `8611971` (G-04 corrective — structured confirm-open handler) — FOUND
