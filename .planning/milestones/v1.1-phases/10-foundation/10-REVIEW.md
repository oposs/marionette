---
phase: 10-foundation
reviewed: 2026-04-09T12:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - frontend/src/app.css
  - frontend/src/lib/components/core/ConnectionBanner.svelte
  - frontend/src/lib/components/core/ErrorBoundary.svelte
  - frontend/src/lib/components/core/Surface.svelte
  - frontend/src/lib/components/feedback/ErrorDisplay.svelte
  - frontend/src/lib/components/feedback/Spinner.svelte
  - frontend/src/lib/components/form/Button.svelte
  - frontend/src/lib/components/form/Checkbox.svelte
  - frontend/src/lib/components/form/Form.svelte
  - frontend/src/lib/components/form/SelectInput.svelte
  - frontend/src/lib/components/form/TextInput.svelte
  - frontend/src/lib/components/layout/Container.svelte
  - frontend/src/lib/components/nav/NavGroup.svelte
  - frontend/src/lib/components/nav/NavItem.svelte
  - frontend/src/lib/components/nav/SideNav.svelte
  - frontend/src/lib/components/popup/ConfirmDialog.svelte
  - frontend/src/lib/components/popup/ModalSurface.svelte
  - frontend/src/lib/components/popup/ToastSurface.svelte
  - frontend/src/lib/components/table/DataTable.svelte
  - frontend/src/lib/utils.ts
  - frontend/components.json
  - frontend/package.json
findings:
  critical: 1
  warning: 5
  info: 2
  total: 8
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-04-09
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

The Flowbite-to-shadcn-svelte migration is clean -- no residual Flowbite imports or dependencies remain. The OKLCH token system in `app.css` is well-structured with proper light/dark mode definitions. Component stubs correctly preserve existing business logic (actions, data binding, dirty tracking). However, two CSS token references are missing from the theme definition, the modal lacks ARIA roles for accessibility, toast colors break in dark mode, and the Spinner uses a dynamic Tailwind class that will be purged at build time.

## Critical Issues

### CR-01: Missing `--destructive-foreground` CSS token causes invisible button text

**File:** `frontend/src/app.css` (missing definition, referenced in multiple components)
**Issue:** `text-destructive-foreground` is used in `Button.svelte:43` and `ConfirmDialog.svelte:54` but neither `--destructive-foreground` nor `--color-destructive-foreground` is defined anywhere in `app.css`. In Tailwind v4, `text-destructive-foreground` resolves to `var(--color-destructive-foreground)` which is undefined, so the destructive buttons will have no text color (inheriting transparent or black depending on browser), making them unreadable.
**Fix:** Add the token to both `:root` and `.dark` blocks in `app.css`, and add the mapping in `@theme inline`:

```css
/* In :root */
--destructive-foreground: oklch(0.985 0 0);

/* In .dark */
--destructive-foreground: oklch(0.985 0 0);

/* In @theme inline */
--color-destructive-foreground: var(--destructive-foreground);
```

## Warnings

### WR-01: Missing `--ring-offset-background` token breaks focus ring styling

**File:** `frontend/src/lib/components/form/SelectInput.svelte:55`, `frontend/src/lib/components/form/TextInput.svelte:67`
**Issue:** Both inputs use `ring-offset-background` and `focus-visible:ring-offset-2` classes, but `--ring-offset-background` / `--color-ring-offset-background` is not defined in `app.css`. The ring offset will render as transparent, causing the focus ring to appear flush against the input border instead of offset.
**Fix:** Either add the token to `app.css`:
```css
/* In :root and .dark, and @theme inline */
--ring-offset-background: var(--background);
--color-ring-offset-background: var(--ring-offset-background);
```
Or replace `ring-offset-background` with `ring-offset-background` using the already-defined `bg-background` token approach: change to `ring-offset-[var(--background)]`.

### WR-02: Spinner dynamic class `size-{value}` will be purged by Tailwind

**File:** `frontend/src/lib/components/feedback/Spinner.svelte:21`
**Issue:** The class `size-{(props.size as string) ?? '6'}` produces dynamic class names like `size-4`, `size-8` at runtime. Tailwind v4 cannot detect these at build time, so they will be purged from the CSS output. Only `size-6` (the default) might work if used elsewhere statically.
**Fix:** Use a safelist in `app.css` (similar to the Grid.svelte safelist already in place), or use a lookup map:
```svelte
<script>
const sizeMap: Record<string, string> = {
  '4': 'size-4',
  '6': 'size-6',
  '8': 'size-8',
  '10': 'size-10',
};
let sizeClass = $derived(sizeMap[(props.size as string) ?? '6'] ?? 'size-6');
</script>

<svg class="animate-spin text-primary {sizeClass}" ...>
```

### WR-03: ModalSurface lacks ARIA role and focus trap

**File:** `frontend/src/lib/components/popup/ModalSurface.svelte:18-36`
**Issue:** The modal overlay and dialog have no `role="dialog"` or `aria-modal="true"` attributes. Screen readers will not announce the modal. Additionally, focus is not trapped inside the modal -- Tab key can move focus to elements behind the backdrop. The `a11y_no_static_element_interactions` ignore comments indicate the linter flagged this.
**Fix:** Add ARIA attributes to the inner dialog div and consider a focus trap:
```svelte
<div
  class="relative w-full max-w-md rounded-lg bg-background shadow-lg"
  role="dialog"
  aria-modal="true"
  aria-labelledby={rootProps.title ? 'modal-title' : undefined}
  onclick={(e) => e.stopPropagation()}
  onkeydown={() => {}}
>
```

### WR-04: Toast hardcoded colors break in dark mode

**File:** `frontend/src/lib/components/popup/ToastSurface.svelte:14-16`
**Issue:** The `success` and `warning` severity classes use hardcoded light-mode Tailwind colors (`bg-green-50 text-green-800`, `bg-yellow-50 text-yellow-800`). In dark mode these will appear as bright light patches against a dark background, and the dark text will have poor contrast against a dark theme.
**Fix:** Use OKLCH semantic tokens or add dark-mode variants:
```typescript
const severityClass: Record<string, string> = {
  success: 'border-green-500/30 bg-green-50 text-green-800 dark:bg-green-950 dark:text-green-200',
  error: 'border-destructive/30 bg-destructive/10 text-destructive',
  warning: 'border-yellow-500/30 bg-yellow-50 text-yellow-800 dark:bg-yellow-950 dark:text-yellow-200',
  info: 'border-border bg-background text-foreground',
};
```

### WR-05: ErrorBoundary `reset` callback is unused

**File:** `frontend/src/lib/components/core/ErrorBoundary.svelte:12`
**Issue:** The `{#snippet failed(error, reset)}` receives a `reset` function but never exposes it in the UI. Users cannot recover from a render error without refreshing the entire page. The error message says "Try refreshing the page" but a reset button would be more user-friendly and is the standard pattern for Svelte error boundaries.
**Fix:** Add a retry button:
```svelte
{#snippet failed(error, reset)}
  <div class="border border-destructive/40 bg-destructive/10 p-4 rounded-md">
    <div class="flex items-center gap-2">
      <AlertCircle class="size-5 text-destructive" />
      <p class="text-destructive text-sm">Something went wrong rendering this component.</p>
    </div>
    <button
      class="mt-2 text-sm text-destructive underline hover:no-underline"
      onclick={reset}
    >
      Try again
    </button>
  </div>
{/snippet}
```

## Info

### IN-01: `cn` utility imported but unused in all reviewed components

**File:** `frontend/src/lib/utils.ts`
**Issue:** The `cn()` utility (clsx + tailwind-merge) is defined but none of the 19 reviewed components import or use it. All components use manual string concatenation for classes instead. This means `tailwind-merge` and `clsx` are unused dependencies. The utility should either be adopted or removed in a future pass.
**Fix:** Consider adopting `cn()` in components that concatenate conditional classes (e.g., Button, TextInput, DataTable) to avoid class conflicts.

### IN-02: SelectInput uses `selected` attribute instead of reactive value binding

**File:** `frontend/src/lib/components/form/SelectInput.svelte:58-61`
**Issue:** The `<select>` does not use a `value` binding. Instead, individual `<option>` elements use `selected={opt.value === value}`. While this works, it is less idiomatic in Svelte and can lead to subtle issues if the derived value changes after initial render -- the DOM `selected` attribute is only set on mount, not reactively updated in all browsers.
**Fix:** Bind the value directly on the `<select>`:
```svelte
<select value={value} onchange={handleChange} ...>
```

---

_Reviewed: 2026-04-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
