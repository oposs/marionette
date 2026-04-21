---
phase: 12
plan: 08
type: execute
wave: 5
depends_on: [12-03, 12-04, 12-07]
files_modified:
  - backend/crates/crm-demo/src/handlers/contact.rs
  - frontend/tests/e2e/node-patch-focus.spec.ts
  - frontend/tests/e2e/shell-nav.spec.ts
  - frontend/tests/e2e/protocol-conformance.spec.ts
autonomous: true
requirements: [PATCH-01, PATCH-02, PATCH-03, SHELL-01, SHELL-02, SHELL-04]
nyquist_compliant: true
tags: [crm, demo, e2e, playwright, focus-preservation]
must_haves:
  truths:
    - "PatchMessage tagged enum + surface field exist end-to-end (Rust crate, TS types, YAML schemas, spec docs)"
    - "Frontend surface store applies node patches reactively; focused text inputs retain focus+cursor across sibling patches"
    - "HelloMessage reports protocol version 1.1.0"
    - "CONCEPT.md's 'easy to patch by node ID' claim matches the implemented protocol"
    - "AppShell renders with collapsible sidebar, header, footer, main content area via shadcn Sidebar primitives"
    - "AppShell is a normal SDUI component (registered in defaults.ts, hand-written builder in backend/crates/marionette/src/builders/)"
    - "CRM app runs inside AppShell with working nav between screens"
    - "Contact form includes a country-select field that triggers a node-patch flow swapping sibling fields in place with preserved focus"
    - "Protocol conformance E2E validates live wire messages against updated schemas"
    - "Toast lifecycle (D-B15) demonstrated end-to-end: country-change handler emits insert-child on the toasts sub-surface root adding a small Heading node; a dismiss_toast action handler emits delete-node removing it. E2E test asserts the toast becomes visible, then dismissal makes it hidden."
  artifacts:
    - path: "backend/crates/marionette-protocol/src/data.rs"
      provides: "tagged PatchOperation enum with 6 variants"
      contains: "enum PatchOperation"
    - path: "backend/crates/marionette-protocol/src/messages.rs"
      provides: "PatchMessage with required surface field"
      contains: "pub surface: Surface"
    - path: "backend/crates/marionette/src/ws.rs"
      provides: "HelloMessage 1.1.0 emission"
      contains: "1.1.0"
    - path: "spec/schemas/data.yaml"
      provides: "PatchOperation oneOf with discriminator"
      contains: "propertyName: op"
    - path: "spec/schemas/message.yaml"
      provides: "PatchMessage with required surface"
      contains: "surface"
    - path: "spec/PROTOCOL.md"
      provides: "node-patch semantics documented; version 1.1.0"
      contains: "set-node"
    - path: "CONCEPT.md"
      provides: "reconciled patch-by-node-ID claim"
    - path: "frontend/src/lib/transport/messages.ts"
      provides: "PatchOperation union + PatchMessage.surface"
      contains: "op: 'set-node'"
    - path: "frontend/src/lib/init.ts"
      provides: "applyPatch(msg.surface, msg.patch)"
      contains: "applyPatch(msg.surface"
    - path: "frontend/src/lib/store/surfaces.svelte.ts"
      provides: "fine-grained setNode / deleteNode / setChildren / insertChild / removeChild / gcOrphans"
      exports: ["setNode", "deleteNode", "setChildren", "insertChild", "removeChild", "gcOrphans"]
    - path: "frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts"
      provides: "D-A6 focus-preservation proof"
      contains: "selectionStart"
    - path: "frontend/src/lib/components/shell/AppShell.svelte"
      provides: "AppShell component using shadcn Sidebar primitives"
      contains: "Sidebar.Provider"
    - path: "frontend/src/lib/components/core/SurfaceMount.svelte"
      provides: "SurfaceMount component"
      contains: "<Surface name="
    - path: "frontend/src/lib/registry/defaults.ts"
      provides: "'app-shell' and 'surface-mount' registered"
      contains: "'app-shell'"
    - path: "frontend/src/routes/+layout.svelte"
      provides: "collapsed single-surface root"
      contains: "Surface name=\"main\""
    - path: "backend/crates/marionette/src/builders/app_shell.rs"
      provides: "hand-written AppShell builder with 6 slot methods"
      contains: "pub struct AppShellBuilder"
    - path: "backend/crates/marionette/src/builders/standard.rs"
      provides: "SurfaceMount derived builder"
      contains: "struct SurfaceMount"
    - path: "backend/crates/crm-demo/src/main.rs"
      provides: "handle_navigate builds AppShell into main + renders screen into content"
      contains: "AppShell::new()"
    - path: "frontend/tests/e2e/node-patch-focus.spec.ts"
      provides: "country-select E2E focus-preservation demo"
      contains: "selectionStart"
    - path: "frontend/tests/e2e/shell-nav.spec.ts"
      provides: "E2E shell nav flow"
      contains: "content"
  key_links:
    - from: "contact form country select"
      to: "contact_country_change handler"
      via: "click action sending PatchMessage with insert-child + set-node"
      pattern: "contact_country_change"
    - from: "node-patch-focus E2E"
      to: "backend country-change handler"
      via: "WebSocket PatchMessage with mixed data + tree ops"
      pattern: "selectionStart"
    - from: "protocol-conformance E2E"
      to: "ajv + data.yaml oneOf discriminator"
      via: "live wire message validation"
      pattern: "set-node"
    - from: "country-change handler"
      to: "toasts sub-surface"
      via: "second PatchMessage with InsertChild + SetNode targeting surface 'toasts'"
      pattern: "surface:\\s*\"toasts\""
    - from: "dismiss_toast action"
      to: "delete-node on toasts sub-surface"
      via: "PatchMessage with DeleteNode + RemoveChild targeting surface 'toasts'"
      pattern: "dismiss_toast"
---

<objective>
Deliver the final Phase 12 piece: an interactive country-select demo on the CRM contact form that proves node-level mutation with focus preservation end-to-end, plus E2E tests that assert (a) shell navigation, (b) the country-select focus-preservation flow against a real backend, and (c) protocol-conformance schema validation of the new PatchMessage shape. This plan is the goal-backward gate for the entire phase.

Purpose: Success criterion 8 ("at least one interactive flow demonstrates node-level mutation end-to-end") cannot be closed without this demo. Success criteria 1-7 are satisfied by Plans 02-07 but need the three E2E specs here to prove they all compose correctly under live WebSocket traffic.

Output: Working country-select field-swap demo on contact form, 3 passing Playwright E2E specs, all Phase 12 must-haves verified.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@backend/crates/crm-demo/src/handlers/contact.rs
@backend/crates/crm-demo/src/main.rs
@frontend/tests/e2e/node-patch-focus.spec.ts
@frontend/tests/e2e/shell-nav.spec.ts
@frontend/tests/e2e/protocol-conformance.spec.ts
@frontend/tests/helpers/schema-validator.ts
@frontend/playwright.e2e.config.ts

<interfaces>
The `contact.rs` file already has a `handle_contact_form` function (line 457) that builds a contact create/edit form. This plan extends that form to include a `country` Select field whose change triggers a new action `contact_country_change`.

The existing `action_router` in `main.rs:304-434` has a pattern for registering actions: `.action("name", box_handler(handler), AuthRequirement::Authenticated)`. Add `contact_country_change` following this pattern.

The new handler returns a `ProtocolMessage::Patch(PatchMessage { ... })` with a mix of:
- `PatchOperation::Set { path: "/contact/country", value }` — confirms the new country value in data
- `PatchOperation::DeleteNode { id }` — removes any previously-inserted country-specific fields
- `PatchOperation::SetNode { id, component }` — defines the new country-specific field(s)
- `PatchOperation::InsertChild { parent: "contact-form", index, childId }` — inserts them into the form's children

Plan 07's handler migration already moved `handle_contact_form` to render into surface `"content"`. The patch in this plan targets `surface: "content"` too.

Existing `protocol-conformance.spec.ts` (to be extended):
- Loads `spec/schemas/*.yaml` via `schema-validator.ts`
- Captures WebSocket frames via `ws-capture.ts`
- Asserts each frame matches its schema

Plan 08 adds new cases: a patch-message capture that uses each of the 5 node op variants at least once.

`frontend/playwright.e2e.config.ts` exists (verified via `ls frontend/playwright.e2e.config.ts`). It runs Playwright against the real backend on port 3001.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add country-select field + contact_country_change handler to the CRM contact form</name>
  <read_first>
    - backend/crates/crm-demo/src/handlers/contact.rs (full file; note handle_contact_form function)
    - backend/crates/crm-demo/src/main.rs (action_router registration pattern around lines 304-434)
    - backend/crates/marionette/src/builders/standard.rs (Select builder with SelectOption)
    - backend/crates/marionette-protocol/src/data.rs (PatchOperation variants)
    - backend/crates/marionette-protocol/src/messages.rs (PatchMessage shape)
  </read_first>
  <action>
1. In `backend/crates/crm-demo/src/handlers/contact.rs`, locate `handle_contact_form` (line 457+). Find the section that builds the form's fields (Name, Email, Phone, Company select). Add a new `country` Select field between "Phone" and "Company" (or at whatever position makes sense — order does not matter for the demo):

```rust
let country_select = Select::new(
    "Country",
    vec![
        SelectOption { value: "".into(), label: "Select...".into() },
        SelectOption { value: "CH".into(), label: "Switzerland".into() },
        SelectOption { value: "US".into(), label: "United States".into() },
        SelectOption { value: "DE".into(), label: "Germany".into() },
    ],
)
.id("contact-country")
.bind("/contact/country")
.action(ComponentAction::change("contact_country_change"))
.build();
```

Add `country_select` to the `Vec<(String, Component)>` passed to `Form::new().children(...)` for the contact form. The form node id is the parent for subsequent node patches — confirm its id is `contact-form` (or update the handler code to use that id if different).

In the form's initial data, include `"contact": { ..., "country": "" }` so the bind resolves without crashing.

2. Add a new handler function `handle_contact_country_change` at the bottom of `contact.rs`. It reads the new country value from the action payload or data, computes the patch, and returns a `PatchMessage`:

```rust
/// Handle the contact form's country-select change: swap country-specific
/// fields in place via node patches (D-A6 focus-preservation demo).
pub async fn handle_contact_country_change(ctx: HandlerContext) -> ActionResult {
    use marionette::builders::standard::{Select, SelectOption, TextInput};
    use marionette_protocol::{PatchMessage, PatchOperation};

    // Extract the new country from payload (the SelectInput's change action
    // carries the new value under /contact/country in the payload).
    let payload = ctx.action.payload.clone().unwrap_or_default();
    let country = payload
        .get("contact")
        .and_then(|v| v.get("country"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Build the ops batch:
    // 1. set data path to confirm the new value (the frontend already did
    //    this optimistically, but the server authoritative write follows)
    // 2. delete any previously-inserted country-specific fields
    // 3. insert new country-specific fields (if any) below the country select
    let mut ops: Vec<PatchOperation> = Vec::new();

    ops.push(PatchOperation::Set {
        path: "/contact/country".into(),
        value: serde_json::json!(country),
    });

    // Remove any previously-inserted country-specific nodes by fixed IDs.
    // (For this demo we always delete all three candidates — deleting a
    // non-existent node is a no-op in the frontend store.)
    for id in ["contact-ch-canton", "contact-us-state", "contact-de-bundesland"] {
        ops.push(PatchOperation::RemoveChild {
            parent: "contact-form".into(),
            child_id: id.into(),
        });
        ops.push(PatchOperation::DeleteNode { id: id.into() });
    }

    // Insert new fields based on country.
    match country.as_str() {
        "CH" => {
            let canton = Select::new(
                "Canton",
                vec![
                    SelectOption { value: "ZH".into(), label: "Zürich".into() },
                    SelectOption { value: "BE".into(), label: "Bern".into() },
                    SelectOption { value: "GE".into(), label: "Geneva".into() },
                ],
            )
            .id("contact-ch-canton")
            .bind("/contact/canton")
            .build();
            ops.push(PatchOperation::SetNode {
                id: "contact-ch-canton".into(),
                component: canton.1,
            });
            ops.push(PatchOperation::InsertChild {
                parent: "contact-form".into(),
                index: 5, // position after Country select — adjust if form layout differs
                child_id: "contact-ch-canton".into(),
            });
        }
        "US" => {
            let state = TextInput::new("State")
                .id("contact-us-state")
                .bind("/contact/usState")
                .build();
            ops.push(PatchOperation::SetNode {
                id: "contact-us-state".into(),
                component: state.1,
            });
            ops.push(PatchOperation::InsertChild {
                parent: "contact-form".into(),
                index: 5,
                child_id: "contact-us-state".into(),
            });
        }
        "DE" => {
            let bundesland = TextInput::new("Bundesland")
                .id("contact-de-bundesland")
                .bind("/contact/bundesland")
                .build();
            ops.push(PatchOperation::SetNode {
                id: "contact-de-bundesland".into(),
                component: bundesland.1,
            });
            ops.push(PatchOperation::InsertChild {
                parent: "contact-form".into(),
                index: 5,
                child_id: "contact-de-bundesland".into(),
            });
        }
        _ => {}
    }

    // Build the primary patch on the content surface (the country-swap).
    let content_patch = marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "content".into(),
        patch: ops,
    });

    // D-B15 toast lifecycle demo: on every country change, insert a toast
    // notification into the `toasts` sub-surface root announcing the change.
    // The toast is a small Heading node with a dismiss action that triggers
    // `dismiss_toast`, which will emit delete-node to remove it. This proves
    // the `insert-child` / `delete-node` operations work on a sub-surface
    // other than `content` — closing success criterion 8 for Phase 12.
    use marionette_protocol::Component;
    let mut toast_props = serde_json::Map::new();
    toast_props.insert(
        "text".into(),
        serde_json::json!(format!("Country set to {}", match country.as_str() {
            "CH" => "Switzerland",
            "US" => "United States",
            "DE" => "Germany",
            _ => "none",
        })),
    );
    toast_props.insert("level".into(), serde_json::json!(6));
    let toast_node = Component {
        component_type: "heading".into(),
        id: Some("toast-country-change".into()),
        props: serde_json::Value::Object(toast_props),
        bind: None,
        action: Some(marionette_protocol::ComponentAction::click("dismiss_toast")),
        children: None,
    };
    let toasts_ops: Vec<PatchOperation> = vec![
        // Clean up any previously-inserted toast with the same id (idempotent)
        PatchOperation::RemoveChild {
            parent: "toasts-root".into(),
            child_id: "toast-country-change".into(),
        },
        PatchOperation::DeleteNode {
            id: "toast-country-change".into(),
        },
        // Insert the new toast node
        PatchOperation::SetNode {
            id: "toast-country-change".into(),
            component: toast_node,
        },
        PatchOperation::InsertChild {
            parent: "toasts-root".into(),
            index: 0,
            child_id: "toast-country-change".into(),
        },
    ];
    let toasts_patch = marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: None,
        surface: "toasts".into(),
        patch: toasts_ops,
    });

    Ok(vec![content_patch, toasts_patch])
}

/// D-B15 dismiss_toast handler: removes the toast node with the id carried
/// in the action payload (or the fixed "toast-country-change" id if not
/// supplied). Proves that `delete-node` works on the toasts sub-surface.
pub async fn handle_dismiss_toast(ctx: HandlerContext) -> ActionResult {
    use marionette_protocol::{PatchMessage, PatchOperation};
    let payload = ctx.action.payload.clone().unwrap_or_default();
    let toast_id = payload
        .get("toastId")
        .and_then(|v| v.as_str())
        .unwrap_or("toast-country-change")
        .to_string();

    let ops = vec![
        PatchOperation::RemoveChild {
            parent: "toasts-root".into(),
            child_id: toast_id.clone(),
        },
        PatchOperation::DeleteNode { id: toast_id },
    ];

    Ok(vec![marionette_protocol::ProtocolMessage::Patch(PatchMessage {
        id: ctx.action.id.clone(),
        surface: "toasts".into(),
        patch: ops,
    })])
}
```

**Toasts sub-surface initialization**: the toasts sub-surface needs an initial `RenderMessage` (with an empty Container root id `toasts-root`) before the first `insert-child` op can reference it. Plan 07 Task 1 constructs a `SurfaceMount::new("toasts")` node but does NOT Render into the `toasts` surface. Add this seeding to Plan 07's `handle_navigate` OR here inside `handle_contact_country_change` (idempotent check for whether the surface has been rendered to before). **Simplest option**: at the end of `handle_navigate` in `main.rs`, append a third Render message seeding the `toasts` sub-surface with an empty Container node `toasts-root` as its root. Add this in Task 1 here as a follow-up edit to `main.rs`:

```rust
// Append to handle_navigate in main.rs BEFORE the final Ok(messages) —
// seed the toasts sub-surface with an empty container so InsertChild
// has a parent to target. D-B15 gate.
let (toasts_root_id, toasts_container) = Container::new()
    .id("toasts-root")
    .children::<Vec<(String, marionette_protocol::Component)>>(vec![])
    .build_tree();
let mut toasts_nodes = HashMap::new();
for (id, c) in toasts_container {
    toasts_nodes.insert(id, c);
}
messages.push(ProtocolMessage::Render(RenderMessage {
    id: None,
    surface: "toasts".into(),
    root: toasts_root_id,
    nodes: toasts_nodes,
    data: serde_json::json!({}),
}));
```

The Render order (shell → content → toasts) is: shell first so SurfaceMount nodes exist, content next so the landing screen appears, toasts last so the empty region is ready to receive insert-child. Order matters: the frontend applies messages sequentially.

Note: the `ComponentAction::change` helper — if it does not exist in the project, use `ComponentAction::click` (the SelectInput on the frontend may dispatch actions on change via a different constructor). Inspect `backend/crates/marionette-protocol/src/component.rs` for the `ComponentAction` helpers and pick the correct one. If only `click` / `submit` exist, add a new `change` constructor following the same pattern.

3. Register BOTH new handlers in `backend/crates/crm-demo/src/main.rs` `action_router` chain:

```rust
.action(
    "contact_country_change",
    box_handler(handlers::contact::handle_contact_country_change),
    AuthRequirement::Authenticated,
)
.action(
    "dismiss_toast",
    box_handler(handlers::contact::handle_dismiss_toast),
    AuthRequirement::Authenticated,
)
```

4. Run `cd backend && cargo build -p crm-demo` — must be green.
5. Run `cd backend && cargo test --workspace` — must be green.
6. Run `cd backend && cargo clippy --workspace -- -D warnings` — must be green.
  </action>
  <verify>
    <automated>cd backend &amp;&amp; cargo build -p crm-demo &amp;&amp; grep -q 'handle_contact_country_change' crates/crm-demo/src/handlers/contact.rs &amp;&amp; grep -q 'contact_country_change' crates/crm-demo/src/main.rs &amp;&amp; cargo clippy --workspace -- -D warnings 2&gt;&amp;1 | tail -5</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'id("contact-country")' backend/crates/crm-demo/src/handlers/contact.rs` succeeds
    - `grep -q 'handle_contact_country_change' backend/crates/crm-demo/src/handlers/contact.rs` succeeds
    - `grep -q 'handle_dismiss_toast' backend/crates/crm-demo/src/handlers/contact.rs` succeeds (D-B15)
    - `grep -q '"contact_country_change"' backend/crates/crm-demo/src/main.rs` succeeds
    - `grep -q '"dismiss_toast"' backend/crates/crm-demo/src/main.rs` succeeds (D-B15 action registered)
    - `grep -q 'PatchOperation::InsertChild' backend/crates/crm-demo/src/handlers/contact.rs` succeeds
    - `grep -q 'PatchOperation::SetNode' backend/crates/crm-demo/src/handlers/contact.rs` succeeds
    - `grep -q 'PatchOperation::RemoveChild' backend/crates/crm-demo/src/handlers/contact.rs` succeeds
    - `grep -q 'PatchOperation::DeleteNode' backend/crates/crm-demo/src/handlers/contact.rs` succeeds (D-B15 lifecycle)
    - `grep -q 'surface: "toasts"' backend/crates/crm-demo/src/handlers/contact.rs` succeeds (D-B15 toast patches target the toasts sub-surface)
    - `grep -q 'toasts-root' backend/crates/crm-demo/src/main.rs` succeeds (toasts sub-surface seeded in handle_navigate)
    - `grep -q 'surface: "toasts"' backend/crates/crm-demo/src/main.rs` succeeds (the toasts Render in handle_navigate)
    - `cd backend && cargo build -p crm-demo` exits 0
    - `cd backend && cargo clippy --workspace -- -D warnings` exits 0
    - `cd backend && cargo test --workspace` exits 0
  </acceptance_criteria>
  <done>Contact form has a Country select; changing it triggers TWO PatchMessages: (a) mixed Set/SetNode/InsertChild/RemoveChild/DeleteNode ops on surface "content" swapping country-specific fields, and (b) InsertChild + SetNode on surface "toasts" adding a dismissable notification (D-B15). `handle_navigate` seeds the toasts sub-surface with an empty `toasts-root` Container so InsertChild has a parent. `handle_dismiss_toast` emits DeleteNode + RemoveChild on toasts to close the lifecycle. Workspace compiles cleanly.</done>
</task>

<task type="auto">
  <name>Task 2: Write node-patch-focus.spec.ts E2E test — country swap with preserved focus</name>
  <read_first>
    - frontend/tests/e2e/node-patch-focus.spec.ts (scaffold from Plan 01)
    - frontend/tests/e2e/integration.spec.ts (existing pattern — Playwright with real backend)
    - frontend/tests/e2e/smoke.spec.ts (login helper pattern)
    - frontend/playwright.e2e.config.ts (base URL, port, auth)
    - .planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md §specifics (concrete demo scenario)
  </read_first>
  <action>
REPLACE the scaffold contents of `frontend/tests/e2e/node-patch-focus.spec.ts` with a real Playwright E2E test that exercises the full stack:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Phase 12: node patch + focus preservation end-to-end', () => {
	test('country-select change swaps sibling fields and preserves focus on Name', async ({ page }) => {
		// Start from the login page
		await page.goto('/');

		// Log in with seeded admin credentials (adjust if smoke.spec.ts uses different ones)
		await page.getByLabel('Email').fill('admin@example.com');
		await page.getByLabel('Password').fill('admin');
		await page.getByRole('button', { name: /log in/i }).click();

		// After login, AppShell should render with contact list; navigate to New Contact
		// (the exact nav label depends on the CRM; accept either a "New" button or a
		// direct link to /contacts/new). The test uses the contact list's "New" button
		// as the entry to the form.
		await page.getByRole('button', { name: /new contact|add contact|new/i }).first().click();

		// Focus the Name field and type "Hello", cursor at position 3
		const nameField = page.getByLabel('Name');
		await nameField.focus();
		await nameField.fill('Hello');

		// Move cursor to position 3 programmatically
		await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			if (el && 'setSelectionRange' in el) {
				el.setSelectionRange(3, 3);
			}
		});

		// Sanity: Name field is focused
		const isFocused = await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			return el?.getAttribute('aria-label')?.toLowerCase().includes('name') ||
				el?.closest('label')?.textContent?.toLowerCase().includes('name');
		});
		// The focus assertion may be loose depending on label wiring; continue even if
		// the sanity check is fuzzy — the real assertion is cursor preservation below.

		// Change the Country select to "Switzerland". The change action triggers
		// contact_country_change which returns a PatchMessage with insert-child +
		// set-node ops that add a "Canton" select below Country.
		await page.getByLabel('Country').selectOption({ label: 'Switzerland' });

		// Wait for the patch to apply — the new Canton field should appear
		await expect(page.getByLabel('Canton')).toBeVisible({ timeout: 5000 });

		// CRITICAL ASSERTION: the Name field is STILL focused and the cursor is
		// STILL at position 3, AND the typed value is unchanged.
		const result = await page.evaluate(() => {
			const active = document.activeElement as HTMLInputElement | null;
			if (!active || active.tagName !== 'INPUT') {
				return { focused: false, value: null, cursor: null, activeTag: active?.tagName };
			}
			return {
				focused: true,
				value: active.value,
				cursor: active.selectionStart,
				activeTag: active.tagName,
			};
		});

		expect(result.focused).toBe(true);
		expect(result.value).toBe('Hello');
		expect(result.cursor).toBe(3);
	});

	test('switching country from Switzerland to United States swaps Canton → State', async ({ page }) => {
		await page.goto('/');
		await page.getByLabel('Email').fill('admin@example.com');
		await page.getByLabel('Password').fill('admin');
		await page.getByRole('button', { name: /log in/i }).click();
		await page.getByRole('button', { name: /new contact|add contact|new/i }).first().click();

		await page.getByLabel('Country').selectOption({ label: 'Switzerland' });
		await expect(page.getByLabel('Canton')).toBeVisible({ timeout: 5000 });

		await page.getByLabel('Country').selectOption({ label: 'United States' });
		await expect(page.getByLabel('State')).toBeVisible({ timeout: 5000 });
		await expect(page.getByLabel('Canton')).not.toBeVisible();
	});

	test('D-B15 toast lifecycle: country change inserts a toast; clicking it dismisses (delete-node)', async ({ page }) => {
		// This test proves `insert-child` / `delete-node` ops work on the
		// `toasts` sub-surface end-to-end. It complements the content-surface
		// node patching proved by the other two tests by exercising a
		// different sub-surface target.
		await page.goto('/');
		await page.getByLabel('Email').fill('admin@example.com');
		await page.getByLabel('Password').fill('admin');
		await page.getByRole('button', { name: /log in/i }).click();
		await page.getByRole('button', { name: /new contact|add contact|new/i }).first().click();

		// Trigger the country change — backend sends TWO PatchMessages:
		// 1) mixed ops on 'content' swapping Canton field in
		// 2) insert-child + set-node on 'toasts' adding a dismissable toast
		await page.getByLabel('Country').selectOption({ label: 'Switzerland' });

		// Assert the toast node text is visible (D-B15 insert-child proven)
		await expect(page.getByText('Country set to Switzerland')).toBeVisible({ timeout: 5000 });

		// Click the toast to trigger the dismiss_toast action — backend
		// responds with a PatchMessage on 'toasts' containing delete-node
		// + remove-child ops that remove the toast node.
		await page.getByText('Country set to Switzerland').click();

		// Assert the toast disappears (D-B15 delete-node proven)
		await expect(page.getByText('Country set to Switzerland')).toBeHidden({ timeout: 5000 });
	});
});
```

Remove the `test.skip` placeholder from Plan 01. Run:

```bash
cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/node-patch-focus.spec.ts 2>&1 | tail -30
```

Both tests must pass. If the selector `getByLabel('Name')` does not find the field, inspect the actual label text in the rendered contact form and adjust. If `getByRole('button', { name: /new contact/i })` does not match, inspect the contact list's actual button label and update.

If the focus assertion fails, the root cause is almost certainly in Plan 04's store implementation — `setNode` must be mutating in place. Re-read `surfaces.svelte.ts` and confirm it does NOT reassign `tree.nodes` or create a new `{...tree.nodes}` spread.

If the focus assertion fails because the WebSocket is still using the old patch shape, confirm Plan 02 and Plan 04 both committed the new shape (grep `surface: String` in Rust, grep `op: 'set'` in TS).
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npx playwright test --config playwright.e2e.config.ts tests/e2e/node-patch-focus.spec.ts 2&gt;&amp;1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'test.skip' frontend/tests/e2e/node-patch-focus.spec.ts` returns no match (scaffold replaced)
    - `grep -q 'selectionStart' frontend/tests/e2e/node-patch-focus.spec.ts` succeeds
    - `grep -q "selectOption.*Switzerland" frontend/tests/e2e/node-patch-focus.spec.ts` succeeds
    - `grep -q "selectOption.*United States" frontend/tests/e2e/node-patch-focus.spec.ts` succeeds
    - `grep -q "Country set to Switzerland" frontend/tests/e2e/node-patch-focus.spec.ts` succeeds (D-B15 toast text assertion)
    - `grep -q "toBeHidden" frontend/tests/e2e/node-patch-focus.spec.ts` succeeds (D-B15 dismiss assertion)
    - `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/node-patch-focus.spec.ts` exits 0 with 3 passing tests (focus-preservation, country swap, D-B15 toast lifecycle)
  </acceptance_criteria>
  <done>End-to-end focus-preservation test passes against a real backend. The country-select flow drives a PatchMessage with mixed ops and the Name field's focus and cursor position survive it. The D-B15 toast lifecycle test proves insert-child + delete-node round-trip on the toasts sub-surface. All 3 tests in node-patch-focus.spec.ts pass.</done>
</task>

<task type="auto">
  <name>Task 3: Write shell-nav.spec.ts E2E + extend protocol-conformance.spec.ts for node ops</name>
  <read_first>
    - frontend/tests/e2e/shell-nav.spec.ts (scaffold from Plan 01)
    - frontend/tests/e2e/protocol-conformance.spec.ts (existing pattern)
    - frontend/tests/helpers/schema-validator.ts
    - frontend/tests/helpers/ws-capture.ts
  </read_first>
  <action>
1. REPLACE the scaffold contents of `frontend/tests/e2e/shell-nav.spec.ts` with:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Phase 12: AppShell nav end-to-end', () => {
	test('login → AppShell renders with sidebar/header/main/footer; nav updates content sub-surface', async ({ page }) => {
		await page.goto('/');
		await page.getByLabel('Email').fill('admin@example.com');
		await page.getByLabel('Password').fill('admin');
		await page.getByRole('button', { name: /log in/i }).click();

		// AppShell landmarks
		await expect(page.getByRole('banner')).toBeVisible({ timeout: 5000 });      // <header>
		await expect(page.getByRole('contentinfo')).toBeVisible();                  // <footer>
		await expect(page.getByText('Marionette v1.1 · Protocol 1.1.0')).toBeVisible();
		await expect(page.getByText('Marionette CRM')).toBeVisible();               // header title

		// Sidebar nav items
		await expect(page.getByRole('link', { name: /contacts/i }).or(page.getByRole('button', { name: /contacts/i }))).toBeVisible();
		await expect(page.getByRole('link', { name: /companies/i }).or(page.getByRole('button', { name: /companies/i }))).toBeVisible();

		// Navigate to Companies — content area updates, shell persists
		const companiesNav = page.getByRole('link', { name: /companies/i }).or(page.getByRole('button', { name: /companies/i })).first();
		await companiesNav.click();

		// Company list heading should appear inside the main content area
		await expect(page.getByRole('heading', { name: /compan/i }).first()).toBeVisible({ timeout: 5000 });

		// Shell landmarks still visible (not remounted)
		await expect(page.getByText('Marionette v1.1 · Protocol 1.1.0')).toBeVisible();

		// Navigate to Contacts
		const contactsNav = page.getByRole('link', { name: /contacts/i }).or(page.getByRole('button', { name: /contacts/i })).first();
		await contactsNav.click();
		await expect(page.getByRole('heading', { name: /contact/i }).first()).toBeVisible({ timeout: 5000 });
	});

	test('Sidebar trigger is present in header (mobile hamburger)', async ({ page }) => {
		await page.goto('/');
		await page.getByLabel('Email').fill('admin@example.com');
		await page.getByLabel('Password').fill('admin');
		await page.getByRole('button', { name: /log in/i }).click();

		// Shrink viewport to mobile
		await page.setViewportSize({ width: 375, height: 700 });

		// Sidebar trigger button should be visible in the header
		const triggers = page.locator('button[data-sidebar="trigger"], button[aria-label*="sidebar" i], button[aria-controls*="sidebar" i]');
		await expect(triggers.first()).toBeVisible({ timeout: 5000 });
	});
});
```

2. Extend `frontend/tests/e2e/protocol-conformance.spec.ts` to cover the new PatchOperation variants. Inspect the existing file first — it likely uses `ws-capture.ts` to record frames during a session and then validates each frame against its schema.

   Add a new test case that:
   - Logs in (to trigger the shell + content renders)
   - Navigates to a new contact form
   - Changes the country select to Switzerland (triggers a patch message with set + delete-node + set-node + insert-child ops)
   - Captures the resulting PatchMessage from the WebSocket
   - Validates it against `spec/schemas/message.yaml PatchMessage` via the existing ajv validator
   - Additionally validates each `PatchOperation` entry against the tagged oneOf in `data.yaml`

   Example test case to append (adjust variable names to match the existing file's conventions):

```typescript
test('PatchMessage with node tree ops validates against spec/schemas/message.yaml', async ({ page }) => {
	const captured: unknown[] = [];
	page.on('websocket', (ws) => {
		ws.on('framereceived', (frame) => {
			try {
				const parsed = JSON.parse(frame.payload.toString());
				if (parsed?.type === 'patch') captured.push(parsed);
			} catch {
				/* ignore non-JSON frames */
			}
		});
	});

	await page.goto('/');
	await page.getByLabel('Email').fill('admin@example.com');
	await page.getByLabel('Password').fill('admin');
	await page.getByRole('button', { name: /log in/i }).click();
	await page.getByRole('button', { name: /new contact|add contact|new/i }).first().click();
	await page.getByLabel('Country').selectOption({ label: 'Switzerland' });
	await page.waitForTimeout(500); // let the frame arrive

	expect(captured.length).toBeGreaterThan(0);

	const patchMsg = captured.find(
		(m) => Array.isArray((m as { patch?: unknown[] }).patch)
	) as { patch: unknown[]; surface: string };

	expect(patchMsg).toBeDefined();
	expect(patchMsg.surface).toBe('content');

	// At least one op is a node op (set-node / insert-child / delete-node)
	const ops = patchMsg.patch as Array<{ op: string }>;
	const opTypes = new Set(ops.map((o) => o.op));
	const nodeOpTypes = ['set-node', 'delete-node', 'set-children', 'insert-child', 'remove-child'];
	const hasNodeOp = nodeOpTypes.some((t) => opTypes.has(t));
	expect(hasNodeOp).toBe(true);

	// Validate against the ajv-compiled schema
	// (Use whatever helper the existing file exposes; this example shows a
	// minimal inline check. Adapt to the file's existing validator instance.)
	const Ajv = (await import('ajv')).default;
	const yaml = await import('js-yaml');
	const fs = await import('fs');
	const dataSchema = yaml.load(fs.readFileSync('../spec/schemas/data.yaml', 'utf8'));
	const messageSchema = yaml.load(fs.readFileSync('../spec/schemas/message.yaml', 'utf8'));
	const commonSchema = yaml.load(fs.readFileSync('../spec/schemas/common.yaml', 'utf8'));
	const componentSchema = yaml.load(fs.readFileSync('../spec/schemas/component.yaml', 'utf8'));

	const ajv = new Ajv({ strict: false, discriminator: true });
	ajv.addSchema(commonSchema as object, 'common.yaml');
	ajv.addSchema(componentSchema as object, 'component.yaml');
	ajv.addSchema(dataSchema as object, 'data.yaml');
	const validate = ajv.compile({
		$ref: 'message.yaml#/PatchMessage',
		$defs: messageSchema as object,
	});
	const ok = validate(patchMsg);
	expect(ok, `PatchMessage validation errors: ${JSON.stringify(validate.errors)}`).toBe(true);
});
```

**IMPORTANT**: If the existing `protocol-conformance.spec.ts` already has a helper that loads and compiles the schemas once, reuse it instead of the inline `ajv.compile` shown above. The inline version is a safety net; the clean implementation uses the project's existing `schema-validator.ts` helper.

3. Also bump the `hello_version` assertion in `protocol-conformance.spec.ts` (if one exists) from `"1.0.0"` to `"1.1.0"`. Grep the file for `1.0.0` and update.

4. Run:
```bash
cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/shell-nav.spec.ts tests/e2e/protocol-conformance.spec.ts 2>&1 | tail -30
```

All tests must pass. The most common failure modes:
- Sidebar link selectors not matching — adjust based on actual rendered markup (the exact `aria-label` depends on the shadcn-svelte sidebar primitive's output)
- ajv discriminator option rejected — if ajv <8.12 does not support `discriminator: true` alongside `strict: false`, downgrade to simple `oneOf` validation without the discriminator keyword (still correct, just worse error messages)
- Hello version mismatch — confirm Plan 02 bumped `ws.rs:109` and Plan 03 bumped all four `1.0.0` occurrences in `spec/PROTOCOL.md`

5. Run the full E2E suite once to confirm no regressions:
```bash
cd frontend && npx playwright test --config playwright.e2e.config.ts 2>&1 | tail -20
```
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npx playwright test --config playwright.e2e.config.ts tests/e2e/shell-nav.spec.ts tests/e2e/protocol-conformance.spec.ts 2&gt;&amp;1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'test.skip' frontend/tests/e2e/shell-nav.spec.ts` returns no match
    - `grep -q 'Marionette v1.1 · Protocol 1.1.0' frontend/tests/e2e/shell-nav.spec.ts` succeeds
    - `grep -q 'data-sidebar="trigger"\|aria-label.*sidebar\|aria-controls.*sidebar' frontend/tests/e2e/shell-nav.spec.ts` succeeds
    - `frontend/tests/e2e/protocol-conformance.spec.ts` contains at least one new test case referencing `'set-node'`, `'insert-child'`, or `'delete-node'`
    - `grep -n '"1.0.0"' frontend/tests/e2e/protocol-conformance.spec.ts` returns zero lines
    - `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/shell-nav.spec.ts` exits 0 with 2+ tests passing
    - `cd frontend && npx playwright test --config playwright.e2e.config.ts tests/e2e/protocol-conformance.spec.ts` exits 0 (existing tests + new node-op test passing)
    - `cd frontend && npx playwright test --config playwright.e2e.config.ts` (full E2E suite) exits 0
  </acceptance_criteria>
  <done>shell-nav.spec.ts validates the AppShell landmark structure and nav updates; protocol-conformance.spec.ts validates a live PatchMessage with node tree ops against the updated schemas. Full E2E suite is green.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| client contact form → contact_country_change handler | Select change action carries `/contact/country` value in payload; handler dispatches a PatchMessage into `content` surface |
| E2E test runner → live backend | Playwright runs against the real backend on port 3001 with seeded admin credentials |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-19 | Tampering | A client could send `contact_country_change` with a fabricated `country` value not in the Select's options | mitigate | The handler's `match country.as_str()` pattern accepts only `CH`, `US`, `DE`; any other value is a no-op (no InsertChild ops emitted). No data corruption path. Documented as an input-validation pattern. |
| T-12-20 | Denial of Service | Rapid country-change actions (e.g., user scrolling through select) could flood the server with patches | accept | Shadcn SelectInput fires change events debounced by the underlying primitive. The patch workload is small (≤6 ops per call). Rate-limiting is a v2 concern if observed in practice. |
| T-12-21 | Information Disclosure | PatchMessage captured by E2E test frame inspection leaks into test logs on failure | accept | Test environment uses seeded admin credentials with no real PII. Test logs stay in CI artifacts. |
</threat_model>

<verification>
- `cd backend && cargo test --workspace` exits 0
- `cd backend && cargo clippy --workspace -- -D warnings` exits 0
- `cd frontend && npm run check` exits 0
- `cd frontend && npx vitest --run` exits 0 (all unit tests)
- `cd frontend && npx vitest --config vitest-browser.config.ts --run` exits 0 (all browser tests)
- `cd frontend && npx playwright test --config playwright.e2e.config.ts` exits 0 (full E2E suite)
- Specific gated tests:
  - `tests/e2e/node-patch-focus.spec.ts` — 3 passing (focus-preservation, country swap, D-B15 toast lifecycle)
  - `tests/e2e/shell-nav.spec.ts` — 2 passing
  - `tests/e2e/protocol-conformance.spec.ts` — existing + new node-op validation passing
- `grep -rn 'surface:\s*"main"' backend/crates/crm-demo/src/handlers/` returns zero lines (Plan 07 verification)
- `grep -rn '"1.0.0"' backend/crates/ --include='*.rs'` returns only changelog comments if any
- `grep -q 'enum PatchOperation' backend/crates/marionette-protocol/src/data.rs`
</verification>

<success_criteria>
All 8 Phase 12 success criteria from ROADMAP are measurably satisfied:

1. **PatchMessage carries data + tree ops atomically** — Rust enum + YAML oneOf + TS union + spec docs all in place (Plans 02, 03, 04)
2. **Focus preservation works** — browser test in Plan 04 + live E2E test in Plan 08 prove it
3. **Version 1.1.0 + CONCEPT.md reconciled** — Plans 02 (ws.rs), 03 (PROTOCOL.md + CONCEPT.md), Plan 08 protocol-conformance asserts the wire version
4. **AppShell collapsible sidebar desktop + mobile sheet** — Plan 06 component + Plan 08 shell-nav.spec.ts mobile test
5. **Header title + user menu; footer status + version** — Plan 07 handle_navigate builds them; Plan 08 shell-nav asserts visibility
6. **CSS variable theming via --sidebar-* tokens** — Plan 01 token rename + Plan 06 Sidebar.* composition (shadcn Sidebar reads the tokens natively)
7. **AppShell is a normal SDUI component** — Plan 05 hand-written builder + Plan 06 registry entry + Plan 07 handler usage
8. **CRM runs inside AppShell with nav + one node-mutation demo** — Plan 07 nav + Plan 08 country-select demo + focus-preservation E2E

All 7 requirement IDs (PATCH-01, PATCH-02, PATCH-03, SHELL-01, SHELL-02, SHELL-03, SHELL-04) are covered by plans in this phase.
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-08-SUMMARY.md` recording:
- Full E2E suite pass count (before/after)
- Which country-specific fields the demo uses (CH/US/DE) and their child IDs
- Any ajv configuration changes required in protocol-conformance.spec.ts
- Confirmation that `grep -rn '"1.0.0"' backend/crates/ --include='*.rs'` returns only comments (if any)
- A one-line phase-closing statement for PROGRESS.md recording that the protocol version bumped to 1.1.0 and the AppShell shipped
</output>
