---
phase: 11-leaf-component-migration
reviewed: 2026-04-09T00:00:00Z
depth: standard
files_reviewed: 24
files_reviewed_list:
  - frontend/src/lib/components/core/ConnectionBanner.svelte
  - frontend/src/lib/components/core/FallbackComponent.svelte
  - frontend/src/lib/components/core/LoadingSkeleton.svelte
  - frontend/src/lib/components/feedback/Spinner.svelte
  - frontend/src/lib/components/form/Button.svelte
  - frontend/src/lib/components/form/Checkbox.svelte
  - frontend/src/lib/components/form/SelectInput.svelte
  - frontend/src/lib/components/form/TextInput.svelte
  - frontend/src/lib/components/layout/Container.svelte
  - frontend/src/lib/components/layout/Grid.svelte
  - frontend/src/lib/components/layout/Heading.svelte
  - frontend/src/lib/components/layout/Text.svelte
  - frontend/src/lib/components/nav/NavGroup.svelte
  - frontend/src/lib/components/nav/NavItem.svelte
  - frontend/src/lib/components/nav/SideNav.svelte
  - frontend/src/lib/components/popup/ConfirmDialog.svelte
  - frontend/src/lib/components/popup/ModalSurface.svelte
  - frontend/src/lib/components/popup/ToastSurface.svelte
  - frontend/src/lib/components/screen/FormScreen.svelte
  - frontend/src/lib/components/screen/TableScreen.svelte
  - frontend/src/lib/components/table/DataTable.svelte
  - frontend/src/lib/index.ts
  - frontend/src/lib/registry/icons.ts
  - frontend/src/lib/utils.ts
findings:
  critical: 3
  warning: 5
  info: 3
  total: 11
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-04-09
**Depth:** standard
**Files Reviewed:** 24
**Status:** issues_found

## Summary

These are SDUI leaf components migrated to shadcn-svelte primitives with lucide-svelte icons. The overall architecture is sound and Svelte 5 `$props`/`$derived`/`$state` patterns are used correctly throughout. Icon resolution is safe (registry lookup with a fallback, no dynamic import or eval). No XSS vectors were found — all text interpolation goes through Svelte's text nodes, not `{@html}`.

Three critical issues were found: the `ToastSurface` exported function cannot work as a module-level singleton in Svelte 5; `ConfirmDialog` renders shadcn Dialog sub-components outside a `Dialog.Root` context, breaking accessibility; and `NavItem` passes `action.name` directly to `sendAction` without a fallback, silently sending `undefined` as the action name.

Five warnings cover logic problems in dirty-state management, dual-use of `action.type`, an effect that can fire an infinite fetch loop, and a console side-effect hidden in template markup.

## Critical Issues

### CR-01: ToastSurface — exported instance method unusable as module API

**File:** `frontend/src/lib/components/popup/ToastSurface.svelte:21`

**Issue:** `export function addToast(...)` inside a Svelte component's `<script>` block is an *instance export*, not a module export. In Svelte 5, calling it requires a reference obtained via `bind:this`. However, the pattern here implies it is intended for use as a singleton imported from the module — which is impossible. Any consumer that does `import { addToast } from '.../ToastSurface.svelte'` will receive `undefined` at import time; only a component instance exposes this function after mounting. The toasts array is also instance-local state, so multiple mounts create independent registries.

**Fix:** Move the toast store and `addToast` function to a standalone `.svelte.ts` store module, then import and call it from within the component:

```typescript
// frontend/src/lib/store/toasts.svelte.ts
interface ToastItem { id: string; severity: string; message: string; duration: number; }
let toasts: ToastItem[] = $state([]);

export function addToast(event: { name: string; hint?: Record<string, unknown> }): void {
    const id = crypto.randomUUID();
    const severity = (event.hint?.severity as string) ?? 'info';
    const message  = (event.hint?.message  as string) ?? event.name;
    const duration = (event.hint?.duration as number) ?? 5000;
    toasts.push({ id, severity, message, duration });
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id); }, duration);
}

export function getToasts() { return toasts; }
```

Then in `ToastSurface.svelte`, import `getToasts` and iterate. Export `addToast` from `index.ts` for dispatcher use.

---

### CR-02: ConfirmDialog — Dialog sub-components rendered without Dialog.Root context

**File:** `frontend/src/lib/components/popup/ConfirmDialog.svelte:39-56`

**Issue:** `Dialog.Header`, `Dialog.Title`, `Dialog.Description`, and `Dialog.Footer` are rendered without a `Dialog.Root` / `Dialog.Content` ancestor in this component. When `ConfirmDialog` is rendered as a child of `ModalSurface`'s `NodeRenderer`, the `Dialog.Content` wrapper is in `ModalSurface.svelte`, not in `ConfirmDialog`. The shadcn-svelte `Dialog.Title` and `Dialog.Description` use Svelte context (set by `Dialog.Root`) to wire accessibility IDs (`aria-labelledby`, `aria-describedby`). Without the context being passed through `NodeRenderer`, these IDs will be empty/undefined, breaking WCAG 2.1 dialog accessibility requirements. In addition, shadcn-svelte `Dialog.Footer` renders a flex row — outside a `Dialog.Content` its styling will be incorrect.

**Fix:** Remove the shadcn Dialog sub-components and render plain markup that is consistent with `ModalSurface`'s wrapping `Dialog.Content`:

```svelte
<div>
    <div class="mb-4">
        {#if title}<h2 class="text-lg font-semibold">{title}</h2>{/if}
        {#if message}<p class="text-sm text-muted-foreground mt-1">{message}</p>{/if}
    </div>
    <div class="flex justify-end gap-2 mt-6">
        <ShadcnButton variant="outline" onclick={handleCancel}>{cancelLabel}</ShadcnButton>
        <ShadcnButton variant={destructive ? 'destructive' : 'default'} onclick={handleConfirm}>{confirmLabel}</ShadcnButton>
    </div>
</div>
```

Alternatively, have `ModalSurface` pass the dialog context object down through `NodeRenderer` so sub-components can consume it.

---

### CR-03: NavItem — undefined action name sent to dispatcher

**File:** `frontend/src/lib/components/nav/NavItem.svelte:34`

**Issue:** `sendAction(action.name, ...)` is called without a fallback. `ComponentAction.name` may be `undefined` (the type definition allows it — `Button.svelte` defensively uses `action.name ?? action.type`). When `action.name` is absent, the dispatcher receives `undefined` as the action name, which will either silently fail or send a malformed WebSocket message to the backend.

**Fix:**
```typescript
sendAction(action.name ?? action.type ?? 'navigate', action.payload as Record<string, unknown> | undefined);
```

## Warnings

### WR-01: Button and FormScreen — action.type dual-use conflates UI variant with protocol name

**File:** `frontend/src/lib/components/form/Button.svelte:45`, `frontend/src/lib/components/screen/FormScreen.svelte:50,102`

**Issue:** `action.type` is used both as a fallback action name (`action.name ?? action.type` sent to dispatcher) and as a button visual variant classifier (`act.type === 'destructive'`). In `FormScreen` line 102, `act.type as ButtonVariant` is passed as the shadcn variant — values like `'destructive'`, `'outline'`, or `'ghost'` are UI tokens, not backend action identifiers. If the backend sends `type: 'destructive'` for styling, that string would also be dispatched as the action name when `name` is absent. This is a semantic collision waiting to cause incorrect backend behavior.

**Fix:** Separate UI variant from action identity. Add an explicit `variant` field to `ToolbarAction` / `ComponentAction` for the button appearance, and stop using `type` as a fallback action name:

```typescript
type ToolbarAction = ComponentAction & { label?: string; icon?: string; variant?: ButtonVariant };
// Then:
sendAction(act.name ?? 'toolbar-action', payload, act.target);
// And:
variant={act.variant ?? 'default'}
```

---

### WR-02: SelectInput — dirty state cleared on value change, not on close

**File:** `frontend/src/lib/components/form/SelectInput.svelte:28-39`

**Issue:** `markDirty(bind)` is called in `handleOpenChange` when the dropdown opens, but `clearDirty` is called in `handleValueChange` when a value is selected. This means: if the user opens the select and then dismisses it without choosing a value, `clearDirty` is never called, leaving the field permanently in a dirty state and potentially running optimistic rollbacks on every subsequent re-render. TextInput correctly pairs mark/clear on focus/blur.

**Fix:** Call `clearDirty` inside `handleOpenChange` when `open === false`:

```typescript
function handleOpenChange(open: boolean) {
    if (open && bind) {
        markDirty(bind);
    } else if (!open && bind) {
        clearDirty(bind, (op) => setData(surface, op.path, op.value));
    }
}
```

And remove the `clearDirty` call from `handleValueChange` (or keep it there to clear dirty immediately on selection — but the close handler must also clear it for the dismiss case).

---

### WR-03: DataTable prefetch effect can fire infinitely when rows fill to capacity

**File:** `frontend/src/lib/components/table/DataTable.svelte:65-69`

**Issue:** The `$effect` fires `sendAction('fetch-rows', ...)` when `visibleEnd >= rows.length - CHUNK_SIZE * 2`. When the data store is updated with new rows in response, `rows` changes, re-evaluating the condition. If `explicitTotalRows` is large and the scroll position keeps the `visibleEnd` near the tail of loaded rows, subsequent fetches will fire on every patch until `rows.length >= explicitTotalRows`. This is expected paginating behavior, but the condition `rows.length < explicitTotalRows` (already present) is the correct guard. However, there is no debounce or in-flight guard: if the server is slow, two renders could both see the same `rows.length` before the response arrives and dispatch duplicate fetch requests.

**Fix:** Add a `let fetching = $state(false)` flag:

```typescript
let fetching = $state(false);

$effect(() => {
    if (
        !fetching &&
        explicitTotalRows > 0 &&
        visibleEnd > 0 &&
        visibleEnd >= rows.length - CHUNK_SIZE * 2 &&
        rows.length < explicitTotalRows
    ) {
        fetching = true;
        sendAction('fetch-rows', { offset: rows.length, limit: CHUNK_SIZE });
    }
});

// Reset fetching when new rows arrive (rows.length increases)
$effect(() => {
    rows.length; // track
    fetching = false;
});
```

---

### WR-04: FallbackComponent — side effect hidden in template expression

**File:** `frontend/src/lib/components/core/FallbackComponent.svelte:15`

**Issue:** The production (`{:else}`) branch uses an IIFE `{(() => { console.warn(...); return ''; })()}` to trigger a side effect inside Svelte template markup. This is non-idiomatic and fragile: Svelte may evaluate template expressions multiple times during reconciliation, causing duplicate console warnings. Side effects belong in `$effect`, not in the render tree.

**Fix:** Lift the warn into an `$effect`:

```typescript
$effect(() => {
    if (!import.meta.env.DEV) {
        console.warn('Unknown component type:', nodeType, 'on surface:', surface);
    }
});
```

And replace the `{:else}` branch with simply `{:else}{/if}` (render nothing in production).

---

## Info

### IN-01: Spinner — props/bind/action/surface declared but never used

**File:** `frontend/src/lib/components/feedback/Spinner.svelte:6-22`

**Issue:** The full SDUI component contract (`props`, `bind`, `action`, `surface`, `children`) is declared in `$props()` but the component renders a hardcoded spinner with no logic. This is not a bug but wastes prop destructuring noise and could mislead maintainers into thinking these props affect the component.

**Fix:** Reduce to only the props that are actually consumed, or add a comment explaining the contract is declared for registry uniformity.

---

### IN-02: LoadingSkeleton — Array constructor used instead of Array.from

**File:** `frontend/src/lib/components/core/LoadingSkeleton.svelte:10`

**Issue:** `{#each Array(lines) as _, i}` creates a sparse array. This works in Svelte's `{#each}` but is a well-known JS footgun — `Array(n)` returns a sparse array with `length === n` but no indices, so `.map()` would skip them. Svelte's `{#each}` iterates by index rather than `for...of`, so it works here, but the intent is unclear.

**Fix:** Use `Array.from({ length: lines })` for clarity:

```svelte
{#each Array.from({ length: lines }) as _, i}
```

---

### IN-03: icons.ts — registerIcon allows arbitrary overwrite of defaults

**File:** `frontend/src/lib/registry/icons.ts:19-21`

**Issue:** `ICON_REGISTRY` is a plain mutable object. Any caller (including server-driven SDUI config if `registerIcon` is exposed) can silently overwrite built-in icons. While there is no direct server path to call `registerIcon` in the current code, the function is exported from `index.ts` and any future feature that processes server config could accidentally (or maliciously) replace icons.

**Fix:** Consider a read-once registration or at minimum document that `registerIcon` must only be called during app initialization before any rendering occurs. No code change required now, but add a comment:

```typescript
// registerIcon must only be called at app boot (before Surface mounts).
// It is not safe to call with untrusted server data.
export function registerIcon(name: string, component: Component): void {
```

---

_Reviewed: 2026-04-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
