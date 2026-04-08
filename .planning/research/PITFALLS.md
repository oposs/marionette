# Pitfalls Research: v1.1 shadcn-svelte Migration

**Project:** Marionette v1.1 — shadcn-svelte + High-Level Components
**Researched:** 2026-04-08
**Confidence:** MEDIUM-HIGH

## Executive Summary

12 pitfalls identified across migration, SDUI integration, virtual scroll, and responsive layout. 5 critical, 5 moderate, 2 minor. The biggest risks are: CSS variable conflicts during Flowbite removal, Sidebar context clashing with Surface rendering, and TanStack Virtual's Svelte 5 incompatibility.

---

## Critical Pitfalls

### 1. Flowbite API Scattered Across 15+ Files

**Risk:** Flowbite uses config-based API (props like `variant`, `size`), shadcn-svelte uses composition patterns (`Dialog.Root`/`Trigger`/`Content`). Different paradigms require rewriting component internals, not just swapping imports.

**Prevention:**
- Map all Flowbite imports before coding: `grep -r "flowbite" frontend/src/`
- Swap in one phase, not incrementally
- Remove Flowbite entirely — both libraries cannot coexist (Tailwind plugin conflicts)

**Detection:** `grep -r "flowbite" frontend/src/` should return zero hits after migration.

**Phase:** Component migration phase

### 2. shadcn-svelte Sidebar Context vs SDUI Surface Clash

**Risk:** shadcn Sidebar needs `Sidebar.Provider` context; Marionette uses `surfaceState["sidebar"]`. Known issues: `useSidebar()` fails in layout components (GitHub #1563), `$derived.by` errors need bits-ui >= 1.3.5 (GitHub #1711).

**Prevention:**
- AppShell SDUI component owns the Provider
- Surfaces render inside the AppShell, not alongside it
- Replace `sidebar.svelte.ts` store with shadcn's built-in state
- Ensure bits-ui >= 1.3.5

**Phase:** AppShell phase

### 3. Virtual Scroll Fixed Row Height Breaks with Real Data

**Risk:** Current `ROW_HEIGHT = 48` constant drives all calculations. Variable content breaks math (jumps, overlaps, blank gaps). TanStack Virtual has Svelte 5 issues (GitHub #866).

**Prevention:**
- Keep custom virtual scroll implementation (~90 lines)
- Enforce fixed height with `white-space: nowrap; overflow: hidden; text-overflow: ellipsis` on all cells
- Add `title` attribute for hover to show truncated content
- Do NOT adopt TanStack Virtual

**Phase:** DataTable phase

### 4. AppShell Breaks Adjacency List Model

**Risk:** AppShell needs named slots (sidebar, header, footer) but `ComponentNode.children` is a flat ordered list. Naive approach requires protocol changes.

**Prevention:**
- Follow FormScreen pattern: use `props.sidebar_items: string[]`, `props.header_items: string[]` to reference child node IDs by name
- This is a component convention, not a protocol change
- Already proven to work in the existing codebase

**Phase:** AppShell phase

### 5. Tailwind v4 + shadcn-svelte CSS Variable Conflict

**Risk:** Flowbite uses raw color classes (`bg-white`, `bg-gray-50`), shadcn uses semantic tokens (`bg-background`, `bg-muted`). Current `Surface.svelte` has hardcoded `bg-white` and `bg-gray-50`. Known Tailwind v4 migration issues (GitHub #2028).

**Prevention:**
- Set up shadcn CSS variables FIRST, remove Flowbite in same commit
- Replace all raw color classes with semantic equivalents
- Use `npx shadcn-svelte@next init` for correct Tailwind v4 config
- Generate fresh `app.css`, copy only custom CSS from old file

**Phase:** Foundation/infrastructure phase

---

## Moderate Pitfalls

### 6. fetchedRanges Stale on Sort/Filter

**Risk:** `handleSort` sends action but doesn't reset `fetchedRanges` Set. Stale data stays visible after server returns new sorted/filtered results.

**Prevention:** Reset `fetchedRanges` whenever sort/filter parameters change.

**Phase:** DataTable phase

### 7. Screen Components Bypass NodeRenderer Contract

**Risk:** FormScreen accesses surface tree directly rather than going through NodeRenderer. This works but is undocumented.

**Prevention:** Keep the pattern but document the contract. Validate missing node IDs gracefully.

**Phase:** Component migration phase

### 8. Breakpoint Mismatch Between AppShell and Content

**Risk:** If AppShell uses one breakpoint for mobile/desktop (e.g., `lg:1024px`) and content components use another (e.g., `md:768px`), layout breaks at intermediate widths.

**Prevention:** Standardize on `md:` (768px) for all mobile/desktop switches. Test at exactly 768px.

**Phase:** AppShell phase

### 9. Svelte 5 Reactivity Depth in Props

**Risk:** Destructuring reactive props to plain variables breaks reactivity tracking.

**Prevention:** Access props through `props.columns` in `$derived`, never destructure to plain variables. Current code does this correctly — maintain the pattern.

**Phase:** All phases

### 10. Icon Library Swap

**Risk:** Flowbite icons and Lucide icons have different default sizes and naming conventions.

**Prevention:** Explicit icon name mapping:
- `ChevronUpOutline` → `ChevronUp`
- `BarsOutline` → `Menu`
- `CloseOutline` → `X`
- `ExclamationCircleOutline` → `AlertCircle`

Keep explicit size classes on all icons.

**Phase:** Component migration phase

---

## Minor Pitfalls

### 11. Toast System Swap

**Risk:** shadcn uses `svelte-sonner` (imperative `toast()` calls), not the component-based approach Flowbite uses.

**Prevention:** Replace `ToastSurface.svelte` with `<Toaster />`, replace `addToast()` store calls with `toast.error()` / `toast.success()`. Toasts are ephemeral client-side UI — don't try to server-drive them.

**Phase:** Component migration phase

### 12. Dynamic Tailwind Classes Don't Work

**Risk:** Tailwind v4 can't JIT dynamic classes like `grid-cols-{columns}`.

**Prevention:** Use inline `style="grid-template-columns: repeat({columns}, 1fr)"` instead.

**Phase:** Form/layout phases

---

## Phase-Specific Warning Summary

| Phase | Pitfalls | Key Mitigation |
|-------|----------|---------------|
| Foundation/CSS | #5 CSS conflicts | Set up shadcn vars before removing Flowbite |
| Component swap | #1 API differences, #7 NodeRenderer, #10 icons, #11 toasts | Map all imports first, swap in one phase |
| AppShell | #2 context clash, #4 adjacency list, #8 breakpoints | AppShell owns Provider, FormScreen pattern for slots |
| DataTable | #3 virtual scroll, #6 stale ranges | Keep custom impl, reset ranges on filter |
| Forms | #12 dynamic classes | Use style attribute for dynamic grids |

---

## Sources

- [shadcn-svelte useSidebar issue — GitHub #1563](https://github.com/huntabyte/shadcn-svelte/discussions/1563)
- [shadcn-svelte $derived.by error — GitHub #1711](https://github.com/huntabyte/shadcn-svelte/discussions/1711)
- [shadcn-svelte Tailwind v4 migration — GitHub #2028](https://github.com/huntabyte/shadcn-svelte/issues/2028)
- [TanStack Virtual Svelte 5 issues — GitHub #866](https://github.com/TanStack/virtual/issues/866)

---

*Pitfalls research for: Marionette v1.1 — shadcn-svelte + High-Level Components*
*Researched: 2026-04-08*
