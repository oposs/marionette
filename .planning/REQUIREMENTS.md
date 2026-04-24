# Requirements: Marionette v1.2

**Defined:** 2026-04-21
**Core Value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI
**Milestone goal:** Ship a dedicated gallery app that serves as visual-iteration harness and SDUI-frontend exerciser, backed by a first-class auto-discoverable demo mechanism colocated with every marionette built-in.

Prior requirements (v1.0, v1.1) are archived in `.planning/milestones/v1.0-REQUIREMENTS.md` and `.planning/milestones/v1.1-REQUIREMENTS.md` respectively. Their validated items live in the `## Validated` section of `PROJECT.md`.

## v1.2 Requirements

### Framework hooks (the auto-discovery spine)

- [ ] **FRAME-01**: `#[gallery_demo]` attribute proc macro exists in `marionette-macros` and applies to a `pub fn name() -> Node` item. Applying it registers the function in a distributed slice at compile time with a stable key derived from the component type name. Misapplication (wrong signature, wrong visibility) produces a clear compiler error that names the violated rule.
- [ ] **FRAME-02**: `marionette` crate exposes a registry-iteration API (e.g. `pub fn registered_demos() -> impl Iterator<Item = DemoEntry>` returning component-type key + `fn() -> Node`) backed by `inventory` or `linkme`. The choice is recorded in a Key Decisions entry with rationale. Iteration order is stable.
- [ ] **FRAME-03**: `gallery` cargo feature on the `marionette` crate gates all demo-related code. Default build (`cargo build -p marionette`) compiles zero demo symbols and zero registry entries — verified by a test that inspects the built artifact size or binary contents. The `gallery-demo` binary enables the feature explicitly.

### Gallery crate skeleton

- [x] **CRATE-01**: New `backend/crates/gallery-demo/` workspace member exists as the 6th Cargo workspace entry (the 5th slot is occupied by `gallery-smoke`, a permanent test-fixture crate landed in Phase 16), with thin backend scaffolding: no auth, no database, no migrations — in-memory `Arc<RwLock<_>>` state only. `cargo run -p gallery-demo` starts the app on its own port against the shared frontend. Validated 2026-04-22 — see [17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md).
- [x] **CRATE-02**: Gallery `main.rs` builds its AppShell navigation by iterating the auto-discovered demo registry — no hand-maintained menu list. Adding a new `#[gallery_demo]` fn anywhere in the workspace causes the gallery to automatically surface it on next build. Validated 2026-04-22 — see [17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md).

### Built-in demos (colocated, pure-fn)

- [x] **DEMO-01**: Every existing built-in component in `backend/crates/marionette/src/builders/` ships a sibling `pub fn gallery_demo() -> Node` annotated with `#[gallery_demo]`, covering all currently-registered SDUI components (Button, TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch, DataTable, AppShell, NavItem, Sidebar pieces, ModalSurface, ConfirmDialog, ToastSurface, FieldSet, FieldSeparator, Container, Heading, plus any others in the current registry). Validated 2026-04-22 — see [17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md).
- [x] **DEMO-02**: Demo contract is enforced by convention (and documented in a short `GALLERY-DEMOS.md` under `backend/crates/marionette/` or equivalent): pure `fn() -> Node`, no external state, no I/O, no fixtures. Composite demos are built by calling other `gallery_demo()` functions directly — `FormScreen::gallery_demo()` invokes `TextInput::gallery_demo()`, `SelectInput::gallery_demo()`, etc. Validated 2026-04-22 — see [17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md).

### Phase 17 UAT gap-closure success criteria (post-hoc)

Defined 2026-04-22 during Phase 17 UAT after `DEMO-01`/`DEMO-02`/`CRATE-01`/`CRATE-02` ran — these gate Phase 17 closure alongside the original four.

- [x] **SC-17-05**: Gap closure for G-01 (Modal lockup) / G-03 (DataTable empty) / G-04 (ConfirmDialog dismiss) / G-06 (Home footer oversized) / G-07 (Modal sub-surface unseeded). Chrome MCP UAT on 2026-04-22 confirms: Modal opens as true Dialog overlay (no tab hang, clean X-close); DataTable renders 5 synthetic rows; ConfirmDialog shows Accept/Reject labels and both flows close dialog + emit matching toast; Home footer renders as `text-xs text-muted-foreground`; no grey LoadingSkeleton bars below footer. Validated 2026-04-22 — see [17-05-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-05-SUMMARY.md).
- [x] **SC-17-06**: Gap closure for G-02 (AppShell nested-sidebar hijack) / G-05 (5 empty demo bodies — error-display, field-set, radio-group, switch, textarea). Chrome MCP UAT on 2026-04-22 confirms: App Shell demo renders 5 labeled slot boxes without replacing the outer gallery sidebar; Error Display renders 3 error boxes from seeded bind paths; Switch renders Wifi (CHECKED) + Bluetooth (unchecked) with `checked-1`/`checked-2` seed alignment; Textarea renders Notes + With description with empty `value`/`value-desc` seed; Radio Group renders 3 options under "Pick one" label (no code change — already-correct static analysis confirmed); Field Set renders "Contact Info" legend + 3 TextInputs + 2 Selects (no code change — already-correct static analysis confirmed). Validated 2026-04-22 — see [17-06-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-06-SUMMARY.md).
- [x] **SC-17-07**: Full 20-demo Chrome MCP re-UAT passes, `17-VERIFICATION.md` flips `status: verified`, ROADMAP/STATE reflect Phase 17 complete. Chrome MCP orchestrator-driven walk on 2026-04-22 against fresh `gallery-demo` server (post-17-08 build) on `:3002`: 20/20 nav entries render correctly; all 7 original gaps (G-01..G-07) closed; G-08 architectural debt resolved via Plan 17-08; 3/3 interactive flows (Modal, ConfirmDialog, Toast) work end-to-end; VERIFICATION.md status flipped to `verified`; ROADMAP marks Phase 17 8/8 Complete; STATE.md rolled forward to Phase 18 hand-off. Validated 2026-04-22 — see [17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md).
- [x] **SC-17-08**: G-08 stranded `Modal` builder primitive removed. `marionette::builders::Modal` struct deleted; modal `gallery_demo()` sibling preserved as a doc-stub host so the modal nav entry still renders; re-exports cleaned up (mod.rs `pub use modal::*;` removed; standard.rs glob-list updated); component-type smoke test renamed `all_19_standard_types` → `all_18_standard_types`; `GALLERY-DEMOS.md` gained a `## Popup composition` section with the canonical form-in-popup recipe; `handle_modal_open` comment refreshed (no more stale `Modal::new` antipattern callout). Cargo build/test/clippy gates all green. Validated 2026-04-22 — see [17-08-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-08-SUMMARY.md).

### Catalog screens (clean showcases)

- [x] **CAT-01**: Gallery includes a Buttons & Actions screen showing every Button variant × size × state (default / destructive / outline / ghost / link × sm/md/lg × normal/disabled/loading/icon-only). Validated 2026-04-23 — see [18-04-SUMMARY.md](phases/18-catalog-screens/18-04-SUMMARY.md).
- [x] **CAT-02**: Gallery includes a Forms screen rendering every input type (text / select / checkbox / switch / radio / textarea) across every visual state (normal / disabled / error / focused / with-description), grouped with `FieldSet` and `FieldSeparator`, including a live validation patch-demo that exercises error-clearing on correction. Validated 2026-04-23 — see [18-05-SUMMARY.md](phases/18-catalog-screens/18-05-SUMMARY.md). *Naming note:* shipped implementation uses shadcn Cards + `FieldSeparator` instead of `FieldSet`; user-observable grouping goal met — naming choice, not feature gap.
- [x] **CAT-03**: Gallery includes a DataTable screen with filter bar, virtualized infinite scroll, column visibility toggle, and per-`ColumnKind` rendering, seeded with enough synthetic rows to exercise virtualization (≥500 rows). Validated 2026-04-23 — see [18-06-SUMMARY.md](phases/18-catalog-screens/18-06-SUMMARY.md).
- [x] **CAT-04**: Gallery includes a Feedback screen showing toast dispatch, confirm dialog flow, modal surface, and empty / loading / error placeholder states side-by-side. Validated 2026-04-23 — see [18-07-SUMMARY.md](phases/18-catalog-screens/18-07-SUMMARY.md).
- [x] **CAT-05**: Gallery includes a Typography & tokens screen rendering the text scale, the lucide-svelte icon catalog (searchable or grid), and OKLCH swatches for every semantic token in `app.css`. Validated 2026-04-23 — see [18-08-SUMMARY.md](phases/18-catalog-screens/18-08-SUMMARY.md).

### Exerciser screens (frontend robustness)

- [x] **EXER-01**: Gallery includes a Nested AppShell screen where an outer AppShell hosts an inner AppShell in its content slot. Demonstrates whether shadcn `SidebarProvider` context, mobile-sheet behaviour, keyboard shortcut handling, and `--sidebar-*` CSS tokens compose under nesting — and captures any gaps as deferred items. Validated 2026-04-24 — see [19-02-SUMMARY.md](phases/19-exerciser-screens/19-02-SUMMARY.md) and [19-VERIFICATION.md](phases/19-exerciser-screens/19-VERIFICATION.md).
- [x] **EXER-02**: Gallery includes a Rapid Patching screen that fires node patches at a configurable interval (default ~500ms) while a text input retains focus. Verifies PATCH-02's focus-preservation invariant under sustained mutation pressure. Validated 2026-04-24 — see [19-03-SUMMARY.md](phases/19-exerciser-screens/19-03-SUMMARY.md) and [19-VERIFICATION.md](phases/19-exerciser-screens/19-VERIFICATION.md).
- [x] **EXER-03**: Gallery includes a Pathological Scale screen combining a DataTable with ≥10 000 synthetic rows and a FormScreen with ≥80 synthetic fields on a single page — captures performance baselines and surfaces scaling issues in the frontend surface store, virtualizer, and SurfaceMount patch application. Validated 2026-04-24 — see [19-04-SUMMARY.md](phases/19-exerciser-screens/19-04-SUMMARY.md) and [19-VERIFICATION.md](phases/19-exerciser-screens/19-VERIFICATION.md).

### Theme tools (force multiplier, scope-flexible)

- [ ] **THEME-01**: Gallery includes a Live Token Editor screen offering controls (color picker / slider / numeric input) for core shadcn theme variables (`--primary`, `--background`, `--foreground`, `--radius`, `--sidebar-*`, etc.). Changes apply via `document.documentElement.style.setProperty()` and re-render all gallery screens in place. An export affordance emits the current token set as a pasteable `@theme`/`:root` block. Scope note: may be scoped down to a subset of tokens or deferred to a follow-up milestone if the owning phase runs long (see seed `gallery-live-token-editor`).

## v1.3+ Requirements

Deferred to a future milestone — tracked for context, not in v1.2 scope.

- **GALLERY-LINT**: CI lint enforcing "every built-in component must have a `#[gallery_demo]`" as a hard rule rather than convention (may emerge from DEMO-01's manual sweep experience)
- **GALLERY-DEMOS-EXPORT**: Publish `marionette-gallery-demo` as a documentation artifact library users can run out of the box (`cargo run -p gallery-demo` already works locally; this is about surfacing it as a shipped story)
- **THEME-EXPORT-WRITE**: Theme editor writes exported tokens directly back to `frontend/src/app.css` on user confirmation, not just clipboard paste

## Out of Scope

| Feature | Reason |
|---------|--------|
| Auth / login in the gallery | The gallery is a design harness; auth would slow boot and reintroduce CRM-style friction |
| Database-backed persistence in the gallery | Stateful fixtures belong in handlers, not durable storage; restart-is-reset is a feature |
| Framework-level composition machinery for composite demos | Composite demos are plain nested fn calls — adding a "demo-combinator" abstraction is premature |
| Auto-generated documentation from demos (e.g. screenshots exported) | Useful, but a v1.3+ concern; v1.2 is about the harness, not the publishing pipeline |
| Demos for third-party components outside the `marionette` crate | Scope-limit: v1.2 ships demos for built-ins only |
| Backend-driven theme tokens (server pushes theme to client) | Theme lives client-side; server-driven theming is a different product direction |

## Traceability

Populated by `/gsd-plan-phase` runs as phases are defined.

| Requirement | Phase | Plan | Status |
|-------------|-------|------|--------|
| FRAME-01 | Phase 16 | 16-02 | Pending |
| FRAME-02 | Phase 16 | 16-01 | Pending |
| FRAME-03 | Phase 16 | 16-01, 16-03 | Pending |
| CRATE-01 | Phase 17 | 17-03, 17-07 | ✅ Validated 2026-04-22 ([17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md)) |
| CRATE-02 | Phase 17 | 17-03, 17-07 | ✅ Validated 2026-04-22 ([17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md)) |
| DEMO-01  | Phase 17 | 17-04, 17-07 | ✅ Validated 2026-04-22 ([17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md)) |
| DEMO-02  | Phase 17 | 17-04, 17-07 | ✅ Validated 2026-04-22 ([17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md)) |
| SC-17-05 | Phase 17 | 17-05 | ✅ Validated 2026-04-22 ([17-05-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-05-SUMMARY.md)) |
| SC-17-06 | Phase 17 | 17-06 | ✅ Validated 2026-04-22 ([17-06-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-06-SUMMARY.md)) |
| SC-17-07 | Phase 17 | 17-07 | ✅ Validated 2026-04-22 ([17-07-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-07-SUMMARY.md)) |
| SC-17-08 | Phase 17 | 17-08 | ✅ Validated 2026-04-22 ([17-08-SUMMARY.md](phases/17-gallery-crate-skeleton-colocated-built-in-demos/17-08-SUMMARY.md)) |
| CAT-01   | Phase 18 | 18-01, 18-04 | ✅ Validated 2026-04-23 ([18-04-SUMMARY.md](phases/18-catalog-screens/18-04-SUMMARY.md)) |
| CAT-02   | Phase 18 | 18-02, 18-05 | ✅ Validated 2026-04-23 ([18-05-SUMMARY.md](phases/18-catalog-screens/18-05-SUMMARY.md)) |
| CAT-03   | Phase 18 | 18-03, 18-06 | ✅ Validated 2026-04-23 ([18-06-SUMMARY.md](phases/18-catalog-screens/18-06-SUMMARY.md)) |
| CAT-04   | Phase 18 | 18-07 | ✅ Validated 2026-04-23 ([18-07-SUMMARY.md](phases/18-catalog-screens/18-07-SUMMARY.md)) |
| CAT-05   | Phase 18 | 18-08 | ✅ Validated 2026-04-23 ([18-08-SUMMARY.md](phases/18-catalog-screens/18-08-SUMMARY.md)) |
| EXER-01  | Phase 19 | 19-02 | ✅ Validated 2026-04-24 ([19-02-SUMMARY.md](phases/19-exerciser-screens/19-02-SUMMARY.md)) |
| EXER-02  | Phase 19 | 19-03 | ✅ Validated 2026-04-24 ([19-03-SUMMARY.md](phases/19-exerciser-screens/19-03-SUMMARY.md)) |
| EXER-03  | Phase 19 | 19-04 | ✅ Validated 2026-04-24 ([19-04-SUMMARY.md](phases/19-exerciser-screens/19-04-SUMMARY.md)) |
| THEME-01 | Phase 20 | — | Pending |

**Coverage:**
- v1.2 requirements: 20 total (16 original + 4 Phase 17 UAT gap-closure SCs added 2026-04-22)
- Mapped to phases: 20 (Phases 16–20)
- Validated: 16 (all of Phase 17 — CRATE-01, CRATE-02, DEMO-01, DEMO-02, SC-17-05, SC-17-06, SC-17-07, SC-17-08 + all of Phase 18 — CAT-01..CAT-05 + all of Phase 19 — EXER-01, EXER-02, EXER-03)
- Unmapped: 0

---
*Requirements defined: 2026-04-21 via /gsd-new-milestone*
*Traceability populated: 2026-04-21 via /gsd-roadmap (v1.2 Phases 16–20)*
*Updated: 2026-04-22 — Phase 17 UAT gap-closure SCs added (SC-17-05/06/07/08); SC-17-05 validated via Chrome MCP UAT*
*Updated: 2026-04-22 — SC-17-06 validated via Chrome MCP UAT (G-02 + G-05 closed; all 7 original Phase 17 gaps now fixed)*
*Updated: 2026-04-22 — SC-17-08 validated via cargo build/test/clippy gates (G-08 stranded Modal builder cleanup; struct deleted, modal nav entry preserved via doc-stub host, GALLERY-DEMOS.md §Popup composition section added)*
*Updated: 2026-04-22 — **Phase 17 COMPLETE**: SC-17-07 validated via full Chrome MCP re-UAT walk; 4 phase requirement IDs (CRATE-01, CRATE-02, DEMO-01, DEMO-02) marked validated; coverage bumps validated count from 3 to 8 (all of Phase 17)*
*Updated: 2026-04-23 — **Phase 18 COMPLETE**: CAT-01..CAT-05 validated via Chrome MCP UAT + goal-backward audit; coverage bumps to 13*
*Updated: 2026-04-24 — **Phase 19 COMPLETE**: EXER-01/02/03 validated via server-driven WebSocket probe + Playwright UAT (desktop + mobile); 2 v1.3 seeds opened (appshell-nestability from 19-02 per D-1; exerciser-instrumentation from 19-05 UAT finding); coverage bumps to 16 — v1.2 now 4/5 phases done, Phase 20 (Live Token Editor) the last remaining phase*
