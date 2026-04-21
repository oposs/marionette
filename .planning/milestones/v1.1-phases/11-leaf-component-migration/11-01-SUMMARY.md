---
phase: 11-leaf-component-migration
plan: 01
subsystem: frontend
tags: [shadcn-svelte, primitives, icons, foundation]
dependency_graph:
  requires: []
  provides: [shadcn-primitives, icon-registry, utils-types]
  affects: [11-02, 11-03, 11-04, 11-05]
tech_stack:
  added: [bits-ui, lucide-svelte-icons-registry]
  patterns: [shadcn-svelte-cli-install, dynamic-icon-lookup-with-fallback]
key_files:
  created:
    - frontend/src/lib/components/ui/button/index.ts
    - frontend/src/lib/components/ui/input/index.ts
    - frontend/src/lib/components/ui/select/index.ts
    - frontend/src/lib/components/ui/checkbox/index.ts
    - frontend/src/lib/components/ui/label/index.ts
    - frontend/src/lib/components/ui/dialog/index.ts
    - frontend/src/lib/components/ui/table/index.ts
    - frontend/src/lib/components/ui/card/index.ts
    - frontend/src/lib/components/ui/skeleton/index.ts
    - frontend/src/lib/components/ui/separator/index.ts
    - frontend/src/lib/components/ui/badge/index.ts
    - frontend/src/lib/registry/icons.ts
  modified:
    - frontend/src/lib/utils.ts
    - frontend/src/lib/index.ts
    - frontend/package.json
    - frontend/package-lock.json
decisions:
  - Used shadcn-svelte CLI v1.2.7 for bulk primitive installation
  - Registered 14 default icons (13 CRM icons + CircleHelp fallback itself)
metrics:
  duration: 2min
  completed: "2026-04-09T15:49:00Z"
---

# Phase 11 Plan 01: Install Primitives and Icon Registry Summary

shadcn-svelte primitives (11 components) installed via CLI, utils.ts extended with WithElementRef/WithoutChildren types, and dynamic icon registry created with 14 default icons and CircleHelp fallback for unknown names.

## Task Results

| Task | Name | Commit | Status |
|------|------|--------|--------|
| 1 | Install shadcn-svelte primitives and fix utils.ts | 4c34a3c | Done |
| 2 | Create dynamic icon registry | 20102a4 | Done |

## What Was Done

### Task 1: Install shadcn-svelte primitives and fix utils.ts

Added `WithElementRef` and `WithoutChildren` type exports to `frontend/src/lib/utils.ts`. Ran `npx shadcn-svelte@latest add` to bulk-install 11 primitives: button, input, select, checkbox, label, dialog, table, card, skeleton, separator, badge. Each primitive was installed into `frontend/src/lib/components/ui/<name>/` with full Svelte 5 component files. Build verified clean.

### Task 2: Create dynamic icon registry

Created `frontend/src/lib/registry/icons.ts` with:
- `registerIcon(name, component)` to add icons to a module-level registry
- `getIcon(name)` returning the registered component or `CircleHelp` fallback
- 14 default icons registered: plus, chevron-up, chevron-down, alert-circle, x, menu, arrow-left, search, filter, pencil, trash (mapped to trash-2), check, loader (mapped to loader-2), circle-help
- Exports added to `frontend/src/lib/index.ts` barrel file

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] npm install required before shadcn CLI**
- **Found during:** Task 1
- **Issue:** shadcn-svelte CLI requires `svelte-kit sync` which needs node_modules present; worktree had no node_modules
- **Fix:** Ran `npm i` before CLI invocation
- **Files modified:** none (node_modules is gitignored)
- **Commit:** part of 4c34a3c

## Verification

- All 11 shadcn-svelte primitive directories exist under `frontend/src/lib/components/ui/`
- `utils.ts` exports `WithElementRef` and `WithoutChildren` types
- `icons.ts` exports `getIcon()` and `registerIcon()` functions
- 14 icon registrations present (13 CRM icons + circle-help)
- Unknown icon names return CircleHelp fallback component
- `npm run build` exits 0 with no TypeScript errors

## Self-Check: PASSED

All 12 key files verified present. Both commits (4c34a3c, 20102a4) verified in git log.
