---
phase: 12
plan: 07
type: execute
wave: 3
depends_on: [12-05, 12-06]
files_modified:
  - backend/crates/crm-demo/src/main.rs
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/crm-demo/src/handlers/company.rs
  - backend/crates/crm-demo/src/handlers/user.rs
  - backend/crates/crm-demo/src/handlers/audit.rs
  - backend/crates/crm-demo/src/handlers/interaction.rs
autonomous: false
requirements: [SHELL-01, SHELL-02, SHELL-04]
nyquist_compliant: true
tags: [backend, crm, handlers, app-shell]
must_haves:
  truths:
    - "handle_navigate builds an AppShell via AppShell::new() and renders it into the main surface"
    - "handle_navigate also renders the initial screen (contact list) into the content sub-surface"
    - "All CRM screen handlers render to surface 'content' (not 'main') — the only Rust code rendering into 'main' is the shell-building site in main.rs"
    - "Login flow still works: build_login_form renders to 'main' (pre-auth), login success triggers handle_navigate which renders shell into 'main' + screen into 'content'"
    - "Header includes a user menu that displays the logged-in user's name from /auth/currentUser"
    - "Footer includes version info ('Marionette v1.1 · Protocol 1.1.0') and a connection status indicator"
  artifacts:
    - path: "backend/crates/crm-demo/src/main.rs"
      provides: "handle_navigate builds AppShell"
      contains: "AppShell::new()"
  key_links:
    - from: "main.rs handle_navigate"
      to: "AppShell builder"
      via: "AppShell::new().sidebar(...).header(...).footer(...).main(...)"
      pattern: "AppShell::new\\(\\)"
---

<objective>
Migrate the CRM demo backend to render into the new AppShell sub-surface architecture: `handle_navigate` builds an AppShell into the `main` surface, renders the initial screen (contact list) into the `content` sub-surface, and all non-auth handlers rename their render target from `"main"` to `"content"`. Header shows the logged-in user; footer shows version + connection status (replacing the retired ConnectionBanner). Matches D-B5, D-B6, D-B9, D-B11, D-B12.

Purpose: Closes Part B's backend half. Together with Plan 06, this makes the CRM demo boot into AppShell with working navigation. Per-screen CRUD cleanup is deferred to Phase 15 (D-B12).

Output: Running `cargo run -p crm-demo` serves a CRM that, after login, shows the shell with sidebar, header, main content area, and footer. Navigation between contact/company/user/audit screens works and updates only the `content` sub-surface.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@backend/crates/crm-demo/src/main.rs
@backend/crates/crm-demo/src/handlers/mod.rs
@backend/crates/crm-demo/src/handlers/contact.rs
@backend/crates/crm-demo/src/handlers/company.rs
@backend/crates/crm-demo/src/handlers/user.rs
@backend/crates/crm-demo/src/handlers/audit.rs
@backend/crates/crm-demo/src/handlers/interaction.rs
@backend/crates/marionette/src/builders/app_shell.rs
@backend/crates/marionette/src/builders/standard.rs

<interfaces>
Current `handle_navigate` in `main.rs:122-188` (verified):
- Calls `handlers::contact::handle_contact_list` to get screen Render messages (surface: "main")
- Builds a `SideNav` with nav items using `NavItem::new(...)` + `SideNav::new().children(...).build_with_children()`
- Pushes a separate Render into surface `"sidebar"` with the SideNav tree
- Returns both messages

Target:
1. Call `handle_contact_list` — it now returns Render into surface "content" (after Task 2 edits)
2. Build an AppShell with sidebar (SideNav), header (container with title + user-menu placeholder), footer (container with version + connection-status placeholder), main (SurfaceMount name="content"), popups (SurfaceMount name="modal"), toasts (SurfaceMount name="toasts")
3. Render the AppShell tree into surface "main"
4. Return `[AppShell-render-into-main, contact-list-render-into-content]`
5. Delete the separate sidebar-surface Render (sidebar is now a slot child of the shell)

Existing handler render sites that must change from `"main"` to `"content"` (from earlier grep):
- `backend/crates/crm-demo/src/main.rs:236` — INSIDE `build_login_form` which STAYS `"main"` because pre-auth login uses a full `main` render, not a content sub-surface (D-B11). **Do NOT change this one.**
- `backend/crates/crm-demo/src/handlers/contact.rs:444, 904` — change to `"content"`
- `backend/crates/crm-demo/src/handlers/company.rs:137, 376` — change to `"content"`
- `backend/crates/crm-demo/src/handlers/user.rs:99, 293` — change to `"content"`
- `backend/crates/crm-demo/src/handlers/audit.rs:209` — change to `"content"`
- `backend/crates/crm-demo/src/handlers/interaction.rs:148` — change to `"content"`

`main.rs:181` (`surface: "sidebar"` for the side-nav render) is DELETED entirely — the side-nav becomes an in-tree slot child of the shell.

Auth user lookup: `Session::from_context(&ctx)` returns `Session { user_id, roles }`. A `/auth/currentUser` data binding in the header's user-menu can come from the Render's `data` field — populate `data.auth.currentUser = { name: "…" }` in the handle_navigate Render and bind the header's user-menu text component to `/auth/currentUser/name`.

HelloMessage protocol version is already 1.1.0 from Plan 02; footer version string renders the literal "Marionette v1.1 · Protocol 1.1.0" as a Heading or Text component in the footer container — no data binding needed unless desired.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Rewrite handle_navigate in main.rs to build AppShell + render screen into content sub-surface</name>
  <read_first>
    - backend/crates/crm-demo/src/main.rs (full file)
    - backend/crates/crm-demo/src/handlers/contact.rs (first ~50 lines + around line 444)
    - backend/crates/marionette/src/builders/app_shell.rs (from Plan 05 — confirm method signatures)
    - backend/crates/marionette/src/builders/standard.rs (confirm SurfaceMount exists)
    - .planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md D-B4, D-B5, D-B6, D-B11, D-B12
  </read_first>
  <action>
1. In `backend/crates/crm-demo/src/main.rs`, update the import block at lines 18-27. Add `AppShell` and `SurfaceMount` to the `marionette::builders::standard` import or create a separate `marionette::builders::app_shell::AppShell` import:

```rust
use marionette::builders::standard::{
    Button, Container, Form, Heading, NavItem, SideNav, SurfaceMount, Text, TextInput,
};
use marionette::builders::app_shell::AppShell;
```

(If `Text` is not already used, skip that addition.)

2. REPLACE the body of `handle_navigate` (currently lines 122-188). New implementation:

```rust
/// Handle the `navigate` action: build AppShell + render the contact list
/// into the `content` sub-surface as the default authenticated view.
async fn handle_navigate(ctx: HandlerContext) -> ActionResult {
    let session = Session::from_context(&ctx)?;
    let is_admin = session.roles.contains(&"admin".to_string());
    let user_name = session.user_id.clone().unwrap_or_else(|| "User".to_string());

    // -- Build the content (screen) render first --
    // The contact_list handler now renders into surface "content" (Task 2).
    let mut content_messages = handlers::contact::handle_contact_list(HandlerContext {
        action: ctx.action.clone(),
        db: ctx.db.clone(),
        session: ctx.session.clone(),
    })
    .await?;

    // -- Build the sidebar sub-tree (SideNav with NavItems) --
    let mut nav_items: Vec<(String, marionette_protocol::Component)> = Vec::new();
    nav_items.push(
        NavItem::new("Home", "/")
            .id("nav-home")
            .action(ComponentAction::click("navigate"))
            .build(),
    );
    nav_items.push(
        NavItem::new("Contacts", "/contacts")
            .id("nav-contacts")
            .action(ComponentAction::click("contact_list"))
            .build(),
    );
    nav_items.push(
        NavItem::new("Companies", "/companies")
            .id("nav-companies")
            .action(ComponentAction::click("company_list"))
            .build(),
    );
    if is_admin {
        nav_items.push(
            NavItem::new("Users", "/users")
                .id("nav-users")
                .action(ComponentAction::click("user_list"))
                .build(),
        );
        nav_items.push(
            NavItem::new("Audit Log", "/audit")
                .id("nav-audit")
                .action(ComponentAction::click("audit_list"))
                .build(),
        );
    }
    let (sidebar_root, sidebar_desc) = SideNav::new()
        .id("shell-side-nav")
        .children(nav_items)
        .build_tree();

    // -- Build the header sub-tree (title + user menu placeholder) --
    // Header = Container with app title Heading + user-menu Heading bound to data path.
    let header_title = Heading::new("Marionette CRM").id("header-title").build();
    let header_user = Heading::new(&format!("User: {}", user_name))
        .id("header-user")
        .build();
    let (header_root, header_desc) = Container::new()
        .id("shell-header")
        .children(vec![header_title, header_user])
        .build_tree();

    // -- Build the footer sub-tree (version + connection status + legal) --
    let footer_version = Heading::new("Marionette v1.1 · Protocol 1.1.0")
        .id("footer-version")
        .build();
    let footer_legal = Heading::new("© 2026 Marionette")
        .id("footer-legal")
        .build();
    let (footer_root, footer_desc) = Container::new()
        .id("shell-footer")
        .children(vec![footer_version, footer_legal])
        .build_tree();

    // -- Build the three sub-surface mounts --
    let (content_mount, _) = SurfaceMount::new("content")
        .id("shell-content-mount")
        .build();
    let (modal_mount, _) = SurfaceMount::new("modal")
        .id("shell-modal-mount")
        .build();
    let (toasts_mount, _) = SurfaceMount::new("toasts")
        .id("shell-toasts-mount")
        .build();

    // -- Assemble the AppShell --
    let mut descendants = Vec::new();
    descendants.extend(sidebar_desc);
    descendants.extend(header_desc);
    descendants.extend(footer_desc);

    let shell_nodes = AppShell::new()
        .id("app-shell-root")
        .sidebar(sidebar_root)
        .header(header_root)
        .footer(footer_root)
        .main((content_mount.0.clone(), content_mount.1))
        .popups((modal_mount.0.clone(), modal_mount.1))
        .toasts((toasts_mount.0.clone(), toasts_mount.1))
        .with_descendants(descendants)
        .build_with_children();

    let mut shell_map = HashMap::new();
    for (id, component) in shell_nodes {
        shell_map.insert(id, component);
    }

    // Initial shell data: auth info lives under /auth/currentUser for future
    // data-bound user menus.
    let shell_data = serde_json::json!({
        "auth": {
            "currentUser": {
                "name": user_name,
                "roles": session.roles,
            }
        },
        "nav": {
            "active": {}
        }
    });

    // -- Compose response messages --
    // Order: shell render first (must exist before content patches/renders
    // can reference its nodes), then the content sub-surface render.
    let mut messages = Vec::new();
    messages.push(ProtocolMessage::Render(RenderMessage {
        id: None,
        surface: "main".into(),
        root: "app-shell-root".into(),
        nodes: shell_map,
        data: shell_data,
    }));
    messages.append(&mut content_messages);

    Ok(messages)
}
```

3. Update `handle_login_action` (currently lines 33-119). The existing line 105 calls `handle_navigate(authenticated_ctx)` and takes the first Render for auth metadata injection. The code at lines 108-116 iterates messages and injects `_auth_user_id` / `_auth_role` into the FIRST Render's data. That first Render is now the shell Render on `main` — that still works. No edits needed here beyond a mental check that the logic still holds. **Confirm by reading the login flow end-to-end after editing handle_navigate.**

4. The existing `build_login_form()` function (lines 196-241) continues to render into surface `"main"` — this is correct per D-B11 (pre-auth login is a full Render, not a patch, and the shell has not been built yet). DO NOT change this function.

5. Delete the unused `use std::collections::HashMap;` if the new code no longer needs it — but it does need it for `shell_map`, so keep the import.

6. Run `cd backend && cargo build -p crm-demo` — EXPECT compile errors if the `handle_contact_list` signature or return type mismatches after Task 2's changes. If Task 2 has already landed, there should be no errors. Otherwise fix forward.

7. Run `cd backend && cargo clippy -p crm-demo -- -D warnings` — must be green.
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo build -p crm-demo 2&gt;&amp;1 | tail -15 &amp;&amp; grep -q 'AppShell::new' backend/crates/crm-demo/src/main.rs &amp;&amp; grep -q 'SurfaceMount::new("content")' backend/crates/crm-demo/src/main.rs</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'AppShell::new()' backend/crates/crm-demo/src/main.rs` succeeds
    - `grep -q 'SurfaceMount::new("content")' backend/crates/crm-demo/src/main.rs` succeeds
    - `grep -q 'SurfaceMount::new("modal")' backend/crates/crm-demo/src/main.rs` succeeds
    - `grep -q 'SurfaceMount::new("toasts")' backend/crates/crm-demo/src/main.rs` succeeds
    - `grep -q 'surface: "sidebar"' backend/crates/crm-demo/src/main.rs` returns zero matches (the separate sidebar Render is gone)
    - `grep -q 'root: "app-shell-root".into()' backend/crates/crm-demo/src/main.rs` succeeds
    - `grep -q '"auth":' backend/crates/crm-demo/src/main.rs` succeeds (shell data contains auth.currentUser)
    - `cd backend && cargo build -p crm-demo` exits 0
    - `cd backend && cargo clippy -p crm-demo -- -D warnings` exits 0
  </acceptance_criteria>
  <done>handle_navigate builds an AppShell into main + delegates screen content to content sub-surface. Sidebar top-level Render is gone. crm-demo compiles cleanly under clippy pedantic.</done>
</task>

<task type="auto">
  <name>Task 2: Rename handler render surfaces from "main" to "content" in all non-auth handlers</name>
  <read_first>
    - backend/crates/crm-demo/src/handlers/contact.rs (lines 444, 904 — search for `surface: "main"`)
    - backend/crates/crm-demo/src/handlers/company.rs (lines 137, 376)
    - backend/crates/crm-demo/src/handlers/user.rs (lines 99, 293)
    - backend/crates/crm-demo/src/handlers/audit.rs (line 209)
    - backend/crates/crm-demo/src/handlers/interaction.rs (line 148)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pitfall 6
  </read_first>
  <action>
1. Verify the complete set of `surface: "main"` occurrences:
   ```bash
   grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/
   ```
   Expected (from pre-research grep):
   - `main.rs:236` (inside `build_login_form`) — **KEEP** as-is (pre-auth login surface)
   - `main.rs:~220` (inside the new `handle_navigate` shell Render from Task 1) — **KEEP** as-is (shell surface is main)
   - `handlers/audit.rs:209`
   - `handlers/contact.rs:444, 904`
   - `handlers/interaction.rs:148`
   - `handlers/company.rs:137, 376`
   - `handlers/user.rs:99, 293`

2. Edit each listed handler file. At the `surface: "main".into()` line, change to `surface: "content".into()`. Leave the rest of the `RenderMessage` construction (id, root, nodes, data) untouched.

   Per-file edits:
   - `handlers/contact.rs` line 444 and line 904 — both `surface: "main"` → `surface: "content"`
   - `handlers/company.rs` line 137 and line 376 — both `surface: "main"` → `surface: "content"`
   - `handlers/user.rs` line 99 and line 293 — both `surface: "main"` → `surface: "content"`
   - `handlers/audit.rs` line 209 — `surface: "main"` → `surface: "content"`
   - `handlers/interaction.rs` line 148 — `surface: "main"` → `surface: "content"`

3. Check for handlers NOT in the initial grep that may have grown new Render sites:
   ```bash
   grep -rn 'RenderMessage {' backend/crates/crm-demo/src/handlers/ | grep -v test
   ```
   For each match, confirm the `surface:` field. If it says `"main"`, change to `"content"`. If it already says something else (`"modal"`, `"content"`), leave it. The `listmonk.rs` and `note.rs` handlers should be inspected — if they render, they render into `"content"`.

4. Verify the final state. Only two Rust source lines in the workspace should construct `surface: "main"`:
   - The shell Render inside `handle_navigate` in `main.rs` (Task 1)
   - `build_login_form` in `main.rs` line ~236

   ```bash
   grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/
   ```
   Must return exactly those two lines (plus possibly comments). If any handler file still returns "main", it is a bug.

5. Run `cd backend && cargo build -p crm-demo` — must be green.

6. Run `cd backend && cargo test --workspace` — all existing handler tests must still pass. If any test asserts `surface == "main"` on a handler that now returns "content", update the assertion to `"content"`.

7. Run `cd backend && cargo clippy --workspace -- -D warnings` — must be green.
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo build -p crm-demo &amp;&amp; grep -rn 'surface:\s*"main"' crates/crm-demo/src/handlers/ ; test $? -eq 1 &amp;&amp; cargo test --workspace 2&gt;&amp;1 | tail -5</automated>
  </verify>
  <acceptance_criteria>
    - `grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/handlers/` returns zero lines (NO handler renders to main)
    - `grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/` returns at most 2 lines (both in `main.rs`)
    - `grep -rc 'surface:\s*"content"' backend/crates/crm-demo/src/handlers/` returns at least 8 (matching the 8 sites listed in RESEARCH Finding 3)
    - `cd backend && cargo build -p crm-demo` exits 0
    - `cd backend && cargo test --workspace` exits 0
    - `cd backend && cargo clippy --workspace -- -D warnings` exits 0
  </acceptance_criteria>
  <done>All 8 non-auth handler render sites rewritten to surface "content". Only main.rs (shell + login) still targets "main". Workspace tests and clippy pedantic green.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Interactive verification — CRM boots into AppShell with working navigation</name>
  <files>backend/crates/crm-demo/src/main.rs, frontend/src/routes/+layout.svelte</files>
  <read_first>
    - backend/crates/crm-demo/src/main.rs (handle_navigate as rewritten by Task 1)
    - frontend/src/lib/components/shell/AppShell.svelte (as implemented in Plan 06)
  </read_first>
  <what-built>
    Plan 05 + Plan 06 + Tasks 1-2 of Plan 07 combined: CRM backend builds AppShell, frontend renders it with shadcn Sidebar, navigation between screens updates `content` sub-surface without replacing the shell.
  </what-built>
  <action>
Run a live end-to-end verification session. Walk through the 10 numbered steps below, noting any failures. This is a blocking human checkpoint — executor pauses for user confirmation before Wave 4 can start.
  </action>
  <how-to-verify>
1. Run the backend: `cd backend && cargo run -p crm-demo` (or via `make dev` if that convenience exists).
2. In a separate terminal, run the frontend dev server: `cd frontend && pnpm dev` (or `npm run dev`).
3. Open `http://localhost:5173` (or whatever port the dev server prints) in a browser.
4. **Login flow**: the login form should still appear on `main` with zero visual changes vs. Phase 11. Log in with `admin@example.com` / the seeded password.
5. **After login**: the page should replace the login form with the AppShell. You should see:
   - A collapsible sidebar on the left (desktop viewport)
   - A header at the top containing the Sidebar trigger (hamburger visible at narrow widths), "Marionette CRM" title, and "User: <your id>" text
   - The contact list as the main content area
   - A footer at the bottom showing "Marionette v1.1 · Protocol 1.1.0" and copyright
6. **Navigation**: click the "Companies" nav item in the sidebar. Expected behavior:
   - The main content area updates to show the company list
   - The sidebar, header, and footer DO NOT re-render (visually no flicker)
   - The browser URL (if routed) updates
7. Repeat for "Contacts", "Users", "Audit Log". Each navigation should update only the content area.
8. **Mobile check**: resize the browser to < 768px width. The sidebar should collapse into a sheet. Click the hamburger (Sidebar.Trigger) in the header — the sheet slides in from the left.
9. **DevTools check**: open the WebSocket frames tab. Observe that clicking a nav item produces exactly ONE `render` message with `surface: "content"`. No `render` with `surface: "main"` after the initial login response.
10. Open the browser console. There should be NO errors. Warnings about missing `children?` snippet on some components are acceptable if they pre-dated Phase 12.

Confirm each of the 10 steps above. If any step fails, abort this checkpoint and return to Task 1 or Task 2 for diagnosis. The most likely failure mode is a `surface-mount` node referencing a sub-surface that was never rendered to — check the WebSocket frames and fix the handler.
  </how-to-verify>
  <resume-signal>Type "approved" if all 10 steps pass, or describe which step failed and what the symptom was.</resume-signal>
  <verify>
    <automated>echo "manual checkpoint — user confirms steps 1-10 visually; no automated command"</automated>
  </verify>
  <acceptance_criteria>
    - All 10 verification steps confirmed pass by the user
    - No console errors in the browser at any step
    - WebSocket frame inspection confirms nav clicks produce only `surface: "content"` renders after the initial shell render
    - User types "approved" to resume
  </acceptance_criteria>
  <done>User confirms the CRM boots into AppShell end-to-end with working navigation, mobile sidebar sheet, and no content-area flicker. Any failures trigger a return to Task 1 or Task 2 for fixes.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| login action → authenticated ctx | `handle_login_action` verifies bcrypt and constructs an authenticated `HandlerContext` that `handle_navigate` trusts |
| session role → nav items | `is_admin` check in `handle_navigate` gates admin-only nav entries (Users, Audit Log) |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-16 | Elevation of Privilege | A non-admin user could see admin nav items if `is_admin` check fails | mitigate | `session.roles.contains(&"admin".to_string())` is the existing check — unchanged from Phase 11. Server-side auth on the actual action handlers (`user_list`, `audit_list`) is enforced by `AuthRequirement::Role("admin")` in `action_router` (main.rs:342, 367), so even if the UI showed the link, the handler rejects the request. |
| T-12-17 | Information Disclosure | Header displays `session.user_id` as user name — if the id is an email, it is leaked into the DOM | accept | The existing auth flow uses an opaque user_id. Svelte escapes the value by default. This is no worse than any other user-bound data. A future SHELL-05 could migrate to a proper display-name field. |
| T-12-18 | Tampering | A non-auth client could send `navigate` action to bypass login | mitigate | `action_router.action("navigate", ..., AuthRequirement::Authenticated)` in main.rs:311 blocks unauthenticated navigate actions. No change. |
</threat_model>

<verification>
- `cd backend && cargo build -p crm-demo` exits 0
- `cd backend && cargo test --workspace` exits 0
- `cd backend && cargo clippy --workspace -- -D warnings` exits 0
- `grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/handlers/` returns zero lines
- `grep -c 'AppShell::new()' backend/crates/crm-demo/src/main.rs` ≥ 1
- Human verification checkpoint (Task 3) confirms end-to-end shell rendering and navigation
</verification>

<success_criteria>
- `handle_navigate` in main.rs builds an AppShell via `AppShell::new()` with all 6 slot methods populated
- The shell is rendered into surface `"main"`; the initial content (contact list) is rendered into surface `"content"`
- Sidebar nav items route via `click` actions (navigate / contact_list / company_list / user_list / audit_list), matching Phase 11 Action conventions
- Admin-only nav items (Users, Audit) are gated by `is_admin`
- All non-auth handler files render to surface `"content"` (8 sites migrated)
- `main.rs` and `build_login_form` are the only remaining Rust sources that construct `surface: "main"`
- Workspace tests and clippy pedantic are green
- Interactive verification checkpoint passes: login → shell + contact list; nav click → content update only; mobile hamburger works
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-07-SUMMARY.md` recording:
- Exact list of handler files + line numbers where `surface: "main"` → `"content"` was applied
- Which nav items are gated by `is_admin`
- Whether the `_auth_user_id` / `_auth_role` injection in `handle_login_action` still works with the shell Render as the first message
- Screenshot filename (if any) from the checkpoint verification
</output>
