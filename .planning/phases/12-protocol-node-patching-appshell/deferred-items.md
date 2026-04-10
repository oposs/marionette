# Phase 12 Deferred Items

Items discovered during execution that are out of scope for the current plan.

## From 12-01-scaffolding

### Pre-existing TypeScript errors in tests/helpers/schema-validator.ts

`npm run check` reports 3 errors about `fs`, `path`, `url` module resolution:
- `tests/helpers/schema-validator.ts:4` — Cannot find module 'fs'
- `tests/helpers/schema-validator.ts:5` — Cannot find module 'path'
- `tests/helpers/schema-validator.ts:6` — Cannot find module 'url'

These predate Phase 12 (present before the shadcn Sidebar install). Likely missing `@types/node` or the test helper tsconfig needs `"types": ["node"]`. Not caused by Plan 12-01 changes.
