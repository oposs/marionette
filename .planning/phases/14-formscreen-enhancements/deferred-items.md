# Phase 14 — Deferred Items

Out-of-scope issues discovered during Phase 14 execution. These are not caused
by Phase 14 changes — logged here so future phases can triage.

---

## `tests/helpers/schema-validator.ts` — missing Node.js type declarations

**Found during:** Plan 14-01, Task 1 (running `npm run check` after shadcn-svelte install)

**Errors reported:**

```
tests/helpers/schema-validator.ts 4:21 "Cannot find module 'fs' or its corresponding type declarations."
tests/helpers/schema-validator.ts 5:23 "Cannot find module 'path' or its corresponding type declarations."
tests/helpers/schema-validator.ts 6:31 "Cannot find module 'url' or its corresponding type declarations."
```

**Root cause:** The test helper imports from bare `'fs'`, `'path'`, `'url'`
rather than the `node:` prefix; the frontend `tsconfig.json` does not include
`@types/node` in its compile unit. Pre-existing — shipped by
`4dc55c0 feat(05-02): add E2E test helpers` long before Phase 14.

**Reproduction:** `cd frontend && npm run check` on `main` at
`cb37e76` (the branch base for Phase 14) produces the same 3 errors.

**Fix (deferred):** Either add `@types/node` to `frontend/devDependencies` and
include it in `tsconfig.json`'s `types` array, or rewrite the imports to use
`node:fs`, `node:path`, `node:url` (Node 18+ native).

**Why deferred:** Unrelated to Phase 14's scope (shadcn Field migration,
NodeRenderer blur race). No Phase 14 plan introduces new code in
`tests/helpers/`. Safe to ignore for Phase 14 success criteria.
