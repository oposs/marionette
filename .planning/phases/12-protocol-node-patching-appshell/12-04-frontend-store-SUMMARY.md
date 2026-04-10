---
phase: 12-protocol-node-patching-appshell
plan: 04
subsystem: frontend
tags: [svelte, store, patch, focus-preservation, fine-grained-reactivity, svelte5-runes]

requires:
  - phase: 12-protocol-node-patching-appshell
    provides: "Plan 12-02 tagged PatchOperation Rust enum + Plan 12-03 JSON Schemas — the wire shape this plan mirrors on the TS side"
provides:
  - "PatchOperation TypeScript discriminated union (6 variants: set, set-node, delete-node, set-children, insert-child, remove-child)"
  - "PatchMessage.surface required field + init.ts patch handler routing by msg.surface (D-A3 bug fix)"
  - "Fine-grained surface mutation API: setNode, deleteNode, setChildren, insertChild, removeChild, gcOrphans"
  - "applyPatch dispatcher that routes both data and node ops from one atomic patch batch"
  - "Walk-and-prune GC (D-A8) scoped per-surface, visited-set bounded for cyclic graphs"
  - "Focus-preservation browser test — canonical D-A6 proof that sibling patches retain focus + cursor"
affects: [12-05-backend-builders, 12-06-frontend-shell-components, 12-07-crm-integration, 12-08-demo-and-e2e]

tech-stack:
  added: []
  patterns:
    - "Svelte 5 $state per-key proxy mutation: mutate `tree.nodes[id]` in place instead of `tree.nodes = { ... }` to scope invalidation to the changed entry"
    - "Tagged discriminated union on TS side exhaustively switched without `as any` casts at call sites"
    - "BFS walk-and-prune GC with visited-set short-circuit (cycle-safe, O(N) bounded)"
    - "Negative-control test documents explicitly what a contract does NOT promise (focused-node replacement)"

key-files:
  created: []
  modified:
    - "frontend/src/lib/transport/messages.ts — PatchOperation 6-variant union + PatchMessage.surface"
    - "frontend/src/lib/init.ts — patch handler routes by msg.surface (D-A3 fix)"
    - "frontend/src/lib/store/surfaces.svelte.ts — rewritten with fine-grained mutators + gcOrphans"
    - "frontend/src/lib/store/data.svelte.ts — applyPatch dispatches on `op` discriminator, runs gcOrphans per batch"
    - "frontend/src/lib/store/dirty.svelte.ts — queue/callback narrowed to PatchOperationSet"
    - "frontend/src/lib/store/optimistic.svelte.ts — narrowed to set ops only"
    - "frontend/src/lib/store/surfaces.svelte.test.ts — 9 unit tests"
    - "frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts — 2 browser tests (canonical D-A6 proof)"
    - "frontend/src/lib/index.ts — re-exports the 6 new store mutators"
    - "frontend/src/lib/store/data.svelte.test.ts — migrated to tagged shape"
    - "frontend/src/lib/store/dirty.svelte.test.ts — migrated to tagged shape"
    - "frontend/src/lib/store/optimistic.svelte.test.ts — migrated to tagged shape"
    - "frontend/src/lib/transport/dispatcher.test.ts — migrated to tagged shape"

key-decisions:
  - "Preserve in-place mutation semantics in setChildren by replacing `parent.children` with a fresh array clone — Svelte 5's $state proxy notifies per-property, so the array reassignment still invalidates only the parent node, not siblings."
  - "GC runs exactly once per applyPatch batch, not per op (D-A8) — ensures O(N) per batch, not O(N*ops)."
  - "Negative-control test for focused-node replacement documents the D-A6 non-promise explicitly in-repo rather than relying on a comment in protocol docs."

patterns-established:
  - "Fine-grained store mutation pattern: all per-key store updates mutate `state[key]` in place; callers that need a fresh tree call setSurfaceTree (the wholesale path)."
  - "applyPatch as a single atomic dispatcher: data ops flow through the dirty queue, node ops flow to the surface store, and GC runs once at the end."
  - "Browser-test pattern for store-level reactivity proofs: render a Surface, drive the store directly (not via protocol messages), assert DOM-level focus/selection state survives mutation."

requirements-completed: [PATCH-01, PATCH-02]

duration: ~35min (Task 2 + Task 3 continuation run; Task 1 was landed in a prior run at 0bb8d02)
completed: 2026-04-10
---

# Phase 12 Plan 04: Frontend Store — Tagged PatchOperation + Fine-Grained Surface Mutation + Focus Preservation

**Frontend mirror of the Phase 12 Part A protocol work: PatchOperation becomes a 6-variant tagged TS union, PatchMessage gains a required `surface` field, init.ts routes patches by surface (D-A3 fix), surfaces.svelte.ts rewrites around in-place per-key mutation for Svelte 5 fine-grained reactivity (D-A6), applyPatch dispatches all 6 op variants with one walk-and-prune GC pass per batch (D-A8), and a browser test proves focused input retains focus + cursor across a sibling patch.**

## Performance

- **Duration:** ~35 min continuation run (Task 2 + Task 3); Task 1 landed earlier at commit 0bb8d02
- **Completed:** 2026-04-10
- **Tasks:** 3 (all committed atomically)
- **Files modified:** 13 across both runs

## Accomplishments

- **Tagged PatchOperation TS union** (Task 1) — mirrors Plan 12-02's Rust `#[serde(tag = "op")]` enum exactly. 6 variants: `set`, `set-node`, `delete-node`, `set-children`, `insert-child`, `remove-child`. TypeScript narrows each case inside the dispatcher; no `as any` casts at call sites.
- **PatchMessage.surface required** (Task 1) — every patch message now carries its target surface, and `init.ts` routes via `applyPatch(msg.surface, msg.patch)`. This fixes the pre-existing latent bug where every patch was hardcoded to `main` regardless of intent, making `sidebar`/`modal`/`toast` data patches silently misroute.
- **Fine-grained surface mutation API** (Task 2) — `surfaces.svelte.ts` exports `setNode`, `deleteNode`, `setChildren`, `insertChild`, `removeChild` alongside the existing `setSurfaceTree`/`getSurfaceTree`/`clearSurfaceTree`. All mutators work on the per-key `$state` proxy entries in place, which is exactly the Svelte 5 reactivity contract required for D-A6: `NodeRenderer.svelte:15` reads `$derived(nodes[nodeId])` per-key, so mutating one key invalidates only that derived and leaves sibling derivations untouched.
- **Walk-and-prune GC** (Task 2) — `gcOrphans(surface)` does a BFS from the surface's root, marks reachable IDs in a `Set`, and deletes any `nodes[id]` not in the set. Visited-set short-circuit bounds cyclic children graphs to O(N) per surface, which mitigates threat **T-12-07** (Denial of Service via cyclic children loop).
- **applyPatch dispatcher** (Task 2) — `data.svelte.ts:applyPatch` now switches on `op.op`: `set` ops flow through the dirty queue (via `setAtPointer` directly against `getStore(surface).data`), and the five node ops delegate to the surface store. A single `gcOrphans(surface)` pass runs after the loop (D-A8) so unreachable nodes created mid-batch are pruned once, not per-op.
- **9 unit tests** (Task 2) — `surfaces.svelte.test.ts` covers every mutator including a `setNode`-in-place assertion (proves `tree.nodes` reference is preserved), a `gcOrphans` orphan-deletion scenario, a deep-descendants scenario, and a missing-surface no-op scenario.
- **2 browser tests** (Task 3) — `surfaces.focus-preservation.browser-test.ts` is the **canonical D-A6 proof**. The positive test drives a real Svelte 5 render of a `Surface` with two text inputs, focuses `field-a`, types "hello", sets the cursor at position 3, calls `setNode(SURFACE, 'field-b', ...)` (a sibling patch), and asserts after `tick()` that `document.activeElement === inputA`, `inputA.selectionStart === 3`, `inputA.selectionEnd === 3`, `inputA.value === 'hello'`, AND that field-b's new label rendered. The negative-control test patches the *focused* node itself and documents explicitly that D-A6 does **not** promise focus preservation for that case.

## Task Commits

1. **Task 1: PatchOperation TS union + PatchMessage.surface + init.ts routing fix** — `0bb8d02` (feat, landed in prior run)
2. **Task 2: Fine-grained surface mutation API + applyPatch dispatcher + 9 unit tests** — `bd9464f` (feat)
3. **Task 3: Focus-preservation browser test (canonical D-A6 proof)** — `a5c0883` (test)

## Files Created/Modified

### Task 1 (committed in 0bb8d02, prior run)

- `frontend/src/lib/transport/messages.ts` — `PatchOperation` rewritten as a tagged union with 6 `op`-discriminated variants (`PatchOperationSet`, `PatchOperationSetNode`, `PatchOperationDeleteNode`, `PatchOperationSetChildren`, `PatchOperationInsertChild`, `PatchOperationRemoveChild`); `PatchMessage.surface: string` added as a required field.
- `frontend/src/lib/init.ts` — patch handler changed from `applyPatch('main', msg.patch)` to `applyPatch(msg.surface, msg.patch)` (D-A3 bug fix).
- `frontend/src/lib/store/dirty.svelte.ts` — `queuePatch` / `clearDirty` narrowed from `PatchOperation` to `PatchOperationSet` (only `set` ops have a JSON Pointer path and can be queued).
- `frontend/src/lib/store/data.svelte.ts` — `applyPatch` temporarily narrowed to `set` ops (Task 2 completed the dispatcher).
- `frontend/src/lib/store/optimistic.svelte.ts` — `applyOptimistic` narrowed to `set` ops.
- `frontend/src/lib/store/data.svelte.test.ts`, `frontend/src/lib/store/dirty.svelte.test.ts`, `frontend/src/lib/store/optimistic.svelte.test.ts`, `frontend/src/lib/transport/dispatcher.test.ts` — 8 call sites migrated from the old `{path, value}` shape to the tagged `{op: 'set', path, value}` shape.

### Task 2 (committed in bd9464f)

- `frontend/src/lib/store/surfaces.svelte.ts` — complete rewrite. Adds `setNode`, `deleteNode`, `setChildren`, `insertChild`, `removeChild`, `gcOrphans`. All mutators are no-ops on a non-existent surface; all mutate per-key in place. `setSurfaceTree` and friends are unchanged.
- `frontend/src/lib/store/data.svelte.ts` — `applyPatch` dispatches on `op.op` and delegates node ops to the surface store; runs one `gcOrphans(surface)` pass after the batch. Now imports the 6 new mutators from `./surfaces.svelte`.
- `frontend/src/lib/store/surfaces.svelte.test.ts` — 9 unit tests replacing the `test.todo` scaffold.
- `frontend/src/lib/index.ts` — re-exports the 6 new mutators alongside `setSurfaceTree`/`getSurfaceTree`/`clearSurfaceTree`.

### Task 3 (committed in a5c0883)

- `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` — 2 browser tests replacing the `test.todo` scaffold. Positive test cursor-position value asserted: **3** (the canonical "cursor mid-word in 'hello' after typing 5 chars then stepping the cursor back to index 3"). Future reference point for extending this test with more scenarios.

## Decisions Made

- **GC runs exactly once per applyPatch batch (D-A8).** Running it per-op would be O(N·ops); batching makes it O(N) per message. Consistent with the plan.
- **`setChildren` uses `parent.children = children.slice()` rather than in-place splice.** A fresh array clone is required because the caller passes a new array reference and Svelte's `$state` proxy handles property-level assignment (not deep identity). The per-key mutation contract is maintained: only `parent.children` is touched, sibling nodes' derivations are not invalidated. This is internal detail and covered by the `setChildren replaces parent.children with new order` unit test.
- **Negative-control test for focused-node replacement.** Rather than simply omitting a test for the "replace focused node" case, the test file explicitly documents the D-A6 non-promise with a passing test that asserts only the new label rendered (not focus state). This prevents a future developer from misreading D-A6 as "patches never lose focus".

## Deviations from Plan

None - plan executed exactly as written.

Task 1 had already been committed in a prior run against the same base; this continuation run executed Task 2 and Task 3 exactly per the plan's `<action>` blocks with no fix-ups. Both `npm run check` and `vitest` passed on the first attempt for both tasks.

## Issues Encountered

None.

Baseline `npm run check` reported 5 pre-existing errors (2 in `src/lib/components/ui/sonner/sonner.svelte` for unresolved `svelte-sonner` / `mode-watcher` module imports, 3 in `tests/helpers/schema-validator.ts` for Node built-in module resolution). All 5 predate Plan 12-04 and are logged in `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md`. Post-Plan-04 `npm run check` reports the same 5 errors — no new errors introduced.

## Verification Run

```bash
$ cd frontend && npm run check 2>&1 | tail -10
1775839612303 ERROR "src/lib/components/ui/sonner/sonner.svelte" 2:70 "Cannot find module 'svelte-sonner' or its corresponding type declarations."
1775839612303 ERROR "src/lib/components/ui/sonner/sonner.svelte" 3:23 "Cannot find module 'mode-watcher' or its corresponding type declarations."
1775839612303 ERROR "tests/helpers/schema-validator.ts" 4:21 "Cannot find module 'fs' or its corresponding type declarations."
1775839612303 ERROR "tests/helpers/schema-validator.ts" 5:23 "Cannot find module 'path' or its corresponding type declarations."
1775839612303 ERROR "tests/helpers/schema-validator.ts" 6:31 "Cannot find module 'url' or its corresponding type declarations."
1775839612303 COMPLETED 933 FILES 5 ERRORS 0 WARNINGS 2 FILES_WITH_PROBLEMS

$ npx vitest --run src/lib/store/surfaces.svelte.test.ts 2>&1 | tail -5
 Test Files  1 passed (1)
      Tests  9 passed (9)

$ npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation 2>&1 | tail -5
 Test Files  1 passed (1)
      Tests  2 passed (2)
```

## Threat Model Follow-Through

Plan 12-04's threat register (T-12-07..T-12-10) dispositions were all honored:

- **T-12-07 (DoS via cyclic children in gcOrphans)** — *mitigate*. `gcOrphans` uses a `reachable` visited-set that short-circuits re-visits; cycles produce a bounded walk. Unit test `gcOrphans preserves deep descendants` verifies transitive reachability. No test explicitly constructs a cyclic graph, but the visited-set invariant is a line-of-sight property of the implementation.
- **T-12-08 (large set-node payload memory)** — *accept*. No action (server is authoritative in this architecture).
- **T-12-09 (password leak via focus-preservation bug)** — *accept* + documented in the focus-preservation test file. The negative-control test explicitly states that focused-node replacement is not covered by D-A6, which aligns with the threat disposition.
- **T-12-10 (crafted surface-targeting bypass)** — *mitigate*. Fixed in Task 1 (commit 0bb8d02) — `applyPatch(msg.surface, msg.patch)`. No hardcoded `'main'` remains in `init.ts`; the pre-fix grep check (`applyPatch('main'`) returns no hits.

## Threat Flags

None. No new security-relevant surface introduced that isn't already in the plan's `<threat_model>`.

## Known Stubs

None. All code paths are wired end-to-end; no placeholder data, no TODO/FIXME, no empty-array-to-UI stubs. The dispatcher handles all 6 op variants, GC runs after every batch, and the focus-preservation contract is proven by a real browser test.

## Next Plan Readiness

- **Plan 12-05 (backend builders)** can now safely emit node patches targeting the surface store — the frontend will apply them correctly (all 6 variants) and preserve focus on sibling inputs under tree mutations.
- **Plan 12-06 (frontend shell components)** — AppShell and SurfaceMount can rely on the in-place mutation semantics for slot content updates without worrying about focus loss.
- **Plan 12-07 (CRM integration)** — the "select country → swap a field in place" demo flow has the frontend primitives it needs (`set-node` + `set-children` via applyPatch) and the D-A6 guarantee it requires.
- **Plan 12-08 (demo and E2E)** — the focus-preservation browser test pattern in this plan is the template for the E2E focus test.

## Self-Check: PASSED

Files verified on disk:
- `frontend/src/lib/store/surfaces.svelte.ts` — FOUND
- `frontend/src/lib/store/surfaces.svelte.test.ts` — FOUND
- `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` — FOUND
- `frontend/src/lib/store/data.svelte.ts` — FOUND (modified)
- `frontend/src/lib/index.ts` — FOUND (modified)
- `frontend/src/lib/transport/messages.ts` — FOUND (Task 1)
- `frontend/src/lib/init.ts` — FOUND (Task 1)

Commits verified in git log:
- `0bb8d02` — FOUND (Task 1, prior run)
- `bd9464f` — FOUND (Task 2)
- `a5c0883` — FOUND (Task 3)

Acceptance criteria spot-check:
- `grep` for 6 node-op cases in `surfaces.svelte.ts` → 6 matches (all 6 exported mutators present)
- `grep "case 'set-node'"` in `data.svelte.ts` → OK
- `grep "case 'insert-child'"` in `data.svelte.ts` → OK
- `grep 'gcOrphans(surface)'` in `data.svelte.ts` → OK
- `grep 'applyPatch(msg\.surface'` in `init.ts` → OK
- `grep "applyPatch('main'"` in `init.ts` → (correctly absent, fix is permanent)

---
*Phase: 12-protocol-node-patching-appshell*
*Completed: 2026-04-10*
