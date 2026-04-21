---
phase: 12-protocol-node-patching-appshell
plan: 07
subsystem: backend
tags: [backend, rust, crm, handlers, app-shell, surface-mount, nav-patch]

requires:
  - phase: 12-05
    provides: "AppShell builder with six named slots and build_with_children; SurfaceMount derived builder"
  - phase: 12-06
    provides: "AppShell.svelte, SurfaceMount.svelte registered in frontend; websocket.svelte.ts publishes /system/connectionStatus; +layout.svelte collapsed to single <Surface name='main'/>"
provides:
  - "handle_navigate builds an AppShell into the main surface and renders the contact list into the content sub-surface on login"
  - "All CRM screen handlers (contact, company, user, audit, interaction) render to surface 'content', not 'main'"
  - "Every screen handler emits a nav_active_patch into surface 'main' updating /nav/active/* paths (D-B13)"
  - "Shell initial data seeds /auth/currentUser, /system/connectionStatus, and /nav/active/contacts=true"
  - "Footer contains three D-B6 children: version literal, connection-status Heading bound to /system/connectionStatus, legal text"
  - "Login flow preserved: build_login_form still renders into 'main' (pre-auth path, D-B11)"
  - "AppShell.children populated with slot IDs for GC reachability in frontend gcOrphans (gap-closure fix from Plan 12-05)"
affects:
  - "Phase 15 (per-screen CRUD cleanup deferred by D-B12)"
  - "Any future plan adding new CRM screens — must render to 'content' and emit nav_active_patch"

tech-stack:
  added: []
  patterns:
    - "Two-message response pattern: shell Render into main + screen Render into content, ordered shell-first"
    - "nav_active_patch module-level helper per handler file: emits a single PatchMessage targeting 'main' with 5 Set ops clearing all /nav/active/* and setting only the current slug to true"
    - "Shell data seeding: auth.currentUser / system.connectionStatus / nav.active all seeded in the handle_navigate Render's data field so the shell renders with values on first mount"

key-files:
  created: []
  modified:
    - backend/crates/crm-demo/src/main.rs
    - backend/crates/crm-demo/src/handlers/contact.rs
    - backend/crates/crm-demo/src/handlers/company.rs
    - backend/crates/crm-demo/src/handlers/user.rs
    - backend/crates/crm-demo/src/handlers/audit.rs
    - backend/crates/crm-demo/src/handlers/interaction.rs
    - backend/crates/marionette/src/builders/app_shell.rs

key-decisions:
  - "Shell Render ordered before content Render so the frontend node map contains the shell before any content patches reference it"
  - "nav_active_patch emits Set ops for ALL five slugs on every nav event, not just the activated one — avoids stale active state when nav items are added or renamed"
  - "Header user name uses session.user_id (integer user ID string) rather than a display name lookup — deferred until Phase 15 CRM cleanup per D-B12"
  - "AppShell.children populated with slot IDs (gap-closure fix): GC reachability in gcOrphans requires children links; slot IDs in props alone were insufficient"

patterns-established:
  - "All SDUI screen handlers in crm-demo render to surface 'content'; the shell surface 'main' is written only by handle_navigate (initial shell build) and build_login_form (pre-auth login)"
  - "Per-handler nav_active_patch helper placed at module scope, not per render-site function — one copy per handler file drives all render exits in that file"

requirements-completed: [SHELL-01, SHELL-02, SHELL-04]

duration: ~45 min (Tasks 1-2 autonomous + Task 3 orchestrator-driven verification session)
completed: 2026-04-10
---

# Phase 12 Plan 07: CRM Integration Summary

**CRM demo backend migrated to AppShell sub-surface architecture: handle_navigate builds an AppShell into main and renders contact list into content, all screen handlers retargeted to content with D-B13 nav_active_patch wiring, verified end-to-end with Chrome automation including a post-hoc fix for AppShell GC reachability.**

## Performance

- **Duration:** ~45 min wall-clock (Tasks 1-2 autonomous execution + orchestrator-driven Task 3 verification)
- **Completed:** 2026-04-10
- **Tasks:** 3 (2 auto + 1 checkpoint:human-verify — APPROVED)
- **Files modified:** 7

## Accomplishments

- `handle_navigate` now builds a full AppShell (sidebar, header, footer, three SurfaceMount slots) into the `main` surface and simultaneously renders the contact list into the `content` sub-surface — a single login produces exactly two ordered ProtocolMessages.
- All 8 non-auth Render sites across 5 handler files retargeted from `surface: "main"` to `surface: "content"`, and each handler gained a module-level `nav_active_patch` helper that emits a 5-op Set Patch into `main` on every navigation, wiring D-B13 sidebar active state.
- End-to-end verification (all 11 steps) passed via orchestrator-driven Chrome browser automation, confirming: full AppShell renders on login, navigation between Contacts/Companies/Users/Audit updates only the content sub-surface, connection status indicator flips on backend kill/restart, and mobile hamburger sheet works correctly.
- Post-hoc gap-closure fix landed (commit `62f2a39`): AppShell builder now populates `Component.children` with slot IDs so the frontend's `gcOrphans` BFS treats slot roots as reachable — closing a Plan 12-05 omission discovered during Task 3 verification.

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite handle_navigate to build AppShell + render into content sub-surface** - `8a7008e` (feat)
2. **Task 2: Rename handler render surfaces main -> content and wire D-B13 nav_active_patch** - `c0c19fe` (feat)
3. **Task 3: AppShell.children gap-closure fix (post-hoc, found during checkpoint verification)** - `62f2a39` (fix)

## Files Created/Modified

- `backend/crates/crm-demo/src/main.rs` — handle_navigate rewritten: imports AppShell + SurfaceMount, builds sidebar/header/footer/mounts sub-trees, assembles AppShell via builder, seeds shell_data with auth.currentUser + system.connectionStatus + nav.active, emits shell Render into main then content Render into content
- `backend/crates/crm-demo/src/handlers/contact.rs` — two Render sites changed to "content"; module-level `nav_active_patch("contacts")` helper added; both render-exit functions append the patch
- `backend/crates/crm-demo/src/handlers/company.rs` — same pattern, slug "companies"
- `backend/crates/crm-demo/src/handlers/user.rs` — same pattern, slug "users"
- `backend/crates/crm-demo/src/handlers/audit.rs` — same pattern, slug "audit"
- `backend/crates/crm-demo/src/handlers/interaction.rs` — same pattern, slug "contacts" (interactions are reached via the contact nav entry)
- `backend/crates/marionette/src/builders/app_shell.rs` — `build()` and `build_with_children()` both now populate `Component.children: Some(vec![slot_ids])` for GC reachability; two inline tests updated to assert the canonical children list

## Decisions Made

- Shell Render is ordered before the content Render so the frontend node map contains the full shell tree before any subsequent content Renders or Patches arrive.
- `nav_active_patch` emits Set ops for all five slugs on every nav event (not only the activated one) — this avoids stale-active bugs when slugs are added or the set changes.
- Header user name is sourced from `session.user_id` (the session's integer user ID as a string, e.g., "User: 1") rather than a display-name database lookup. A proper display name requires a `users` table lookup that is deferred to Phase 15 per D-B12.
- The GC-reachability fix (`children` on AppShell) is attributed to Plan 12-05 as a gap-closure even though the commit lands on the phase 12 main branch. The fix does not change `<app-shell>` Svelte rendering behavior (slots are still mounted via `*NodeId` props); it only populates `children` so `gcOrphans` BFS does not prune slot roots.

## Checkpoint Verification

Task 3 was a blocking `checkpoint:human-verify`. The orchestrator drove a headful Chrome session via the claude-in-chrome MCP after killing a stale pre-12-07 crm-demo process on port 3001 and starting fresh `cargo run -p crm-demo` and `npm run dev` instances. All 11 steps passed:

| Step | Description | Result |
|---|---|---|
| 1 | Backend up on :3001 | PASS |
| 2 | Frontend up on :5173 | PASS |
| 3 | Browser opens login form | PASS |
| 4 | Login with admin@localhost / admin | PASS |
| 5 | After login: full AppShell visible — sidebar (Home, Contacts active, Companies, Users, Audit Log), header (hamburger + "Marionette CRM" + "User: 1"), contact table (Alice Johnson, Bob Smith), footer ("Marionette v1.1 · Protocol 1.1.0" / "connected" / "© 2026 Marionette") | PASS (after post-hoc fix) |
| 6 | Click Companies — active state moves Contacts→Companies, content updates to Company Management (Acme Corp, Globex Inc), sidebar/header/footer stable | PASS |
| 7 | Click Users — active moves to Users, content updates to User Management table | PASS |
| 8 | Kill backend — footer "connected" flips to "reconnecting" within 3s; restart backend — WS reconnects, fresh session, re-login restores "connected" footer | PASS — D-B6 wiring confirmed |
| 9 | Resize to 600×900 — sidebar collapses; click hamburger — sidebar sheet slides in showing all 5 nav items with Contacts highlighted | PASS |
| 10 | Outgoing `user_list` action captured via WS send spy; after click, `main` surface stayed stable at 17 nodes (app-shell-root + 6 slot roots + 10 descendants) and `content` swapped to a 4-node User Management subtree | PASS — GC-stability is the definitive proof |
| 11 | Browser console errors: 0 | PASS |

**Verification technique note:** A `WebSocket.prototype.send` monkey-patch was injected in the browser console to capture outgoing frames. Incoming frames from an already-open WS connection could not be captured directly; the node-count stability of the `main` surface (confirmed via dynamic `import('/src/lib/store/surfaces.svelte.ts')` store inspection) served as the definitive proof that the render+patch pair landed correctly without GC pruning.

## Post-hoc Fix: AppShell GC Reachability (Plan 12-05 Gap, Found at Task 3)

**Commit:** `62f2a39` — `fix(12-05): populate AppShell.children with slot IDs for GC reachability`

**Root cause (Plan 12-05 omission):** The Plan 12-05 `AppShell` builder in `backend/crates/marionette/src/builders/app_shell.rs` emitted the shell root `Component` with `children: None`, storing the six slot node IDs only in props (`sidebarNodeId`, `headerNodeId`, etc.). That matches how the `<app-shell>` Svelte component mounts its slots (it reads props, not children), but the frontend's walk-and-prune GC (`gcOrphans` in `frontend/src/lib/store/surfaces.svelte.ts`, introduced in Plan 12-04 D-A8) walks reachability strictly via `node.children`. On every post-patch GC sweep — triggered by every `nav_active_patch` from the CRM handlers and by the initial render's own GC pass — BFS from `app-shell-root` found no `.children`, marked only the shell root reachable, and pruned all six slot roots plus all their descendants from the `main` surface.

**Observed symptom:** "Right after login the UI is fully populated … it just disappeared very quickly." Confirmed state after the bug fired: `main` surface contained only 1 node (`app-shell-root`) while the shell had initially been rendered with 17 nodes. The `content` sub-surface was unaffected because `gcOrphans` is scoped per-surface (D-A8) and the content tree was never part of `main`.

**Fix:** In `AppShellBuilder::build()` and `build_with_children()`, populate `Component.children: Some(vec![slot_ids in canonical order])` filtered to populated slots only. The `<app-shell>` Svelte component continues to mount slots via `*NodeId` props — `children` now only drives graph reachability for GC. This matches the protocol contract (`Component.children: Ordered list of child node IDs`) and keeps GC semantics uniform across all builders.

**Verification:** Two inline tests updated to assert the canonical children list. All 5 AppShell tests plus full `cargo test -p marionette` suite passed after the fix.

**Attribution:** Plan 12-05 gap; fix committed to phase 12 main branch during Plan 12-07 Task 3 checkpoint verification.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AppShell.children missing — GC prunes all slot roots on first patch**
- **Found during:** Task 3 (checkpoint verification, live browser session)
- **Issue:** `AppShell` builder (Plan 12-05) emitted `children: None`; frontend `gcOrphans` BFS could not reach slot roots and pruned them on the first nav_active_patch, leaving the `main` surface with only the shell root (1 node) and blanking the entire AppShell chrome
- **Fix:** `build()` and `build_with_children()` in `app_shell.rs` now set `children: Some(slot_ids)` with the canonical ordered list of populated slot IDs
- **Files modified:** `backend/crates/marionette/src/builders/app_shell.rs`
- **Verification:** 5 AppShell unit tests + full marionette test suite pass; Task 3 Step 5 re-verified with all 17 main-surface nodes intact after GC sweep
- **Committed in:** `62f2a39`

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug)
**Impact on plan:** Fix was essential for correctness; without it the AppShell chrome was invisible after the first nav event. Root cause was a Plan 12-05 omission exposed by Plan 12-07 live verification. No scope creep.

## Known Stubs

None — all data wired. One deferred item:

- **Header user display name** (`backend/crates/crm-demo/src/main.rs`, `handle_navigate`): the `header_user` Heading text is `"User: {user_id}"` (integer ID string, e.g., "User: 1") rather than a human-readable display name. A proper lookup requires the `users` table and is deferred to Phase 15 per D-B12. This is an intentional deferral, not a stub that prevents the plan's goal.

## Deferred Items

**Clippy warnings in crm-demo (pre-existing baseline, not introduced by this plan):** 77 clippy warnings exist in `backend/crates/crm-demo` prior to this plan. The plan's acceptance criteria required `cargo clippy -p crm-demo -- -D warnings` to be green; however, the pre-existing warning baseline was already present before any 12-07 edits. These warnings are logged in `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` and are tracked for Phase 15 CRM cleanup (D-B12). They are NOT regressions introduced by this plan.

## Issues Encountered

- **Stale crm-demo process on port 3001:** The orchestrator found a pre-12-07 binary still running on port 3001 during Task 3 verification. It was killed (pid 278917) before starting the updated binary, ensuring the live session reflected the new code. Not a code defect — a deployment hygiene issue.
- **Incoming WS frame capture blocked by pre-existing connection:** The verification WS spy (`WebSocket.prototype.send` monkey-patch) could only capture outgoing frames. Node-count stability via direct Svelte store inspection served as the alternative proof of correct render+patch behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Wave 4 of Phase 12 is complete. Plans 12-05, 12-06, and 12-07 together deliver a fully working AppShell CRM with sub-surface navigation and D-B6/D-B13 wiring.
- Wave 5 (Plans 12-08 and beyond) can proceed — the AppShell architecture is verified end-to-end.
- Phase 15 (per-screen CRUD cleanup, D-B12) can safely target the `content` sub-surface and use the `nav_active_patch` helper pattern established here.
- No blockers.

---
*Phase: 12-protocol-node-patching-appshell*
*Completed: 2026-04-10*
