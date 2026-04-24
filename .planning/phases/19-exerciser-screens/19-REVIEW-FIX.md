---
phase: 19-exerciser-screens
fixed_at: 2026-04-24T14:45:00Z
review_path: .planning/phases/19-exerciser-screens/19-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 19: Code Review Fix Report

**Fixed at:** 2026-04-24T14:45:00Z
**Source review:** `.planning/phases/19-exerciser-screens/19-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (Critical + Warning; Info excluded by scope `critical_warning`)
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: Unbounded 30-second `setTimeout` in perf auto-arm is never cancellable

**Files modified:** `frontend/src/lib/exer03/perf.svelte.ts`
**Commit:** 796acf0
**Applied fix:** Took WR-01's second acceptable option ("add a code comment explicitly documenting the known-bounded leak"). Refactoring the module-level auto-arm into a cancellable `autoArm()` closure parallel to EXER-02 would be a non-trivial structural change that the reviewer explicitly flagged as deferred-to-v1.3. Added a block comment in `perf.svelte.ts` above the `if (typeof window !== 'undefined')` guard that (a) enumerates the three lingering resources (t+30s setTimeout, scroll listener until first fire, MutationObserver on document.body), (b) calls out that the behaviour is intentionally non-symmetric with EXER-02's `autoArm()` cleanup, and (c) points future maintainers at the v1.3 seed `.planning/seeds/v1.3-exerciser-instrumentation.md` for the cancellability work. No runtime behaviour change.

### WR-02: `void import(...).then(...)` silently swallows module-load rejections

**Files modified:** `frontend/src/lib/init.ts`
**Commit:** f766d82
**Applied fix:** Replaced the three `void import(...)` statements with explicit `.catch((e) => console.error('[marionette] failed to load <module>', e))` handlers, matching the error-hygiene style of the rest of `init.ts`. The `exer02` chain keeps its `.then((m) => m.autoArm())` and adds a single `.catch` at the tail so both a load failure and an `autoArm()` throw surface through the same handler. Added a block comment above the imports explaining the rationale (UAT would otherwise see "invariants never fire" with no console signal). Verified `src/lib/init.patchprobe.test.ts` (5 tests) still passes and no new `tsc --noEmit` errors appear in `init.ts`.

## Skipped Issues

None.

---

_Fixed: 2026-04-24T14:45:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
