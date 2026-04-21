---
phase: 13
plan: 04
type: execute
wave: 2
depends_on: [13-01]
files_modified:
  - frontend/src/lib/components/table/DataTableActions.svelte
  - frontend/src/lib/components/table/DataTableActions.browser-test.ts
autonomous: true
requirements: [TABLE-01]
must_haves:
  truths:
    - "`DataTableActions.svelte` renders a shadcn `DropdownMenu` trigger (three-dots / Ellipsis button) + content with one `DropdownMenu.Item` per element of `items` prop"
    - "Each item label is rendered via Svelte text interpolation `{item.label}` — NEVER via `{@html}` (XSS mitigation)"
    - "Clicking an item dispatches `sendAction(item.action.name ?? item.action.type, item.action.payload, item.action.target)` via the `$lib/transport/dispatcher` module"
    - "Component accepts `items: { label: string; action: ComponentAction }[]` as a prop (the shape CRM handlers already produce in their row data)"
    - "Component handles `items: []` by rendering the trigger but an empty menu (or no-op trigger) — does NOT crash"
    - "Malicious `label` strings containing `<script>` tags render as literal text, not as executed HTML"
  artifacts:
    - path: "frontend/src/lib/components/table/DataTableActions.svelte"
      provides: "Per-row actions DropdownMenu component"
    - path: "frontend/src/lib/components/table/DataTableActions.browser-test.ts"
      provides: "XSS mitigation proof + click dispatch test"
  key_links:
    - from: "DataTable.svelte (Plan 05) cell renderer for column.kind='actions'"
      to: "DataTableActions via renderComponent"
      via: "renderComponent(DataTableActions, { items })"
      pattern: "renderComponent\\(DataTableActions"
    - from: "DataTableActions click handler"
      to: "sendAction dispatcher"
      via: "import { sendAction } from '$lib/transport/dispatcher'"
      pattern: "sendAction\\("
---

<objective>
Create a minimal, reusable Svelte component that renders the `actions` cell kind as a shadcn DropdownMenu. This component is the missing piece in the current CRM: three list handlers (`contact.rs`, `company.rs`, `user.rs`) already ship per-row `actions` arrays in their JSON, but the old DataTable just `String()`s them, producing the latent `[object Object]` bug confirmed in research (13-RESEARCH.md Summary paragraph 1). Plan 05's rewritten DataTable will wire `column.kind: 'actions'` to this component via `renderComponent`.

Purpose: Isolate the per-row action menu into its own testable component. Plan 05's DataTable rewrite stays focused on table mechanics (virtualizer, filter bar, column visibility) and delegates the actions cell to this component. Parallelizable with Plan 03 (pure-Svelte vs pure-Rust, no file overlap).

Output: One small Svelte component + one browser test covering render, click dispatch, and XSS escape behavior.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/13-datatable-enhancements/13-CONTEXT.md
@.planning/phases/13-datatable-enhancements/13-RESEARCH.md
@.planning/phases/13-datatable-enhancements/13-VALIDATION.md
@.planning/codebase/CONVENTIONS.md
@.planning/codebase/TESTING.md
@frontend/src/lib/transport/dispatcher.ts
@frontend/src/lib/transport/messages.ts
@frontend/src/lib/components/table/DataTable.browser-test.ts
@frontend/src/lib/components/form/Button.svelte

<interfaces>
<!-- Executor MUST read these BEFORE writing the component. -->

Row action data shape produced by CRM handlers (from `contact.rs:423-426`):
```json
{
  "id": 42,
  "name": "Alice Johnson",
  "actions": [
    { "label": "Edit",   "action": { "type": "click", "name": "contact_edit",   "payload": { "contact_id": 42 } } },
    { "label": "Delete", "action": { "type": "click", "name": "contact_delete", "payload": { "contact_id": 42 } } }
  ]
}
```

Existing `ComponentAction` TypeScript type (frontend/src/lib/transport/messages.ts:76-90):
```typescript
export interface ComponentAction {
  type: string;
  name?: string;
  target?: string;
  idPath?: string;
  variant?: string;
  [key: string]: unknown;
}
```

Existing `sendAction` signature after Plan 01:
```typescript
export function sendAction(
  name: string,
  payload?: Record<string, unknown>,
  source?: string,
  optimistic?: { patch: PatchOperation[] }
): string;
```

Shadcn-svelte DropdownMenu import (path set up by Plan 01):
```typescript
import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
```

Typical shadcn DropdownMenu composition:
```svelte
<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon">
        <EllipsisVertical class="size-4" />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end">
    {#each items as item (item.label)}
      <DropdownMenu.Item onSelect={() => handleClick(item)}>
        {item.label}
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
</DropdownMenu.Root>
```

Browser test pattern (`DataTable.browser-test.ts:1-16`):
```typescript
import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import DataTableActions from './DataTableActions.svelte';

vi.mock('$lib/transport/dispatcher', () => ({ sendAction: vi.fn() }));
import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => { vi.clearAllMocks(); });
```

Lucide icon for the trigger (from `@lucide/svelte` — already installed):
```typescript
import EllipsisVertical from '@lucide/svelte/icons/ellipsis-vertical';
```
</interfaces>

<research_references>
- 13-RESEARCH.md Summary paragraph 1 — the `[object Object]` bug is CONFIRMED real in three CRM handlers (`contact.rs:423-426`, `company.rs:126-129`, `user.rs:87-90`)
- 13-RESEARCH.md §Pattern 3 (Per-kind cell renderers) — `DataTableActions.svelte` is explicitly the target of `renderComponent(DataTableActions, { items: ... })` in the actions case
- 13-CONTEXT.md §D-F1 — `actions` kind "expects rowData[col.key] to be an array of {label, action} objects, renders as a DropdownMenu"
- 13-VALIDATION.md rows 23, 27 — tests prove actions DropdownMenu renders and `item.label` is XSS-escaped
- Security §XSS via column cell rendering — mitigation: "Svelte text interpolation; DataTableActions.svelte renders item.label via text, not `{@html}`"
</research_references>
</context>

<mcp_tool_usage>
Use the `svelte` MCP server (`mcp__svelte__*`) when writing `DataTableActions.svelte`. In particular:
1. Query for the correct Svelte 5 + shadcn-svelte DropdownMenu composition pattern (snippet `child` prop, `onSelect` handler vs `onclick`)
2. Confirm the `{#snippet child({ props })}` syntax for passing trigger props through to a nested Button
3. After writing the file, re-invoke svelte MCP to validate no bit-ui/runes issues

Do NOT guess the DropdownMenu API — the shadcn-svelte recipe uses a specific `child` snippet pattern that's easy to get wrong.
</mcp_tool_usage>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Write DataTableActions.svelte + browser test covering render, click, and XSS escape</name>
  <files>
    frontend/src/lib/components/table/DataTableActions.svelte,
    frontend/src/lib/components/table/DataTableActions.browser-test.ts
  </files>
  <read_first>
    - frontend/src/lib/components/ui/dropdown-menu/ (the full CLI-generated primitive — understand its Root/Trigger/Content/Item namespace structure; the Plan 01 CLI install wrote this)
    - frontend/src/lib/components/ui/button/ (existing shadcn Button used as the trigger)
    - frontend/src/lib/transport/messages.ts §ComponentAction (type shape)
    - frontend/src/lib/transport/dispatcher.ts §sendAction (Plan 01 extended signature)
    - frontend/src/lib/components/table/DataTable.browser-test.ts (browser test pattern reference)
    - .planning/codebase/CONVENTIONS.md §Svelte 5 Component Patterns
    - .planning/phases/13-datatable-enhancements/13-RESEARCH.md §Pattern 3 (the canonical shape)
    - Query the svelte MCP: ask for the correct Svelte 5 shadcn-svelte DropdownMenu composition pattern and for XSS-safe string interpolation rules
  </read_first>
  <behavior>
    - Given `items = [{label: "Edit", action: {type: "click", name: "contact_edit", payload: {id: 42}}}]`, clicking "Edit" calls `sendAction("contact_edit", {id: 42}, undefined)`
    - Given `items = [{label: "Delete", action: {type: "click", name: "contact_delete", payload: {contact_id: 7}, target: "modal"}}]`, clicking "Delete" calls `sendAction("contact_delete", {contact_id: 7}, "modal")`
    - Given `items = [{label: "<script>alert(1)</script>", action: {type: "click", name: "noop"}}]`, the DOM contains the literal text `<script>alert(1)</script>` (i.e., the angle brackets are HTML-escaped by Svelte's text interpolation) and NO `<script>` element exists as a child of the menu item
    - Given `items = [{label: "Fallback name", action: {type: "click"}}]` (no `name` present), clicking falls back to `sendAction("click", undefined, undefined)` using `action.type`
    - Given `items = []`, the component renders the trigger but NO menu items (or an empty menu content)
  </behavior>
  <action>
    **Step 1 — Invoke the svelte MCP** to get the idiomatic Svelte 5 shadcn-svelte DropdownMenu composition pattern. Ask specifically:
    > "Show me the correct Svelte 5 syntax for a shadcn-svelte DropdownMenu with a Trigger that wraps a shadcn Button (ghost variant, icon-only, lucide EllipsisVertical), Content aligned end, and Items rendered from a reactive `items` array. Include the `child` snippet pattern for passing trigger props through to the Button."

    Capture the exact syntax the MCP returns.

    **Step 2 — Write `frontend/src/lib/components/table/DataTableActions.svelte`.**

    Target shape:

    ```svelte
    <script lang="ts">
      import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
      import { Button } from '$lib/components/ui/button';
      import EllipsisVertical from '@lucide/svelte/icons/ellipsis-vertical';
      import { sendAction } from '$lib/transport/dispatcher';
      import type { ComponentAction } from '$lib/transport/messages';

      /**
       * Per-row action item. Matches the shape CRM handlers already produce
       * (contact.rs:423, company.rs:126, user.rs:87). Phase 13 Plan 05's
       * DataTable wires `column.kind: 'actions'` to this component via
       * `renderComponent(DataTableActions, { items: row.original[col.key] })`.
       */
      export interface ActionItem {
        label: string;
        action: ComponentAction;
      }

      let {
        items = [],
      }: {
        items?: ActionItem[];
      } = $props();

      function handleSelect(item: ActionItem) {
        const name = item.action.name ?? item.action.type;
        const payload = (item.action.payload as Record<string, unknown> | undefined) ?? undefined;
        const target = item.action.target;
        sendAction(name, payload, target);
      }
    </script>

    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Button {...props} variant="ghost" size="icon" aria-label="Row actions">
            <EllipsisVertical class="size-4" />
          </Button>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        {#each items as item (item.label)}
          <DropdownMenu.Item onSelect={() => handleSelect(item)}>
            {item.label}
          </DropdownMenu.Item>
        {/each}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
    ```

    CRITICAL syntax notes that the svelte MCP will confirm:
    - The label inside the menu item MUST be `{item.label}` (text interpolation), NEVER `{@html item.label}`. This is the ONLY XSS mitigation in this file.
    - `onSelect` is the shadcn-svelte DropdownMenu.Item event that fires on both click and keyboard activation. Use `onSelect`, not `onclick`, unless the svelte MCP says otherwise.
    - If the CLI-generated DropdownMenu primitive exposes Trigger with a different child-snippet shape (e.g., not `{ props }` but `{ triggerProps }` or similar), match it exactly. The Plan 01 CLI install is the source of truth.

    After writing the file, RE-INVOKE the svelte MCP with the file contents and ask: "Does this component use idiomatic Svelte 5 + shadcn-svelte patterns? Are there any issues with the DropdownMenu composition, the snippet pattern, or reactive bindings?" Fix any issues the MCP identifies.

    **Step 3 — Write `frontend/src/lib/components/table/DataTableActions.browser-test.ts`.**

    ```typescript
    import { render } from 'vitest-browser-svelte';
    import { expect, test, vi, beforeEach } from 'vitest';
    import DataTableActions from './DataTableActions.svelte';

    vi.mock('$lib/transport/dispatcher', () => ({
      sendAction: vi.fn(),
    }));
    import { sendAction } from '$lib/transport/dispatcher';

    beforeEach(() => {
      vi.clearAllMocks();
    });

    test('renders DropdownMenu trigger even for empty items', async () => {
      const screen = await render(DataTableActions, { props: { items: [] } });
      await expect.element(screen.getByLabelText('Row actions')).toBeVisible();
    });

    test('renders one menu item per action after trigger click', async () => {
      const items = [
        { label: 'Edit', action: { type: 'click', name: 'contact_edit', payload: { id: 42 } } },
        { label: 'Delete', action: { type: 'click', name: 'contact_delete', payload: { id: 42 } } },
      ];
      const screen = await render(DataTableActions, { props: { items } });
      await screen.getByLabelText('Row actions').click();
      await expect.element(screen.getByText('Edit')).toBeVisible();
      await expect.element(screen.getByText('Delete')).toBeVisible();
    });

    test('dispatches sendAction on item click with name + payload + target', async () => {
      const items = [
        { label: 'Delete', action: { type: 'click', name: 'contact_delete', payload: { contact_id: 7 }, target: 'modal' } },
      ];
      const screen = await render(DataTableActions, { props: { items } });
      await screen.getByLabelText('Row actions').click();
      await screen.getByText('Delete').click();
      expect(sendAction).toHaveBeenCalledWith('contact_delete', { contact_id: 7 }, 'modal');
    });

    test('falls back to action.type when action.name is missing', async () => {
      const items = [
        { label: 'Raw', action: { type: 'custom_action' } },
      ];
      const screen = await render(DataTableActions, { props: { items } });
      await screen.getByLabelText('Row actions').click();
      await screen.getByText('Raw').click();
      expect(sendAction).toHaveBeenCalledWith('custom_action', undefined, undefined);
    });

    test('escapes malicious labels via text interpolation (XSS mitigation)', async () => {
      const evil = '<script>window.__pwned = true</script>';
      const items = [
        { label: evil, action: { type: 'click', name: 'noop' } },
      ];
      const screen = await render(DataTableActions, { props: { items } });
      await screen.getByLabelText('Row actions').click();

      // The literal text must appear, escaped — getByText matches the raw string
      await expect.element(screen.getByText(evil, { exact: true })).toBeVisible();

      // No actual <script> element should have been created as a descendant
      // of the menu content. Query the DOM directly.
      const scripts = document.querySelectorAll('script');
      for (const s of Array.from(scripts)) {
        expect(s.textContent ?? '').not.toContain('__pwned');
      }
      // And the global pollution didn't happen
      expect((window as unknown as { __pwned?: boolean }).__pwned).toBeUndefined();
    });
    ```

    **Step 4 — Run the browser test.**

    ```bash
    cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTableActions.browser-test.ts
    ```

    Iterate until all 5 tests pass. If a test fails because of a DropdownMenu API mismatch (e.g., the CLI-generated primitive uses a different trigger snippet convention), adapt the component to the real primitive and re-run.
  </action>
  <verify>
    <automated>cd frontend && npx tsc --noEmit && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTableActions.browser-test.ts</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/src/lib/components/table/DataTableActions.svelte` exists
    - `grep -c "{@html" frontend/src/lib/components/table/DataTableActions.svelte` == 0 (NO raw-HTML interpolation)
    - `grep -c "sendAction" frontend/src/lib/components/table/DataTableActions.svelte` == 1 (single dispatch path)
    - `grep -c "DropdownMenu" frontend/src/lib/components/table/DataTableActions.svelte` >= 3 (Root + Trigger + Content + Item)
    - `grep -c 'aria-label="Row actions"' frontend/src/lib/components/table/DataTableActions.svelte` == 1
    - `frontend/src/lib/components/table/DataTableActions.browser-test.ts` exists with 5 passing tests
    - `cd frontend && npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTableActions.browser-test.ts` exits 0 with `5 passed`
    - `cd frontend && npx tsc --noEmit` exits 0
    - The XSS test (test 5) specifically asserts both `getByText(evil, { exact: true })` renders the literal and `window.__pwned` is undefined
  </acceptance_criteria>
  <done>Component implemented, svelte MCP validated, all 5 browser tests green, XSS mitigation proven.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Server-supplied row data → DataTableActions `items` prop | Untrusted. `item.label` could contain malicious HTML/script; `item.action.name/payload` could be a crafted action target. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-13-04-01 | Tampering (XSS) | `item.label` rendered into menu item | mitigate | Svelte text interpolation (`{item.label}`) auto-escapes. Test 5 proves `<script>` tags render as literal text and do NOT execute. NEVER use `{@html}` on `item.label`. |
| T-13-04-02 | Tampering (Action forging) | Server could send an `action.name` the current user isn't authorized for | accept | Client cannot decide authorization — all enforcement is server-side. The action will be rejected by the backend router if the session lacks the required role. This is a server-side concern, not a client-side one. |
| T-13-04-03 | I (Information disclosure) | Action payload logs to console on error | accept | No new logging added. Existing dispatcher error handling (from prior phases) applies. |

No HIGH severity threats. XSS is the only real risk and it's structurally mitigated by Svelte's interpolation.
</threat_model>

<verification>
```bash
cd frontend
npx tsc --noEmit
npx vitest --config vitest-browser.config.ts --run src/lib/components/table/DataTableActions.browser-test.ts
```

Both MUST exit 0. 5 tests green.
</verification>

<success_criteria>
- `DataTableActions.svelte` exists and renders an accessible DropdownMenu per row
- All 5 browser tests pass: empty-items render, item list render, click dispatch with full payload, fallback name resolution, XSS escape proof
- No `{@html}` in the file
- TypeScript compiles
- svelte MCP confirms idiomatic usage
</success_criteria>

<output>
After completion, create `.planning/phases/13-datatable-enhancements/13-04-datatable-actions-component-SUMMARY.md` recording:
- The exact DropdownMenu Trigger snippet pattern used (the `{ props }` destructuring shape) — Plan 05 will reuse the same pattern for the "Columns" trigger
- Any svelte MCP feedback that required a rewrite
- Any deviations from the stub code (e.g., if the CLI-generated DropdownMenu primitive named its Item event `onclick` instead of `onSelect`)
</output>
