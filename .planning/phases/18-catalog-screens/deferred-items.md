# Phase 18 — Deferred Items

Items discovered during Phase 18 execution that are OUT OF SCOPE for the
current plan but should be captured for later attention.

## Pre-existing `svelte-check` errors in `frontend/src/lib/utils/virtualizer.svelte.ts`

**Discovered during:** Plan 18-02 (SelectInput / Checkbox / Switch / RadioGroup
blur dispatch) while running `pnpm check` for the acceptance-criteria gate.

**Symptom:** `svelte-check` reports 3 errors in `src/lib/utils/virtualizer.svelte.ts`:
- Line 56:8 — `Cannot find module '@tanstack/virtual-core' or its corresponding type declarations.`
- Line 99:15 — `Parameter 'inst' implicitly has an 'any' type.`
- Line 99:21 — `Parameter 'sync' implicitly has an 'any' type.`

**Why it's pre-existing:** `virtualizer.svelte.ts` was last modified in Phase 13
(commit `87b17b6`). It directly imports from `@tanstack/virtual-core`, which is
only declared as a transitive dep of `@tanstack/svelte-virtual` in `package.json`.
Under pnpm's strict node_modules layout, direct imports of transitive packages
are not resolvable without an explicit top-level dependency entry or a
`.pnpmfile.cjs` hoist directive.

**Why not fixed in 18-02:** Out of scope — this plan touches 4 form components,
not the virtualizer. The errors are not caused by our changes and would occur
on the base commit as well. The tests of this plan (SelectInput, Checkbox,
Switch, RadioGroup) all pass.

**Candidate fix:** Add `"@tanstack/virtual-core": "^3.14.0"` to
`frontend/package.json` dependencies, or hoist it via pnpm settings. Best picked
up by a dedicated cleanup plan (tracked with the other pre-existing ESLint
baseline drift).
