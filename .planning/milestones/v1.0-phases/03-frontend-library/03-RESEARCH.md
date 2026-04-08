# Phase 3: Frontend Library - Research

**Researched:** 2026-03-20
**Domain:** Svelte 5 component library, reactive data store, WebSocket messaging, server-driven UI rendering
**Confidence:** HIGH

## Summary

Phase 3 builds the complete Marionette Svelte library: a reactive data store bound via JSON Pointer paths, a component registry with dynamic rendering from adjacency lists, WebSocket message handling, multi-surface rendering, and the full component vocabulary (navigation, forms, layout, tables, popups, feedback). The library lives in `frontend/src/lib/` and is consumed by the CRM demo in `frontend/src/routes/`.

The core technical challenges are: (1) building a deeply reactive data store using Svelte 5 `$state()` that supports JSON Pointer get/set with dirty field tracking and optimistic updates, (2) dynamic component rendering from a flat adjacency list using Svelte 5's native dynamic component support (no `<svelte:component>` needed), (3) WebSocket connection lifecycle with reconnection/backoff, and (4) virtual scrolling for data tables. All interactive/styled components wrap Flowbite Svelte 1.31.0.

**Primary recommendation:** Build the store as a `.svelte.ts` module using `$state()` deep reactivity with `$derived` for JSON Pointer resolution. Use Svelte 5's native dynamic component rendering (`<Thing />` where Thing is a variable) for the NodeRenderer. Wrap Flowbite Svelte components as thin Marionette components that add data binding and action dispatch. Use `@tanstack/svelte-virtual` for virtual scroll (with Svelte 5 compatibility caveats -- may need a thin custom virtualizer if TanStack is broken).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Single Svelte 5 `$state()` store holding all protocol data
- JSON Pointer paths resolve into the store via get/set helpers
- Components bind via derived signals from the store
- Patch operations update the store reactively
- Focus-based dirty field tracking: mark dirty on focus, clean on blur, skip incoming patches while dirty, apply pending on blur
- Optimistic updates: snapshot affected paths, restore on error (no event-sourcing)
- Action-driven URL routing: SvelteKit router bypassed, backend render messages include route field, frontend updates URL via history.pushState, browser back/forward send navigation actions
- Static component registry: plain object mapping type strings to Svelte component constructors, extensible via register(), unknown types render visible fallback
- Recursive NodeRenderer traverses adjacency list, looks up type in registry, renders children by ID
- Named Surface components (main, sidebar, modal, toast), each with independent component tree
- Interactive/styled controls: thin wrappers around Flowbite Svelte components
- Container components use Flowbite Card/Section wrappers
- Virtual scrolling for data-table: appears as full table, rows load progressively, backend provides total count, frontend requests chunks on scroll, server-side sort
- No client-side pagination -- virtual scroll replaces it

### Claude's Discretion
- Virtual scroll chunk size and prefetch strategy
- Exact WebSocket reconnection backoff parameters (within exponential backoff spec)
- Component prop type definitions (TypeScript interfaces)
- Error boundary implementation around individual components
- Toast auto-dismiss timing and animation
- Loading skeleton design for surfaces

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FRONT-01 | Reactive data store with JSON Pointer binding | $state() deep reactivity in .svelte.ts module, json-ptr library for RFC 6901 path resolution, $derived for component bindings |
| FRONT-02 | Component registry with dynamic rendering from adjacency list | Svelte 5 native dynamic components (variable as tag), static Map registry, recursive NodeRenderer |
| FRONT-03 | Message handling (send actions, receive renders/patches/events) | WebSocket API, typed message discriminator on `type` field, store dispatch pattern |
| FRONT-04 | Multi-surface renderer (main, sidebar, modal, toast) | Named Surface components with independent state, Flowbite Modal/Toast for overlays |
| FRONT-05 | WebSocket connection management with reconnection | Native WebSocket + exponential backoff, hello message handshake, keepalive via transport ping/pong |
| FRONT-06 | Optimistic update handling with rollback on failure | Snapshot/restore pattern keyed by correlation ID, applied before action send |
| FRONT-07 | Dirty field tracking (skip patches to actively edited fields) | Focus/blur event tracking per JSON Pointer path, pending patch queue |
| FRONT-08 | URL routing (reflect route in URL, handle browser nav) | history.pushState/replaceState, popstate listener, initial navigate action on connect |
| FRONT-10 | Navigation components (side-nav, nav-item, nav-group) | Flowbite Sidebar/SidebarGroup/SidebarItem wrappers with bind + action |
| FRONT-11 | Form components (form, text-input, select, checkbox, button) | Flowbite Input/Select/Checkbox/Button wrappers with bind:value + action dispatch |
| FRONT-12 | Layout components (container, grid/flex, heading, text) | Flowbite Card + native HTML with Tailwind classes, no heavy wrapper needed |
| FRONT-13 | Table components (data-table with virtual scroll) | @tanstack/svelte-virtual or custom virtualizer + Flowbite Table components |
| FRONT-14 | Popup components (modal, toast, confirm-dialog) | Flowbite Modal/Toast wrappers, surface-targeted rendering |
| FRONT-15 | Feedback components (spinner/loading, error display) | Flowbite Spinner, custom error boundary + loading skeleton |
| FRONT-16 | Flowbite styling integration | Flowbite Svelte 1.31.0 already installed, Tailwind v4 configured |
| FRONT-20 | Unit test framework (Vitest) for component logic | Vitest 4.1.0 already configured, test .svelte.ts store logic in unit tests |
| FRONT-21 | Component tests using vitest-browser-svelte + Playwright | vitest-browser-svelte 2.1.0 + @vitest/browser 4.1.0 + playwright |
| FRONT-22 | Data store unit tests | Pure Vitest unit tests for store get/set/patch/dirty/optimistic |
| FRONT-23 | Message handling unit tests | Pure Vitest unit tests for message dispatch, render processing |
| FRONT-24 | E2E test framework (Playwright) | @playwright/test 1.58.2 for full user flow testing |
| FRONT-25 | Visual regression testing with Playwright screenshots | toHaveScreenshot() with baseline images, CI-generated baselines |
| FRONT-26 | Component visual snapshots | Per-component screenshot tests in vitest-browser or Playwright |
| FRONT-27 | Full-page visual snapshots | Playwright full-page screenshots for key assembled views |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| svelte | 5.54.0 | UI framework | Already installed, Svelte 5 runes provide the reactive primitives needed |
| @sveltejs/kit | 2.55.0 | App framework | Already installed, provides routing shell and build tooling |
| flowbite-svelte | 1.31.0 | UI component library | Already installed, provides styled components (Input, Table, Modal, Toast, Sidebar, etc.) |
| flowbite-svelte-icons | 3.1.0 | Icon library | Already installed, provides ChevronUp/Down, ExclamationTriangle, etc. |
| tailwindcss | 4.2.0 | CSS utility framework | Already installed, Flowbite Svelte depends on it |
| json-ptr | 3.1.1 | JSON Pointer (RFC 6901) | Complete RFC 6901 implementation, ESM/CJS/UMD builds, handles path parsing and resolution |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tanstack/svelte-virtual | 3.13.23 | Virtual scrolling | For data-table virtual scroll; Svelte 5 compatibility may require workarounds |
| vitest | 4.1.0 | Test runner | Already installed, unit and component tests |
| @vitest/browser | 4.1.0 | Browser test mode | Component tests in real browser via Playwright |
| vitest-browser-svelte | 2.1.0 | Svelte component rendering in tests | Render + interact with Svelte components in vitest browser mode |
| @playwright/test | 1.58.2 | E2E + visual regression | Full user flow tests and screenshot comparison |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| json-ptr | Hand-rolled JSON Pointer | RFC 6901 has edge cases (escaped slashes, empty segments) -- use library |
| @tanstack/svelte-virtual | Custom virtual scroller | TanStack has Svelte 5 compatibility issues; may need custom implementation with fixed 48px row height |
| flowbite-svelte 1.31.0 | flowbite-svelte 2.0.0-next.9 | Next version is still in beta, stick with stable 1.31.0 |

**Installation:**
```bash
cd frontend
npm install json-ptr @tanstack/svelte-virtual
npm install -D @vitest/browser vitest-browser-svelte @playwright/test
npx playwright install chromium
```

## Architecture Patterns

### Recommended Project Structure
```
frontend/src/lib/
  store/
    data.svelte.ts       # Main reactive data store ($state)
    pointer.ts           # JSON Pointer get/set/resolve helpers using json-ptr
    dirty.svelte.ts      # Dirty field tracking (focus/blur per path)
    optimistic.svelte.ts # Optimistic snapshot/restore
  transport/
    websocket.svelte.ts  # WebSocket connection, reconnection, send/receive
    messages.ts          # Message type definitions (TS interfaces from protocol schemas)
    dispatcher.ts        # Incoming message router (render/patch/event/error handlers)
  registry/
    registry.ts          # Component registry (Map<string, Component>)
    defaults.ts          # Register all built-in Marionette components
  components/
    core/
      NodeRenderer.svelte     # Recursive adjacency list renderer
      Surface.svelte          # Named surface container
      ErrorBoundary.svelte    # Catches component render errors
      FallbackComponent.svelte # Unknown type placeholder (dev mode)
      LoadingSkeleton.svelte  # Animated loading placeholder
    nav/
      SideNav.svelte          # Wraps Flowbite Sidebar
      NavItem.svelte          # Wraps Flowbite SidebarItem
      NavGroup.svelte         # Wraps Flowbite SidebarGroup
    form/
      Form.svelte             # Form container with error display
      TextInput.svelte        # Wraps Flowbite Input
      SelectInput.svelte      # Wraps Flowbite Select
      Checkbox.svelte         # Wraps Flowbite Checkbox
      Button.svelte           # Wraps Flowbite Button
    layout/
      Container.svelte        # Wraps Flowbite Card or div
      Grid.svelte             # CSS Grid/Flex layout
      Heading.svelte          # h1-h6 with Tailwind typography
      Text.svelte             # Paragraph/span text
    table/
      DataTable.svelte        # Virtual scroll table with sort
    popup/
      ModalSurface.svelte     # Modal overlay using Flowbite Modal
      ToastSurface.svelte     # Toast stack using Flowbite Toast
      ConfirmDialog.svelte    # Confirm dialog
    feedback/
      Spinner.svelte          # Wraps Flowbite Spinner
      ErrorDisplay.svelte     # Error message display
  routing/
    router.svelte.ts    # URL sync: pushState, popstate, initial navigate
  index.ts              # Library entry point -- re-exports everything
```

### Pattern 1: Reactive Data Store with JSON Pointer Binding

**What:** A single `$state()` object holds all protocol data per surface. Components derive their values via JSON Pointer resolution.

**When to use:** Every component that displays or edits data.

**Example:**
```typescript
// store/data.svelte.ts
import { JsonPointer } from 'json-ptr';

// Per-surface data stores
const surfaces: Record<string, { data: Record<string, unknown> }> = $state({});

export function getStore(surface: string) {
  if (!surfaces[surface]) {
    surfaces[surface] = { data: {} };
  }
  return surfaces[surface];
}

export function getData(surface: string, pointer: string): unknown {
  const store = getStore(surface);
  return JsonPointer.get(store.data, pointer);
}

export function setData(surface: string, pointer: string, value: unknown): void {
  const store = getStore(surface);
  JsonPointer.set(store.data, pointer, value, true);
}

export function applyPatch(surface: string, operations: PatchOperation[]): void {
  for (const op of operations) {
    // Skip if path is dirty (being edited)
    if (!isDirty(op.path)) {
      setData(surface, op.path, op.value);
    } else {
      queuePatch(op.path, op);
    }
  }
}
```

**Critical note on $state() in .svelte.ts files:** Runes only work in `.svelte.ts` (or `.svelte.js`) files, NOT in plain `.ts` files. The file extension is a compiler signal. Any module that uses `$state`, `$derived`, or `$effect` MUST use the `.svelte.ts` extension.

**Critical note on exported $state:** You cannot directly reassign an exported `$state` variable from another module. Instead, export an object and mutate its properties, or export accessor functions. The store pattern above uses an object (`surfaces`) whose properties are mutated.

### Pattern 2: Dynamic Component Rendering (NodeRenderer)

**What:** Svelte 5 allows using a variable as a component tag directly. No `<svelte:component>` needed.

**When to use:** The NodeRenderer that maps adjacency list nodes to actual Svelte components.

**Example:**
```svelte
<!-- components/core/NodeRenderer.svelte -->
<script lang="ts">
  import { getComponent } from '$lib/registry/registry';
  import { getData, setData } from '$lib/store/data.svelte';
  import FallbackComponent from './FallbackComponent.svelte';
  import ErrorBoundary from './ErrorBoundary.svelte';

  let { nodeId, nodes, surface } = $props();

  let node = $derived(nodes[nodeId]);
  let Component = $derived(getComponent(node?.type) ?? FallbackComponent);
</script>

{#if node}
  {#if !node.visible || getData(surface, node.visible)}
    <ErrorBoundary>
      <Component
        props={node.props ?? {}}
        bind={node.bind}
        action={node.action}
        {surface}
      >
        {#if node.children}
          {#each node.children as childId (childId)}
            <svelte:self nodeId={childId} {nodes} {surface} />
          {/each}
        {/if}
      </Component>
    </ErrorBoundary>
  {/if}
{/if}
```

**Key insight:** In Svelte 5, `<Component />` where `Component` is a variable works natively. When the variable changes, the component re-renders. This makes the registry lookup pattern trivial.

### Pattern 3: Thin Flowbite Wrapper

**What:** Each Marionette component wraps a Flowbite Svelte component, adding data binding via JSON Pointer and action dispatch.

**When to use:** Every interactive component in the vocabulary.

**Example:**
```svelte
<!-- components/form/TextInput.svelte -->
<script lang="ts">
  import { Input } from 'flowbite-svelte';
  import { getData, setData } from '$lib/store/data.svelte';
  import { dispatch } from '$lib/transport/dispatcher';
  import { markDirty, clearDirty } from '$lib/store/dirty.svelte';

  let { props, bind, action, surface } = $props();

  let value = $derived(bind ? getData(surface, bind) as string : '');

  function handleInput(e: Event) {
    if (bind) {
      setData(surface, bind, (e.target as HTMLInputElement).value);
    }
  }

  function handleFocus() {
    if (bind) markDirty(bind);
  }

  function handleBlur() {
    if (bind) clearDirty(bind);
    if (action?.type === 'submit' || props?.submitOnBlur) {
      // dispatch blur-triggered actions
    }
  }
</script>

<Input
  value={value}
  label={props?.label}
  placeholder={props?.placeholder}
  required={props?.required}
  type={props?.inputType ?? 'text'}
  oninput={handleInput}
  onfocus={handleFocus}
  onblur={handleBlur}
/>
```

### Pattern 4: WebSocket with Reconnection

**What:** Managed WebSocket connection with exponential backoff reconnection.

**Example:**
```typescript
// transport/websocket.svelte.ts
let socket: WebSocket | null = $state(null);
let connected = $state(false);
let reconnectDelay = 1000;
const MAX_DELAY = 30000;

export function connect(url: string) {
  socket = new WebSocket(url);

  socket.onopen = () => {
    connected = true;
    reconnectDelay = 1000; // Reset on success
    // Send initial navigate action with current URL
  };

  socket.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    handleMessage(msg); // Route to dispatcher
  };

  socket.onclose = () => {
    connected = false;
    scheduleReconnect(url);
  };
}

function scheduleReconnect(url: string) {
  const jitter = reconnectDelay * 0.2 * (Math.random() * 2 - 1);
  const delay = Math.min(reconnectDelay + jitter, MAX_DELAY);
  setTimeout(() => connect(url), delay);
  reconnectDelay = Math.min(reconnectDelay * 2, MAX_DELAY);
}
```

### Anti-Patterns to Avoid

- **Using .ts instead of .svelte.ts for reactive modules:** Runes are compiler features. Files without the `.svelte` prefix in their extension will not be compiled by the Svelte compiler, so `$state()`, `$derived()`, etc. will not work. Always use `.svelte.ts` for any module that uses runes.

- **Reassigning exported $state variables:** `export let count = $state(0)` then setting `count = 5` from another module breaks reactivity. Export objects and mutate properties, or export getter/setter functions.

- **Destructuring $state objects:** `let { name } = $state({ name: 'Alice' })` loses reactivity. The destructured `name` is a snapshot, not a reactive binding. Access properties through the proxy: `state.name`.

- **Using <svelte:component> in Svelte 5:** While it still works, it is unnecessary. Use dynamic component tags directly: `<Component />` where `Component` is a `$derived` or `$state` variable.

- **Nesting component trees (not using adjacency list):** The protocol uses flat adjacency lists. Never convert to nested structures -- traverse the flat map by ID reference.

- **Patching data for dirty fields:** Always check dirty state before applying a server patch. Clobbering user input mid-edit is the most common SDUI bug.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Pointer parsing | Custom path splitter | `json-ptr` library | RFC 6901 has edge cases: escaped `~0` (`~`), escaped `~1` (`/`), empty segments, root pointer |
| Virtual scrolling | Custom scroll handler | `@tanstack/svelte-virtual` (if compatible) or known algorithm | Scroll position math, overscan, resize observers -- deceptively complex |
| UI component styling | Custom CSS for inputs/buttons/tables | Flowbite Svelte wrappers | Consistent design, accessibility, dark mode -- already solved |
| WebSocket reconnection | Interval-based retry | Exponential backoff with jitter | Thundering herd problem without jitter |
| Message type discrimination | if/else chains | Discriminated union on `type` field with TypeScript | Protocol schema already defines tagged union |
| Screenshot comparison | Pixel-diff algorithm | Playwright `toHaveScreenshot()` | Built-in pixelmatch, threshold config, baseline management |

**Key insight:** The Marionette library's value is in the glue (store, binding, registry, rendering) -- not in reimplementing UI controls or virtual scroll algorithms.

## Common Pitfalls

### Pitfall 1: Svelte 5 Runes in Wrong File Extension
**What goes wrong:** `$state()` silently becomes a regular function call, producing a non-reactive object.
**Why it happens:** Developer creates `store.ts` instead of `store.svelte.ts`.
**How to avoid:** Enforce `.svelte.ts` extension for ALL files that use runes. Lint or naming convention.
**Warning signs:** Components don't update when store values change.

### Pitfall 2: Deep vs Shallow Reactivity Confusion
**What goes wrong:** Setting a nested property on a `$state.raw()` object doesn't trigger updates. Or, deep proxy on a large dataset causes performance issues.
**Why it happens:** Misunderstanding when to use `$state()` (deep proxy) vs `$state.raw()` (assignment-only reactivity).
**How to avoid:** Use `$state()` for the main data store (it needs deep reactivity for JSON Pointer path updates). Use `$state.raw()` only for large immutable datasets like table row caches where you always replace entire objects.
**Warning signs:** Nested data updates not reflected in UI, or excessive re-renders on large data.

### Pitfall 3: Circular Re-render in Two-Way Binding
**What goes wrong:** Component reads from store via `$derived`, writes back on input, which triggers the derived again, causing infinite loop or jank.
**Why it happens:** Input handler calls `setData()` which updates the $state, which re-derives the value, which updates the input.
**How to avoid:** Svelte's `$derived` is read-only and glitch-free -- it won't cause loops. But avoid `$effect` that writes back to the same state it reads. Use event handlers (oninput) for writes, `$derived` for reads. Never use `$effect` for two-way binding.
**Warning signs:** Console warnings about "effect cycle", UI flickering on input.

### Pitfall 4: Surface State Isolation
**What goes wrong:** Patching data in the modal surface accidentally affects the main surface.
**Why it happens:** Sharing a single flat data object across all surfaces.
**How to avoid:** Each surface MUST have its own data store. The protocol specifies independent state per surface. Patch messages target specific data paths within a surface's data scope.
**Warning signs:** Opening a modal changes values on the page behind it.

### Pitfall 5: @tanstack/svelte-virtual Svelte 5 Incompatibility
**What goes wrong:** Virtual scroll renders empty content or breaks with Svelte 5 reactivity.
**Why it happens:** @tanstack/svelte-virtual was written for Svelte 4's store-based reactivity. Svelte 5's proxy-based reactivity may not trigger the same update paths.
**How to avoid:** Test the library early. If broken, implement a minimal custom virtualizer -- with fixed 48px row height (per UI spec), the math is straightforward: `visibleStart = Math.floor(scrollTop / 48)`, render `visibleStart - overscan` to `visibleStart + visibleCount + overscan`.
**Warning signs:** Empty table body, rows not appearing on scroll.

### Pitfall 6: WebSocket Message Ordering
**What goes wrong:** Optimistic patches get overwritten by stale server responses.
**Why it happens:** Network latency causes server responses to arrive out of order with new user actions.
**How to avoid:** Use correlation IDs (`id` field on messages). Track pending optimistic updates by ID. When a server response arrives with a matching ID, it supersedes the optimistic state. Responses without matching pending IDs are applied normally.
**Warning signs:** UI reverts user changes momentarily then re-applies them.

### Pitfall 7: Patch to Null Means Delete
**What goes wrong:** Patching a path with `value: null` is treated as "set to null" instead of "delete this key."
**Why it happens:** Protocol says setting value to null removes an item from a keyed collection.
**How to avoid:** In the store's `setData`, check if value is `null` and the parent is a keyed collection -- if so, delete the key from the parent object rather than setting it to null.
**Warning signs:** Deleted items show as null entries instead of disappearing.

## Code Examples

### TypeScript Interfaces from Protocol Schemas

```typescript
// transport/messages.ts
// Derived from spec/schemas/message.yaml

export interface HelloMessage {
  type: 'hello';
  version: string;
}

export interface RenderMessage {
  type: 'render';
  id?: string;
  surface: string;
  root: string;
  nodes: Record<string, ComponentNode>;
  data: Record<string, unknown>;
}

export interface PatchMessage {
  type: 'patch';
  id?: string;
  patch: PatchOperation[];
}

export interface ActionMessage {
  type: 'action';
  id?: string;
  name: string;
  source?: string;
  payload?: Record<string, unknown>;
  optimistic?: { patch: PatchOperation[] };
}

export interface EventMessage {
  type: 'event';
  id?: string;
  name: string;
  surface?: string;
  hint?: Record<string, unknown>;
}

export interface ErrorMessage {
  type: 'error';
  id?: string;
  errors: ValidationError[];
}

export type ProtocolMessage =
  | HelloMessage
  | RenderMessage
  | PatchMessage
  | ActionMessage
  | EventMessage
  | ErrorMessage;

// From spec/schemas/component.yaml
export interface ComponentNode {
  type: string;
  props?: Record<string, unknown>;
  children?: string[];
  bind?: string;      // JSON Pointer
  action?: ComponentAction;
  visible?: string;   // JSON Pointer to boolean
}

export interface ComponentAction {
  type: string;
  name?: string;
  target?: string;
  idPath?: string;
  [key: string]: unknown; // additionalProperties: true
}

// From spec/schemas/data.yaml
export interface PatchOperation {
  path: string;   // JSON Pointer
  value: unknown;
}

export interface ValidationError {
  path?: string;
  message: string;
}
```

### Component Registry

```typescript
// registry/registry.ts
import type { Component } from 'svelte';

const registry = new Map<string, Component>();

export function register(type: string, component: Component): void {
  registry.set(type, component);
}

export function getComponent(type: string): Component | undefined {
  return registry.get(type);
}

export function registerAll(components: Record<string, Component>): void {
  for (const [type, component] of Object.entries(components)) {
    registry.set(type, component);
  }
}
```

### Dirty Field Tracking

```typescript
// store/dirty.svelte.ts
const dirtyPaths = new Set<string>();
const pendingPatches = new Map<string, PatchOperation[]>();

export function markDirty(path: string): void {
  dirtyPaths.add(path);
}

export function clearDirty(path: string): void {
  dirtyPaths.delete(path);
  // Apply any queued patches for this path
  const queued = pendingPatches.get(path);
  if (queued) {
    pendingPatches.delete(path);
    for (const op of queued) {
      // Apply to store (import from data.svelte.ts)
    }
  }
}

export function isDirty(path: string): boolean {
  // Check if the path or any parent path is dirty
  for (const dirty of dirtyPaths) {
    if (path === dirty || path.startsWith(dirty + '/')) {
      return true;
    }
  }
  return false;
}

export function queuePatch(path: string, op: PatchOperation): void {
  if (!pendingPatches.has(path)) {
    pendingPatches.set(path, []);
  }
  pendingPatches.get(path)!.push(op);
}
```

### Optimistic Update with Snapshot/Restore

```typescript
// store/optimistic.svelte.ts
import { getData, setData, applyPatch } from './data.svelte';

interface OptimisticEntry {
  surface: string;
  snapshots: Map<string, unknown>; // path -> original value
}

const pending = new Map<string, OptimisticEntry>(); // correlationId -> entry

export function applyOptimistic(
  correlationId: string,
  surface: string,
  operations: PatchOperation[]
): void {
  const snapshots = new Map<string, unknown>();
  for (const op of operations) {
    snapshots.set(op.path, getData(surface, op.path));
    setData(surface, op.path, op.value);
  }
  pending.set(correlationId, { surface, snapshots });
}

export function confirmOptimistic(correlationId: string): void {
  pending.delete(correlationId); // Server confirmed, discard snapshot
}

export function rollbackOptimistic(correlationId: string): void {
  const entry = pending.get(correlationId);
  if (entry) {
    for (const [path, value] of entry.snapshots) {
      setData(entry.surface, path, value);
    }
    pending.delete(correlationId);
  }
}
```

### vitest-browser-svelte Component Test

```typescript
// Source: https://vitest.dev/api/browser/svelte
import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import TextInput from '$lib/components/form/TextInput.svelte';

test('TextInput renders with label and binds value', async () => {
  const screen = await render(TextInput, {
    props: { label: 'Email', placeholder: 'Enter email' },
    bind: '/user/email',
    surface: 'main',
  });

  await expect.element(screen.getByLabelText('Email')).toBeVisible();
  await screen.getByLabelText('Email').fill('test@example.com');
  // Verify store was updated
});
```

### Playwright Visual Regression Test

```typescript
// Source: https://playwright.dev/docs/test-snapshots
import { test, expect } from '@playwright/test';

test('data table matches visual snapshot', async ({ page }) => {
  // Setup: mock WebSocket, send render message with table data
  await page.goto('/');
  await expect(page.locator('[data-surface="main"]')).toBeVisible();
  await expect(page).toHaveScreenshot('data-table.png', {
    maxDiffPixels: 100,
  });
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `<svelte:component this={X}>` | `<X />` (dynamic variable as tag) | Svelte 5 (Oct 2024) | No special syntax for dynamic components |
| Svelte stores (`writable`, `readable`) | `$state()` runes in `.svelte.ts` | Svelte 5 (Oct 2024) | Simpler API, deep reactivity, no subscribe/unsubscribe |
| `$:` reactive declarations | `$derived()` and `$effect()` | Svelte 5 (Oct 2024) | Explicit dependency tracking, no surprises |
| @testing-library/svelte + jsdom | vitest-browser-svelte + Playwright | 2024-2025 | Real browser rendering, no jsdom quirks |
| Slot-based component composition | Snippet-based composition | Svelte 5 (Oct 2024) | More flexible content projection |

**Deprecated/outdated:**
- `<svelte:component>`: Still works but unnecessary in Svelte 5
- Svelte stores (`writable`/`readable`): Not deprecated but superseded by runes for new code
- `$:` reactive declarations: Superseded by `$derived` and `$effect`
- `$$props` / `$$restProps`: Replaced by `$props()` with destructuring

## Open Questions

1. **@tanstack/svelte-virtual Svelte 5 Compatibility**
   - What we know: GitHub issues report it doesn't work with Svelte 5. The library was designed for Svelte 4's store-based reactivity.
   - What's unclear: Whether version 3.13.23 has resolved these issues, or if workarounds exist.
   - Recommendation: Test early in Wave 0. If broken, implement minimal custom virtualizer -- fixed row height (48px) makes the math straightforward. A custom ~80-line implementation is viable.

2. **Patch Message Surface Scoping**
   - What we know: Render messages target a specific surface. Patch messages have path but no surface field.
   - What's unclear: How to know which surface a patch targets. The protocol schema shows patches don't have a surface field.
   - Recommendation: Patches likely apply to a "global" data scope or the most recent surface context. Clarify with protocol author -- may need to track which surface "owns" which data paths based on render messages.

3. **Error Boundary in Svelte 5**
   - What we know: Svelte doesn't have built-in error boundaries like React's `componentDidCatch`.
   - What's unclear: Best pattern for catching render errors from dynamic components in Svelte 5.
   - Recommendation: Use `try/catch` in `$effect` or wrap component mounting. Another approach: use Svelte's `onMount` error handling and a sentinel. Research the `boundary` experimental feature or use `{#key}` blocks that can reset on error.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 + vitest-browser-svelte 2.1.0 + @playwright/test 1.58.2 |
| Config file | frontend/vite.config.ts (Vitest uses Vite config) |
| Quick run command | `cd frontend && npx vitest run --reporter=verbose` |
| Full suite command | `cd frontend && npx vitest run && npx playwright test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FRONT-01 | Data store get/set/patch via JSON Pointer | unit | `cd frontend && npx vitest run src/lib/store/ -x` | No -- Wave 0 |
| FRONT-02 | Registry lookup + NodeRenderer renders components | browser-component | `cd frontend && npx vitest run --browser src/lib/components/core/ -x` | No -- Wave 0 |
| FRONT-03 | Message dispatch routes to correct handler | unit | `cd frontend && npx vitest run src/lib/transport/ -x` | No -- Wave 0 |
| FRONT-04 | Multi-surface independent rendering | browser-component | `cd frontend && npx vitest run --browser src/lib/components/core/Surface -x` | No -- Wave 0 |
| FRONT-05 | WebSocket connect/reconnect/backoff | unit | `cd frontend && npx vitest run src/lib/transport/websocket -x` | No -- Wave 0 |
| FRONT-06 | Optimistic apply + rollback on error | unit | `cd frontend && npx vitest run src/lib/store/optimistic -x` | No -- Wave 0 |
| FRONT-07 | Dirty tracking skips patches, applies on blur | unit | `cd frontend && npx vitest run src/lib/store/dirty -x` | No -- Wave 0 |
| FRONT-08 | URL updates on render, popstate sends navigate | unit | `cd frontend && npx vitest run src/lib/routing/ -x` | No -- Wave 0 |
| FRONT-10 | Nav components render and dispatch actions | browser-component | `cd frontend && npx vitest run --browser src/lib/components/nav/ -x` | No -- Wave 0 |
| FRONT-11 | Form components bind values and dispatch | browser-component | `cd frontend && npx vitest run --browser src/lib/components/form/ -x` | No -- Wave 0 |
| FRONT-12 | Layout components render children | browser-component | `cd frontend && npx vitest run --browser src/lib/components/layout/ -x` | No -- Wave 0 |
| FRONT-13 | DataTable virtual scroll + sort | browser-component | `cd frontend && npx vitest run --browser src/lib/components/table/ -x` | No -- Wave 0 |
| FRONT-14 | Modal/toast/confirm render in correct surface | browser-component | `cd frontend && npx vitest run --browser src/lib/components/popup/ -x` | No -- Wave 0 |
| FRONT-15 | Spinner/error display render states | browser-component | `cd frontend && npx vitest run --browser src/lib/components/feedback/ -x` | No -- Wave 0 |
| FRONT-16 | Flowbite styling applied correctly | visual | `cd frontend && npx playwright test --grep visual` | No -- Wave 0 |
| FRONT-22 | Store binding + patching + dirty | unit | `cd frontend && npx vitest run src/lib/store/ -x` | No -- Wave 0 |
| FRONT-23 | Message handling dispatch | unit | `cd frontend && npx vitest run src/lib/transport/ -x` | No -- Wave 0 |
| FRONT-25 | Visual regression baselines | visual | `cd frontend && npx playwright test --grep screenshot` | No -- Wave 0 |
| FRONT-26 | Component visual snapshots | visual | `cd frontend && npx playwright test --grep component-visual` | No -- Wave 0 |
| FRONT-27 | Full-page visual snapshots | visual | `cd frontend && npx playwright test --grep page-visual` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cd frontend && npx vitest run -x`
- **Per wave merge:** `cd frontend && npx vitest run && npx playwright test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `frontend/vitest.workspace.ts` or browser config -- configure @vitest/browser with Playwright provider
- [ ] `frontend/playwright.config.ts` -- Playwright E2E and visual regression configuration
- [ ] `frontend/tests/` -- Playwright E2E test directory
- [ ] Install: `npm install -D @vitest/browser vitest-browser-svelte @playwright/test && npx playwright install chromium`
- [ ] `frontend/src/lib/store/data.svelte.test.ts` -- store unit test scaffolding
- [ ] `frontend/src/lib/transport/messages.test.ts` -- message type tests

## Sources

### Primary (HIGH confidence)
- `spec/PROTOCOL.md` -- Authoritative protocol specification, all message types and data binding rules
- `spec/schemas/message.yaml`, `component.yaml`, `data.yaml`, `common.yaml` -- Machine-readable protocol schemas
- Svelte 5 official docs ($state, $derived, dynamic components, mount) -- https://svelte.dev/docs/svelte/$state, https://svelte.dev/docs/svelte/v5-migration-guide
- Vitest browser mode API docs -- https://vitest.dev/api/browser/svelte
- Playwright visual comparison docs -- https://playwright.dev/docs/test-snapshots

### Secondary (MEDIUM confidence)
- Flowbite Svelte 1.31.0 component APIs -- https://flowbite-svelte.com/docs/forms/input-field
- @tanstack/svelte-virtual -- https://tanstack.com/virtual/latest/docs/framework/svelte/svelte-virtual (Svelte 5 compat uncertain)
- json-ptr npm package -- https://www.npmjs.com/package/json-ptr (3.1.1, ESM support confirmed)
- Svelte 5 shared state patterns -- https://mainmatter.com/blog/2025/03/11/global-state-in-svelte-5/

### Tertiary (LOW confidence)
- @tanstack/svelte-virtual Svelte 5 status -- https://github.com/TanStack/virtual/issues/866 (may be resolved, needs validation)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all core libraries are already installed and verified, versions confirmed against npm registry
- Architecture: HIGH -- patterns directly derived from Svelte 5 official docs and protocol specification
- Pitfalls: HIGH -- documented from official Svelte 5 migration guide and community reports
- Testing: MEDIUM -- vitest-browser-svelte 2.1.0 API verified but real SvelteKit project integration needs validation
- Virtual scroll: LOW -- @tanstack/svelte-virtual Svelte 5 compatibility unconfirmed, may need custom implementation

**Research date:** 2026-03-20
**Valid until:** 2026-04-20 (stable ecosystem, 30-day window appropriate)
