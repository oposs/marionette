---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 08
subsystem: marionette-builders
tags: [gallery, gap-closure, modal, popup, builder-cleanup, dead-code]

requires:
  - Plan 17-05 (popups-global layout-root refactor — landed 2026-04-22; commits 154b7ff..e000d07)
  - Plan 17-04 (19-demo sweep + GALLERY-DEMOS.md author contract)
  - marionette::gallery registry + linkme (Phase 16)

provides:
  - Stranded `Modal` builder struct removed from `backend/crates/marionette/src/builders/modal.rs`
  - Cleaned re-export chain (mod.rs `pub use modal::*;` removed; standard.rs `modal::*` removed)
  - Component-type smoke test renamed `all_19_standard_types` → `all_18_standard_types` (the `"modal"` row + expected entry both gone)
  - GALLERY-DEMOS.md `## Popup composition` section with the canonical "form in popup" recipe (Container → Heading + Form(TextInput × 2) + Container(Button × 2))
  - Refreshed `handle_modal_open` comment pointing at GALLERY-DEMOS.md §Popup composition (no more stale `Modal::new` antipattern callout)
  - Modal `gallery_demo()` sibling preserved in the (now struct-less) modal.rs file so the `modal` nav entry continues to render its trigger Button + explainer Text body

affects:
  - Plan 17-07 (full 20-demo Phase 17 re-UAT — modal nav entry will be exercised; behavior should be identical to 17-05's Chrome MCP UAT result)

tech-stack:
  added: []
  patterns:
    - "Compositional popups: handler authors emit any SDUI tree (Container, Form, TextInput, Button, Heading, Text, …) into the `modal` sub-surface; ModalSurface.svelte (layout-root singleton, Plan 17-05) wraps in `<Dialog.Root>`/`<Dialog.Content>` automatically. No dedicated `Modal::new(...)` wrapper needed."
    - "Builder-file-as-demo-host: when a builder struct is removed but its `gallery_demo()` sibling must survive (because the demo body itself is still meaningful, just not the wrapper struct), the file remains as a doc-stub + `gallery_demo()` only — `pub mod modal;` stays in mod.rs but `pub use modal::*;` is dropped (nothing public to re-export)."

key-files:
  created: []
  modified:
    - backend/crates/marionette/src/builders/modal.rs (struct deleted; gallery_demo() preserved; doc-comment stub explains the cleanup)
    - backend/crates/marionette/src/builders/mod.rs (`pub use modal::*;` removed; smoke test renamed + Modal row + "modal" string removed)
    - backend/crates/marionette/src/builders/standard.rs (`modal::*,` removed from re-export shim)
    - backend/crates/marionette/GALLERY-DEMOS.md (added §Popup composition with canonical form-in-popup recipe)
    - backend/crates/gallery-demo/src/handlers/modal.rs (file header + handle_modal_open comment refreshed; no more "(not Modal::new)" stale antipattern callout)

key-decisions:
  - "Delete the Modal STRUCT but PRESERVE the modal `gallery_demo()` sibling. The plan's Task 1 wording 'Delete the Modal struct + gallery_demo sibling' conflicts with must_haves.truths #7 ('The gallery `modal` nav entry still renders correctly after the cleanup (its demo body never referenced Modal::new)') and Task 6 ('The Modal demo body must render as it did before'). Resolution: re-create modal.rs as a doc-stub + the unchanged `gallery_demo()` fn. The struct is gone (no more dead `type = \"modal\"` SDUI nodes) but the demo nav entry is preserved (its body uses Button + Text + Container; never touched `Modal::new(...)`). Documented as a Rule 3 (blocking) + Rule 2 (critical-functionality) auto-fix."
  - "GALLERY-DEMOS.md recipe uses the actual builder API, not the plan's draft. The plan's draft recipe called `TextInput::new()` with no args + `.label(...)` — but the real API is `TextInput::new(label)` positional (no `.label()` method exists). Fixed in the published recipe per Rule 1 (a wrong-API doc-recipe would mislead handler authors). Added a 'Note on builder APIs used above' callout to make the positional convention explicit."
  - "`pub mod modal;` retained in mod.rs but `pub use modal::*;` dropped. The retained `pub mod modal;` keeps the `gallery_demo()` sibling discoverable by `crate::builders::modal::gallery_demo` in gallery.rs's force-link table; the dropped `pub use` reflects that there are no public types in the module to re-export."

requirements-completed: [SC-17-08]

metrics:
  duration: "~3h 15min wall clock (plan reading + 5 commits + deviation auto-fix + gates + SUMMARY)"
  tasks-completed: "6/6 (Tasks 1-2 merged into one commit due to compile interlock; Task 3 was a no-op verification; Task 6 gates all green via cargo)"
  completed-date: 2026-04-22
---

# Phase 17 Plan 08: G-08 stranded Modal builder cleanup — Summary

**Stranded `Modal` builder struct (dead since Plan 17-05's popups-global refactor unregistered the `'modal': ModalSurface` SDUI dispatch) deleted, re-export chain + smoke-test smoke-test cleaned, GALLERY-DEMOS.md gained a canonical `## Popup composition` recipe, and the modal nav entry's demo body preserved as a doc-stub host — popups are now officially compositional, not primitive-based.**

## Performance

- **Duration:** ~3h 15min wall clock
- **Started:** 2026-04-22T17:31:25Z
- **Completed:** 2026-04-22T20:46Z
- **Tasks:** 6 of 6 (Tasks 1+2 in one commit; Task 3 no-op; Task 6 gates green)
- **Files modified:** 5 (4 backend Rust + 1 GALLERY-DEMOS.md)
- **Commits:** 5 (4 implementation + this finalization commit; deviation fix is included as a separate atomic commit)

## Accomplishments

- **`Modal` struct deleted** — `backend/crates/marionette/src/builders/modal.rs` no longer hosts `#[derive(ComponentBuilder)] pub struct Modal { title, size }`. After Plan 17-05's `'modal': ModalSurface` unregistration in `defaults.ts`, this struct produced dead SDUI nodes; it is now gone.
- **Re-export chain + smoke test cleaned** — `pub use modal::*;` removed from both `builders/mod.rs` and `builders/standard.rs`. The `all_19_standard_types` smoke test renamed to `all_18_standard_types`, with `Modal::new("x")` row + `"modal"` expected-entry both removed; test passes.
- **`## Popup composition` documentation** — GALLERY-DEMOS.md now has a top-level section explaining the layout-root ModalSurface → emit-any-tree pattern, the empty-Container close-sentinel, the canonical "form in popup" recipe (using the REAL builder APIs), and when to reach for `ConfirmDialog` instead.
- **Handler comment refreshed** — `gallery-demo/src/handlers/modal.rs` no longer references the (now-deleted) `Modal::new` antipattern; comments point at GALLERY-DEMOS.md §Popup composition for the general pattern.
- **Modal nav entry preserved** — The `modal` nav entry's demo body (trigger Button "Open modal" + explainer Text in a Container) continues to render. Its `gallery_demo()` sibling lives in the (now struct-less) modal.rs file as the doc-stub's only inhabitant.

## Task Commits

Each task was committed atomically (with one merge for compile interlock and one deviation fix):

1. **Tasks 1+2: Delete Modal struct + clean re-exports + smoke-test** — `8d71f4b` (fix) — Tasks 1 and 2 merged into a single commit because deleting modal.rs without simultaneously removing `pub mod modal;` would break the build (Rule 3 — atomic compile-correctness).
2. **Task 3: Update test-only callsites in sibling builders** — *no commit* — verification step. `grep -rn "Modal::" backend/crates/marionette/src/` returned nothing post-Tasks-1+2; no `Modal::` references in `gallery.rs`. The plan's Task 3 wording was a verification task with no required edits. The `SurfaceMount::new("modal")` callsites in `app_shell.rs` / `surface_mount.rs` remained untouched per the plan's `<safety>` block (they reference the modal sub-surface NAME — a routing target string, not the deleted Modal builder).
3. **Deviation fix: Restore modal `gallery_demo()` sibling** — `84e40cf` (fix) — Task 1's wholesale file deletion violated must_haves.truths #7 ("modal nav entry still renders") and broke `gallery.rs`'s `crate::builders::modal::gallery_demo` force-link with `error[E0433]: failed to resolve: could not find `modal` in `builders``. Recreated modal.rs as a doc-stub + the unchanged `gallery_demo()` fn; restored `pub mod modal;` in mod.rs (but kept `pub use modal::*;` dropped — nothing public to re-export). Rule 3 (blocking compile) + Rule 2 (missing critical functionality).
4. **Task 4: Document popup composition in GALLERY-DEMOS.md** — `c81cd23` (docs) — Added `## Popup composition` section with the canonical form-in-popup recipe (using the REAL builder APIs after Rule 1 correction of the plan's draft) and cross-references to handlers/modal.rs, confirm_dialog.rs, and ModalSurface.svelte.
5. **Task 5: Refresh handle_modal_open handler comment** — `09bc02e` (docs) — File header + inline comment in `gallery-demo/src/handlers/modal.rs` no longer reference the deleted `Modal::new` antipattern; both now point at GALLERY-DEMOS.md §Popup composition.

**Plan finalization:** _(this commit)_ — `docs(17-08): finalize SUMMARY + tracking` — Creates this 17-08-SUMMARY.md, updates STATE.md / ROADMAP.md / REQUIREMENTS.md to record SC-17-08 validated.

**Total commits in this plan:** 5 (4 task-aligned + 1 finalization). Plus 1 deviation auto-fix commit `84e40cf` interleaved between Tasks 1+2 and Task 4.

_Task 6 (gates) is a verification step; produced no commit. Cargo gates all passed cleanly — see §Issues Encountered._

## Files Created/Modified

### Backend (Rust)

- `backend/crates/marionette/src/builders/modal.rs` — Wholesale rewrite: struct + impl deleted; doc-comment stub at file head explains the architectural cleanup + cross-refs to GALLERY-DEMOS.md §Popup composition; `gallery_demo()` sibling preserved (unchanged body — Button "Open modal" trigger + explainer Text in a Container, dispatching `gallery-demo/modal-open`).
- `backend/crates/marionette/src/builders/mod.rs` — `pub use modal::*;` removed (line ~62 in the original); `pub mod modal;` retained (with new doc-comment explaining why nothing is re-exported); smoke test renamed `all_19_standard_types` → `all_18_standard_types` with `Modal::new("x").build().1.r#type` row removed and `"modal"` removed from the expected-types vec; both vecs now have 18 entries (matching). New comment in the test body cross-refs Plan 17-08 cleanup.
- `backend/crates/marionette/src/builders/standard.rs` — `modal::*,` removed from the re-export shim glob list; otherwise untouched.
- `backend/crates/gallery-demo/src/handlers/modal.rs` — File header rewritten to describe the post-17-05 / post-17-08 pattern cleanly + link to GALLERY-DEMOS.md §Popup composition; `handle_modal_open` inline comment dropped the stale `Modal::new` antipattern callout, kept the "ModalSurface supplies the Dialog chrome; body is inner-only" guidance.

### Documentation

- `backend/crates/marionette/GALLERY-DEMOS.md` — Appended `## Popup composition` top-level section (~117 lines added). Covers: layout-root ModalSurface pattern, empty-Container close-sentinel, canonical form-in-popup recipe (Container → Heading + Form(TextInput × 2) + Container(Button × 2)), "Note on builder APIs used above" callout (the REAL APIs: `TextInput::new(label)` positional, `Button::new(label)` positional, `Container::new()` arg-free, `Form::new()` arg-free), `ConfirmDialog`-vs-compose guidance, and cross-references to handlers/modal.rs, confirm_dialog.rs, and ModalSurface.svelte.

### Tracking (this commit)

- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-08-SUMMARY.md` (this file)
- `.planning/STATE.md` — Plan 17-08 complete; completed_plans counter bumped from 10 → 11; current plan position advanced; decision logged.
- `.planning/ROADMAP.md` — Phase 17 row Plans Complete → 7/8.
- `.planning/REQUIREMENTS.md` — SC-17-08 marked validated 2026-04-22 with link to this summary.

## Decisions Made

### D-A (Plan 17-08): Delete Modal STRUCT, preserve modal `gallery_demo()` sibling

**Decision:** `backend/crates/marionette/src/builders/modal.rs` is rewritten as a doc-stub + the unchanged `gallery_demo()` fn. The struct is gone; the demo entry survives.

**Rationale:** The plan's Task 1 wording ("Delete the Modal struct + gallery_demo sibling") conflicts with must_haves.truths #7 ("The gallery modal nav entry still renders correctly after the cleanup") and Task 6's verification ("The Modal demo body must render as it did before"). The conflict is real — deleting the file wholesale broke the gallery.rs force-link table (`error[E0433]: failed to resolve: could not find modal in builders`) AND would have removed the modal nav entry from the gallery's auto-discovered registry. Resolution: rebuild the file with ONLY the demo + a doc-comment stub explaining the cleanup. The struct's dead-code problem is fixed; the demo's "trigger Button + explainer Text in a Container" body never touched `Modal::new(...)` and survives unchanged.

**Pre-deployment posture (no back-compat):** No demo nav entry was lost; no struct survived. Both halves of the cleanup achieved.

**Consequence:** Future contributors can use modal.rs as the canonical pattern for "demo file whose builder struct was removed but demo body is still useful": doc-comment stub + `pub mod` in mod.rs + NO `pub use` re-export (nothing public to re-export). This pattern is now available for any future builder removal where the demo body should outlast the struct.

### D-B (Plan 17-08): GALLERY-DEMOS.md recipe uses REAL builder APIs, not the plan's draft

**Decision:** The `## Popup composition` recipe published in GALLERY-DEMOS.md uses `TextInput::new(label)` positional, `Button::new(label)` positional, etc. The plan's draft recipe used `TextInput::new()` no-arg + `.label(...)` — neither pattern exists in the codebase.

**Rationale:** Static cross-check against `backend/crates/marionette/src/builders/text_input.rs` (`pub label: String` as the required positional ctor arg; no `.label()` method derived by `ComponentBuilder`) and `button.rs` (same shape) confirmed the plan's draft would not compile if a handler author copy-pasted it. Rule 1 (auto-fix bug) — wrong-API documentation actively misleads.

**Consequence:** Added a "Note on builder APIs used above" callout to GALLERY-DEMOS.md to make the positional convention explicit. Future builder-method changes (rename, removal) should update this callout.

### D-C (Plan 17-08): `pub mod modal;` retained but `pub use modal::*;` dropped

**Decision:** `builders/mod.rs` keeps `pub mod modal;` (so `gallery.rs`'s test-only force-link `crate::builders::modal::gallery_demo` resolves) but drops `pub use modal::*;` (because the module now exports zero public types — only a feature-gated `gallery_demo()` fn the macro registers via linkme).

**Rationale:** Glob-re-exporting an empty namespace (or a namespace with only a `#[cfg(feature = "gallery")]` fn that's already registered via linkme) does nothing. The `pub mod` declaration is still required because Rust's module system needs a `mod` statement somewhere for the file to be part of the crate.

**Consequence:** This is the canonical shape for "demo-only module" going forward. If a future builder is removed but its demo survives, the same shape applies.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking + Rule 2 - Missing Critical] Restored `modal` gallery_demo sibling after Task 1's wholesale file deletion**
- **Found during:** Task 6 (cargo test --workspace --features gallery)
- **Issue:** Task 1 instructed deleting `backend/crates/marionette/src/builders/modal.rs` entirely (struct + gallery_demo sibling). After deletion, `cargo test --workspace --features gallery` failed with `error[E0433]: failed to resolve: could not find modal in builders` (gallery.rs's `all_in_scope_keys_present` test references `crate::builders::modal::gallery_demo` to force-link the linkme registration). Additionally, must_haves.truths #7 promises "The gallery modal nav entry still renders correctly after the cleanup" — wholesale deletion broke that promise.
- **Fix:** Re-created `backend/crates/marionette/src/builders/modal.rs` as a doc-stub + the unchanged `gallery_demo()` fn (no struct, no `pub use modal::*;` in mod.rs). Restored `pub mod modal;` in mod.rs to make the file part of the crate again. The `gallery_demo()` body (trigger Button + explainer Text in a Container) is identical to the pre-17-08 implementation and never referenced `Modal::new(...)`.
- **Files modified:** `backend/crates/marionette/src/builders/modal.rs` (recreated), `backend/crates/marionette/src/builders/mod.rs` (`pub mod modal;` restored)
- **Verification:** `cargo test --workspace --features gallery` passes (all `ok`); `cargo test -p marionette --features gallery --lib gallery::` passes (`all_in_scope_keys_present` and `skipped_keys_not_present` both green); `cargo clippy -p marionette --features gallery -- -D warnings` clean.
- **Committed in:** `84e40cf` (separate atomic commit between Tasks 1+2 and Task 4 — kept distinct for traceability)

**2. [Rule 1 - Bug Fix] Corrected GALLERY-DEMOS.md recipe to use real builder APIs**
- **Found during:** Task 4 (writing the popup composition section)
- **Issue:** Plan's draft recipe used `TextInput::new()` (no-arg) + `.label(...)` and `Button::new("Save").variant("default")`. Neither `TextInput::new()` (the constructor requires `label: String` positional per `text_input.rs:9-30`) nor a chainable `.label()` setter exists. Copy-pasting the plan's recipe would not compile.
- **Fix:** Adjusted the recipe to use the REAL APIs: `TextInput::new("Name")` positional, `Button::new("Cancel").variant("outline")`, `Container::new()` arg-free. Added a "Note on builder APIs used above" callout to GALLERY-DEMOS.md making the positional convention explicit.
- **Files modified:** `backend/crates/marionette/GALLERY-DEMOS.md`
- **Verification:** Cross-checked builder method names against `text_input.rs`, `button.rs`, `container.rs`, `form.rs`, `heading.rs`. The published recipe is syntactically valid Rust against the current crate APIs.
- **Committed in:** `c81cd23` (Task 4 commit — the fix landed in the same edit as the section addition)

**3. [Scope Boundary - Out-of-Scope Discovery] `.planning/config.json` has uncommitted modification**
- **Found during:** Pre-Task-1 git status check
- **Issue:** `.planning/config.json` had a single-line change (`"_auto_chain_active": true` → `"false"`). Not introduced by this plan; appears to be an environment flag toggled outside this plan's scope.
- **Fix:** Left untouched. Per scope boundary rule ("Only auto-fix issues DIRECTLY caused by the current task's changes"), out-of-scope. Logged here for awareness.
- **Files modified:** None by this plan (the config.json modification was pre-existing in the working tree).
- **Verification:** `git status --short` confirms `.planning/config.json` is still unstaged at the end of this plan; no plan commit touched it.
- **Committed in:** Not committed by this plan.

---

**Total deviations:** 2 auto-fixed (1 Rule 3 + Rule 2 — modal demo restoration; 1 Rule 1 — recipe API correction) + 1 out-of-scope discovery noted.

**Impact on plan:** Both auto-fixes were essential — the modal restoration unblocked the build and preserved a documented success criterion; the recipe correction prevents future handler authors from being misled by published-but-wrong code. Neither expanded plan scope; both refined plan execution to match real-world API surface and stated must_haves. Plan duration ~3h 15min vs. estimate "no estimate (autonomous, ~1-2h expected)" — the deviation discovery + investigation accounted for ~1h of that.

## Issues Encountered

**Build failure during Task 6 gate** — `cargo test --workspace --features gallery` failed with `error[E0433]: failed to resolve: could not find modal in builders` after Task 1's wholesale deletion of modal.rs. Root cause: `gallery.rs`'s `all_in_scope_keys_present` test force-links every builder module (`let _modal = crate::builders::modal::gallery_demo;`) to ensure linkme's distributed slice picks them up under `cargo test`. Without `pub mod modal;`, the path doesn't resolve.

**Resolution:** Re-introduced `pub mod modal;` (with the new doc-stub modal.rs hosting only `gallery_demo()`). The plan's Task 1 wording was incompatible with both the must_haves.truths #7 ("modal nav entry still renders") and the existing test infrastructure. See Deviations §Auto-fix #1.

**Live UAT note (`make gallery-dev` restart):** A pre-existing `gallery-demo` server was running on port 3002 from before this plan started (PID 1388701, started 19:19). The sandbox denied `kill` / `pkill` commands targeting it. Cargo build/test/clippy gates all passed cleanly, which is the authoritative correctness check; the modal demo's `gallery_demo()` body is byte-identical to its pre-17-08 form (no behavioral change), so a live UAT is functionally redundant. Plan 17-07's full Phase 17 Chrome MCP re-UAT will exercise the modal nav entry with the freshly-rebuilt server, providing the live confirmation.

## User Setup Required

None — no external service configuration. The cleanup is purely backend Rust + Markdown.

## Next Plan Readiness

- **Plan 17-07 (full Phase 17 re-UAT)** — Now READY. With Plans 17-05 (G-01/03/04/06/07), 17-06 (G-02/05), and 17-08 (G-08) all complete, all 8 surfaced Phase 17 gaps are fixed. 17-07's job is the full 20-demo Chrome MCP re-walk + `17-VERIFICATION.md` `status: verified` flip + ROADMAP/STATE phase-close updates. The modal nav entry will be exercised — its body is unchanged, so behavior should match 17-05's Chrome MCP UAT (Modal opens as true Dialog overlay, X-close + backdrop-click both dismiss).
- **Phase 17 NOT marked complete** — Per the orchestrator's instruction, this plan completion advances the phase to 7/8 plans, NOT 8/8. The phase-close gate is Plan 17-07's responsibility.

**Note for Plan 17-07 executor:** The `modal` nav entry in the gallery is now hosted by a doc-stub modal.rs (the Modal struct is gone). The demo body is byte-identical to before; a Chrome MCP click on "Modal" → "Open modal" should still produce the Dialog overlay. If the live binary on :3002 still shows the OLD modal demo body, restart with `make gallery-dev` to pick up Plan 17-08's tree (the running PID 1388701 was started before this plan).

## Threat Flags

None. No new trust-boundary-adjacent surface introduced. The cleanup REDUCES surface (one fewer SDUI component type, one fewer publicly-exported builder struct). No data path / authorization / network change.

T-17.08-01 (handler author confusion from stranded primitive — would lead to dead `type = "modal"` SDUI nodes silently failing to render) is NOW MITIGATED — the primitive is gone, the doc-stub modal.rs and GALLERY-DEMOS.md §Popup composition both explicitly direct authors to the compositional pattern.

## Known Stubs

The `backend/crates/marionette/src/builders/modal.rs` file is now a "demo-only stub" (doc-comment + `gallery_demo()` only). This is INTENTIONAL — see Decisions §D-A. It is NOT a placeholder awaiting future content; it is the final shape for this builder slot.

## Deferred / Tracked Separately

Unchanged from prior Phase 17 plans:
- **Toast global-overlay refactor** — Deferred from 17-05; not addressed here.
- **W-06 ErrorDisplay `message` field dead-state** — Phase 18 CAT-04 polish.
- **Pre-existing crm-demo clippy::pedantic drift** — `deferred-items.md`.
- **Pre-existing frontend ESLint baseline** — `deferred-items.md`.
- **Pre-existing ConfirmDialog browser-test failures** — `deferred-items.md`.

No new deferred items added by Plan 17-08.

---
*Phase: 17-gallery-crate-skeleton-colocated-built-in-demos*
*Completed: 2026-04-22*

## Self-Check: PASSED

Files verified present:
- `backend/crates/marionette/src/builders/modal.rs` — FOUND (doc-stub + gallery_demo)
- `backend/crates/marionette/src/builders/mod.rs` — FOUND (smoke test renamed; modal re-export removed)
- `backend/crates/marionette/src/builders/standard.rs` — FOUND (modal::* removed from re-export chain)
- `backend/crates/marionette/GALLERY-DEMOS.md` — FOUND (Popup composition section added at line 200)
- `backend/crates/gallery-demo/src/handlers/modal.rs` — FOUND (Modal::new antipattern callout removed; GALLERY-DEMOS.md cross-ref added)
- `.planning/phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-08-SUMMARY.md` — FOUND (this file)

Commits verified present (via `git log --oneline`):
- `8d71f4b` (Tasks 1+2 — Modal struct deletion + re-export cleanup) — FOUND
- `84e40cf` (Deviation auto-fix — modal gallery_demo restoration) — FOUND
- `c81cd23` (Task 4 — GALLERY-DEMOS.md popup composition section) — FOUND
- `09bc02e` (Task 5 — handle_modal_open comment refresh) — FOUND

Finalization commit `docs(17-08): finalize SUMMARY + tracking` will be authored together with SUMMARY + STATE.md + ROADMAP.md + REQUIREMENTS.md updates as the final atomic commit of this plan.
