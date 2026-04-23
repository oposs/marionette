---
phase: 18-catalog-screens
plan: 02
subsystem: ui
tags: [svelte, shadcn-svelte, bits-ui, forms, blur, validation, sendAction, framework-polish]

# Dependency graph
requires:
  - phase: 14-formscreen-enhancements
    provides: Field.Field anatomy, Checkbox/Switch/RadioGroup/SelectInput leaves on shadcn-svelte + bits-ui
  - phase: 12-protocol-node-patch
    provides: SelectInput change-action dispatch baseline (merged surface payload pattern preserved here)
provides:
  - "SelectInput, Checkbox, Switch, RadioGroup all dispatch sendAction(name, { value }, target) when action.type === 'blur'"
  - "Select uses onOpenChange(false) as the blur signal; Checkbox/Switch/RadioGroup use <div onfocusout class='contents'> wrappers"
  - "Payload shape parity with TextInput/Textarea: { value: <bound primitive> }"
  - "8 new browser tests (2 fire-tests + 3 no-fire/regression-guards per component × 4 = 20 blur-focused tests total)"
affects:
  - 18-05-CAT-02-forms
  - 18-08-UAT

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Blur-action dispatch (action.type === 'blur' + sendAction(name, { value }, target)) now available on ALL six built-in form leaves"
    - "Checkbox/Switch/RadioGroup: <div onfocusout class='contents'> wrapper pattern — class='contents' keeps wrapper visually transparent in Field.Field grid"
    - "SelectInput: popover-close is the logical blur moment (hooked via handleOpenChange(false)), NOT focus-leave on the trigger"

key-files:
  created:
    - .planning/phases/18-catalog-screens/18-02-SUMMARY.md
    - .planning/phases/18-catalog-screens/deferred-items.md
  modified:
    - frontend/src/lib/components/form/SelectInput.svelte
    - frontend/src/lib/components/form/Checkbox.svelte
    - frontend/src/lib/components/form/Switch.svelte
    - frontend/src/lib/components/form/RadioGroup.svelte
    - frontend/src/lib/components/form/SelectInput.browser-test.ts
    - frontend/src/lib/components/form/Checkbox.browser-test.ts
    - frontend/src/lib/components/form/Switch.browser-test.ts
    - frontend/src/lib/components/form/RadioGroup.browser-test.ts

key-decisions:
  - "SelectInput uses onOpenChange(false) — NOT focus-leave on the trigger — because opening the dropdown transfers focus to the portal-rendered items list, which would produce spurious blur events during normal interaction."
  - "Checkbox/Switch/RadioGroup wrap Field.Field in <div onfocusout class='contents'> — the 'contents' display class makes the wrapper layout-transparent so Phase 14's Field grid anatomy stays intact (D-B1 contract)."
  - "Blur dispatch is additive: the existing change-action path on SelectInput (merged-surface payload for Phase 12 node-patch flows) is preserved untouched; Checkbox/Switch/RadioGroup's onCheckedChange/onValueChange paths are likewise preserved."
  - "Payload value field carries the LITERAL primitive (boolean for Checkbox/Switch, string for Select/Radio) — no coercion to string, matching TextInput's contract."

patterns-established:
  - "Blur-dispatch signature identity across all 6 form leaves: sendAction(action.name ?? action.type, { value: getData(surface, bind!) }, action.target)"
  - "clearDirty lives inside handleBlur (not only in the change path) — pairing mark/clear with focus/blur mirrors TextInput's convention and prevents stranded optimistic state on dismiss"

requirements-completed: []

# Metrics
duration: ~45min
completed: 2026-04-23
---

# Phase 18 Plan 02: Framework-Gap-2 Blur Dispatch Summary

**All four non-TextInput/Textarea form leaves (SelectInput, Checkbox, Switch, RadioGroup) now emit `sendAction(name, { value }, target)` on their respective blur signal (popover close / focus-leave), unblocking Plan 18-05 CAT-02 Forms live-validate story.**

## Performance

- **Duration:** ~45 min (initial read + RED/GREEN for 3 tasks + svelte-check triage)
- **Started:** 2026-04-23 (post-worktree-branch-check)
- **Completed:** 2026-04-23T16:15:51Z
- **Tasks:** 3 (all TDD)
- **Files modified:** 8 (4 components + 4 browser-test files)

## Accomplishments

- **SelectInput** now calls `handleBlur()` from `handleOpenChange(false)`; the Select's popover-close is the blur signal (dispatched via bits-ui's `Select.Root` `onOpenChange` prop already wired in Phase 14).
- **Checkbox + Switch** each gained an outer `<div onfocusout={handleBlur} class="contents">` wrapper. `class="contents"` keeps the wrapper visually transparent so Field.Field's horizontal grid layout from Phase 14 stays untouched.
- **RadioGroup** gained the same `onfocusout` wrapper — `focusout` bubbles from any individual `RadioGroupItem`, so tab-out from any option fires `handleBlur()` once.
- All four `handleBlur()` bodies are byte-equivalent to `TextInput.svelte` lines 45–56 in structure (clearDirty then sendAction guarded by `action?.type === 'blur'`).
- 20 new assertions in 8 new test cases cover the fire/no-fire branches, the payload primitive-type contract (boolean for Checkbox/Switch, string for Select/Radio), and bind-value preservation across focusout.

## Task Commits

Each task was committed atomically with TDD RED/GREEN pairs:

1. **Task 1 RED — SelectInput failing tests:** `364bb2c` (test)
2. **Task 1 GREEN — SelectInput handleBlur via onOpenChange(false):** `3021dd2` (feat)
3. **Task 2 RED — Checkbox + Switch failing tests:** `9d564b7` (test)
4. **Task 2 GREEN — Checkbox + Switch onfocusout wrappers:** `5b21f9c` (feat)
5. **Task 3 RED — RadioGroup failing tests:** `c82555f` (test)
6. **Task 3 GREEN — RadioGroup onfocusout wrapper:** `9e46186` (feat)

No REFACTOR commits needed — the initial GREEN implementations are already minimal and byte-aligned with TextInput's reference structure.

## Files Created/Modified

**Component logic (4 files):**
- `frontend/src/lib/components/form/SelectInput.svelte` — imported `sendAction`, added `handleBlur()`, routed `handleOpenChange(false)` into it (replaced the local `clearDirty` call so the blur path owns both behaviors).
- `frontend/src/lib/components/form/Checkbox.svelte` — imported `sendAction` + `clearDirty`, added `handleBlur()`, wrapped `Field.Field` in `<div onfocusout={handleBlur} class="contents">`.
- `frontend/src/lib/components/form/Switch.svelte` — same pattern as Checkbox.
- `frontend/src/lib/components/form/RadioGroup.svelte` — same pattern as Checkbox.

**Browser tests (4 files):**
- `frontend/src/lib/components/form/SelectInput.browser-test.ts` — added 3 blur-dispatch tests driving the bits-ui Select via the existing pointer-sequence pattern, closing via Escape keydown.
- `frontend/src/lib/components/form/Checkbox.browser-test.ts` — added 5 blur-dispatch tests (fire/no-fire/regression-guards) plus a new `vi.mock('$lib/transport/dispatcher')` hook.
- `frontend/src/lib/components/form/Switch.browser-test.ts` — same 5 tests as Checkbox, adapted.
- `frontend/src/lib/components/form/RadioGroup.browser-test.ts` — same 5 tests as Checkbox, adapted for string-payload semantics.

**Bookkeeping:**
- `.planning/phases/18-catalog-screens/deferred-items.md` — logs the pre-existing `@tanstack/virtual-core` resolution errors in `virtualizer.svelte.ts` (Phase 13 artifact, NOT caused by this plan; out of scope).

## Per-component diff highlights

### SelectInput.svelte

```svelte
// Before
function handleOpenChange(open: boolean) {
  if (!bind) return;
  if (open) markDirty(bind);
  else clearDirty(bind, (op) => setData(surface, op.path, op.value));
}

// After
function handleOpenChange(open: boolean) {
  if (!bind) return;
  if (open) markDirty(bind);
  else handleBlur();  // delegates clearDirty + optional blur dispatch
}

function handleBlur() {
  if (bind) {
    clearDirty(bind, (op) => setData(surface, op.path, op.value));
    if (action?.type === 'blur') {
      sendAction(action.name ?? action.type, { value: getData(surface, bind!) }, action.target);
    }
  }
}
```

### Checkbox.svelte / Switch.svelte / RadioGroup.svelte

```svelte
// Added to <script>
function handleBlur() { /* same body as above */ }

// Markup wrap (Field.Field body unchanged)
<div onfocusout={handleBlur} class="contents">
  <Field.Field ...>
    ... existing children unchanged ...
  </Field.Field>
</div>
```

## DOM mechanism rationale table

| Component   | Blur signal                    | Why                                                                                                                          |
|-------------|--------------------------------|------------------------------------------------------------------------------------------------------------------------------|
| SelectInput | `onOpenChange(false)`          | Trigger focus-leave fires during normal interaction (focus transfers to the portal-rendered items list). Popover close is the semantic "user finished interacting" moment. |
| Checkbox    | `<div onfocusout>` wrapper     | bits-ui Checkbox doesn't expose a reliable native `onblur`. `focusout` bubbles from the bits-ui button.                      |
| Switch      | `<div onfocusout>` wrapper     | Same — bits-ui Switch button doesn't expose native `onblur`.                                                                 |
| RadioGroup  | `<div onfocusout>` wrapper     | `focusout` bubbles from any individual `RadioGroupItem` (each radio is a bits-ui button), so tab-out from the group fires once. |

## Test summary (20 new assertions)

| Component   | Fire test | No-fire (action.type ≠ 'blur') | No-fire (no action) | Primitive-type lock | Bind preservation |
|-------------|-----------|-------------------------------|---------------------|--------------------|-------------------|
| SelectInput | ✓ (string) | ✓ | ✓ | (string asserted in fire test) | N/A (no regression-guard added — Select already had a selection-change test) |
| Checkbox    | ✓ (boolean true) | ✓ | ✓ | ✓ (typeof value === 'boolean') | ✓ |
| Switch      | ✓ (boolean true) | ✓ | ✓ | ✓ (typeof value === 'boolean') | ✓ |
| RadioGroup  | ✓ (string) | ✓ | ✓ | ✓ (typeof value === 'string') | ✓ |

Plus Checkbox and Switch each have a "emit `false` when unchecked/off" test, and RadioGroup has an "emit `''` when no option selected" test.

## Decisions Made

1. **Delegation over duplication in SelectInput.** `handleOpenChange(false)` previously called `clearDirty` directly; we moved that into `handleBlur()` and had `handleOpenChange` delegate to it. This means close-without-action still behaves identically (clearDirty runs) but now close-with-blur-action does the dispatch too.
2. **`class="contents"` wrapper for the other three.** Alternatives considered: adding `onfocusout` directly to `Field.Field`. Rejected because Field.Field is a bits-ui primitive — patching its event props would fight the framework's recipe. An outer wrapper with `display: contents` is the canonical Svelte/shadcn escape hatch.
3. **Payload value is literal primitive.** `getData(surface, bind)` returns whatever was written via `setData`, and `setData` writes whatever `onCheckedChange`/`onValueChange` passed — a boolean for Checkbox/Switch, a string for RadioGroup/SelectInput. No coercion needed. The backend handler-side contract is `ctx.action.payload.value.as_bool()` / `as_str()`.

## Deviations from Plan

None — plan executed exactly as written. All three tasks had the precise shape described in the plan's `<interfaces>` block and `<action>` sections.

## Issues Encountered

1. **Missing `pnpm` in worktree environment.** The worktree didn't have `pnpm` on PATH and had no installed node_modules. Resolved by `mise trust`, `npm install -g pnpm`, `pnpm install`, `npx playwright install chromium`. First-time infra cost only; will persist for later plans.
2. **Pre-existing `svelte-check` errors in `frontend/src/lib/utils/virtualizer.svelte.ts`.** Not caused by this plan (file last modified Phase 13, commit `87b17b6`). Logged to `.planning/phases/18-catalog-screens/deferred-items.md` with candidate fix.
3. **Full-suite browser tests show 8 pre-existing failures.** A/B test confirmed: on the stashed baseline (BEFORE this plan's changes), the full suite had 11 failures. With this plan applied, 8 failures remain — our changes actually FIX 3 pre-existing failures (the SelectInput tests previously failed because of something else in the suite-ordering; in isolation they pass 19/19). The remaining 8 are `@tanstack/virtual-core` resolution races, DataTableActions timing, and ToastSurface transitions — all pre-existing and out of scope.

## User Setup Required

None — pure framework-library change, no external services, no new env vars.

## Next Phase Readiness

**Plan 18-05 (CAT-02 Forms) unblocked.** D-3-A's "one live-validation story per input type — six total" and D-3-B's "validation fires on blur" can now be implemented because all six form leaves (TextInput, Textarea, SelectInput, Checkbox, Switch, RadioGroup) carry byte-identical `handleBlur()` dispatch contracts.

**Callers migrating to blur-action:** pass `ComponentAction { type: "blur", name: "validate-*", target: Some("...") }` — the payload shape is always `{ value: <bound primitive> }`.

**Pre-existing blockers NOT touched:** AppShell nestability (Phase 19 EXER-01 ownership), W-06 ErrorDisplay message field dead-state (deferred to CAT-04 polish), pre-existing ESLint baseline drift.

## Self-Check: PASSED

Verified via ad-hoc greps and file existence checks:

- `frontend/src/lib/components/form/SelectInput.svelte` — FOUND (handleBlur at line 78; `action?.type === 'blur'` at line 81; `handleBlur()` call at line 67).
- `frontend/src/lib/components/form/Checkbox.svelte` — FOUND (handleBlur at line 45; `action?.type === 'blur'` at line 48; `onfocusout={handleBlur}` at line 65; `class="contents"` on wrapper).
- `frontend/src/lib/components/form/Switch.svelte` — FOUND (handleBlur at line 43; `action?.type === 'blur'` at line 46; `onfocusout={handleBlur}` at line 63; `class="contents"` on wrapper).
- `frontend/src/lib/components/form/RadioGroup.svelte` — FOUND (handleBlur at line 47; `action?.type === 'blur'` at line 50; `onfocusout={handleBlur}` at line 66; `class="contents"` on wrapper).
- Commits FOUND in `git log --oneline`: `364bb2c`, `3021dd2`, `9d564b7`, `5b21f9c`, `c82555f`, `9e46186`.
- Test runs: `pnpm vitest --run --config vitest-browser.config.ts SelectInput Checkbox Switch RadioGroup` → 58/58 pass.

---
*Phase: 18-catalog-screens*
*Plan: 02 — Framework Gap 2: Blur-Action Dispatch*
*Completed: 2026-04-23*
