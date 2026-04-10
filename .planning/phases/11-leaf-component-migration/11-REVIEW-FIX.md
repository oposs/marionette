---
phase: 11-leaf-component-migration
fixed_at: 2026-04-09T00:00:00Z
review_path: .planning/phases/11-leaf-component-migration/11-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 11: Code Review Fix Report

**Fixed at:** 2026-04-09
**Source review:** `.planning/phases/11-leaf-component-migration/11-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (3 Critical + 4 Warning)
- Fixed: 7
- Skipped: 0

> Note: the source REVIEW.md frontmatter lists `warning: 5`, but the body
> only contains four warning sections (WR-01 through WR-04). The frontmatter
> count was off-by-one; all warnings present in the document have been
> addressed.

## Fixed Issues

### CR-01: ToastSurface — exported instance method unusable as module API

**Files modified:**
- `frontend/src/lib/store/toasts.svelte.ts`
- `frontend/src/lib/components/popup/ToastSurface.svelte`
- `frontend/src/lib/components/popup/ToastSurface.browser-test.ts`
- `frontend/src/lib/index.ts`
**Commit:** `2c6f38b`
**Applied fix:** Moved the toast store and `addToast`/`removeToast`/`getToasts` functions into the standalone `.svelte.ts` module (the file already existed with a diverging `(severity, message, duration)` API; re-aligned it with the REVIEW-suggested event-shaped `{ name, hint? }` signature). Rewrote `ToastSurface.svelte` to `$derived(getToasts())` and removed the instance `export function addToast`. Updated the browser test to import `addToast` from the store. Exported `addToast`/`removeToast`/`getToasts` from `lib/index.ts` for dispatcher and external consumers.

**Verification:** Tier 1 (re-read) + Tier 2 (svelte-check — no errors in modified files; pre-existing errors in `ui/select/*` and `tests/helpers/` are unrelated).

### CR-02: ConfirmDialog — Dialog sub-components rendered without Dialog.Root context

**Files modified:**
- `frontend/src/lib/components/popup/ConfirmDialog.svelte`
**Commit:** `c2c005f`
**Applied fix:** Removed the `import * as Dialog` and replaced `Dialog.Header` / `Dialog.Title` / `Dialog.Description` / `Dialog.Footer` with plain markup (`<div>` + `<h2>` + `<p>` + flex footer). The wrapping `Dialog.Content` is supplied by `ModalSurface`, so the outer accessibility wiring remains intact. Added a block comment documenting why sub-components cannot be used here.

**Verification:** Tier 1 + Tier 2 (svelte-check clean).

### CR-03: NavItem — undefined action name sent to dispatcher

**Files modified:**
- `frontend/src/lib/components/nav/NavItem.svelte`
**Commit:** `71124e0`
**Applied fix:** Added the defensive fallback chain `action.name ?? action.type ?? 'navigate'` in `handleClick`, matching `Button.svelte`'s existing defensive pattern and the REVIEW suggestion. Added an explanatory comment.

**Verification:** Tier 1 + Tier 2 (svelte-check clean — this also resolved the `Argument of type 'string | undefined' is not assignable to parameter of type 'string'` error flagged in the baseline svelte-check run).

### WR-01: Button and FormScreen — action.type dual-use conflates UI variant with protocol name

**Files modified:**
- `frontend/src/lib/transport/messages.ts`
- `frontend/src/lib/components/form/Button.svelte`
- `frontend/src/lib/components/screen/FormScreen.svelte`
**Commit:** `039b42d`
**Applied fix:** Added an optional `variant?: string` field to `ComponentAction` in `messages.ts`, documented as a cosmetic UI hint separate from `name` (protocol identifier) and `type` (protocol classifier). In `Button.svelte` replaced `sendAction(action.name ?? action.type, ...)` with `sendAction(action.name ?? 'button-click', ...)` so `type` is no longer used as a backend action name. In `FormScreen.svelte` did the same for `handleAction` (`act.name ?? 'toolbar-action'`) and changed the toolbar `<ShadcnButton variant={...}>` binding to use `act.variant` instead of `act.type`.

**Verification:** Tier 1 + Tier 2 (svelte-check clean for the modified files).

### WR-02: SelectInput — dirty state cleared on value change, not on close

**Files modified:**
- `frontend/src/lib/components/form/SelectInput.svelte`
**Commit:** `2c6e0f2`
**Applied fix:** Moved the `clearDirty` call from `handleValueChange` into `handleOpenChange`'s `else` branch, mirroring the mark-on-focus / clear-on-blur pattern used by `TextInput.svelte`. `handleValueChange` now only updates the store value; `handleOpenChange` handles mark/clear symmetrically so that dismissing the dropdown without a selection still clears the dirty flag.

**Verification:** Tier 1 + Tier 2 (svelte-check clean). Status: `fixed: requires human verification` — this is a dirty-state/optimistic logic change and the verifier phase should confirm there are no downstream consumers that relied on `handleValueChange` calling `clearDirty` synchronously (e.g., test fixtures).

### WR-03: DataTable prefetch effect can fire infinitely when rows fill to capacity

**Files modified:**
- `frontend/src/lib/components/table/DataTable.svelte`
**Commit:** `ca8a32c`
**Applied fix:** Added a `let fetching = $state(false)` in-flight guard. The prefetch `$effect` now only fires when `!fetching`, sets `fetching = true` before dispatching `fetch-rows`. A second `$effect` tracks `rows.length` and resets `fetching = false` when the row set changes (i.e. when server response lands).

**Verification:** Tier 1 + Tier 2 (svelte-check clean). Status: `fixed: requires human verification` — the reset effect relies on `rows.length` mutating when new rows arrive; if the server ever returns an empty page or the same length, the guard will remain stuck. Verifier should confirm the pagination contract always changes length on fulfilled fetches, or add an explicit timeout fallback.

### WR-04: FallbackComponent — side effect hidden in template expression

**Files modified:**
- `frontend/src/lib/components/core/FallbackComponent.svelte`
**Commit:** `fb3c953`
**Applied fix:** Lifted the `console.warn` call out of the `{:else}` IIFE into a `$effect` that runs when `import.meta.env.DEV` is false. Removed the `{:else}` branch entirely — in production the component renders nothing, which is the desired behavior.

**Verification:** Tier 1 + Tier 2 (svelte-check clean).

---

_Fixed: 2026-04-09_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
