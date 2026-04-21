---
phase: 14-formscreen-enhancements
plan: 04
subsystem: ui
tags: [form, field, checkbox, shadcn-svelte, svelte5, backend-builder, horizontal-orientation]

# Dependency graph
requires:
  - phase: 14-01
    provides: "shadcn-svelte Field family primitives (Field.Field with orientation=horizontal, Field.Label, Field.Description, Field.Error)"
  - phase: 14-02
    provides: "TextInput.svelte exemplar for D-B1 Shared Leaf Anatomy + backend .description/.full_width builder extension template"
  - phase: 14-03
    provides: "SelectInput.svelte additional Wave-1 Field-anatomy reference"
provides:
  - "Checkbox.svelte rewritten with internal Field.Field (orientation=\"horizontal\") wrap per D-B1 (Shared Leaf Anatomy) — 14-UI-SPEC §Component Visual Contracts – Checkbox"
  - "Checkbox supports props.description (D-B3, Field.Description below the row)"
  - "Checkbox supports props.full_width (D-C4, col-span-full wrapper override)"
  - "Mount-time UUID fallback for field id (D-B4) — Field.Label for/checkbox id matching preserved"
  - "Click-label-to-toggle works natively via <label for> association + bind write-through"
  - "Backend Checkbox builder accepts .description(...) and .full_width(...) helpers (existing .disabled preserved)"
  - "12 new Checkbox browser-test assertions covering Field.Field anatomy, horizontal orientation, attribute-presence semantics, label-click toggle, full_width override"
  - "3 new backend serialization tests"
affects:
  - "14-05 (Textarea new) — same Field anatomy pattern (default vertical orientation)"
  - "14-06 (RadioGroup + Switch new) — same Field anatomy pattern; Switch also uses horizontal orientation"
  - "14-07 (FieldSet new) — consumes per-field col-span-full contract established here"
  - "14-08 (CRM migration) — contact-form opt-in/consent checkbox handlers consume the rewritten component"

# Tech tracking
tech-stack:
  added: []  # No new libraries — Plan 14-01 already installed Field primitives
  patterns:
    - "Horizontal Field.Field orientation via orientation=\"horizontal\" prop — the Field primitive's tailwind-variants config maps this to flex-row + items-center + field-label auto-grow (ui/field/field.svelte line 9-18) AND sets data-orientation=\"horizontal\" on the wrapper for CSS hooks"
    - "Native click-label-to-toggle via <label for={id}> + <ShadcnCheckbox id={id} ...> — no onclick wiring needed; the browser's built-in label-for association delegates the click through to the bits-ui checkbox, which fires onCheckedChange and runs the bind write"
    - "Checkbox-first source order inside Field.Field (control before label) — matches 14-UI-SPEC.md Checkbox layout (16px checkbox + 8px gap + label). The Field.Field CSS handles the 8px gap via its base gap-3 utility; horizontal-orientation variant adds items-center alignment"

key-files:
  created:
    - ".planning/phases/14-formscreen-enhancements/14-04-SUMMARY.md"
  modified:
    - "frontend/src/lib/components/form/Checkbox.svelte (rewritten with horizontal Field.Field anatomy + stable id + description + full_width)"
    - "frontend/src/lib/components/form/Checkbox.browser-test.ts (12 new assertions, total 15 tests)"
    - "backend/crates/marionette/src/builders/standard.rs (Checkbox struct + description + full_width + 3 tests)"

key-decisions:
  - "Used orientation=\"horizontal\" as an explicit prop on <Field.Field> — the Field primitive (Plan 14-01 install) exposes orientation as a typed variant (vertical | horizontal | responsive) via tailwind-variants, so no class-based workaround or custom CSS was needed. data-orientation=\"horizontal\" on the wrapper is the test-stable hook; CSS utility classes (flex-row items-center) come from the recipe."
  - "Relied on the native <label for={id}> click-through mechanism for toggle-on-label-click. bits-ui's CheckboxPrimitive.Root is a <button role=\"checkbox\">, and browsers natively forward label clicks to an associated button via the for attribute — so setting Field.Label for={fieldId} and passing id={fieldId} to ShadcnCheckbox produces the contract without any onclick wiring."
  - "Placed ShadcnCheckbox BEFORE Field.Label in source order inside Field.Field. Visually this matches 14-UI-SPEC.md (checkbox 16px + 8px gap + label). Both would look identical in horizontal orientation regardless of order because the Field.Label variant has flex-auto (fills remaining space), but putting the control first is semantically closer to the 'checkbox | label' reading order and makes the test assertion 'checkbox is descendant of wrapper' more readable."

patterns-established:
  - "Horizontal Field.Field wrap for Checkbox-style leaves: <Field.Field orientation=\"horizontal\" data-invalid={hasError || undefined} class={full_width ? 'col-span-full' : undefined}><Control id={id} aria-invalid={hasError || undefined}/><Field.Label for={id}>{label}</Field.Label>{#if description && !hasError}<Field.Description>{description}</Field.Description>{/if}{#if error}<Field.Error>{error}</Field.Error>{/if}</Field.Field>"
  - "Backend Checkbox struct shape (mirrors TextInput/Select pattern): label + disabled + description (Option<String>) + full_width (Option<bool>), all #[builder(optional)] except label. Same template will land on Switch in Plan 14-06."

requirements-completed: [FORM-01]

# Metrics
duration: 6m 31s
completed: 2026-04-17
---

# Phase 14 Plan 04: Checkbox Field Anatomy (Horizontal Orientation) Summary

**Checkbox rewritten with shadcn Field.Field anatomy using `orientation="horizontal"` — the one structural difference from TextInput / SelectInput mandated by 14-UI-SPEC for the Checkbox primitive (checkbox + label inline on the same row, description / error flowing beneath). Backend Checkbox builder gains description + full_width helpers — FORM-01 compliance for the third leaf in Wave 1.**

## Performance

- **Duration:** 6 min 31 s
- **Started:** 2026-04-17T22:50:48Z
- **Completed:** 2026-04-17T22:57:19Z
- **Tasks:** 3
- **Files modified:** 3 (2 frontend + 1 backend)
- **Files created:** 1 (this SUMMARY)

## Accomplishments

- `Checkbox.svelte` now wraps its control in a single `<Field.Field orientation="horizontal">` with `<ShadcnCheckbox id={fieldId}>` followed by `<Field.Label for={fieldId}>` on the same row, plus conditional `<Field.Description>` / `<Field.Error>` below (the shadcn Field horizontal-orientation recipe handles the inline layout via its `flex-row items-center` variant). Attribute-presence semantics (`data-invalid / aria-invalid = hasError || undefined`) neutralize shadcn Pitfall #4.
- The `orientation="horizontal"` prop sets `data-orientation="horizontal"` on the wrapper (see `ui/field/field.svelte` line 48) — a stable CSS/test hook that distinguishes Checkbox/Switch-style leaves from the vertical-by-default leaves used for TextInput/SelectInput/Textarea/RadioGroup.
- `props.description` (D-B3) and `props.full_width` (D-C4) wired end-to-end (backend builder → protocol → Svelte). The `col-span-full` override on the Field.Field wrapper is identical to the TextInput / SelectInput pattern.
- Field.Label `for={fieldId}` matches the `<ShadcnCheckbox id={fieldId}>` — both resolve to handler-supplied `props.id` or a mount-time `crypto.randomUUID()` fallback (D-B4). A dedicated browser test proves clicking the label toggles the checkbox via the native `<label for>` → button-click forwarding, and the toggle writes `true` to the bound path via `setData(surface, bind, val === true)`.
- Backend `Checkbox` struct gains `.description(...)` and `.full_width(...)` fluent helpers via the existing `ComponentBuilder` derive. Three new unit tests pin the serialization (description + full_width present, omitted when unset, legacy `.disabled(true)` still works).
- The unused `children: Snippet` prop (declared in the old source but never rendered) is removed — Checkbox is a leaf, not a parent.

## Task Commits

1. **Task 1: Extend Checkbox browser tests with Field-anatomy assertions** — `99a52d0` (test)
2. **Task 2: Rewrite Checkbox.svelte with horizontal Field.Field anatomy** — `82d3b75` (feat)
3. **Task 3: Extend backend Checkbox builder with description + full_width** — `4d5a591` (feat)

## Checkbox.svelte Before / After

**Before** (`frontend/src/lib/components/form/Checkbox.svelte`, 41 lines):

```svelte
<script lang="ts">
    import { Checkbox as ShadcnCheckbox } from '$lib/components/ui/checkbox';
    import { Label } from '$lib/components/ui/label';
    import { getData, setData } from '$lib/store/data.svelte';
    import type { ComponentAction } from '$lib/transport/messages';
    import type { Snippet } from 'svelte';

    let {
        props = {},
        bind,
        action,
        surface,
        children,
    }: { /* includes unused children: Snippet */ } = $props();

    let checked = $derived(bind ? ((getData(surface, bind) as boolean) ?? false) : false);

    function handleCheckedChange(val: boolean | 'indeterminate') {
        if (bind) setData(surface, bind, val === true);
    }
</script>

<div class="flex items-center gap-2">
    <ShadcnCheckbox {checked} onCheckedChange={handleCheckedChange} disabled={props.disabled as boolean}/>
    {#if props.label}<Label class="font-semibold">{props.label}</Label>{/if}
</div>
```

**After** (61 lines, +20 net):

```svelte
<script lang="ts">
    import * as Field from '$lib/components/ui/field';
    import { Checkbox as ShadcnCheckbox } from '$lib/components/ui/checkbox';
    import { getData, setData } from '$lib/store/data.svelte';
    import type { ComponentAction } from '$lib/transport/messages';

    let { props = {}, bind, action, surface }: {
        props: Record<string, unknown>;
        bind?: string;
        action?: ComponentAction;
        surface: string;
    } = $props();

    // D-B4: stable id — handler-supplied wins; mount-time UUID fallback.
    const fallbackId = crypto.randomUUID();
    let fieldId = $derived((props.id as string) ?? fallbackId);

    let checked = $derived(bind ? ((getData(surface, bind) as boolean) ?? false) : false);
    let fieldError = $derived(
        bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
    );
    let hasError = $derived(!!fieldError);

    function handleCheckedChange(val: boolean | 'indeterminate') {
        if (bind) setData(surface, bind, val === true);
    }
</script>

<Field.Field
    orientation="horizontal"
    data-invalid={hasError || undefined}
    class={props.full_width ? 'col-span-full' : undefined}
>
    <ShadcnCheckbox
        id={fieldId}
        {checked}
        onCheckedChange={handleCheckedChange}
        disabled={props.disabled as boolean}
        aria-invalid={hasError || undefined}
    />
    {#if props.label}
        <Field.Label for={fieldId}>{props.label}</Field.Label>
    {/if}
    {#if props.description && !hasError}
        <Field.Description>{props.description}</Field.Description>
    {/if}
    {#if fieldError}
        <Field.Error>{fieldError}</Field.Error>
    {/if}
</Field.Field>
```

Net diff:

1. Outer `<div class="flex items-center gap-2">` replaced with `<Field.Field orientation="horizontal">` — the shadcn recipe handles the inline layout via its horizontal variant (`flex-row items-center` + `[&>[data-slot=field-label]]:flex-auto`).
2. `<Label class="font-semibold">` replaced with `<Field.Label for={fieldId}>` — click-to-toggle works natively via `<label for>` forwarding.
3. `id={fieldId}` + `aria-invalid={hasError || undefined}` now live on `<ShadcnCheckbox>` (both pass through via `...restProps` to the bits-ui `CheckboxPrimitive.Root`).
4. `data-invalid={hasError || undefined}` on the Field.Field wrapper.
5. `class={props.full_width ? 'col-span-full' : undefined}` on the wrapper for the per-field FieldSet grid override.
6. Conditional `<Field.Description>` (suppressed on error) and `<Field.Error>` follow the shadcn Field recipe pattern.
7. `children: Snippet` prop removed — Checkbox is a leaf, not a parent.

### Note on `orientation="horizontal"` — the primitive API matched the plan

The plan flagged that the shadcn Field primitive's actual orientation API should be verified before writing the test. Verification: `frontend/src/lib/components/ui/field/field.svelte` exports a `fieldVariants` tailwind-variants config with `orientation: { vertical, horizontal, responsive }` and accepts `orientation` as a typed `FieldOrientation` prop on the `<Field>` component (default `"vertical"`). Setting `orientation="horizontal"` does two things: (1) swaps class sets to `flex-row items-center [&>[data-slot=field-label]]:flex-auto` and friends, (2) writes `data-orientation="horizontal"` onto the wrapper div. The plan's spec'd API (`orientation="horizontal"`) was correct — no deviation required. The test asserts the `data-orientation` attribute as the stable hook.

## Backend Checkbox Struct Before / After

**Before** (`backend/crates/marionette/src/builders/standard.rs:85-91`):

```rust
#[derive(ComponentBuilder)]
#[component(type = "checkbox")]
pub struct Checkbox {
    pub label: String,
    #[builder(optional)]
    pub disabled: Option<bool>,
}
```

**After**:

```rust
#[derive(ComponentBuilder)]
#[component(type = "checkbox")]
pub struct Checkbox {
    pub label: String,
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered below the checkbox row via shadcn
    /// `Field.Description` (Phase 14 D-B3). Hidden while an
    /// `/_errors/{bind}` entry is active (the error replaces the
    /// description per the shadcn recipe).
    #[builder(optional)]
    pub description: Option<String>,
    /// When `true`, the field's `Field.Field` wrapper spans every column
    /// of its parent `FieldSet` grid (Phase 14 D-C4). Used for consent
    /// checkboxes that should take the full FieldSet row.
    #[builder(optional)]
    pub full_width: Option<bool>,
}
```

The `ComponentBuilder` derive auto-generates `.description(impl Into<String>)` and `.full_width(bool)` setters. Existing handler call sites (`Checkbox::new("Active").disabled(true).build()`) continue to compile — the new fields are `Option<...>` with `#[builder(optional)]`.

## Test Count Delta

| File | Before | After | Delta |
|------|--------|-------|-------|
| `frontend/src/lib/components/form/Checkbox.browser-test.ts` | 3 | 15 | +12 |
| `backend/crates/marionette/src/builders/standard.rs::tests` (checkbox coverage) | 0 (shared only through `all_19_standard_types`) | 3 | +3 |

Plan's acceptance bar was `≥ 5` browser-test blocks; landed with 15 (well above).

New backend tests:
- `checkbox_serializes_description_and_full_width`
- `checkbox_omits_new_optionals_when_not_set`
- `checkbox_preserves_existing_disabled_field`

Full marionette unit-test count is now 53 (up from 50 in Plan 14-03) — +3 for the new Checkbox coverage.

## Decisions Made

- **Mount-time `crypto.randomUUID()` fallback captured in a plain `const`, not `$state`.** Same pattern as TextInput (Plan 14-02) and SelectInput (Plan 14-03). Accessed via `$derived((props.id as string) ?? fallbackId)` so handler-supplied `props.id` wins; the fallback is captured once per component instance and is stable across rerenders. Safe per STACK.md (SPA-only, no SSR).
- **Used the Field primitive's typed `orientation="horizontal"` prop directly.** The plan flagged that the API should be verified; verification confirmed the prop is exposed exactly as spec'd. No class-based workaround needed. The `data-orientation="horizontal"` attribute on the wrapper (set by the Field primitive itself) is the stable test hook.
- **Relied on native `<label for={id}>` → control click-forwarding.** bits-ui's CheckboxPrimitive.Root renders as `<button role="checkbox">` and the browser natively forwards `<label for>` clicks to the associated button, which fires the button's `onclick`, which bits-ui translates to `onCheckedChange`. No manual `onclick` wiring on Field.Label is necessary — the label-click-to-toggle test uses a plain `label.click()` and verifies `getData(surface, '/optedIn')` flipped.
- **Source order: ShadcnCheckbox first, Field.Label second.** Visually identical to label-first under horizontal orientation (Field.Field's `flex-row items-center` + `[&>[data-slot=field-label]]:flex-auto` places the label next to the control regardless of source order). Picking control-first makes the DOM semantics read as "checkbox | label" top-down and simplifies test assertions like "checkbox is descendant of wrapper".

## Deviations from Plan

None. The plan executed exactly as written. All three tasks landed first-try:

- Task 1: 15 test blocks (≥ 5 acceptance bar met; 11 RED while implementation pending).
- Task 2: Component rewrite landed cleanly; all 15 tests flipped GREEN on the first run. `orientation="horizontal"` worked exactly as the primitive exposed it (no API drift from the plan's spec).
- Task 3: Backend builder + three tests compiled and passed cleanly; crm-demo still builds; all 53 marionette unit tests pass.

No auto-fixes required, no blocking issues, no architectural escalations.

## Issues Encountered

- **Parallel-worktree hygiene (not a plan concern):** the worktree's `frontend/node_modules/` and `.svelte-kit/` were empty at plan start. Ran `npm ci` (~11 s) + `npx svelte-kit sync` (~1 s) once to hydrate. Identical cost to Plans 14-02 and 14-03 (documented there as well). Not tracked as a deviation.
- **First-Write path confusion (corrected before commit):** the initial Task 1 Write targeted the main checkout path (`/home/oetiker/checkouts/marionette/frontend/...`) instead of the worktree path (`/home/oetiker/checkouts/marionette/.claude/worktrees/agent-a60245b7/frontend/...`). Detected immediately when the subsequent vitest run reported only 3 tests (the baseline). Reverted the main checkout file with `git checkout HEAD -- ...` and re-Wrote to the worktree path. No work was committed before the correction; no deviation from the plan.

## User Setup Required

None — no external service configuration required.

## Verification Commands (all passing as of commit `4d5a591`)

```bash
# Frontend
cd frontend && npx vitest --config vitest-browser.config.ts \
  src/lib/components/form/Checkbox.browser-test.ts --run       # 15/15 green
cd frontend && npx vitest --config vitest-browser.config.ts \
  src/lib/components/form/TextInput.browser-test.ts \
  src/lib/components/form/Form.browser-test.ts \
  src/lib/components/form/SelectInput.browser-test.ts --run    # 42/42 green (no regression)
cd frontend && npm run check                                   # 3 pre-existing errors only

# Backend
cd backend && cargo test -p marionette checkbox                # 3/3 green
cd backend && cargo test -p marionette                         # 53 + 6 + 3 + 5 + 1 ignored, all green
cd backend && cargo build -p crm-demo                          # clean

# Horizontal orientation hook preserved
grep "orientation=\"horizontal\"" \
  frontend/src/lib/components/form/Checkbox.svelte             # 1 match
```

## Next Phase Readiness

- **Wave 1 complete:** Plans 14-02 (TextInput), 14-03 (SelectInput), and 14-04 (Checkbox) have all landed the D-B1 Shared Leaf Anatomy. Three of the six form leaves are now FORM-01 compliant.
- **Wave 2 (Plans 14-05 Textarea, 14-06 RadioGroup/Switch):** Textarea follows the TextInput template (vertical orientation). RadioGroup follows the SelectInput template structurally (label above, control area below — vertical). Switch follows THIS plan's horizontal-orientation template — the Checkbox → Switch migration is near-mechanical: swap `ShadcnCheckbox` for `Switch`, swap `onCheckedChange` signature (boolean vs boolean|'indeterminate'), keep everything else identical.
- **Wave 3 (14-07 FieldSet):** The `col-span-full` contract is now wired on three leaves (TextInput, SelectInput, Checkbox). FieldSet's grid can consume it end-to-end.
- **Wave 4 (14-08 CRM migration):** Existing `Checkbox::new("X").disabled(true).build()` call sites continue to compile unchanged. Contact-form consent checkboxes (opt-in marketing, terms acceptance) can now opt into `.description("...")` and `.full_width(true)` when the migration plan lands.

No blockers. No open questions.

## Known Stubs

None. Every rendered element has a concrete data source (handler-provided `props.label / description`, store-provided `checked` via `bind` and `/_errors/{bind}`). No hardcoded empty props, no "coming soon" text, no TODO markers.

## Threat Flags

None. The surface introduced by this plan is identical in shape to TextInput (Plan 14-02) and SelectInput (Plan 14-03) — text injection via `props.label`/`props.description` is mitigated by Svelte's auto-escaping of `{expression}` interpolation (no `{@html}`); click-synthesis on disabled is handled natively by bits-ui (disabled pointer-events). Server re-validates every boolean write via the standard action-dispatch flow; no new network endpoint or trust boundary is introduced.

## Self-Check: PASSED

Verified post-SUMMARY:

```
FOUND: frontend/src/lib/components/form/Checkbox.svelte
FOUND: frontend/src/lib/components/form/Checkbox.browser-test.ts
FOUND: backend/crates/marionette/src/builders/standard.rs
FOUND: .planning/phases/14-formscreen-enhancements/14-04-SUMMARY.md
FOUND: commit 99a52d0 (Task 1 — RED tests)
FOUND: commit 82d3b75 (Task 2 — Checkbox rewrite, GREEN)
FOUND: commit 4d5a591 (Task 3 — backend builder + 3 new tests)
```

---

*Phase: 14-formscreen-enhancements*
*Completed: 2026-04-17*
