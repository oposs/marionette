---
phase: 11-leaf-component-migration
plan: 03
subsystem: frontend
tags: [shadcn-svelte, dialog, table, toast, popup, browser-tests]
dependency_graph:
  requires: [11-01]
  provides: [shadcn-dialog-modal, shadcn-table, semantic-toast]
  affects: [11-05]
tech_stack:
  added: []
  patterns: [shadcn-dialog-composable, shadcn-table-composable, dialog-root-context-wrapper-for-tests]
key_files:
  created:
    - frontend/src/lib/components/popup/ModalSurface.browser-test.ts
    - frontend/src/lib/components/popup/ConfirmDialog.browser-test.ts
    - frontend/src/lib/components/popup/ConfirmDialogTestWrapper.svelte
    - frontend/src/lib/components/popup/ToastSurface.browser-test.ts
    - frontend/src/lib/components/table/DataTable.browser-test.ts
  modified:
    - frontend/src/lib/components/popup/ModalSurface.svelte
    - frontend/src/lib/components/popup/ConfirmDialog.svelte
    - frontend/src/lib/components/popup/ToastSurface.svelte
    - frontend/src/lib/components/table/DataTable.svelte
decisions:
  - ConfirmDialog renders Dialog sub-components (Header/Footer/Title/Description) without its own Dialog.Root since it lives inside ModalSurface Dialog.Content
  - ToastSurface kept as custom component with semantic tokens (no shadcn Toast/Sonner since SDUI drives toast state)
  - Created ConfirmDialogTestWrapper.svelte to provide Dialog.Root context for isolated browser tests
  - ToastSurface dismiss test uses real timers since Svelte fly transitions use requestAnimationFrame
metrics:
  duration: 5min
  completed: "2026-04-09T15:57:00Z"
---

# Phase 11 Plan 03: Popup and Table Component Migration Summary

ModalSurface migrated to shadcn Dialog with focus trap/portal/Escape, ConfirmDialog uses Dialog.Header/Footer + shadcn Button variants, ToastSurface restyled with semantic tokens and repositioned to bottom-right, DataTable uses shadcn Table sub-components with preserved virtual scroll. All 14 browser tests pass across 4 test files.

## Task Results

| Task | Name | Commit | Status |
|------|------|--------|--------|
| 1 | Migrate ModalSurface, ConfirmDialog, ToastSurface, DataTable | 2dec1d1 | Done |
| 2 | Write browser tests for popup and table components | 89684c3 | Done |

## What Was Done

### Task 1: Migrate popup and table components to shadcn primitives

**ModalSurface.svelte:** Replaced custom `<div class="fixed inset-0 ...">` overlay with shadcn `Dialog.Root` + `Dialog.Content`. Uses `open={isOpen}` one-way bind from SDUI surface tree existence. `onOpenChange` dispatches `close-modal` action. Removed all `a11y_no_static_element_interactions` ignore comments. Gains focus trap, portal isolation, and Escape key handling from bits-ui Dialog primitive.

**ConfirmDialog.svelte:** Replaced hand-rolled `<div class="p-4">` with `Dialog.Header`, `Dialog.Title`, `Dialog.Description`, and `Dialog.Footer`. Replaced inline Tailwind button classes with `ShadcnButton` component using `variant="outline"` for cancel and `variant="destructive"|"default"` for confirm. ConfirmDialog does NOT render its own Dialog.Root -- it renders inside ModalSurface's Dialog.Content.

**ToastSurface.svelte:** Replaced hardcoded color classes (`bg-green-50`, `bg-yellow-50`, `text-green-800`) with semantic tokens (`bg-primary/10`, `bg-destructive/10`, `bg-card`, `text-foreground`). Repositioned from `fixed top-4 left-4 right-4` to `fixed bottom-4 right-4` per UI-SPEC. Added `shadow-lg` to toast cards. Replaced `&times;` character with lucide `X` icon component.

**DataTable.svelte:** Replaced raw HTML `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>` with shadcn `Table.Root`, `Table.Header`, `Table.Body`, `Table.Row`, `Table.Head`, `Table.Cell`. Updated header styling to `text-muted-foreground text-xs uppercase font-semibold`. Changed cell text from `text-muted-foreground` to `text-foreground`. Changed row hover from `hover:bg-accent` to `hover:bg-muted/50`. All virtual scroll and sort logic preserved unchanged.

### Task 2: Write browser tests for popup and table components

Created 4 browser test files with 14 total tests:

- **ModalSurface** (3 tests): no-render when no surface tree, renders dialog when tree exists, dispatches close-modal on close button click
- **ConfirmDialog** (4 tests): renders title and message, renders confirm/cancel buttons, dispatches action on confirm, dispatches close-modal on cancel
- **ToastSurface** (3 tests): empty state, renders toast after addToast, removes toast on dismiss click
- **DataTable** (4 tests): renders column headers, renders rows from bound data, dispatches sort on header click, dispatches select-row on row click

Created `ConfirmDialogTestWrapper.svelte` to provide the required `Dialog.Root` context for standalone ConfirmDialog testing (bits-ui Dialog.Title requires Dialog.Root ancestor context).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ConfirmDialog.Title requires Dialog.Root context**
- **Found during:** Task 2
- **Issue:** bits-ui Dialog.Title throws "Context Dialog.Root not found" when ConfirmDialog is rendered without a Dialog.Root ancestor
- **Fix:** Created ConfirmDialogTestWrapper.svelte that wraps ConfirmDialog inside Dialog.Root + Dialog.Content for browser tests
- **Files modified:** ConfirmDialog.browser-test.ts, ConfirmDialogTestWrapper.svelte (new)
- **Commit:** 89684c3

**2. [Rule 1 - Bug] ToastSurface dismiss test failed with fake timers**
- **Found during:** Task 2
- **Issue:** Svelte fly transitions use requestAnimationFrame which fake timers don't advance, so dismissed toast remained in DOM
- **Fix:** Used real timers for the dismiss test with explicit setTimeout waits for transition completion
- **Files modified:** ToastSurface.browser-test.ts
- **Commit:** 89684c3

**3. [Rule 1 - Bug] Svelte quoted attribute warning on DataTable**
- **Found during:** Task 2
- **Issue:** `class="{expr}"` syntax on Table.Head triggers Svelte deprecation warning about quoted attributes on components
- **Fix:** Changed to unquoted `class={expr}` syntax
- **Files modified:** DataTable.svelte
- **Commit:** 89684c3

## Verification

- ModalSurface uses shadcn Dialog with `Dialog.Root`, `Dialog.Content`, `onOpenChange`
- ConfirmDialog uses `Dialog.Header`, `Dialog.Footer`, `Dialog.Title`, `Dialog.Description` + `ShadcnButton`
- ToastSurface uses only semantic token classes (no `bg-green-50`, `bg-yellow-50`, or other raw colors)
- DataTable uses `Table.Root`, `Table.Header`, `Table.Body`, `Table.Row`, `Table.Head`, `Table.Cell`
- All 14 browser tests pass across 4 test files
- `npm run build` exits 0

## Self-Check: PASSED

All 9 key files verified present. Both commits (2dec1d1, 89684c3) verified in git log.
