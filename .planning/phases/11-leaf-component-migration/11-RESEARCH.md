# Phase 11: Leaf Component Migration - Research

**Researched:** 2026-04-09
**Domain:** Svelte 5 SDUI components, shadcn-svelte primitives, lucide-svelte icons
**Confidence:** HIGH

## Summary

Phase 11 migrates all 18 registered SDUI components from HTML+Tailwind stubs (created in Phase 10) to use shadcn-svelte primitives. The existing stubs already use OKLCH semantic tokens (`bg-background`, `text-foreground`, etc.) and the SDUI interface contract (`surface`, `props`, `bind`, `action`) is preserved. The migration is primarily about replacing hand-written HTML elements with shadcn-svelte primitive components for consistency, accessibility, and maintainability.

The codebase is clean: Flowbite is fully removed (zero imports), `@lucide/svelte` is already installed and used in 3 components, `cn()` utility exists, and `bits-ui` + `tailwind-variants` + `tw-animate-css` are all installed. The `$lib/components/ui/` directory does not yet exist -- shadcn primitives must be installed via CLI before migration begins.

**Primary recommendation:** Install all needed shadcn-svelte primitives in one batch, then migrate components in dependency order (leaf components first, then composites). Build a dynamic icon registry for server-driven icon names. Rewrite all browser tests from scratch targeting shadcn markup.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Pass-through with styling -- SDUI components become mostly pass-through to shadcn primitives, adding only SDUI-specific logic (bind, action, surface). Shadcn variants used directly.
- **D-02:** Bulk install upfront -- Install all needed shadcn-svelte primitives in one batch before starting component migration.
- **D-03:** Compose from shadcn parts for components without direct shadcn equivalents (SideNav, NavGroup, DataTable, Surface, NodeRenderer). Build from shadcn building blocks where possible.
- **D-04:** shadcn Toast (bits-ui Radix Toast primitive) -- not Sonner. **IMPORTANT: Research found this decision is based on incorrect information -- see Open Questions.**
- **D-05:** Dynamic icon registry -- Build a registry mapping string names to lucide-svelte components. Server sends icon name, frontend resolves at runtime.
- **D-06:** Fallback placeholder icon for unknown icon names -- show lucide `CircleHelp`.
- **D-07:** Rewrite browser tests from scratch -- delete existing `.browser-test.ts` files, write new ones targeting shadcn structure.
- **D-08:** Full test coverage -- every one of the 18 migrated components gets a browser test.

### Claude's Discretion
- Specific shadcn primitive mapping per component (which shadcn component maps to which SDUI component)
- Icon registry implementation details (Map vs object, lazy loading vs eager)
- Test assertion granularity per component
- Migration ordering within the phase

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-01 | All existing SDUI components re-implemented with shadcn-svelte primitives | Component mapping table, shadcn primitive list, architecture patterns section |
| COMP-02 | All icons migrated from flowbite-svelte-icons to lucide-svelte | Already done in Phase 10 (zero flowbite imports). Icon registry needed for dynamic server-driven icons. |
</phase_requirements>

## Standard Stack

### Core (already installed)

| Library | Version | Purpose | Verified |
|---------|---------|---------|----------|
| bits-ui | ^2.17.3 | Headless accessible primitives (runtime dep of shadcn-svelte) | [VERIFIED: package.json] |
| @lucide/svelte | ^1.7.0 | Icon library replacing flowbite-svelte-icons | [VERIFIED: package.json] |
| tailwind-variants | 3.2.2 | Variant styling (used by shadcn-svelte components) | [VERIFIED: npm ls] |
| clsx | ^2.1.1 | Conditional class utility | [VERIFIED: package.json] |
| tailwind-merge | ^3.5.0 | Tailwind class deduplication | [VERIFIED: package.json] |
| tw-animate-css | ^1.4.0 | Animation utilities for shadcn-svelte | [VERIFIED: package.json] |

### CLI Tool

| Tool | Version | Purpose |
|------|---------|---------|
| shadcn-svelte | 1.2.7 | Component scaffolding CLI (npx, not installed) | [VERIFIED: npx output] |

### shadcn Primitives to Install

These primitives must be added via `npx shadcn-svelte@latest add` before migration:

| Primitive | Maps To (SDUI) | Rationale |
|-----------|-----------------|-----------|
| button | Button | Direct mapping with variant/size support |
| input | TextInput | Styled input with focus/error states |
| select | SelectInput | Composable select with bits-ui popover |
| checkbox | Checkbox | Accessible checkbox with bindable checked state |
| label | TextInput, SelectInput, Checkbox | Form field labels |
| dialog | ModalSurface, ConfirmDialog | Accessible modal with overlay, portal, close |
| table | DataTable | Semantic table parts (Root, Header, Head, Body, Row, Cell) |
| card | Container (card variant) | Bordered card styling |
| skeleton | LoadingSkeleton | Animated loading placeholders |
| separator | FormScreen section dividers | Visual dividers between form sections |
| badge | (future use, low cost) | May be useful for status indicators |

**Installation command:**
```bash
cd frontend && npx shadcn-svelte@latest add -y button input select checkbox label dialog table card skeleton separator badge
```

[VERIFIED: shadcn-svelte CLI tested with `add -y button` -- installs to `$lib/components/ui/button/`] [CITED: shadcn-svelte.com/docs/components/*]

### utils.ts Gap

The shadcn-svelte CLI generates components that import `WithElementRef` from `$lib/utils.js`, but the current `utils.ts` only exports `cn()`. The `WithElementRef` type must be added:

```typescript
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
```

[VERIFIED: Button component generated by CLI imports `WithElementRef` from `$lib/utils.js`; type not present in current utils.ts]

## Architecture Patterns

### Component Mapping (Claude's Discretion)

| SDUI Component | shadcn Primitive | Migration Complexity | Notes |
|----------------|-----------------|---------------------|-------|
| **Button** | `Button` | Low | Map `props.color=red` to `variant="destructive"`, `props.outline` to `variant="outline"` |
| **TextInput** | `Input` + `Label` | Low | Wrap shadcn Input, keep bind/dirty/blur logic |
| **SelectInput** | `Select.*` | Medium | Composable API (Root/Trigger/Content/Item); must translate flat options array |
| **Checkbox** | `Checkbox` + `Label` | Low | bits-ui checkbox with `bind:checked` pattern |
| **Form** | None (keep as-is) | Minimal | Already correct HTML form; just renders children |
| **Container** | `Card` (card variant only) | Low | Card variant uses shadcn Card; plain container stays as div |
| **Grid** | None (keep as-is) | Minimal | CSS grid layout; no shadcn equivalent needed. Fix dynamic class pitfall. |
| **Heading** | None (keep as-is) | Minimal | Semantic HTML heading; no shadcn primitive exists |
| **Text** | None (keep as-is) | Minimal | Semantic HTML text; no shadcn primitive exists |
| **Spinner** | `Spinner` (if available) or keep SVG | Minimal | Current SVG spinner is fine |
| **ErrorDisplay** | None (keep styling) | Minimal | Already uses lucide AlertCircle icon |
| **DataTable** | `Table.*` | Medium | Replace raw `<table>` with shadcn Table sub-components |
| **ModalSurface** | `Dialog.*` | Medium | Replace custom overlay with Dialog.Root/Content/Overlay/Portal |
| **ConfirmDialog** | `Dialog.*` + `Button` | Medium | Use Dialog sub-components + shadcn Buttons |
| **ToastSurface** | Custom (see Open Questions) | Medium | D-04 conflict -- see below |
| **SideNav** | Composed from `Button` variant=ghost | Low | Nav wrapper; children render NavItems |
| **NavItem** | `Button` variant=ghost | Low | Use shadcn Button with ghost variant for nav items |
| **NavGroup** | None (keep as div) | Minimal | Simple grouping div |
| **NodeRenderer** | None (keep as-is) | None | Renderer logic, not visual |
| **Surface** | None (keep as-is) | None | Surface container, not visual |
| **ConnectionBanner** | None (keep as-is) | Minimal | Notification banner; not a form/dialog component |
| **ErrorBoundary** | None (keep as-is) | None | Svelte boundary, not visual |
| **FallbackComponent** | None (keep as-is) | Minimal | Dev-only debug display |
| **LoadingSkeleton** | `Skeleton` | Low | Replace custom animation with shadcn Skeleton |
| **FormScreen** | Composed (Card, Separator, Button) | Medium | Use Card for sections, Separator between them, shadcn Button for back |
| **TableScreen** | Composed (Button) | Low | Toolbar buttons use shadcn Button |

### Pattern 1: SDUI Pass-Through to shadcn Primitive

**What:** SDUI component accepts protocol props, translates them to shadcn variant props, delegates rendering.
**When:** Component has a direct shadcn equivalent (Button, Input, Checkbox, Select, Dialog).

```svelte
<!-- Source: Codebase pattern + shadcn-svelte docs -->
<script lang="ts">
  import { Button as ShadcnButton } from '$lib/components/ui/button';
  import { sendAction } from '$lib/transport/dispatcher';
  import type { ComponentAction } from '$lib/transport/messages';
  import type { Snippet } from 'svelte';

  let {
    props = {},
    bind,
    action,
    surface,
    children,
  }: {
    props: Record<string, unknown>;
    bind?: string;
    action?: ComponentAction;
    surface: string;
    children?: Snippet;
  } = $props();

  let variant = $derived(
    (props.color as string) === 'red' ? 'destructive' as const
    : (props.outline as boolean) ? 'outline' as const
    : 'default' as const
  );

  function handleClick() {
    if (action) {
      // ... existing action dispatch logic preserved ...
    }
  }
</script>

<ShadcnButton {variant} disabled={props.disabled as boolean} onclick={handleClick}>
  {props.label ?? ''}
</ShadcnButton>
```

### Pattern 2: Composable Select (bits-ui based)

**What:** shadcn Select uses composable API, not a flat props API.
**When:** Migrating SelectInput which currently uses native `<select>`.

```svelte
<script lang="ts">
  import * as Select from '$lib/components/ui/select';
  // ... standard SDUI props ...

  let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
  let options = $derived(
    (props.options as Array<{ value: string; label: string }>) ?? []
  );

  function handleValueChange(newValue: string) {
    if (bind) {
      setData(surface, bind, newValue);
    }
  }
</script>

<div class="w-full">
  {#if props.label}
    <Label>{props.label}</Label>
  {/if}
  <Select.Root type="single" value={value} onValueChange={handleValueChange}>
    <Select.Trigger>
      <span>{options.find(o => o.value === value)?.label ?? props.placeholder ?? 'Select...'}</span>
    </Select.Trigger>
    <Select.Content>
      {#each options as opt (opt.value)}
        <Select.Item value={opt.value} label={opt.label} />
      {/each}
    </Select.Content>
  </Select.Root>
</div>
```

[CITED: shadcn-svelte.com/docs/components/select]

### Pattern 3: Dialog for Modal

**What:** Replace custom overlay div with shadcn Dialog composable.
**When:** Migrating ModalSurface and ConfirmDialog.

```svelte
<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  // ... existing imports ...
  let isOpen = $derived(tree !== undefined);
</script>

<Dialog.Root open={isOpen} onOpenChange={(open) => { if (!open) handleClose(); }}>
  <Dialog.Content>
    {#if tree}
      <NodeRenderer nodeId={tree.root} nodes={tree.nodes} surface="modal" />
    {/if}
  </Dialog.Content>
</Dialog.Root>
```

[CITED: shadcn-svelte.com/docs/components/dialog]

### Pattern 4: Dynamic Icon Registry (D-05)

**What:** Map string icon names from server to lucide-svelte component references.
**When:** Server sends `props.icon: "plus"` and frontend must resolve to `Plus` component.

```typescript
// $lib/registry/icons.ts
import type { Component } from 'svelte';
import CircleHelp from '@lucide/svelte/icons/circle-help';

// Eager registry -- all icons loaded upfront for simplicity
// Tree-shaking still works since only registered icons are imported
const ICON_REGISTRY: Record<string, Component> = {};

export function registerIcon(name: string, component: Component): void {
  ICON_REGISTRY[name] = component;
}

export function getIcon(name: string): Component {
  return ICON_REGISTRY[name] ?? CircleHelp; // D-06: fallback
}

// Register common icons used by the CRM demo
import Plus from '@lucide/svelte/icons/plus';
import ChevronUp from '@lucide/svelte/icons/chevron-up';
import ChevronDown from '@lucide/svelte/icons/chevron-down';
import AlertCircle from '@lucide/svelte/icons/alert-circle';
import X from '@lucide/svelte/icons/x';
import Menu from '@lucide/svelte/icons/menu';
import ArrowLeft from '@lucide/svelte/icons/arrow-left';
import Search from '@lucide/svelte/icons/search';
import Filter from '@lucide/svelte/icons/filter';
import Pencil from '@lucide/svelte/icons/pencil';
import Trash2 from '@lucide/svelte/icons/trash-2';
import Check from '@lucide/svelte/icons/check';
import Loader2 from '@lucide/svelte/icons/loader-2';

// Registration
const defaults: Record<string, Component> = {
  plus: Plus, 'chevron-up': ChevronUp, 'chevron-down': ChevronDown,
  'alert-circle': AlertCircle, x: X, menu: Menu, 'arrow-left': ArrowLeft,
  search: Search, filter: Filter, pencil: Pencil, trash: Trash2,
  check: Check, loader: Loader2,
};
for (const [name, comp] of Object.entries(defaults)) {
  registerIcon(name, comp);
}
```

[ASSUMED: Exact icon names needed by CRM demo -- verify against backend component builders]

### Anti-Patterns to Avoid

- **Wrapping shadcn in extra divs:** shadcn components already handle layout. Don't wrap `<ShadcnButton>` in unnecessary `<div>` elements.
- **Overriding shadcn styles with inline Tailwind:** Use shadcn `variant` props instead of adding custom Tailwind classes that duplicate what variants do.
- **Dynamic Tailwind classes:** `grid-cols-${cols}` does NOT work with Tailwind v4. Use `style="grid-template-columns: repeat({cols}, 1fr)"` instead. Current Grid.svelte has this bug. [VERIFIED: Grid.svelte line 36 uses `grid-cols-${cols}`]
- **Breaking the SDUI contract:** Every component MUST still accept `surface`, `props`, `bind?`, `action?`. shadcn primitives are internal implementation details.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Button variants | Custom class logic | shadcn Button `variant` prop | Consistent design tokens, focus/disabled states |
| Modal overlay | Custom fixed div + backdrop | shadcn Dialog.Root/Content/Overlay | Accessibility (focus trap, escape, aria), portal rendering |
| Select dropdown | Native `<select>` element | shadcn Select composable | Accessible, styled, keyboard navigation, portal |
| Checkbox styling | Custom checkbox CSS | shadcn Checkbox | Accessible, indeterminate state, consistent styling |
| Loading skeleton | Custom pulse animation | shadcn Skeleton | Consistent animation, already themed |
| Form labels | Custom label styling | shadcn Label | Accessible association, consistent typography |

## Common Pitfalls

### Pitfall 1: shadcn Select Value Binding

**What goes wrong:** shadcn Select uses `onValueChange` callback, not native `onchange` event. Trying to use `e.currentTarget.value` fails.
**Why it happens:** bits-ui Select is a custom component, not a native `<select>`.
**How to avoid:** Use `onValueChange={(val) => setData(surface, bind, val)}` pattern.
**Warning signs:** Select value doesn't update in data store after selection.

### Pitfall 2: Dialog Open State Management

**What goes wrong:** ModalSurface derives `isOpen` from surface tree existence. Dialog.Root expects a boolean `open` prop and calls `onOpenChange` when user tries to close.
**Why it happens:** Two state management systems (SDUI surface tree vs Dialog internal state) can conflict.
**How to avoid:** Use `open={isOpen}` (one-way from SDUI) and `onOpenChange` to dispatch `close-modal` action. Never let Dialog manage its own open state.
**Warning signs:** Modal doesn't close, or closes but surface tree is stale.

### Pitfall 3: WithElementRef Missing from utils.ts

**What goes wrong:** shadcn-svelte CLI generates components that import `WithElementRef` from `$lib/utils.js`. Type doesn't exist, causing TypeScript errors.
**Why it happens:** CLI may not update utils.ts when adding components to an existing project.
**How to avoid:** Add `WithElementRef` and `WithoutChildren` types to `$lib/utils.ts` before running `npx shadcn-svelte add`.
**Warning signs:** TypeScript errors on first component add.

[VERIFIED: Tested `npx shadcn-svelte@latest add -y button` -- generated component imports `WithElementRef` from `$lib/utils.js` which is not present]

### Pitfall 4: Grid Dynamic Class Bug

**What goes wrong:** Current Grid.svelte uses `grid-cols-${cols}` which Tailwind v4 cannot JIT-compile for dynamic values.
**Why it happens:** Tailwind scans source code at build time; template literals produce unknown class names.
**How to avoid:** Use inline `style` attribute: `style="grid-template-columns: repeat({cols}, 1fr)"`.
**Warning signs:** Grid renders as single column regardless of `cols` prop.

[VERIFIED: Grid.svelte line 36 uses dynamic class interpolation]

### Pitfall 5: Checkbox onCheckedChange vs onchange

**What goes wrong:** shadcn Checkbox uses `onCheckedChange` callback from bits-ui, not native `onchange`.
**Why it happens:** bits-ui Checkbox is a custom component rendering a button+indicator, not a native checkbox input.
**How to avoid:** Use `onCheckedChange={(val) => setData(surface, bind, val)}`.
**Warning signs:** Checkbox visual state changes but data store not updated.

### Pitfall 6: LoadingSkeleton Hardcoded Colors

**What goes wrong:** Current LoadingSkeleton uses `bg-gray-200 dark:bg-gray-700` -- raw colors not in the OKLCH token system.
**Why it happens:** Phase 10 stub didn't fully convert this component.
**How to avoid:** Replace with shadcn Skeleton component which uses `bg-primary/10` or `bg-muted`.

[VERIFIED: LoadingSkeleton.svelte uses `bg-gray-200 dark:bg-gray-700`]

### Pitfall 7: ToastSurface Has Dual State

**What goes wrong:** ToastSurface.svelte has its own `toasts` state AND there's a separate `toasts.svelte.ts` store. They duplicate each other.
**Why it happens:** Both were created independently during Phase 10 stub work.
**How to avoid:** Consolidate to one approach. Since ToastSurface is an SDUI component registered as `'toast'` in defaults.ts and renders via Surface, keep the component-level state and remove or deprecate the store.

[VERIFIED: Both `ToastSurface.svelte` and `$lib/store/toasts.svelte.ts` maintain independent toast arrays]

## Code Examples

### shadcn Button Import Pattern
```typescript
// Source: Verified from CLI-generated button component
import { Button, buttonVariants } from '$lib/components/ui/button';
// Variants: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
// Sizes: 'default' | 'xs' | 'sm' | 'lg' | 'icon' | 'icon-xs' | 'icon-sm' | 'icon-lg'
```

### shadcn Dialog Pattern
```svelte
<!-- Source: shadcn-svelte.com/docs/components/dialog -->
<Dialog.Root open={isOpen} onOpenChange={handleOpenChange}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description>{message}</Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={handleCancel}>{cancelLabel}</Button>
      <Button variant={destructive ? 'destructive' : 'default'} onclick={handleConfirm}>
        {confirmLabel}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
```

### shadcn Table Pattern
```svelte
<!-- Source: shadcn-svelte.com/docs/components/table -->
<Table.Root>
  <Table.Header>
    <Table.Row>
      {#each columns as col (col.key)}
        <Table.Head>{col.label}</Table.Head>
      {/each}
    </Table.Row>
  </Table.Header>
  <Table.Body>
    {#each visibleRows as [rowKey, rowData] (rowKey)}
      <Table.Row>
        {#each columns as col (col.key)}
          <Table.Cell>{String(rowData[col.key] ?? '')}</Table.Cell>
        {/each}
      </Table.Row>
    {/each}
  </Table.Body>
</Table.Root>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flowbite prop-based API (`<Button color="red">`) | shadcn-svelte variant API (`<Button variant="destructive">`) | shadcn-svelte v1.0+ | All component props must be translated |
| Native `<select>` | bits-ui composable Select | shadcn-svelte v1.0+ | SelectInput needs significant rewrite |
| Custom modal overlay div | Dialog composable with portal | shadcn-svelte v1.0+ | ModalSurface gets accessibility for free |
| tailwind-variants (tv) | Used by shadcn-svelte components | Current | Already installed, used in generated button |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | CRM demo uses ~13 specific lucide icon names (plus, chevron-up, etc.) | Icon Registry pattern | Wrong icon names means icons show as fallback; easily fixable |
| A2 | Select.Root accepts `onValueChange` callback for value changes | Select pattern | Must verify against actual bits-ui API; could be `onchange` |
| A3 | Dialog.Root accepts `open` and `onOpenChange` props | Dialog pattern | Must verify; critical for ModalSurface |
| A4 | FormScreen/TableScreen screen components are in scope for this phase | Component mapping | CONTEXT.md lists 18 components in defaults.ts; screens are not in defaults.ts but listed in code_context |

## Open Questions

1. **Toast Decision D-04 Conflicts with Reality**
   - What we know: D-04 says "shadcn Toast (bits-ui Radix Toast primitive) -- not Sonner." However, bits-ui does NOT have a toast component (confirmed: feature request only, GitHub discussion #1173). shadcn-svelte's toast IS Sonner (`svelte-sonner`).
   - What's unclear: Should we use Sonner (the shadcn-svelte standard), keep the current custom ToastSurface (which works and is SDUI-driven), or build a custom toast on bits-ui primitives?
   - Recommendation: **Keep the current custom ToastSurface** for this phase. It works, it's SDUI-driven (server sends toast events to the `toast` surface), and replacing it with Sonner would break the SDUI pattern. The styling can be updated to use shadcn tokens (already partially done). Revisit in a later phase if needed.
   - [VERIFIED: bits-ui toast is only a feature request -- github.com/huntabyte/bits-ui/discussions/1173] [CITED: shadcn-svelte.com/docs/components/sonner]

2. **Screen Components (FormScreen, TableScreen) Scope**
   - What we know: defaults.ts has 18 registrations. FormScreen and TableScreen are NOT in defaults.ts -- they are rendered differently (via direct import in screen routing, not via SDUI registry).
   - What's unclear: Are they in scope for Phase 11?
   - Recommendation: Include them since CONTEXT.md code_context lists them as "Components to Migrate" and they have hardcoded `text-gray-*` classes that need semantic token replacement. But they are lower priority than the 18 registered components.

3. **FallbackComponent Uses Hardcoded `border-red-500 bg-red-50 text-red-700`**
   - What we know: Dev-only component uses raw color classes.
   - Recommendation: Replace with `border-destructive bg-destructive/10 text-destructive` for consistency, but this is cosmetic and low priority.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 4.1 + vitest-browser-svelte 2.1 + Playwright/Chromium |
| Config file | `frontend/vitest-browser.config.ts` |
| Quick run command | `cd frontend && npx vitest run --config vitest-browser.config.ts --reporter=verbose` |
| Full suite command | `cd frontend && npx vitest run --config vitest-browser.config.ts` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-01-btn | Button renders with shadcn variant, dispatches action | browser | `npx vitest run --config vitest-browser.config.ts src/lib/components/form/Button.browser-test.ts` | Exists (rewrite) |
| COMP-01-input | TextInput renders with shadcn Input, bind works | browser | same pattern | Exists (rewrite) |
| COMP-01-select | SelectInput renders with shadcn Select, bind works | browser | same pattern | Does not exist |
| COMP-01-check | Checkbox renders with shadcn Checkbox, bind works | browser | same pattern | Does not exist |
| COMP-01-form | Form renders children, submits action | browser | same pattern | Does not exist |
| COMP-01-dialog | ModalSurface opens/closes with Dialog | browser | same pattern | Does not exist |
| COMP-01-confirm | ConfirmDialog confirm/cancel dispatch | browser | same pattern | Does not exist |
| COMP-01-toast | ToastSurface displays and dismisses toasts | browser | same pattern | Does not exist |
| COMP-01-table | DataTable renders with shadcn Table, sort works | browser | Exists (rewrite) |
| COMP-01-nav | SideNav/NavItem/NavGroup render correctly | browser | Exists for SideNav (rewrite) |
| COMP-01-layout | Container/Grid/Heading/Text render correctly | browser | Does not exist |
| COMP-01-feedback | Spinner/ErrorDisplay render correctly | browser | Does not exist |
| COMP-01-core | ConnectionBanner/ErrorBoundary/LoadingSkeleton/FallbackComponent | browser | NodeRenderer exists (rewrite), Surface exists (rewrite) |
| COMP-01-screen | FormScreen/TableScreen render correctly | browser | Does not exist |
| COMP-02 | Zero flowbite imports, all icons from @lucide/svelte | grep check | `grep -r "flowbite" frontend/src/ && echo FAIL || echo PASS` | N/A (already passing) |

### Sampling Rate
- **Per task commit:** Quick run of changed component test file
- **Per wave merge:** Full browser test suite
- **Phase gate:** Full suite green + `grep -r "flowbite" frontend/src/` returns nothing

### Wave 0 Gaps
- [ ] 12+ new `.browser-test.ts` files needed (SelectInput, Checkbox, Form, ModalSurface, ConfirmDialog, ToastSurface, Container, Grid, Heading, Text, Spinner, ErrorDisplay, FormScreen, TableScreen)
- [ ] 6 existing `.browser-test.ts` files to rewrite (Button, TextInput, DataTable, SideNav, NodeRenderer, Surface)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A (no auth changes) |
| V3 Session Management | No | N/A (no session changes) |
| V4 Access Control | No | N/A (SDUI is server-driven; access control is backend) |
| V5 Input Validation | Minimal | Server-side validation via SDUI protocol (unchanged) |
| V6 Cryptography | No | N/A |

No security concerns for this phase -- it is a pure UI component reimplementation with no changes to data flow, authentication, or authorization. The SDUI protocol contract (server controls all data and actions) is preserved.

## Sources

### Primary (HIGH confidence)
- Frontend codebase -- all 18+ component files read and analyzed
- `frontend/package.json` -- dependency versions verified
- `frontend/components.json` -- shadcn-svelte CLI config verified
- `frontend/src/app.css` -- OKLCH tokens verified
- `frontend/vitest-browser.config.ts` -- test config verified
- shadcn-svelte CLI tested (`npx shadcn-svelte@latest add -y button`) -- verified output structure

### Secondary (MEDIUM confidence)
- [shadcn-svelte.com/docs/components/dialog](https://www.shadcn-svelte.com/docs/components/dialog) -- Dialog API
- [shadcn-svelte.com/docs/components/select](https://www.shadcn-svelte.com/docs/components/select) -- Select API
- [shadcn-svelte.com/docs/components/checkbox](https://www.shadcn-svelte.com/docs/components/checkbox) -- Checkbox API
- [shadcn-svelte.com/docs/components/sonner](https://www.shadcn-svelte.com/docs/components/sonner) -- Toast (Sonner) API
- [shadcn-svelte.com/docs/components/table](https://www.shadcn-svelte.com/docs/components/table) -- Table API
- [shadcn-svelte.com/docs/components/button](https://www.shadcn-svelte.com/docs/components/button) -- Button API

### Tertiary (LOW confidence)
- [github.com/huntabyte/bits-ui/discussions/1173](https://github.com/huntabyte/bits-ui/discussions/1173) -- bits-ui Toast feature request (confirms no native toast)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all dependencies verified in package.json, CLI tested
- Architecture: HIGH -- all 18+ components read, patterns derived from actual code
- Pitfalls: HIGH -- 7 pitfalls identified from actual codebase analysis (Grid bug, LoadingSkeleton colors, WithElementRef missing, toast dual state)
- Component mapping: MEDIUM -- shadcn API details from docs, not hands-on testing of every component

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (30 days -- stable ecosystem)
