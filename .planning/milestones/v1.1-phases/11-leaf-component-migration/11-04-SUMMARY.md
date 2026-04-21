---
phase: 11-leaf-component-migration
plan: 04
subsystem: frontend-components
tags: [layout, nav, core, feedback, shadcn, browser-tests]
dependency_graph:
  requires: [11-01]
  provides: [layout-components, nav-components, core-components, feedback-components]
  affects: [frontend/src/lib/components/layout, frontend/src/lib/components/nav, frontend/src/lib/components/core, frontend/src/lib/components/feedback]
tech_stack:
  added: []
  patterns: [shadcn-card, shadcn-skeleton, shadcn-button-ghost, inline-grid-style, lucide-loader2]
key_files:
  created:
    - frontend/src/lib/components/layout/Container.browser-test.ts
    - frontend/src/lib/components/layout/Grid.browser-test.ts
    - frontend/src/lib/components/layout/Heading.browser-test.ts
    - frontend/src/lib/components/layout/Text.browser-test.ts
    - frontend/src/lib/components/nav/NavItem.browser-test.ts
    - frontend/src/lib/components/nav/NavGroup.browser-test.ts
    - frontend/src/lib/components/core/LoadingSkeleton.browser-test.ts
    - frontend/src/lib/components/core/FallbackComponent.browser-test.ts
    - frontend/src/lib/components/core/ConnectionBanner.browser-test.ts
    - frontend/src/lib/components/feedback/Spinner.browser-test.ts
    - frontend/src/lib/components/feedback/ErrorDisplay.browser-test.ts
  modified:
    - frontend/src/lib/components/layout/Container.svelte
    - frontend/src/lib/components/layout/Grid.svelte
    - frontend/src/lib/components/layout/Heading.svelte
    - frontend/src/lib/components/layout/Text.svelte
    - frontend/src/lib/components/nav/SideNav.svelte
    - frontend/src/lib/components/nav/NavItem.svelte
    - frontend/src/lib/components/nav/NavGroup.svelte
    - frontend/src/lib/components/core/LoadingSkeleton.svelte
    - frontend/src/lib/components/core/FallbackComponent.svelte
    - frontend/src/lib/components/core/ConnectionBanner.svelte
    - frontend/src/lib/components/feedback/Spinner.svelte
    - frontend/src/lib/components/nav/SideNav.browser-test.ts
    - frontend/src/lib/components/core/NodeRenderer.browser-test.ts
    - frontend/src/lib/components/core/Surface.browser-test.ts
decisions:
  - "Container card variant uses shadcn Card.Root instead of manual border/bg classes"
  - "Grid uses inline style grid-template-columns instead of dynamic Tailwind grid-cols-N"
  - "NavItem uses shadcn Button ghost variant with getIcon for icon support"
  - "Spinner replaced inline SVG with Loader2 from lucide"
  - "ConnectionBanner uses Loader2 instead of inline SVG spinner"
  - "Heading h1 changed from text-[28px] to text-xl per UI-SPEC"
metrics:
  duration: "6 min"
  completed: "2026-04-09T15:58:23Z"
  tasks_completed: 2
  tasks_total: 2
  test_count: 33
  test_files: 14
---

# Phase 11 Plan 04: Layout/Nav/Core/Feedback Component Migration Summary

Migrated 11 components to shadcn-svelte primitives with semantic tokens, fixed Grid dynamic Tailwind bug (Pitfall 4) and LoadingSkeleton hardcoded colors (Pitfall 6), added icon support to NavItem, and created 14 browser test files with 33 passing tests.

## Tasks Completed

### Task 1: Migrate layout, nav, core, and feedback components
**Commit:** 102bf8e

Migrated 11 components (ErrorBoundary, ErrorDisplay, NodeRenderer, Surface unchanged -- already correct):

- **Container**: Card variant now uses `Card.Root` from shadcn instead of manual `border-border bg-card rounded-lg`
- **Grid**: Replaced dynamic `grid-cols-${cols}` Tailwind class with inline `style="grid-template-columns: repeat(N, 1fr)"` (Pitfall 4 fix)
- **Heading**: Changed h1 from `text-[28px]` to `text-xl` per UI-SPEC, added `text-foreground`
- **Text**: Added muted variant support (`text-muted-foreground` when `props.muted=true`), added `text-foreground`
- **SideNav**: Added `bg-sidebar-background` and `flex flex-col` per UI-SPEC
- **NavItem**: Replaced manual button with shadcn `Button variant="ghost"`, added `getIcon` icon support, uses `bg-sidebar-accent text-sidebar-accent-foreground` for active state
- **NavGroup**: Added optional uppercase group label with `text-muted-foreground`, `mt-2` for 8px top margin
- **LoadingSkeleton**: Replaced `animate-pulse bg-gray-200 dark:bg-gray-700` divs with shadcn `Skeleton` component (Pitfall 6 fix)
- **FallbackComponent**: Replaced `border-red-500 bg-red-50 text-red-700` with `border-destructive bg-destructive/10 text-destructive`
- **ConnectionBanner**: Changed from `bg-accent text-accent-foreground` to `bg-destructive text-primary-foreground`, replaced inline SVG with `Loader2`, added `font-semibold` and `justify-center`
- **Spinner**: Replaced inline SVG with `Loader2` from lucide, uses `text-primary`

### Task 2: Write browser tests for all components
**Commit:** c409936

Created 11 new test files and updated 3 existing ones. 14 test files total with 33 tests:

- Layout: Container (2), Grid (2), Heading (2), Text (2)
- Nav: SideNav (3, rewritten), NavItem (3), NavGroup (2)
- Core: LoadingSkeleton (2), FallbackComponent (2), ConnectionBanner (2), NodeRenderer (4, existing), Surface (3, updated)
- Feedback: Spinner (2), ErrorDisplay (2)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Heading h1 used non-standard size**
- **Found during:** Task 1
- **Issue:** h1 used `text-[28px]` arbitrary value instead of UI-SPEC's `text-xl` (20px)
- **Fix:** Changed to `text-xl` to match UI-SPEC typography contract
- **Files modified:** Heading.svelte
- **Commit:** 102bf8e

**2. [Rule 1 - Bug] Spinner used dynamic Tailwind class**
- **Found during:** Task 1
- **Issue:** `size-{props.size}` is a dynamic Tailwind class that won't be compiled (same pattern as Grid Pitfall 4)
- **Fix:** Replaced entire inline SVG with Loader2 lucide component at fixed `size-6`
- **Files modified:** Spinner.svelte
- **Commit:** 102bf8e

**3. [Rule 2 - Missing functionality] Surface.browser-test.ts assertion outdated**
- **Found during:** Task 2
- **Issue:** Surface test checked for `.animate-pulse` class which no longer exists after LoadingSkeleton migration to shadcn Skeleton
- **Fix:** Updated to check for `[data-slot="skeleton"]` selector
- **Files modified:** Surface.browser-test.ts
- **Commit:** c409936

## Verification Results

- Grid uses inline style: `grid-template-columns: repeat(N, 1fr)` -- PASS
- LoadingSkeleton uses shadcn Skeleton component -- PASS
- FallbackComponent uses `border-destructive bg-destructive/10 text-destructive` -- PASS
- ConnectionBanner uses `bg-destructive text-primary-foreground` -- PASS
- NavItem supports icons via `getIcon` registry -- PASS
- No `bg-gray-*`, `text-gray-*`, `border-red-*`, `bg-red-*` in modified files -- PASS
- All 14 browser test files pass (33 tests) -- PASS
- `npm run build` exits 0 -- PASS

## Self-Check: PASSED

All 13 created files verified present. Both commit hashes (102bf8e, c409936) verified in git log.
