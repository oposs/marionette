---
phase: 17
slug: gallery-crate-skeleton-colocated-built-in-demos
status: uat-incomplete
verified_by: chrome-mcp-uat-walk
verified_at: 2026-04-22
---

# Phase 17 Verification Report

**Verification method:** Chrome MCP UAT walk against SC #5 ("every nav entry produces a screen, not an error surface") and interactive flows for trigger-based components.

**Summary:** 11 of 21 demos render and behave as designed. 9 demos fail SC #5 (empty content, client lockup, or non-dismissing overlay). 1 page has visible UX issues (Home footer + unseeded sub-surfaces). Framework-level deliverables (CRATE-01 gallery-demo crate, CRATE-02 auto-discovered nav, DEMO-02 GALLERY-DEMOS.md, D-Z1 Vec<Node> signature, per-component file refactor) all pass — the gaps are all in the gallery-demo handlers and specific demo-fn bodies.

---

## Success Criteria (from ROADMAP §Phase 17)

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | `cargo run -p gallery-demo` starts the app on its own port (3002) against the shared frontend with no auth, no database, no migrations — only `Arc<RwLock<_>>` in-memory state | ✅ PASS | Server alive on :3002; `curl /api/health` returns `ok`; no sea_orm migrator / bcrypt referenced in gallery-demo/src/main.rs |
| 2 | AppShell navigation is built at runtime by iterating `registered_demos()`; adding a new `#[gallery_demo]` and rebuilding causes the entry to appear in nav without touching the gallery binary | ✅ PASS | Sidebar shows 20 nav entries derived from the registry — alphabetical order, one NavItem per key |
| 3 | Every currently-registered built-in component has a `pub fn gallery_demo() -> Vec<Node>` sibling annotated with `#[gallery_demo]` | ✅ PASS | `cargo test -p marionette --features gallery --lib gallery::builtin_coverage` passes; 20 registered (19 in-scope + 1 smoke) |
| 4 | `GALLERY-DEMOS.md` documents the pure-fn contract | ✅ PASS | `backend/crates/marionette/GALLERY-DEMOS.md` exists, covers contract + skip list + bind-path convention + composite rule |
| 5 | **Each built-in demo renders without panicking when visited in the running gallery; clicking every nav entry produces a screen, not an error surface** | ❌ **FAIL** | 9 demos fail (see §Gaps below); 1 demo (Modal) locks up the client when interacted with |

---

## Gaps (ordered by severity)

### BLOCKER gaps (SC #5 failures)

#### G-01 — Modal demo: client lockup on "Open modal" trigger
- **Symptom:** Clicking the "Open modal" button hangs the browser tab (Chrome extension disconnects). Server remains responsive.
- **Demo key:** `modal`
- **Files implicated:** `backend/crates/gallery-demo/src/handlers/modal.rs`, `backend/crates/marionette/src/builders/modal.rs` (demo body), `frontend/src/lib/components/popup/ModalSurface.svelte`
- **Hypothesis:** The `gallery-demo/modal-open` handler either (a) isn't registered, (b) returns a malformed Render to the `modal` sub-surface, or (c) produces a patch that sends `ModalSurface.svelte` into a render loop
- **Fix scope:** Trace handler registration → inspect the Render message sent → test in browser with console open

#### G-02 — App Shell demo: hijacks outer gallery sidebar
- **Symptom:** Clicking "App Shell" in the nav replaces the gallery's own sidebar (20 demo entries) with the inner AppShell demo's sidebar (Dashboard/Reports/Settings). Nav navigation is lost until page refresh.
- **Demo key:** `app-shell`
- **Files implicated:** `backend/crates/marionette/src/builders/app_shell.rs` (the `gallery_demo()` fn at the bottom of the hand-written builder file)
- **Root cause:** The demo body uses `SurfaceMount("sidebar")` / `SurfaceMount("header")` / `SurfaceMount("footer")` whose surface names are GLOBAL — they collide with the outer gallery's same-named surfaces, replacing outer content.
- **Fix scope:** Rewrite `AppShell::gallery_demo()` to NOT use nested SurfaceMounts. Options: (a) render a canonical AppShell structure inline using Container+Heading for the sidebar/header/main slots (no SurfaceMount), (b) render a read-only description of what an AppShell looks like using Text/Heading only (simpler but less interesting), (c) investigate frontend for scoped surface names (out of Phase 17 scope). Preferred: (a).

#### G-03 — Data Table: empty body (column headers render; no rows)
- **Symptom:** Visiting the Data Table demo shows column headers (ID, Name, Email, Created) and the Columns toggle button, but zero rows.
- **Demo key:** `data-table`
- **Files implicated:** `backend/crates/gallery-demo/src/handlers/fetch_rows.rs`, `backend/crates/gallery-demo/src/handlers/show.rs` (seed routine for `data-table` key), `backend/crates/marionette/src/builders/data_table.rs` (demo body)
- **Hypothesis:** Either (a) the demo's DataTable component doesn't declare `source = "demo-rows"`, (b) `fetch-rows` handler isn't registered, (c) handler doesn't match the `"demo-rows"` source name, or (d) handler returns empty
- **Fix scope:** Verify all four; likely a one-line fix in whichever is wrong

#### G-04 — ConfirmDialog demo: Accept/Reject buttons don't dismiss
- **Symptom:** Clicking "Open confirm" opens the dialog (though inline below footer, not as overlay). Clicking Confirm or Cancel does NOT close the dialog. The dialog persists across nav clicks.
- **Demo key:** `confirm-dialog`
- **Files implicated:** `backend/crates/gallery-demo/src/handlers/confirm.rs`, possibly `backend/crates/marionette/src/builders/confirm_dialog.rs` (demo body)
- **Hypothesis:** `gallery-demo/confirm-accept` and `gallery-demo/confirm-reject` handlers either don't clear the `modal` sub-surface, or aren't wired to the Confirm/Cancel button actions. Related to G-07 (open overlays render in-flow, not as dialogs).
- **Fix scope:** Verify both actions are registered AND emit a Render clearing the `modal` surface on dismissal

#### G-05 — Empty-content demos: 5 demos render zero content
- **Symptom:** Visiting any of these 5 demos shows header and footer but completely empty main content area.
- **Affected demos:** `error-display`, `field-set`, `radio-group`, `switch`, `textarea`
- **Files implicated:** `backend/crates/marionette/src/builders/{error_display,field_set,radio_group,switch,textarea}.rs` — specifically each file's `gallery_demo()` sibling fn
- **Hypothesis:** Shared root cause across these 5 demos. Candidates:
  1. The fn returns `vec![]` (empty) instead of `vec![root_tuple, ...descendants]`
  2. The fn returns the root only, without the descendants that `Container::new().children(vec![...]).build_tree()` produces
  3. The `.build_tree()` vs `.build_with_children()` choice returns a shape incompatible with `Vec<Node>` (see RESEARCH.md §Pitfall 4)
  4. The builders' required fields (e.g., `RadioGroup.options`) are constructed but the resulting children aren't flattened
- **Fix scope:** Inspect each of the 5 gallery_demo fns, compare against working leaf demos (button, checkbox, heading) that DO render, identify the pattern difference, apply one fix repeated 5 times

### HIGH gaps (visible Home UX issues)

#### G-06 — Home footer text oversized
- **Symptom:** Footer displays "Marionette Gallery · v1.2" and "connected" as giant bold `<h2 class="text-xl font-semibold">` headings inside the footer, overriding the footer's `text-xs text-muted-foreground` styling.
- **Files implicated:** `backend/crates/gallery-demo/src/handlers/navigate.rs:51,54`
- **Fix:** Replace `Heading::new("...")` with `Text::new("...")` for both footer version and footer status.

#### G-07 — Modal and Toasts sub-surfaces unseeded; render in-flow with skeleton placeholders
- **Symptom 1:** Home page shows horizontal grey bars below the footer (`LoadingSkeleton` rendering because the `modal` and `toasts` sub-surfaces have no initial root).
- **Symptom 2:** When Modal / ConfirmDialog / Toast demos open, their content renders inline below the footer rather than as a true overlay/fixed-position toast.
- **Files implicated:** `backend/crates/gallery-demo/src/handlers/navigate.rs`
- **Root cause:** Gallery-demo's `navigate` handler sends only the shell Render. It never seeds the `modal` and `toasts` sub-surfaces with empty-Container roots. CRM demo does this (crm-demo/src/main.rs ~lines 302-309).
- **Fix:** Add two initial Render messages to `handle_navigate` — one for `modal` with an empty Container root, one for `toasts` with an empty Container root. Matches CRM's pattern.
- **Note:** The in-flow rendering (Symptom 2) may be a separate frontend concern. Phase 17 fix addresses Symptom 1 (skeletons gone); if Symptom 2 persists after seeding, record as a Phase 19 EXER-01 exerciser concern.

### MEDIUM gaps (architectural hygiene)

#### G-08 — Stranded `Modal` builder after popups-global architectural fix
- **Symptom:** `backend/crates/marionette/src/builders/modal.rs` defines `#[derive(ComponentBuilder)] pub struct Modal { title, size }` with `#[component(type = "modal")]`, plus a `gallery_demo()` sibling. Plan 17-05's architectural fix (2026-04-22) mounted `ModalSurface` as a layout-root singleton in `+layout.svelte` and removed `'modal': ModalSurface` from `frontend/src/lib/registry/defaults.ts`. After that change, any node of type `"modal"` emitted into the SDUI tree falls through to the unknown-type fallback — the `Modal` builder now produces dead nodes.
- **Surfaced by:** Chrome MCP UAT walk of the Plan 17-05 architectural fix, 2026-04-22.
- **Demo key impact:** The `modal` demo page itself still renders (its `gallery_demo()` builds only `Button` + `Text` children in a `Container`; it never calls `Modal::new(...)`). The stranded primitive is the `Modal` builder struct, not the demo page's body.
- **Files implicated:**
  - `backend/crates/marionette/src/builders/modal.rs` — struct `Modal` + `gallery_demo()` sibling
  - `backend/crates/marionette/src/builders/mod.rs:23, 54` — `pub mod modal; pub use modal::*;`
  - `backend/crates/marionette/src/builders/standard.rs` — compatibility re-export
  - `backend/crates/marionette/src/builders/mod.rs:94` — test callsite `Modal::new("x").build()`
  - `backend/crates/marionette/GALLERY-DEMOS.md` — any mention of `Modal::new`
  - `backend/crates/gallery-demo/src/handlers/modal.rs` — comment references `(not Modal::new)` pattern
- **Root cause:** The `Modal` component type was ModalSurface's SDUI-dispatch key. Plan 17-05's move of ModalSurface to a layout-root static mount dropped that dispatch registration but left the producer intact.
- **User-architectural decision (2026-04-22):** Popups must work independent of any other component (AppShell included). The popup toolbox stays compositional — authors emit any SDUI tree (`Container` with `Form`, `TextInput`, `Button`, …) into the `modal` sub-surface and `ModalSurface.svelte` (layout-root) wraps it in `<Dialog.Root>`/`<Dialog.Content>` automatically. No dedicated `Modal` wrapper primitive is needed. `ConfirmDialog` remains as the canonical structured-accept-cancel variant.
- **Fix scope:** Delete the `Modal` struct + `gallery_demo()` sibling. Remove the `modal` module publish from `mod.rs` (and the `Modal::new("x")` test expectation — update to use a different component-type smoke or delete). Remove the `standard.rs` re-export line. Update `GALLERY-DEMOS.md` with a short "popup composition" recipe + the canonical "form in popup" example (Container → Heading + Form(TextInput + TextInput) + Container(Button(cancel) + Button(save))). Update `handlers/modal.rs` comment that currently reads "(not Modal::new)" to explain the compositional pattern instead.
- **Classification:** MEDIUM. Compiles today; primitive is dead and misleading to handler authors. Not an SC #5 failure (no demo breaks), but a documented-toolbox integrity issue surfaced by 17-05's architectural fix.

---

## Behavior audit — passing demos (for regression guard)

For reference, these 11 demos + Home-content render correctly. Any fix plan must not regress them:
- `home` (content; header + subheader + 20-tile grid)
- `button` (3 instances: Primary, Disabled, Destructive)
- `checkbox` (3 instances: Unchecked, With description, Disabled)
- `form` (Form with 3 TextInputs + 2 Selects + Submit button — composite nesting works)
- `grid` (3-col layout A-F)
- `heading` (3 levels)
- `select` (2 Selects: Fruit, Disabled)
- `smoke` ("gallery-smoke" text)
- `spinner` (3 spinners)
- `text` (3 text blocks at varying lengths)
- `text-input` (3 inputs: Label, Disabled, With description)
- `toast` ("Fire toast" button → "Demo toast from gallery-demo/toast-fire" appears)

---

## Not a gap — non-issues ruled out

- **Pink border around viewport** — Chrome MCP remote-driving visual artifact, NOT a frontend bug.
- **Button full-width styling** — cosmetic; Container default. Phase 18 polish.
- **Grid A-F plain styling** — cosmetic; Phase 18 polish.
- **Sub-surface in-flow rendering for Toast** — acceptable for Phase 17 ("feels alive" bar met — toast appears on click). Phase 18 CAT-04 Feedback screen will properly overlay toasts.

---

## Out-of-phase deferrals (record but do not block Phase 17 closure)

- **Sub-surface overlay positioning** — if modal/toasts continue to render in-flow after seeding, that's a frontend `Surface.svelte` / `ModalSurface.svelte` concern, not a Phase 17 skeleton concern. Record as Phase 19 EXER-01 or Phase 18 CAT-04 input.
- **Home tile grid styling** — cosmetic; Phase 18 CAT-05 Typography & tokens territory.

---

## Gap closure criteria

Phase 17 is complete when:
1. G-01 through G-05 are fixed (9 demos pass SC #5 on a fresh Chrome walk)
2. G-06 and G-07 are fixed (Home page footer is small muted text; no skeleton bars below footer)
3. All 11 currently-passing demos remain passing (no regression)
4. `cd backend && cargo test --workspace --features gallery` exits 0
5. `cd backend && cargo clippy --workspace --features gallery -- -D warnings` stays clean
6. Chrome MCP UAT walk of all 20 nav entries: 20/20 produce a screen (not an error surface or empty content); interactive flows on Modal + ConfirmDialog + Toast work (open + dismiss cleanly)

---

*Verified: 2026-04-22 via Chrome MCP UAT walk*
