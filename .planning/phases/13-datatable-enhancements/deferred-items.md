# Phase 13 — Deferred Items

Items discovered during execution that are out of scope for the current plan.

## Pre-existing tsc errors (observed at Phase 13 Wave 0 start)

These errors exist at the Phase 13 base commit (`555e355f`) and are NOT caused by Plan 13-01 changes. The project's canonical typecheck is `svelte-check` (`npm run check`), not raw `tsc --noEmit`. The pre-existing `tsc --noEmit` failures are:

- `src/lib/components/ui/badge/index.ts` — re-exports named members from `*.svelte` which raw tsc treats as default exports only (works under svelte-check's Svelte-aware module resolution)
- `src/lib/components/ui/button/index.ts` — same pattern
- `tests/helpers/schema-validator.ts` — Node built-ins (`fs`, `path`, `url`) not resolved because `@types/node` is not in tsconfig for the frontend test helpers (these files are only run under Playwright E2E which has its own resolution)

**Disposition:** deferred. None are introduced or aggravated by this plan. A future cleanup plan can choose to (a) switch to svelte-check as the type gate, or (b) add proper type re-exports in the badge/button barrels.

Plan 13-01 uses `npm run check` (svelte-check) as the effective TypeScript gate.
