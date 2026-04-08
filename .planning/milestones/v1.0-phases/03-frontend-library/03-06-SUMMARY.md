---
phase: 03-frontend-library
plan: 06
subsystem: testing
tags: [vitest-browser-svelte, playwright, visual-regression, e2e, browser-tests, screenshots]

# Dependency graph
requires:
  - phase: 03-frontend-library
    provides: "All 18 SDUI components, registry, data store, surfaces, init"
provides:
  - "19 browser component tests in 6 files running in real Chromium"
  - "Playwright E2E smoke tests verifying app load and demo mode"
  - "Visual regression baselines for sidebar, form, data-table, and full page"
  - "Demo page rendering all component types without backend"
affects: [05-integration]

# Tech tracking
tech-stack:
  added: ["@vitest/browser-playwright"]
  patterns: [vitest-browser-svelte-testing, visual-regression-baseline, demo-mode-fallback]

key-files:
  created:
    - frontend/src/lib/components/core/NodeRenderer.browser-test.ts
    - frontend/src/lib/components/core/Surface.browser-test.ts
    - frontend/src/lib/components/form/TextInput.browser-test.ts
    - frontend/src/lib/components/form/Button.browser-test.ts
    - frontend/src/lib/components/nav/SideNav.browser-test.ts
    - frontend/src/lib/components/table/DataTable.browser-test.ts
    - frontend/tests/e2e/smoke.spec.ts
    - frontend/tests/visual/components.spec.ts
    - frontend/tests/visual/full-page.spec.ts
  modified:
    - frontend/vitest-browser.config.ts
    - frontend/playwright.config.ts
    - frontend/src/routes/+page.svelte
    - frontend/src/routes/+layout.svelte
    - frontend/src/lib/index.ts

key-decisions:
  - "Vitest browser provider migrated to @vitest/browser-playwright factory API (Vitest 4 breaking change)"
  - "Demo page falls back to demo mode after 2s WebSocket connection timeout"
  - "Layout renders named Surface components for sidebar/main/modal/toast"
  - "Exported setSurfaceTree/getSurfaceTree/clearSurfaceTree from public API for demo/testing"

patterns-established:
  - "Browser test pattern: import from vitest-browser-svelte, register components, render via NodeRenderer"
  - "Visual regression pattern: toHaveScreenshot with maxDiffPixels tolerance, snapshotDir for baselines"
  - "Demo mode pattern: initMarionette + setTimeout fallback populates surfaces with sample data"

requirements-completed: [FRONT-21, FRONT-24, FRONT-25, FRONT-26, FRONT-27]

# Metrics
duration: 6min
completed: 2026-03-20
---

# Phase 3 Plan 6: Browser Tests, E2E Framework, and Visual Regression Summary

**19 browser component tests in Chromium via vitest-browser-svelte, Playwright E2E smoke tests, visual regression baselines for sidebar/form/table/full-page, and demo page rendering all component types without backend**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-20T11:19:03Z
- **Completed:** 2026-03-20T11:25:52Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments
- 6 browser test files (19 tests) covering NodeRenderer, Surface, TextInput, Button, SideNav, DataTable in real Chromium
- Playwright E2E framework with 7 tests: 3 smoke tests + 3 component visual snapshots + 1 full-page snapshot
- Demo page with fallback mode populating sidebar nav, form with text inputs, and data table with sample CRM data
- Multi-surface layout rendering ConnectionBanner, sidebar, main, modal, and toast surfaces
- All 3 test tiers operational: unit (44 tests), browser-component (19 tests), E2E+visual (7 tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: Browser component tests with vitest-browser-svelte** - `34c89de` (feat)
2. **Task 2: Playwright E2E framework, visual regression tests, and demo page** - `3b3826d` (feat)

## Files Created/Modified
- `frontend/vitest-browser.config.ts` - Updated with $lib alias and @vitest/browser-playwright provider
- `frontend/playwright.config.ts` - Added webServer, toHaveScreenshot config, snapshotDir
- `frontend/src/lib/components/core/NodeRenderer.browser-test.ts` - 4 tests: render, nesting, fallback, visibility
- `frontend/src/lib/components/core/Surface.browser-test.ts` - 3 tests: skeleton, tree render, data-surface attr
- `frontend/src/lib/components/form/TextInput.browser-test.ts` - 4 tests: label, binding, input update, dirty tracking
- `frontend/src/lib/components/form/Button.browser-test.ts` - 3 tests: label, action dispatch, color/size
- `frontend/src/lib/components/nav/SideNav.browser-test.ts` - 2 tests: children render, navigate action
- `frontend/src/lib/components/table/DataTable.browser-test.ts` - 3 tests: columns/rows, sort, virtual scroll height
- `frontend/tests/e2e/smoke.spec.ts` - 3 E2E smoke tests
- `frontend/tests/visual/components.spec.ts` - 3 component visual snapshots (sidebar, form, data-table)
- `frontend/tests/visual/full-page.spec.ts` - 1 full-page visual snapshot
- `frontend/src/routes/+page.svelte` - Demo page with initMarionette and demo mode fallback
- `frontend/src/routes/+layout.svelte` - Multi-surface layout with ConnectionBanner
- `frontend/src/lib/index.ts` - Added surface tree exports to public API

## Decisions Made
- Migrated vitest browser provider from string `'playwright'` to `playwright()` factory (Vitest 4 breaking change in @vitest/browser)
- Demo page uses 2-second timeout before falling back to demo mode (balances startup speed vs connection chance)
- Layout renders all 4 surface types (sidebar, main, modal, toast) plus ConnectionBanner for complete SDUI layout
- Exported setSurfaceTree/getSurfaceTree/clearSurfaceTree from $lib public API for demo and testing use

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed @vitest/browser-playwright for Vitest 4 compatibility**
- **Found during:** Task 1
- **Issue:** Vitest 4 changed browser.provider from string to factory function, breaking the existing config
- **Fix:** Installed @vitest/browser-playwright package, changed config to use `playwright()` factory import
- **Files modified:** frontend/vitest-browser.config.ts, frontend/package.json
- **Verification:** Browser tests run successfully
- **Committed in:** 34c89de (Task 1 commit)

**2. [Rule 3 - Blocking] Exported surface tree functions from $lib public API**
- **Found during:** Task 2
- **Issue:** Demo page needed setSurfaceTree but it was not exported from $lib/index.ts
- **Fix:** Added setSurfaceTree, getSurfaceTree, clearSurfaceTree exports to $lib/index.ts
- **Files modified:** frontend/src/lib/index.ts
- **Verification:** Demo page compiles and renders correctly
- **Committed in:** 3b3826d (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes required for task completion. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 3 (Frontend Library) is complete with all components, stores, transport, and comprehensive testing
- 3 test tiers operational: unit tests (Vitest), browser component tests (vitest-browser-svelte), E2E/visual (Playwright)
- Demo page enables manual testing and visual regression without a running backend
- Ready for Phase 4 (Backend Library) development

## Self-Check: PASSED

All 9 created files verified present. Commits 34c89de and 3b3826d verified in git log.

---
*Phase: 03-frontend-library*
*Completed: 2026-03-20*
