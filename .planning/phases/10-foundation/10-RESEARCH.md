# Phase 10: Foundation - Research

**Researched:** 2026-04-08
**Domain:** shadcn-svelte initialization, OKLCH CSS theming, Flowbite removal
**Confidence:** HIGH

## Summary

This phase replaces Flowbite with shadcn-svelte as the sole component framework. The work has three clear stages: (1) run `npx shadcn-svelte@latest init` to scaffold the shadcn infrastructure (components.json, utils.ts with `cn()` helper, base dependencies), (2) rewrite `app.css` from Flowbite plugin + primary color mapping to OKLCH semantic tokens with `@theme inline`, and (3) replace all 19 Flowbite import sites with minimal HTML+Tailwind stubs, then remove Flowbite packages.

The codebase has 14 unique component files importing from `flowbite-svelte` or `flowbite-svelte-icons`, plus `app.css` and `+layout.svelte`. Each Flowbite component is used in a thin wrapper pattern -- the SDUI interface contract (`surface`, `props`, `bind`, `action`) stays unchanged; only the rendering internals get stubbed. This is a clean mechanical replacement.

**Primary recommendation:** Run shadcn-svelte init first, then rewrite app.css with OKLCH Zinc tokens, then stub components file-by-file (each must compile), then remove Flowbite packages as the final step.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use **Default** base style (not New York) -- rounded, spacious, standard business app aesthetics
- **D-02:** Border radius set to **0.25rem** -- subtle rounding for a professional/technical look
- **D-03:** Use **CSS variables mode** for theming -- enables runtime theme switching, standard shadcn approach
- **D-04:** Primary color: **Zinc/Neutral** -- understated, lets content speak, clean professional look
- **D-05:** Dark mode: **Claude's Discretion** -- pragmatic approach (define tokens if easy, don't block on toggle wiring)
- **D-06:** **Stub-first approach** -- replace each Flowbite import with a minimal HTML+Tailwind stub that compiles, then remove Flowbite packages
- **D-07:** **Drop Flowbite's dark mode** `@custom-variant dark` -- shadcn handles dark mode via its own class strategy
- **D-08:** Stubs are **minimal HTML + Tailwind** -- just enough to compile and render something visible
- **D-09:** Stubs **preserve the full SDUI interface contract** (`surface`, `props`, `bind`, `action`) -- backend doesn't change

### Claude's Discretion
- Dark mode token definition and toggle wiring (D-05) -- Claude picks the pragmatic approach based on effort
- Any ordering details within the stub-first removal process

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FOUND-01 | shadcn-svelte CLI initialized with bits-ui, lucide-svelte, tw-animate-css, clsx+tailwind-merge dependencies installed | Standard Stack section: exact packages and versions verified via npm registry |
| FOUND-02 | app.css rewritten with OKLCH semantic color tokens and shadcn theme system (no Flowbite plugin) | Architecture Patterns section: complete OKLCH Zinc token set and @theme inline block from official docs |
| FOUND-03 | All Flowbite packages removed with zero residual imports | Flowbite Import Inventory: complete list of 14 files + 2 config sites needing changes |
</phase_requirements>

## Standard Stack

### Add (via shadcn-svelte init + manual)

| Package | Version | Purpose | Why Standard | Confidence |
|---------|---------|---------|--------------|------------|
| shadcn-svelte (CLI only) | 1.2.7 | Scaffolding tool | Copies component source into project; not a runtime dependency | HIGH [VERIFIED: npm registry] |
| bits-ui | ^2.17.3 | Headless accessible primitives | Installed automatically by shadcn-svelte CLI; runtime dependency for all shadcn components | HIGH [VERIFIED: npm registry] |
| @lucide/svelte | ^1.7.0 | Icon library | Replaces flowbite-svelte-icons; shadcn-svelte default icon set | HIGH [VERIFIED: npm registry] |
| clsx | ^2.1.1 | Conditional class strings | Used by `cn()` helper; standard shadcn pattern | HIGH [VERIFIED: npm registry] |
| tailwind-merge | ^3.5.0 | Tailwind class deduplication | Used by `cn()` helper to merge conflicting Tailwind classes | HIGH [VERIFIED: npm registry] |
| tw-animate-css | ^1.4.0 | Animation utilities | Required by shadcn-svelte theming; replaces tailwindcss-animate for Tailwind v4 | HIGH [VERIFIED: npm registry] |

### Remove

| Package | Current Version | Reason |
|---------|----------------|--------|
| flowbite-svelte | ^1.31.0 | Replaced entirely by shadcn-svelte stubs |
| flowbite-svelte-icons | ^3.1.0 | Replaced by @lucide/svelte |

### Keep (No Changes)

| Package | Notes |
|---------|-------|
| tailwindcss ^4.2 | Already compatible; shadcn-svelte uses CSS-native theming |
| @tailwindcss/vite ^4.2 | Already installed; no plugin changes needed |
| svelte ^5.53 | Already compatible with shadcn-svelte and bits-ui |
| All other existing deps | No changes |

**Installation:**
```bash
cd frontend
npx shadcn-svelte@latest init
# CLI installs: bits-ui, clsx, tailwind-merge, tw-animate-css, @lucide/svelte
# Then after stubs are complete:
npm uninstall flowbite-svelte flowbite-svelte-icons
```

## Architecture Patterns

### shadcn-svelte Init Artifacts

Running `npx shadcn-svelte@latest init` in the `frontend/` directory creates: [CITED: shadcn-svelte.com/docs/installation/sveltekit]

1. **`components.json`** -- CLI configuration file at project root
2. **`$lib/utils.ts`** -- The `cn()` class merge helper
3. **Overwrites `app.css`** -- Replaces with shadcn theme variables (we will customize this)

The CLI prompts for:
- Base color: select **Zinc** (per D-04)
- CSS file: `src/app.css`
- Import aliases: keep defaults (`$lib`, `$lib/components`, `$lib/utils`)

**components.json expected structure:** [ASSUMED]
```json
{
  "$schema": "https://shadcn-svelte.com/schema.json",
  "tailwind": {
    "css": "src/app.css",
    "baseColor": "zinc"
  },
  "aliases": {
    "lib": "$lib",
    "components": "$lib/components",
    "utils": "$lib/utils",
    "ui": "$lib/components/ui",
    "hooks": "$lib/hooks"
  },
  "typescript": true
}
```

**`$lib/utils.ts` content (standard across all shadcn):** [CITED: shadcn-svelte.com/docs/theming]
```typescript
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(...inputs));
}
```

### app.css Rewrite (OKLCH Zinc Theme)

The new `app.css` must contain these sections: [CITED: shadcn-svelte.com/docs/theming, shadcn-svelte.com/docs/migration/tailwind-v4]

```css
@import 'tailwindcss';
@import 'tw-animate-css';

@custom-variant dark (&:is(.dark *));

/* Safelist dynamic grid-cols classes used by Grid.svelte */
@source inline("grid-cols-1 grid-cols-2 grid-cols-3 grid-cols-4 grid-cols-5 grid-cols-6 md:grid-cols-1 md:grid-cols-2 md:grid-cols-3 md:grid-cols-4 md:grid-cols-5 md:grid-cols-6");

:root {
  --radius: 0.25rem;
  --background: oklch(1 0 0);
  --foreground: oklch(0.141 0.005 285.823);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.141 0.005 285.823);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.141 0.005 285.823);
  --primary: oklch(0.21 0.006 285.885);
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.967 0.001 286.375);
  --secondary-foreground: oklch(0.21 0.006 285.885);
  --muted: oklch(0.967 0.001 286.375);
  --muted-foreground: oklch(0.552 0.016 285.938);
  --accent: oklch(0.967 0.001 286.375);
  --accent-foreground: oklch(0.21 0.006 285.885);
  --destructive: oklch(0.577 0.245 27.325);
  --border: oklch(0.92 0.004 286.32);
  --input: oklch(0.92 0.004 286.32);
  --ring: oklch(0.705 0.015 286.067);
  --sidebar-background: oklch(0.985 0 0);
  --sidebar-foreground: oklch(0.141 0.005 285.823);
  --sidebar-primary: oklch(0.21 0.006 285.885);
  --sidebar-primary-foreground: oklch(0.985 0 0);
  --sidebar-accent: oklch(0.967 0.001 286.375);
  --sidebar-accent-foreground: oklch(0.21 0.006 285.885);
  --sidebar-border: oklch(0.92 0.004 286.32);
  --sidebar-ring: oklch(0.705 0.015 286.067);
}

.dark {
  --background: oklch(0.141 0.005 285.823);
  --foreground: oklch(0.985 0 0);
  --card: oklch(0.21 0.006 285.885);
  --card-foreground: oklch(0.985 0 0);
  --popover: oklch(0.21 0.006 285.885);
  --popover-foreground: oklch(0.985 0 0);
  --primary: oklch(0.92 0.004 286.32);
  --primary-foreground: oklch(0.21 0.006 285.885);
  --secondary: oklch(0.274 0.006 286.033);
  --secondary-foreground: oklch(0.985 0 0);
  --muted: oklch(0.274 0.006 286.033);
  --muted-foreground: oklch(0.705 0.015 286.067);
  --accent: oklch(0.274 0.006 286.033);
  --accent-foreground: oklch(0.985 0 0);
  --destructive: oklch(0.704 0.191 22.216);
  --border: oklch(1 0 0 / 10%);
  --input: oklch(1 0 0 / 15%);
  --ring: oklch(0.552 0.016 285.938);
  --sidebar-background: oklch(0.21 0.006 285.885);
  --sidebar-foreground: oklch(0.985 0 0);
  --sidebar-primary: oklch(0.985 0 0);
  --sidebar-primary-foreground: oklch(0.21 0.006 285.885);
  --sidebar-accent: oklch(0.274 0.006 286.033);
  --sidebar-accent-foreground: oklch(0.985 0 0);
  --sidebar-border: oklch(1 0 0 / 10%);
  --sidebar-ring: oklch(0.552 0.016 285.938);
}

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --color-sidebar-background: var(--sidebar-background);
  --color-sidebar-foreground: var(--sidebar-foreground);
  --color-sidebar-primary: var(--sidebar-primary);
  --color-sidebar-primary-foreground: var(--sidebar-primary-foreground);
  --color-sidebar-accent: var(--sidebar-accent);
  --color-sidebar-accent-foreground: var(--sidebar-accent-foreground);
  --color-sidebar-border: var(--sidebar-border);
  --color-sidebar-ring: var(--sidebar-ring);
  --radius-sm: calc(var(--radius) - 0.125rem);
  --radius-md: var(--radius);
  --radius-lg: calc(var(--radius) + 0.125rem);
  --radius-xl: calc(var(--radius) + 0.25rem);
}
```

**Key notes:**
- `--radius: 0.25rem` per D-02
- Dark mode tokens defined (per D-05 discretion: defining them costs nothing, actual toggle wiring deferred)
- `@custom-variant dark (&:is(.dark *))` is shadcn's approach (D-07: drops Flowbite's `&:where(.dark, .dark *)`)
- Grid safelist classes preserved from current app.css
- No `@plugin "flowbite/plugin"` or `@source` Flowbite directives

### Flowbite Color Class to Semantic Token Mapping

Hardcoded Flowbite/raw color classes in component stubs must use semantic tokens: [VERIFIED: codebase grep]

| Current Raw Class | Semantic Replacement |
|-------------------|---------------------|
| `bg-white` | `bg-background` |
| `bg-gray-50` | `bg-muted` |
| `bg-gray-100` | `bg-muted` |
| `text-gray-500` | `text-muted-foreground` |
| `text-gray-600` | `text-muted-foreground` |
| `text-gray-700` | `text-foreground` |
| `text-gray-900` | `text-foreground` |
| `border-gray-200` | `border-border` |
| `bg-red-50` | `bg-destructive/10` |
| `text-red-600` | `text-destructive` |
| `bg-yellow-50` | `bg-accent` (or keep raw for connection banner) |
| `hover:bg-gray-50` | `hover:bg-accent` |
| `hover:bg-gray-100` | `hover:bg-accent` |

### Surface.svelte Color Updates

`Surface.svelte` has hardcoded layout classes that must update: [VERIFIED: codebase read]
```
main: 'bg-white ...' -> 'bg-background ...'
sidebar: 'bg-gray-50 border-r border-gray-200 ...' -> 'bg-sidebar-background border-r border-sidebar-border ...'
```

## Flowbite Import Inventory

Complete list of files needing stub replacement: [VERIFIED: grep of frontend/src]

### Files importing from `flowbite-svelte` (11 files)

| File | Flowbite Imports | Stub Complexity |
|------|-----------------|-----------------|
| `components/core/ConnectionBanner.svelte` | `Spinner` | Trivial: SVG spinner animation |
| `components/form/Button.svelte` | `Button`, `ButtonProps` | Simple: `<button>` with Tailwind classes |
| `components/form/TextInput.svelte` | `Input`, `Label`, `Helper` | Simple: `<input>`, `<label>`, `<p>` elements |
| `components/form/SelectInput.svelte` | `Select`, `Label` | Simple: `<select>`, `<label>` elements |
| `components/form/Checkbox.svelte` | `Checkbox` | Simple: `<input type="checkbox">` with label |
| `components/form/Form.svelte` | `Helper` | Trivial: `<p>` with error styling |
| `components/nav/SideNav.svelte` | `Sidebar`, `SidebarWrapper` | Trivial: `<nav>` + `<div>` wrapper |
| `components/nav/NavItem.svelte` | `SidebarItem` | Simple: `<a>` with active state classes |
| `components/nav/NavGroup.svelte` | `SidebarGroup` | Trivial: `<div>` wrapper |
| `components/layout/Container.svelte` | `Card` | Simple: `<div>` with border/shadow |
| `components/feedback/ErrorDisplay.svelte` | `Alert` | Simple: `<div>` with red background |
| `components/feedback/Spinner.svelte` | `Spinner` | Trivial: SVG spinner animation |
| `components/popup/ModalSurface.svelte` | `Modal`, `ModalProps` | Medium: overlay + centered dialog |
| `components/popup/ToastSurface.svelte` | `Toast`, `ToastProps` | Simple: colored `<div>` with dismiss |
| `components/popup/ConfirmDialog.svelte` | `Modal`, `Button` | Medium: reuses ModalSurface pattern |

### Files importing from `flowbite-svelte-icons` (4 files)

| File | Icons Used | Lucide Equivalent |
|------|-----------|-------------------|
| `routes/+layout.svelte` | `BarsOutline`, `CloseOutline` | `Menu`, `X` |
| `components/core/ErrorBoundary.svelte` | `ExclamationCircleOutline` | `AlertCircle` |
| `components/feedback/ErrorDisplay.svelte` | `ExclamationCircleOutline` | `AlertCircle` |
| `components/table/DataTable.svelte` | `ChevronUpOutline`, `ChevronDownOutline` | `ChevronUp`, `ChevronDown` |

### CSS/Config (2 files)

| File | What to Change |
|------|---------------|
| `app.css` | Remove `@plugin "flowbite/plugin"`, `@source` Flowbite directives, `@custom-variant dark` (Flowbite style) |
| `package.json` | Remove `flowbite-svelte`, `flowbite-svelte-icons` from dependencies |

**Total: 16 files need changes** (14 components + app.css + package.json)

### Files with NO Flowbite imports (no changes needed)

These component files use only Tailwind classes and internal imports:
- `Grid.svelte`, `Heading.svelte`, `Text.svelte` -- pure HTML + Tailwind
- `FormScreen.svelte`, `TableScreen.svelte` -- use NodeRenderer only
- `Surface.svelte` -- hardcoded color classes need semantic token update (see above)
- `NodeRenderer.svelte`, `LoadingSkeleton.svelte` -- no Flowbite

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Class merging | Manual string concat | `cn()` from clsx + tailwind-merge | Handles Tailwind specificity conflicts correctly |
| Accessible dialog | Custom modal overlay | bits-ui Dialog (Phase 11) | Focus trapping, escape handling, ARIA attributes |
| Spinner animation | Complex CSS animation | Simple SVG + `animate-spin` class | Standard Tailwind pattern, 3 lines |
| Icon system | Custom SVG components | @lucide/svelte | 1500+ icons, consistent sizing, tree-shakeable |

**Key insight:** In this phase, stubs are intentionally simple HTML+Tailwind. The "don't hand-roll" items become relevant in Phase 11 when stubs are replaced with proper shadcn-svelte primitives.

## Common Pitfalls

### Pitfall 1: Running shadcn init overwrites app.css

**What goes wrong:** `npx shadcn-svelte@latest init` overwrites `app.css` with its own template, losing the grid safelist classes.
**Why it happens:** The CLI generates a fresh CSS file as part of initialization.
**How to avoid:** Run init first, then manually add back the `@source inline(...)` safelist line. Or save the safelist before init and re-add it after.
**Warning signs:** `grid-cols-{n}` classes stop working in Grid.svelte.

### Pitfall 2: Flowbite Tailwind plugin conflicts with shadcn tokens

**What goes wrong:** If `@plugin "flowbite/plugin"` is still in app.css when shadcn tokens are added, Flowbite's color definitions clash with shadcn's semantic tokens.
**Why it happens:** Both define `--color-primary` and similar tokens.
**How to avoid:** Remove Flowbite plugin line BEFORE adding shadcn tokens. Do both in the same app.css rewrite.
**Warning signs:** Unexpected colors, especially on primary/secondary buttons.

### Pitfall 3: Stub compiles but breaks SDUI contract

**What goes wrong:** A stub replaces a Flowbite component but drops the `oninput`, `onchange`, or `onclick` handler, breaking data binding or actions.
**Why it happens:** Focusing on visual output and forgetting the event wiring that makes SDUI work.
**How to avoid:** Every stub MUST preserve: (1) event handlers that call `setData`/`sendAction`, (2) `$derived` bindings from `getData`, (3) the full props interface. The business logic stays identical; only the rendering HTML changes.
**Warning signs:** Form fields don't update data store, buttons don't fire actions.

### Pitfall 4: Forgetting to update Surface.svelte hardcoded colors

**What goes wrong:** Surface.svelte has `bg-white`, `bg-gray-50`, `border-gray-200` in its layout classes map. After removing Flowbite, these raw colors still work but don't respond to dark mode.
**Why it happens:** Surface.svelte doesn't import Flowbite so it doesn't show up in "flowbite" greps.
**How to avoid:** Update `layoutClasses` in Surface.svelte to use semantic tokens (`bg-background`, `bg-sidebar-background`, `border-sidebar-border`).
**Warning signs:** Sidebar and main area ignore dark mode theme.

### Pitfall 5: ModalSurface needs overlay + portal behavior

**What goes wrong:** A naive stub replaces Flowbite's `<Modal>` with just a `<div>` but loses the backdrop overlay, centering, and escape-to-close behavior.
**Why it happens:** Flowbite's Modal handles a lot: portal rendering, backdrop, focus trap, close-on-escape.
**How to avoid:** The stub must include: (1) fixed position overlay with `bg-black/50`, (2) centered content div, (3) onclick on backdrop to close, (4) conditional rendering via `{#if isOpen}`. Full accessibility comes in Phase 11 with bits-ui Dialog.
**Warning signs:** Modal content renders inline instead of as an overlay.

## Code Examples

### Stub Pattern: Button (representative example)

```svelte
<!-- Source: derived from existing Button.svelte, replacing FlowbiteButton -->
<script lang="ts">
	import { sendAction } from '$lib/transport/dispatcher';
	import { getAllData } from '$lib/store/data.svelte';
	import type { ComponentAction, PatchOperation } from '$lib/transport/messages';
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

	function handleClick() {
		if (action) {
			const optimisticField = action.optimistic as
				| { patch: PatchOperation[] }
				| undefined;
			const surfaceData = getAllData(surface) ?? {};
			const payload = {
				...(action.payload as Record<string, unknown> ?? {}),
				...surfaceData
			};
			sendAction(
				action.name ?? action.type,
				payload,
				action.target,
				optimisticField ? { patch: optimisticField.patch } : undefined
			);
		}
	}

	let colorClass = $derived(
		(props.color as string) === 'red'
			? 'bg-destructive text-destructive-foreground hover:bg-destructive/90'
			: (props.outline as boolean)
				? 'border border-input bg-background hover:bg-accent hover:text-accent-foreground'
				: 'bg-primary text-primary-foreground hover:bg-primary/90'
	);
</script>

<button
	type="button"
	class="inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 w-full md:w-auto disabled:opacity-50 disabled:pointer-events-none {colorClass}"
	disabled={props.disabled as boolean}
	onclick={handleClick}
>
	{props.label ?? ''}
</button>
```

### Stub Pattern: TextInput (form field with data binding)

```svelte
<!-- Source: derived from existing TextInput.svelte -->
<script lang="ts">
	import { getData, setData } from '$lib/store/data.svelte';
	import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
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

	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);

	function handleInput(e: Event) {
		if (bind) {
			const target = e.currentTarget as HTMLInputElement;
			setData(surface, bind, target.value);
		}
	}

	function handleFocus() {
		if (bind) markDirty(bind);
	}

	function handleBlur() {
		if (bind) {
			clearDirty(bind, (op) => setData(surface, op.path, op.value));
			if (action?.type === 'blur') {
				sendAction(
					action.name ?? action.type,
					{ value: getData(surface, bind!) },
					action.target
				);
			}
		}
	}
</script>

<div class="w-full">
	{#if props.label}
		<label class="mb-2 block text-sm font-medium text-foreground">
			{props.label}
		</label>
	{/if}
	<input
		type={(props.type as string) ?? 'text'}
		placeholder={props.placeholder as string}
		required={props.required as boolean}
		disabled={props.disabled as boolean}
		{value}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={handleBlur}
		class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {fieldError ? 'border-destructive' : ''}"
	/>
	{#if fieldError}
		<p class="mt-1 text-sm text-destructive">{fieldError}</p>
	{:else if props.helperText}
		<p class="mt-1 text-sm text-muted-foreground">{props.helperText}</p>
	{/if}
</div>
```

### Icon Replacement Pattern

```svelte
<!-- Before (Flowbite) -->
<script>
	import { ChevronUpOutline } from 'flowbite-svelte-icons';
</script>
<ChevronUpOutline class="w-4 h-4 inline" />

<!-- After (Lucide) -->
<script>
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
</script>
<ChevronUp class="size-4 inline" />
```

### Icon Name Mapping (Complete)

```typescript
// flowbite-svelte-icons -> @lucide/svelte
// Import pattern: import IconName from '@lucide/svelte/icons/icon-name'
const iconMap = {
	'BarsOutline': 'menu',           // Menu (hamburger)
	'CloseOutline': 'x',             // X (close)
	'ExclamationCircleOutline': 'alert-circle',  // AlertCircle
	'ChevronUpOutline': 'chevron-up',            // ChevronUp
	'ChevronDownOutline': 'chevron-down',        // ChevronDown
};
```

## Dark Mode Recommendation (D-05 -- Claude's Discretion)

**Recommendation: Define dark mode tokens, skip toggle wiring.**

Rationale:
- The OKLCH Zinc dark mode token set is already documented above (from official shadcn-svelte docs)
- Including `.dark { ... }` in app.css costs zero runtime effort
- The `@custom-variant dark (&:is(.dark *))` directive is already in shadcn's standard template
- Semantic token classes like `bg-background` automatically respect dark mode when `.dark` class is on `<html>`
- No toggle UI or localStorage wiring needed in this phase -- that's a future concern
- Stubs that use semantic tokens (instead of raw colors) will automatically support dark mode when a toggle is added later

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | vitest 4.1 + vitest-browser-svelte 2.1 (Playwright/Chromium) |
| Config file | `frontend/vitest-browser.config.ts` (browser), `frontend/vite.config.ts` (unit) |
| Quick run command | `cd frontend && npx vitest run --config vitest-browser.config.ts` |
| Full suite command | `cd frontend && npx vitest run && npx vitest run --config vitest-browser.config.ts` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FOUND-01 | shadcn-svelte artifacts exist (components.json, utils.ts, cn()) | smoke | `test -f frontend/components.json && test -f frontend/src/lib/utils.ts` | N/A (file check) |
| FOUND-02 | app.css uses OKLCH tokens, no Flowbite plugin refs | smoke | `grep -c 'oklch' frontend/src/app.css && ! grep -q 'flowbite' frontend/src/app.css` | N/A (grep check) |
| FOUND-03 | Zero Flowbite imports in source | smoke | `! grep -rq 'flowbite' frontend/src/` | N/A (grep check) |
| ALL | Frontend compiles and dev server starts | build | `cd frontend && npm run build` | N/A (build check) |
| ALL | Existing unit tests still pass | unit | `cd frontend && npx vitest run` | Yes (6 test files) |
| ALL | Existing browser tests still pass | browser | `cd frontend && npx vitest run --config vitest-browser.config.ts` | Yes (6 test files) |

### Sampling Rate
- **Per task commit:** `cd frontend && npm run build` (must succeed)
- **Per wave merge:** Full test suite (unit + browser)
- **Phase gate:** `npm run build` + `grep -rq flowbite frontend/src/` returns nothing + all tests green

### Wave 0 Gaps
None -- existing test infrastructure covers all phase requirements. No new test files needed for this phase since validation is primarily build success + absence of Flowbite references.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | components.json schema for Tailwind v4 projects includes `tailwind.baseColor: "zinc"` | Architecture Patterns | Low -- CLI generates this interactively, exact field names may differ slightly |
| A2 | `@lucide/svelte` uses `import X from '@lucide/svelte/icons/x'` import pattern | Code Examples | Medium -- import path structure may differ; verify after install |
| A3 | Dark mode sidebar tokens (`--sidebar-*`) are part of standard Zinc theme | Architecture Patterns | Low -- sidebar tokens are standard shadcn but exact OKLCH values may differ |
| A4 | `tw-animate-css` is auto-installed by shadcn-svelte CLI init | Standard Stack | Low -- if not, manual `npm install tw-animate-css` is trivial |

## Open Questions

1. **Exact shadcn-svelte init behavior with existing app.css**
   - What we know: CLI overwrites app.css with its template
   - What's unclear: Does it merge or fully replace? Does it detect Tailwind v4 automatically?
   - Recommendation: Back up app.css before init, then merge safelist line after

2. **ConfirmDialog stub shares ModalSurface overlay logic**
   - What we know: ConfirmDialog currently renders inside ModalSurface's `<Modal>` wrapper
   - What's unclear: Should the stub have its own overlay or rely on ModalSurface?
   - Recommendation: ConfirmDialog renders inside ModalSurface (current pattern) -- it's a content component, not a standalone overlay. Keep as-is but replace Flowbite Button imports with stub buttons.

## Sources

### Primary (HIGH confidence)
- [shadcn-svelte theming docs](https://www.shadcn-svelte.com/docs/theming) -- OKLCH Zinc token values
- [shadcn-svelte Tailwind v4 migration](https://www.shadcn-svelte.com/docs/migration/tailwind-v4) -- @theme inline pattern, tw-animate-css
- [shadcn-svelte installation](https://www.shadcn-svelte.com/docs/installation/sveltekit) -- init process, CLI prompts
- [shadcn-svelte components.json](https://www.shadcn-svelte.com/docs/components-json) -- configuration schema
- npm registry -- all package versions verified 2026-04-08

### Secondary (MEDIUM confidence)
- [shadcn/ui Tailwind v4 docs](https://ui.shadcn.com/docs/tailwind-v4) -- @theme inline block structure (React version, adapted for Svelte)
- Codebase grep -- all 16 Flowbite import sites verified

### Tertiary (LOW confidence)
- Dark mode sidebar token OKLCH values -- extrapolated from Zinc light mode values and shadcn convention

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all versions verified against npm registry
- Architecture: HIGH -- OKLCH tokens from official shadcn-svelte docs, app.css pattern from migration guide
- Flowbite inventory: HIGH -- verified by grep, all 16 files identified
- Stub patterns: HIGH -- derived directly from existing component source code
- Pitfalls: HIGH -- based on codebase analysis and prior research in PITFALLS.md

**Research date:** 2026-04-08
**Valid until:** 2026-05-08 (stable: shadcn-svelte and Tailwind v4 are mature)
