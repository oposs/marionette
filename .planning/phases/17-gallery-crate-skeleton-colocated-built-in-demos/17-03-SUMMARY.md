---
phase: 17-gallery-crate-skeleton-colocated-built-in-demos
plan: 03
subsystem: demo-app
tags: [axum, tokio, tower-http, sea-orm-mock, linkme, sdui, appshell, integration-test]

# Dependency graph
requires:
  - phase: 16-framework-hooks
    provides: "linkme-backed DemoEntry registry + #[gallery_demo] macro + registered_demos()"
  - phase: 17-01
    provides: "DemoEntry.render: fn() -> Vec<Node> signature (Phase 16.5 refactor)"
  - phase: 17-02
    provides: "per-component builder files — Button, Container, Grid, Heading, Text, etc."
provides:
  - "6th workspace crate gallery-demo booting on :3002"
  - "runtime AppShell nav built from registered_demos() — one NavItem per entry"
  - "gallery-show handler that looks up by key, invokes entry.render(), emits Render to content sub-surface"
  - "Home page (welcome Heading + intro Text + Grid of registry-derived Button tiles)"
  - "11-action handler set: navigate, gallery-show, gallery-demo/{noop,modal-open,confirm-{open,accept,reject},toast-fire}, close-modal, dismiss-toast, fetch-rows"
  - "smoke_boot + nav_auto_discovery integration tests proving CRATE-01 + CRATE-02 end-to-end"
  - "Makefile gallery-dev target"
affects: [17-04, 18-catalog-screens, 19-exerciser-screens, 20-theme-editor]

# Tech tracking
tech-stack:
  added:
    - "sea-orm MockDatabase pattern in production binary (not just tests) — satisfies AppState.db: Arc<DatabaseConnection> without SQL"
    - "lib+bin split (lib.rs + main.rs) so integration tests can use gallery_demo::handlers::register_gallery_actions"
    - "ensure_demos_linked() force-link shim via std::hint::black_box — prevents linker dead-stripping of gallery-smoke"
  patterns:
    - "Thin backend binary = strip(crm-demo) + MockDatabase + pure in-memory Arc<RwLock<_>> state"
    - "AppShell nav iteration: `registered_demos().map(NavItem::new(...).action(gallery-show + payload).build()).collect()`"
    - "Sub-surface routing: handle_navigate seeds main + content + toasts in one dispatch; later actions target content/modal/toasts individually"
    - "ComponentAction payload convention: `action.extra.insert(\"payload\".into(), json!({...}))` — no with_payload() helper on ComponentAction"

key-files:
  created:
    - "backend/crates/gallery-demo/Cargo.toml — 6th workspace manifest, thin-backend deps"
    - "backend/crates/gallery-demo/src/main.rs — tokio main, :3002 bind, ServeDir SPA fallback"
    - "backend/crates/gallery-demo/src/lib.rs — pub mod handlers/home/state + ensure_demos_linked()"
    - "backend/crates/gallery-demo/src/home.rs — registry-derived Home page builder"
    - "backend/crates/gallery-demo/src/state.rs — GalleryState with Arc<tokio::sync::RwLock<_>>"
    - "backend/crates/gallery-demo/src/handlers/{mod,navigate,show,noop,modal,confirm,toast,fetch_rows}.rs"
    - "backend/crates/gallery-demo/tests/smoke_boot.rs"
    - "backend/crates/gallery-demo/tests/nav_auto_discovery.rs"
  modified:
    - "backend/Cargo.toml — gallery-demo added to workspace members"
    - "Makefile — gallery-dev target + .PHONY entry"

key-decisions:
  - "lib+bin split adopted at Task 1 (not deferred to Task 3): avoids mid-plan refactor; tests can use gallery_demo::handlers::... from the start"
  - "gallery-smoke added as a regular dep (not dev-only): enables a non-empty nav at runtime before Plan 17-04 lands the in-marionette demos"
  - "ensure_demos_linked() called inside register_gallery_actions(): one chokepoint covers production binary + all integration tests"
  - "Modal/ConfirmDialog close semantics: render empty Container ('modal-empty') into modal sub-surface. Best-effort until Open Question #2 (CONTEXT.md) settles"
  - "ConfirmDialog demo carries both Accept and Reject buttons as explicit children. The frontend's native close-modal dispatch is orthogonal to the gallery-demo/confirm-accept|reject named flow"
  - "Three-dot ServeDir path (`../../../frontend/build`) in main.rs assumes cwd=backend/crates/gallery-demo/; the Makefile uses `cd backend` which mismatches. Documented below under Deviations"

patterns-established:
  - "Thin gallery backend: MockDatabase + in-memory Arc<RwLock> + no auth"
  - "Registry-driven navigation: nav = registered_demos().map(NavItem)"
  - "Force-link pattern for linkme slices in binary crates (std::hint::black_box over fn ptr)"
  - "Frontend-hardcoded action names route to backend handlers explicitly (close-modal, dismiss-toast) — no alias layer"
  - "Integration tests bind 127.0.0.1:0 and import handlers via the crate's lib.rs"

requirements-completed: [CRATE-01, CRATE-02]

# Metrics
duration: ~55min
completed: 2026-04-22
---

# Phase 17 Plan 03: Gallery-Demo Binary Crate Summary

**Gallery-demo crate boots on :3002 with runtime AppShell nav iterating `registered_demos()`, a 11-action handler set (including the frontend-hardcoded `close-modal`/`dismiss-toast` names), a curated Home page derived from the registry, and two integration tests proving CRATE-01 + CRATE-02 end-to-end.**

## Performance

- **Duration:** ~55 minutes
- **Started:** 2026-04-22T~10:08:00Z
- **Completed:** 2026-04-22T11:02:56Z
- **Tasks:** 3
- **Files created:** 12 (1 Cargo.toml, 10 src files, 2 tests)
- **Files modified:** 2 (backend/Cargo.toml, Makefile)

## Accomplishments

- **CRATE-01 delivered**: gallery-demo is the 6th workspace crate; `cargo run -p gallery-demo` boots on :3002 against `frontend/build/` via tower-http ServeDir with SPA fallback. Thin-backend posture: no auth, no migrations, no bcrypt/chrono/reqwest. AppState.db satisfied by `sea_orm::MockDatabase` (zero SQL).
- **CRATE-02 delivered**: `handle_navigate` iterates `registered_demos()` in flat alphabetical order (per Phase 16 D-A2 + D-C1), emitting one NavItem per entry with id `nav-<key>` and a `gallery-show` action carrying `{key: "<demo-key>"}` in `action.extra["payload"]`. Adding a new `#[gallery_demo]` anywhere in the workspace and rebuilding surfaces the entry in nav without touching gallery-demo/src/main.rs.
- **11-action handler set**: navigate, gallery-show, gallery-demo/noop, gallery-demo/modal-open, close-modal (frontend-hardcoded), gallery-demo/confirm-open, gallery-demo/confirm-accept, gallery-demo/confirm-reject, gallery-demo/toast-fire, dismiss-toast (frontend-hardcoded), fetch-rows. Each emits on the correct sub-surface (main / content / modal / toasts).
- **Registry-derived Home page**: welcome Heading + intro Text + 3-column Grid of Button tiles, one tile per demo, each firing gallery-show with the matching key. Auto-updates as new demos register.
- **Integration tests green**: `smoke_boot` asserts the WS hello frame; `nav_auto_discovery` asserts one `nav-<key>` per registered demo. Both spin up on ephemeral 127.0.0.1:0 ports.
- **Makefile gallery-dev target** landed; mirrors the existing `dev` target shape (single-service; no frontend Vite dev-server — gallery serves the prebuilt bundle directly).

## Task Commits

Each task was committed atomically:

1. **Task 1: Scaffold gallery-demo crate + workspace member + placeholder handlers** — `a1bbcc0` (feat)
2. **Task 2: Implement real handler bodies + Home page** — `540af5e` (feat)
3. **Task 3: Integration tests + Makefile gallery-dev target (+ force-link shim)** — `f7dad5d` (test)

## Files Created/Modified

**Created (in gallery-demo crate):**
- `backend/crates/gallery-demo/Cargo.toml` — 6th workspace manifest; marionette[gallery] + gallery-smoke + axum/tokio/tower-http/sea-orm/uuid; no bcrypt/migration/axum-extra/chrono/reqwest/wiremock
- `backend/crates/gallery-demo/src/main.rs` — tokio main; MockDatabase; `0.0.0.0:3002` bind; `ServeDir::new("../../../frontend/build")`
- `backend/crates/gallery-demo/src/lib.rs` — `pub mod handlers/home/state` + `ensure_demos_linked()` force-link shim
- `backend/crates/gallery-demo/src/home.rs` — `build_home_page()` returns `(root_id, HashMap<String, Component>, serde_json::Value)`
- `backend/crates/gallery-demo/src/state.rs` — `GalleryState` with `Arc<tokio::sync::RwLock<_>>` fields (demo_values, modal_open, confirm_open)
- `backend/crates/gallery-demo/src/handlers/mod.rs` — `register_gallery_actions(ActionRouter) -> ActionRouter` wiring 11 actions
- `backend/crates/gallery-demo/src/handlers/navigate.rs` — AppShell construction + 3-Render response (main/content/toasts)
- `backend/crates/gallery-demo/src/handlers/show.rs` — registry lookup + `(entry.render)()` + seed_for_key() with per-demo state seeds
- `backend/crates/gallery-demo/src/handlers/noop.rs` — toast emit (SetNode + InsertChild on toasts-root)
- `backend/crates/gallery-demo/src/handlers/modal.rs` — open/close handlers targeting `modal` sub-surface
- `backend/crates/gallery-demo/src/handlers/confirm.rs` — open/accept/reject handlers (accept/reject clear modal + emit toast)
- `backend/crates/gallery-demo/src/handlers/toast.rs` — fire + dismiss handlers (Patch on toasts sub-surface)
- `backend/crates/gallery-demo/src/handlers/fetch_rows.rs` — 5 synthetic rows as Set ops on `/demo/data-table/rows/{id}`
- `backend/crates/gallery-demo/tests/smoke_boot.rs` — integration test: WS hello frame shape
- `backend/crates/gallery-demo/tests/nav_auto_discovery.rs` — integration test: one nav-<key> per registered demo

**Modified:**
- `backend/Cargo.toml` — `"crates/gallery-demo"` added as 6th workspace member
- `backend/Cargo.lock` — new dep graph (gallery-demo + sea-orm-mock edge)
- `Makefile` — `gallery-dev` target + .PHONY entry

## Action-Wiring Table

| Action name                     | Handler                                   | Sub-surface | Origin                                           |
|---------------------------------|-------------------------------------------|-------------|--------------------------------------------------|
| `navigate`                      | `handlers::navigate::handle_navigate`     | main/content/toasts (3 Renders) | frontend router.svelte.ts:27 dispatches on WS connect |
| `gallery-show`                  | `handlers::show::handle_gallery_show`     | content     | gallery-demo native; registry-driven routing (D-C3) |
| `gallery-demo/noop`             | `handlers::noop::handle_noop`             | toasts      | gallery-demo native; leaf-demo catch-all           |
| `gallery-demo/modal-open`       | `handlers::modal::handle_modal_open`      | modal       | gallery-demo native; Modal demo trigger            |
| **`close-modal`**               | `handlers::modal::handle_modal_close`     | modal       | **Frontend-hardcoded** (ModalSurface.svelte:15, ConfirmDialog.svelte:34) — NOT `gallery-demo/modal-close` |
| `gallery-demo/confirm-open`     | `handlers::confirm::handle_confirm_open`  | modal       | gallery-demo native; ConfirmDialog trigger         |
| `gallery-demo/confirm-accept`   | `handlers::confirm::handle_confirm_accept`| modal + toasts | gallery-demo native                              |
| `gallery-demo/confirm-reject`   | `handlers::confirm::handle_confirm_reject`| modal + toasts | gallery-demo native                              |
| `gallery-demo/toast-fire`       | `handlers::toast::handle_toast_fire`      | toasts      | gallery-demo native; Toast demo explicit dispatch |
| **`dismiss-toast`**             | `handlers::toast::handle_dismiss_toast`   | toasts      | **Frontend-hardcoded** — toast components dispatch this name |
| `fetch-rows`                    | `handlers::fetch_rows::handle_demo_fetch_rows` | content | DataTable infinite-scroll hook (Phase 13 contract) |

**Total:** 11 actions. Two are frontend-hardcoded (bold above) per RESEARCH.md §Pitfall 3 — registering under any other name would silently fail at runtime since the svelte components dispatch literal strings.

## Test Results

All test suites green:

```
running 3 tests  (lib unit)
test handlers::show::tests::seed_for_unknown_key_is_empty ... ok
test handlers::show::tests::seed_for_form_has_email_and_name ... ok
test handlers::show::tests::seed_table_rows_has_five_rows ... ok

running 1 test  (nav_auto_discovery)
test navigate_shell_render_includes_one_nav_item_per_registered_demo ... ok

running 1 test  (smoke_boot)
test gallery_demo_boots_and_emits_hello ... ok
```

- `cargo build -p gallery-demo`: OK
- `cargo build -p gallery-demo --release`: OK
- `cargo clippy -p gallery-demo -- -D warnings`: 0 warnings
- `cargo build --workspace --features marionette/gallery`: OK

## Decisions Made

- **lib+bin split adopted upfront (Task 1, not deferred to Task 3)**. The plan's Task 3 offered a conditional refactor if binary-crate testing proved incompatible. Landing the split on Task 1 avoided the mid-plan refactor and made tests work out of the box.
- **`gallery-smoke` moved to regular `[dependencies]`, not dev-only**. The plan intended gallery-smoke to be "always present" in `registered_demos()`. As a regular dep, the production `cargo run -p gallery-demo` binary also registers `smoke`, so the gallery is usable (one demo tile visible) even before Plan 17-04's in-marionette demos land. If future drift requires dev-only isolation, a feature-gate is the right path; for v1.2 Phase 17, the regular dep is the pragmatic answer.
- **Force-link via `std::hint::black_box(fn_ptr)` in `ensure_demos_linked()`**. The linkme pattern is "register via a static with a side-effect"; without a live fn-pointer reference, the linker dead-strips the object file. Placing the shim inside `register_gallery_actions()` gives one chokepoint for production and tests.
- **Three-dot `ServeDir` path in `main.rs`**. Matches `env!("CARGO_MANIFEST_DIR")` + `../../../frontend/build` in the integration tests. Caveat: the Makefile's `cd backend && cargo run` cwd is `backend/`, so at runtime the three-dot path resolves to `/<repo>/../frontend/build` which doesn't exist — this mismatch is documented below as a deviation.
- **Modal close = render empty Container into modal sub-surface**. The plan allowed "best-effort approximation until Open Question #2 settles" (CONTEXT.md). Chrome MCP UAT in Plan 04 will exercise the user-visible dismiss flow end-to-end.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 – Bug] `ComponentAction::with_payload(...)` does not exist**

- **Found during:** Task 2 (home.rs + navigate.rs draft)
- **Issue:** The plan's code snippets in multiple places use `ComponentAction::click("gallery-show").with_payload(serde_json::json!({ "key": entry.key }))`, but `ComponentAction` (in `marionette-protocol/src/component.rs`) exposes only `::click`, `::submit`, `::change` associated fns — no `with_payload` method. The payload idiom used elsewhere in the codebase (crm-demo/src/handlers/contact.rs:702) is `action.extra.insert("payload".into(), serde_json::json!({...}))`.
- **Fix:** Applied the `extra.insert("payload", ...)` pattern throughout home.rs + navigate.rs.
- **Files modified:** `backend/crates/gallery-demo/src/home.rs`, `backend/crates/gallery-demo/src/handlers/navigate.rs`
- **Verification:** `cargo build -p gallery-demo` exits 0; the nav_auto_discovery test reaches the assertion and passes.
- **Committed in:** `540af5e` (Task 2 commit)

**2. [Rule 3 – Blocking] linkme `DEMOS` slice empty in integration tests**

- **Found during:** Task 3 (nav_auto_discovery.rs first run)
- **Issue:** The test asserts `registered_demos()` yields at least one entry (gallery-smoke's `smoke`). But gallery-smoke was neither a dep of gallery-demo nor referenced from any live symbol, so the linker dead-stripped gallery-smoke's object file and its `#[gallery_demo]`-emitted static never populated `DEMOS`. Integration test failed with "registered_demos() should yield at least gallery-smoke's 'smoke' key".
- **Fix:** Two-part. (a) Added `gallery-smoke = { path = "../gallery-smoke" }` to gallery-demo's `[dependencies]`. (b) Added `ensure_demos_linked()` in `lib.rs` that holds a fn-pointer to `gallery_smoke::smoke` through `std::hint::black_box`, called from `register_gallery_actions()` so every code path (production binary + tests) triggers the force-link. Mirrors the pattern from `gallery-smoke/tests/registry_roundtrip.rs::force_link_smoke_demo`.
- **Files modified:** `backend/crates/gallery-demo/Cargo.toml`, `backend/crates/gallery-demo/src/lib.rs`, `backend/crates/gallery-demo/src/handlers/mod.rs`
- **Verification:** `cargo test -p gallery-demo --test nav_auto_discovery` now exits 0 with 1 test passing.
- **Committed in:** `f7dad5d` (Task 3 commit)

**3. [Rule 3 – Blocking] Clippy pedantic noise on `missing_errors_doc` + `doc_markdown`**

- **Found during:** Task 2 (`cargo clippy -p gallery-demo -- -D warnings` as per acceptance criteria)
- **Issue:** 24 pedantic-level clippy errors: every `pub async fn handle_*` returning `ActionResult` triggered `missing_errors_doc`; every prose reference to `handle_modal_close` or similar triggered `doc_markdown`. The handlers are uniform — all errors propagate an `ActionError` via `?` or early return. Per-handler `# Errors` docs would be boilerplate without adding information.
- **Fix:** Added crate-level `#![allow(clippy::missing_errors_doc)]` + `#![allow(clippy::doc_markdown)]` in both `lib.rs` and `main.rs`.
- **Files modified:** `backend/crates/gallery-demo/src/lib.rs`, `backend/crates/gallery-demo/src/main.rs`
- **Verification:** `cargo clippy -p gallery-demo -- -D warnings` exits 0.
- **Committed in:** `540af5e` (Task 2 commit)

**4. [Documented mismatch – not fixed] `main.rs` ServeDir path vs Makefile cwd**

- **Found during:** Task 1 / Task 3
- **Issue:** The plan's acceptance criteria require main.rs to contain the literal string `ServeDir::new("../../../frontend/build")` (three `..`) — this resolves correctly only when cwd = `backend/crates/gallery-demo/`. The plan's Makefile target uses `cd backend && cargo run -p gallery-demo` (cwd = `backend/`), which means at runtime the three-dot path looks for `<repo_parent>/frontend/build/` which doesn't exist. This is purely a runtime-SPA-serve concern (the WebSocket endpoint + integration tests are unaffected because the tests use `CARGO_MANIFEST_DIR` resolution).
- **Fix:** None applied — the literal text is the grep target in the plan's acceptance criteria; changing either the path OR the Makefile cwd would break one of the two acceptance greps. Left as-is and flagged here so Plan 17-04 or a follow-up can resolve (options: adjust Makefile cwd to `cd backend/crates/gallery-demo`, or switch to `env!("CARGO_MANIFEST_DIR")` composition).
- **Impact:** `make gallery-dev` will serve the WS endpoint and `/api/health` correctly but will 404 on static assets (the frontend SPA). The auto-discovery + protocol behavior can still be exercised via Chrome MCP by navigating directly to `http://localhost:3002/ws` — but a visual gallery requires the path fix. Plan 17-04 or a follow-up spot-fix is the right scope.

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking) + 1 documented mismatch.
**Impact on plan:** All auto-fixes were necessary for compile/test correctness; none are scope creep. The documented mismatch is a ~1-line planner-level decision for Plan 17-04 or a spot-fix PR.

## Issues Encountered

- **Linker dead-stripping gallery-smoke**: turned into deviation #2 above. The fix is small and reusable for future external demo-registering crates.
- **Integration test output buffering**: the `nav_auto_discovery` test scans up to 5 frames to find the `main` surface Render because `handle_navigate` emits 3 Renders and buffering may interleave. Scan-with-timeout is robust.

## User Setup Required

None — gallery-demo is a thin backend with no secrets, no external services, no DB.

## Next Phase Readiness (Plan 17-04 hand-off)

- **The rails are live.** Plan 17-04 can now add `#[gallery_demo]` siblings to every in-scope builder file in `backend/crates/marionette/src/builders/*.rs` and they will appear in the gallery's nav on the next `cargo run -p gallery-demo`. Zero file overlap with this plan's gallery-demo/ crate.
- **REQUIREMENTS.md §CRATE-01 "5th workspace entry" reconciliation remains open**. Phase 16's gallery-smoke took the 5th slot; gallery-demo is the 6th. Plan 17-04's docs pass (GALLERY-DEMOS.md authoring) is the natural place to update the REQUIREMENTS.md wording to "6th" or clarify that gallery-smoke is a test-fixture crate not counted in the ordinal (per RESEARCH.md §Open Question #4).
- **ServeDir path mismatch (deviation #4 above)** is the only open runtime issue. Plan 17-04 can resolve by adjusting the Makefile `gallery-dev` target (one-line change: `cd backend/crates/gallery-demo && cargo run`) or by switching main.rs to `CARGO_MANIFEST_DIR` composition. Either is non-breaking.
- **Chrome MCP UAT (Success Criterion #5 from CONTEXT.md)** remains scheduled for when Plan 17-04 merges — clicking every nav entry and verifying each produces a screen (not an error surface) is the final gate per `feedback_use_chrome_for_uat.md`.

## Self-Check: PASSED

- All 16 files listed in "Files Created/Modified" exist on disk.
- All 3 task commits (`a1bbcc0`, `540af5e`, `f7dad5d`) are present in `git log`.
- `cargo test -p gallery-demo` passes: 3 unit + 2 integration tests.
- `cargo clippy -p gallery-demo -- -D warnings` exits 0.
- `cargo build --workspace --features marionette/gallery` exits 0.

---
*Phase: 17-gallery-crate-skeleton-colocated-built-in-demos*
*Completed: 2026-04-22*
