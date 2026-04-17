---
phase: 14-formscreen-enhancements
plan: 02
subsystem: ui
tags: [form, field, text-input, shadcn-svelte, svelte5, backend-builder, bug-fix]

# Dependency graph
requires:
  - phase: 14-01
    provides: "shadcn-svelte Field family primitives (Field.Field, Field.Label, Field.Description, Field.Error, Field.Group) + NodeRenderer D-E2 unmount fix"
  - phase: 13
    provides: "TextInput props.input_type (backend-authoritative snake_case) — D-H4a baseline preserved"
provides:
  - "TextInput.svelte rewritten with internal Field.Field wrap per D-B1 (Shared Leaf Anatomy)"
  - "TextInput supports props.description (D-B3, helperText retired — no back-compat alias)"
  - "TextInput supports props.full_width (D-C4, col-span-full wrapper override)"
  - "Mount-time UUID fallback for field id (D-B4) — Field.Label for/input id matching preserved"
  - "Backend TextInput builder accepts .description(...) and .full_width(...) helpers"
  - "Form.svelte wraps children in Field.Group with space-y-6 (D-A3, 24px sibling-FieldSet rhythm)"
  - "Form.svelte error banner upgraded to UI-SPEC Form banner styling (bg-destructive/10, border-destructive/50, text-destructive, rounded-md)"
  - "14 new TextInput browser-test assertions covering Field.Field anatomy, attribute-presence semantics (Pitfall #4), label-click focus, id stability"
  - "2 new Form browser-test assertions covering Field.Group wrap + banner styling"
affects:
  - "14-03 (SelectInput rewrite) — same Field anatomy pattern + .description/.full_width additions"
  - "14-04 (Checkbox rewrite) — same Field anatomy pattern"
  - "14-05 (Textarea new) — same Field anatomy pattern"
  - "14-06 (RadioGroup + Switch new) — same Field anatomy pattern"
  - "14-07 (FieldSet new) — Field.Group space-y-6 rhythm + per-field full_width consumed"
  - "14-08 (CRM migration + FormScreen delete) — every handler call site consumes the rewritten TextInput"

# Tech tracking
tech-stack:
  added: []  # No new libraries — Plan 01 already installed Field/Textarea/RadioGroup/Switch primitives
  patterns:
    - "Per-leaf internal Field.Field wrap (D-B1) — TextInput is the exemplar for Wave 2+3 leaves"
    - "Attribute-presence semantics — `attr={value || undefined}` pattern for data-invalid + aria-invalid (shadcn convention, Pitfall #4)"
    - "Mount-time UUID fallback with $derived override — stable id across rerenders while honoring handler-supplied props.id"
    - "Surface pre-seeding in browser tests — setFullState(surface, {}) before render-with-bind to avoid state_unsafe_mutation inside $derived (Svelte 5 rule)"
    - "cleanup() between renders in a single test — vitest-browser-svelte shares the DOM root across render() calls, so multi-render tests must cleanup"

key-files:
  created: []
  modified:
    - "frontend/src/lib/components/form/TextInput.svelte (rewritten with Field.Field anatomy)"
    - "frontend/src/lib/components/form/TextInput.browser-test.ts (14 new assertions, total 22 tests)"
    - "frontend/src/lib/components/form/Form.svelte (Field.Group space-y-6 wrap + banner upgrade)"
    - "frontend/src/lib/components/form/Form.browser-test.ts (2 new assertions, total 4 tests)"
    - "backend/crates/marionette/src/builders/standard.rs (TextInput + description/full_width + 3 tests)"

key-decisions:
  - "Mount-time crypto.randomUUID() fallback for fieldId — captured in a plain const (not $state), referenced via $derived so handler-supplied props.id wins. Safe per STACK.md (SPA-only, no SSR). Stable across rerenders."
  - "Split the spec'd single data-invalid/aria-invalid test into four separate tests (no-error and error cases as distinct test() blocks) because vitest-browser-svelte shares the DOM root across render() calls within one test — queries against baseElement return the FIRST match, not the most recent."
  - "Surface pre-seeding (setFullState(surface, {})) added to no-error tests. The initial render of a TextInput with bind but no prior setData would otherwise trigger getStore auto-create inside $derived, which Svelte 5 rejects as state_unsafe_mutation. The seed is idempotent: surfaces initialize to {} anyway, and real server-driven flows always deliver data before bind-sensitive components mount."

patterns-established:
  - "Field.Field wrap per leaf: <Field.Field data-invalid={hasError || undefined} class={full_width ? 'col-span-full' : undefined}><Field.Label for={id}>{label}</Field.Label><Control .../><Field.Description>{description}</Field.Description>(no error)<Field.Error>{error}</Field.Error></Field.Field>"
  - "Backend TextInput-style struct: description + full_width as Option<String> / Option<bool> with #[builder(optional)] — the same shape will land on SelectInput, Checkbox, Textarea, RadioGroup, Switch in Wave 1/2"
  - "Form.svelte = <form class='shrink-0 overflow-y-auto'>{banner}?<Field.Group class='space-y-6'>{children}</Field.Group></form> — UI-SPEC authoritative"

requirements-completed: [FORM-01]

# Metrics
duration: 10min
completed: 2026-04-17
---

# Phase 14 Plan 02: TextInput Field Anatomy + Form Grouping Summary

**TextInput rewritten with shadcn Field.Field anatomy (label/input/description/error), Form.svelte wraps children in Field.Group space-y-6, and backend TextInput builder gains description + full_width helpers — FORM-01 compliance for the most-used form leaf.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-04-17T22:17:54Z
- **Completed:** 2026-04-17T22:28:26Z
- **Tasks:** 3
- **Files modified:** 5 (3 frontend, 1 backend, 1 new-pattern doc pass)

## Accomplishments

- TextInput.svelte now follows the D-B1 Shared Leaf Anatomy from 14-UI-SPEC.md: a single `<Field.Field>` wrap with `Field.Label for={id}`, `<Input id={id}>`, conditional `<Field.Description>`, and conditional `<Field.Error>`. Attribute-presence semantics (`data-invalid / aria-invalid = hasError || undefined`) neutralize shadcn Pitfall #4.
- `props.helperText` is fully gone (D-B3). The only helper-text source is `props.description`. `grep -r 'helperText' frontend/src` returns zero matches; `grep -rn 'helper_text' backend/crates` returns zero matches — the no-back-compat rename landed clean.
- `props.full_width` (D-C4) maps to `col-span-full` on the Field.Field wrapper — the per-field FieldSet override is wired end-to-end (backend builder → protocol → Svelte).
- Backend `TextInput` struct gains `.description(...)` and `.full_width(...)` fluent helpers via the existing `ComponentBuilder` derive. Three new unit tests pin the serialization (description stringified, full_width boolean, both omitted when unset).
- `Form.svelte` children are now wrapped in `<Field.Group class="space-y-6">` (D-A3, 24px sibling-FieldSet rhythm per 14-UI-SPEC.md §Spacing Scale rule 4). The error banner matches the UI-SPEC styling (`bg-destructive/10 border border-destructive/50 text-destructive rounded-md p-4 mb-4`).
- D-E1 password regression test stays green — Phase 13's `props.input_type`-only fix is preserved under the new Field.Field wrap.
- Field.Label `for={fieldId}` matches the `<Input id={fieldId}>` — both resolve to handler-supplied `props.id` or a mount-time `crypto.randomUUID()` fallback (D-B4). Label-click focus works, id is stable across rerenders, two component instances produce distinct ids.

## Task Commits

1. **Task 1: Extend TextInput browser tests with Field anatomy assertions** - `48d83d1` (test)
2. **Task 2: Rewrite TextInput.svelte with Field.Field wrap + full_width + stable id** - `9771bfd` (feat)
3. **Task 3: Add description/full_width to backend TextInput builder + D-A3 Form.svelte tweak** - `9a2b847` (feat)

## TextInput Before / After

**Before** (`frontend/src/lib/components/form/TextInput.svelte`):

```svelte
<script lang="ts">
    import { Input } from '$lib/components/ui/input';
    import { Label } from '$lib/components/ui/label';
    ...
    let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
    let fieldError = $derived(
        bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
    );
    ...
</script>

<div class="grid w-full gap-2">
    {#if props.label}
        <Label class="font-semibold">{props.label}</Label>
    {/if}
    <Input
        type={(props.input_type as string) ?? 'text'}
        ...
        class={fieldError ? 'border-destructive' : ''}
    />
    {#if fieldError}
        <p class="text-xs text-destructive">{fieldError}</p>
    {:else if props.helperText}
        <p class="text-xs text-muted-foreground">{props.helperText}</p>
    {/if}
</div>
```

**After**:

```svelte
<script lang="ts">
    import * as Field from '$lib/components/ui/field';
    import { Input } from '$lib/components/ui/input';
    ...
    const fallbackId = crypto.randomUUID();
    let fieldId = $derived((props.id as string) ?? fallbackId);
    let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
    let fieldError = $derived(
        bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
    );
    let hasError = $derived(!!fieldError);
    ...
</script>

<Field.Field
    data-invalid={hasError || undefined}
    class={props.full_width ? 'col-span-full' : undefined}
>
    {#if props.label}
        <Field.Label for={fieldId}>{props.label}</Field.Label>
    {/if}
    <Input
        id={fieldId}
        type={(props.input_type as string) ?? 'text'}
        ...
        aria-invalid={hasError || undefined}
    />
    {#if props.description && !hasError}
        <Field.Description>{props.description}</Field.Description>
    {/if}
    {#if fieldError}
        <Field.Error>{fieldError}</Field.Error>
    {/if}
</Field.Field>
```

Net diff: the ad-hoc `<div class="grid w-full gap-2"><Label>...<Input class={error?'border-destructive':''}><p>...` layout is replaced with the shadcn `Field.*` recipe. The `Input` primitive's built-in `aria-invalid:ring-destructive aria-invalid:border-destructive` utilities now carry the error state — no more `class={fieldError ? 'border-destructive' : ''}` ad-hoc override. The `helperText` fallback branch is gone.

## Form.svelte Before / After

**Before**:

```svelte
<form onsubmit={handleSubmit} class="space-y-4 shrink-0 overflow-y-auto">
    {#if Array.isArray(formErrors) && formErrors.length > 0}
        <div class="rounded-lg bg-destructive/10 p-4">
            {#each formErrors as error}
                <p class="text-sm text-destructive">{error}</p>
            {/each}
        </div>
    {/if}
    {@render children?.()}
</form>
```

**After**:

```svelte
<form onsubmit={handleSubmit} class="shrink-0 overflow-y-auto">
    {#if Array.isArray(formErrors) && formErrors.length > 0}
        <div
            class="bg-destructive/10 border border-destructive/50 text-destructive rounded-md p-4 mb-4"
        >
            {#each formErrors as error}
                <p class="text-sm">{error}</p>
            {/each}
        </div>
    {/if}
    <Field.Group class="space-y-6">
        {@render children?.()}
    </Field.Group>
</form>
```

Changes: `space-y-4` moved off `<form>` onto a new inner `<Field.Group class="space-y-6">` (24px rhythm between sibling FieldSets — UI-SPEC §Spacing Scale rule 4). Banner gained `border border-destructive/50 text-destructive rounded-md mb-4` + per-error `<p class="text-sm">` (color cascades from parent).

## Backend TextInput Struct Before / After

**Before** (`backend/crates/marionette/src/builders/standard.rs:23-35`):

```rust
#[derive(ComponentBuilder)]
#[component(type = "text-input")]
pub struct TextInput {
    pub label: String,
    #[builder(optional)] pub placeholder: Option<String>,
    #[builder(optional)] pub required: Option<bool>,
    #[builder(optional)] pub input_type: Option<String>,
    #[builder(optional)] pub disabled: Option<bool>,
}
```

**After**:

```rust
#[derive(ComponentBuilder)]
#[component(type = "text-input")]
pub struct TextInput {
    pub label: String,
    #[builder(optional)] pub placeholder: Option<String>,
    #[builder(optional)] pub required: Option<bool>,
    #[builder(optional)] pub input_type: Option<String>,
    #[builder(optional)] pub disabled: Option<bool>,
    /// Helper text rendered below the input via shadcn `Field.Description`
    /// (Phase 14 D-B3). Replaces the retired `helperText` prop — pre-deployment
    /// posture, no back-compat alias.
    #[builder(optional)] pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column of
    /// its parent `FieldSet` grid (Phase 14 D-C4).
    #[builder(optional)] pub full_width: Option<bool>,
}
```

The `ComponentBuilder` derive auto-generates `.description(impl Into<String>)` and `.full_width(bool)` setters — handler usage is `TextInput::new("Email").description("Optional helper text.").full_width(true).build()`.

## Test Count Delta

| File | Before | After | Delta |
|------|--------|-------|-------|
| `frontend/src/lib/components/form/TextInput.browser-test.ts` | 8 | 22 | +14 |
| `frontend/src/lib/components/form/Form.browser-test.ts` | 2 | 4 | +2 |
| `backend/crates/marionette/src/builders/standard.rs::tests` (text_input coverage) | 1 | 4 | +3 |

Plan advertised `+7` new TextInput cases and `+2` new Form cases. Actual +14 on TextInput: I split the spec'd single "data-invalid present-or-omitted" test into two (no-error and error) separate test() blocks, same for aria-invalid (see Deviations §1), and added a separate "full_width omitted" complement for full_width. These are all spec'd behaviors — the split is for test-harness reliability, not additional scope.

## Decisions Made

- **Mount-time UUID fallback captured in a plain `const`, not `$state`.** Svelte 5's `const` at the top of `<script>` runs once per component instance and is not reactive. Accessing it through `$derived((props.id as string) ?? fallbackId)` keeps the id stable across rerenders while letting handler-supplied `props.id` win. An alternative with `$state.raw` was evaluated but offered no benefit — the value never needs to change.
- **Surface pre-seeding in no-error browser tests** (`setFullState(surface, {})` before render). The reactive data store's `getStore` auto-creates a per-surface entry on first read, which mutates the `$state`-backed `surfaces` map. Svelte 5 treats that as `state_unsafe_mutation` when it happens inside a `$derived`. Seeding the surface to `{}` up-front makes the first `getData` call a pure read. Real server-driven flows don't hit this path — the server always sends a full-state patch before a bind-sensitive component mounts.
- **Two-render tests split into single-render tests where possible.** vitest-browser-svelte reuses the same DOM root across `render()` calls within one test; `baseElement.querySelector('[data-slot="field"]')` returns the first match in document order. Splitting "present-or-omitted" tests removes the ambiguity. Where a genuine two-render semantic IS needed (id uniqueness across independent instances), I added a `cleanup()` call between them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Two-render tests collided on a shared DOM root**

- **Found during:** Task 2 (first GREEN-run of the browser-test suite).
- **Issue:** The plan's spec for the data-invalid/aria-invalid tests had each `test(...)` perform two renders (one no-error, one with-error) and query each screen's `baseElement`. vitest-browser-svelte shares the same DOM root across successive `render()` calls in a single test, so both `screen1.baseElement.querySelector(...)` and `screen2.baseElement.querySelector(...)` returned the FIRST Field.Field wrapper (the no-error one), making the error assertion fail — and similarly for the stable-id test where the second render's id comparison returned the first instance's id for both.
- **Fix:** Split the two merged tests into four single-render `test(...)` blocks (no-error and error as distinct tests) for data-invalid and aria-invalid. Added a `cleanup()` call between the two renders in the stable-id uniqueness test (where both instances genuinely must exist in the same test to compare ids across independent mounts).
- **Files modified:** `frontend/src/lib/components/form/TextInput.browser-test.ts`
- **Verification:** `npx vitest .../TextInput.browser-test.ts --run` exits 0, 22/22 pass.
- **Committed in:** `9771bfd` (Task 2 commit).

**2. [Rule 1 - Bug] `state_unsafe_mutation` in no-error tests with bind set**

- **Found during:** Task 2 first test run (GREEN attempt).
- **Issue:** The plan's spec for the no-error data-invalid/aria-invalid tests called `render(TextInput, { props: { ... }, bind: '/email', surface: 'test-no-err' })` without any prior store interaction for that surface. The reactive data store's `getStore` auto-creates a per-surface entry on first access, mutating `$state surfaces`. When this happens inside a `$derived` (as it does during TextInput's initial `fieldError` derivation), Svelte 5 rejects it as `state_unsafe_mutation`.
- **Fix:** Added `setFullState(surface, {})` at the top of the no-error tests to pre-create the surface entry before render. This is a test-only concern — production flows always receive a full-state patch from the server before a bind-sensitive leaf mounts.
- **Files modified:** `frontend/src/lib/components/form/TextInput.browser-test.ts`
- **Verification:** state_unsafe_mutation no longer fires; 22/22 tests pass.
- **Committed in:** `9771bfd` (Task 2 commit, rolled in).

---

**Total deviations:** 2 auto-fixed (both Rule 1 — test-only bugs in the spec'd test code, not behavior or scope changes).
**Impact on plan:** Zero scope creep. All spec'd TextInput behaviors are asserted; the auto-fixes tightened the test-harness mechanics to match vitest-browser-svelte's DOM-sharing behavior and Svelte 5's $derived rules. Final test count (22) exceeds the plan's ≥10 acceptance bar.

## Issues Encountered

- **npm ci missing in worktree** — the worktree's `frontend/node_modules` was empty at plan start. Ran `npm ci` once (~20s) to hydrate, then `npx svelte-check sync` to regenerate `.svelte-kit/tsconfig.json`. One-shot, not a blocker; subsequent vitest runs resolved immediately. This is a parallel-worktree hygiene cost, not a Plan 14-02 concern.
- **vitest "Vite unexpectedly reloaded a test" warning** — shown once per cold test run as Vite pre-bundles tailwind-variants/bits-ui/json-ptr/tailwind-merge. The test still passes on the same run (the reload is the optimized-deps dance completing). No action; pre-warming optimizeDeps is out of scope.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **Wave 1 siblings unblocked:** Plan 14-03 (SelectInput rewrite) and Plan 14-04 (Checkbox rewrite) can adopt the exact same Field.Field-wrap pattern landed here. The `description + full_width` builder extensions follow the template in `backend/crates/marionette/src/builders/standard.rs`'s new TextInput shape.
- **Wave 2 leaves (14-05 Textarea, 14-06 RadioGroup/Switch):** The shadcn primitives are already installed (Plan 14-01); the Field-anatomy pattern from TextInput is the template. New builders mirror the TextInput extension.
- **Wave 3 (14-07 FieldSet):** Consumes the `space-y-6` rhythm on Form.svelte + the per-field `full_width = col-span-full` contract.
- **Wave 4 (14-08 CRM migration):** Every existing `TextInput::new(...)` call site in `handlers/contact.rs` continues to compile — the new `description / full_width` helpers are optional and backward-compatible in the builder direction (even though the Svelte helperText prop was removed, no CRM handler used it per Plan 01's grep audit).

No blockers. No open questions.

## Known Stubs

None. Every rendered element has a concrete data source (handler-provided `props.label / description / placeholder / input_type`, store-provided `value` and `/_errors/{bind}`). No hardcoded empty props, no "coming soon" text, no TODO markers.

## Self-Check: PASSED

Verified post-SUMMARY:

```
FOUND: frontend/src/lib/components/form/TextInput.svelte
FOUND: frontend/src/lib/components/form/TextInput.browser-test.ts
FOUND: frontend/src/lib/components/form/Form.svelte
FOUND: frontend/src/lib/components/form/Form.browser-test.ts
FOUND: backend/crates/marionette/src/builders/standard.rs
FOUND: .planning/phases/14-formscreen-enhancements/14-02-SUMMARY.md
FOUND: commit 48d83d1 (Task 1 — RED tests)
FOUND: commit 9771bfd (Task 2 — TextInput rewrite, GREEN)
FOUND: commit 9a2b847 (Task 3 — backend builder + Form.svelte tweak)
```

---

*Phase: 14-formscreen-enhancements*
*Completed: 2026-04-17*
