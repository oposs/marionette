# Phase 13 — Deferred Items

Items discovered during execution that are out of scope for the current plan.

## Pre-existing tsc errors (observed at Phase 13 Wave 0 start)

These errors exist at the Phase 13 base commit (`555e355f`) and are NOT caused by Plan 13-01 changes. The project's canonical typecheck is `svelte-check` (`npm run check`), not raw `tsc --noEmit`. The pre-existing `tsc --noEmit` failures are:

- `src/lib/components/ui/badge/index.ts` — re-exports named members from `*.svelte` which raw tsc treats as default exports only (works under svelte-check's Svelte-aware module resolution)
- `src/lib/components/ui/button/index.ts` — same pattern
- `tests/helpers/schema-validator.ts` — Node built-ins (`fs`, `path`, `url`) not resolved because `@types/node` is not in tsconfig for the frontend test helpers (these files are only run under Playwright E2E which has its own resolution)

**Disposition:** deferred. None are introduced or aggravated by this plan. A future cleanup plan can choose to (a) switch to svelte-check as the type gate, or (b) add proper type re-exports in the badge/button barrels.

Plan 13-01 uses `npm run check` (svelte-check) as the effective TypeScript gate.

## Pre-existing popup browser-test failures (observed during Plan 13-05)

Five failing tests surfaced in `src/lib/components/popup/` while Plan 13-05 was running the full browser-test suite for regression checking:

- `ConfirmDialog.browser-test.ts` — 4 tests fail (render title/message, render buttons, dispatch confirm, dispatch close-modal)
- `ToastSurface.browser-test.ts` — 1 test fails (remove toast on dismiss click)

**Disposition:** **pre-existing**, not introduced by Plan 13-05. Verified by stashing Plan 13-05 changes and re-running the popup test suite against the baseline commit (`5c2b27b test(13-05): rewrite DataTable.browser-test.ts harness` — note: test-only RED-phase commit, the underlying DataTable.svelte was still the pre-rewrite version); the same 5 failures reproduce with zero code changes from 13-05 loaded. Likely root cause: the Tailwind-class-not-applied layout issue that Plan 13-05 worked around with inline styles in DataTable.svelte — popup components rely on layout classes (`flex`, `hidden`, etc.) that are no-ops in the browser-test harness because `src/app.css` isn't imported by `vitest-browser.config.ts`. Out of scope for Plan 13-05 per SCOPE BOUNDARY — logged here so a future popup-suite fix plan can (a) inline the critical layout styles or (b) load `src/app.css` in `vitest-browser.config.ts`.
