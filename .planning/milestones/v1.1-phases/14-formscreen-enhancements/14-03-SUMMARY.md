---
phase: 14-formscreen-enhancements
plan: 03
subsystem: ui
tags: [form, field, select-input, shadcn-svelte, svelte5, backend-builder, bits-ui]

# Dependency graph
requires:
  - phase: 14-01
    provides: "shadcn-svelte Field family primitives (Field.Field, Field.Label, Field.Description, Field.Error)"
  - phase: 14-02
    provides: "TextInput.svelte exemplar for D-B1 Shared Leaf Anatomy (Field.Field wrap pattern) + backend .description/.full_width builder extension template"
  - phase: 12
    provides: "Country-select change-action dispatch flow (D-A6 focus preservation, D-B15 toast lifecycle) — MUST NOT regress"
provides:
  - "SelectInput.svelte rewritten with internal Field.Field wrap per D-B1 (Shared Leaf Anatomy)"
  - "SelectInput supports props.description (D-B3, Field.Description)"
  - "SelectInput supports props.full_width (D-C4, col-span-full wrapper override)"
  - "Mount-time UUID fallback for field id (D-B4) — Field.Label for/Select.Trigger id matching preserved"
  - "Backend Select builder accepts .description(...), .full_width(...), .placeholder(...), .disabled(...) helpers"
  - "Phase 12 country-select change-action dispatch preserved byte-identical"
  - "13 new SelectInput browser-test assertions covering Field.Field anatomy + change-action dispatch contract"
affects:
  - "14-04 (Checkbox rewrite) — same Field anatomy pattern"
  - "14-05 (Textarea new) — same Field anatomy pattern + bits-ui Select-Trigger-style description-omitted-on-error convention"
  - "14-06 (RadioGroup + Switch new) — same Field anatomy pattern"
  - "14-08 (CRM migration) — contact-form company/country Select handlers consume the rewritten component"

# Tech tracking
tech-stack:
  added: []  # No new libraries — Plan 14-01 already installed Field primitives; bits-ui already present
  patterns:
    - "Attribute-presence semantics on Select.Trigger: `aria-invalid={hasError || undefined}` — shadcn Pitfall #4 rule applied consistently across TextInput + SelectInput"
    - "Manual pointerdown + pointerup + click event dispatch in browser tests for bits-ui Select — the SelectTriggerState.onpointerdown gate requires the full pointer sequence; plain locator.click() skips the pointer-down path and portal never mounts in headless Chromium"
    - "Mocking $lib/transport/dispatcher with vi.mock for dispatch-contract tests — mirrors Button.browser-test.ts + Form.browser-test.ts pattern"

key-files:
  created: []
  modified:
    - "frontend/src/lib/components/form/SelectInput.svelte (rewritten with Field.Field anatomy)"
    - "frontend/src/lib/components/form/SelectInput.browser-test.ts (13 new assertions, total 16 tests)"
    - "backend/crates/marionette/src/builders/standard.rs (Select struct + 4 new optional fields + 3 tests)"
    - "frontend/.gitignore (added .vitest-attachments/ and **/__screenshots__/ runtime artifacts)"

key-decisions:
  - "Used data-slot=\"select-trigger\" attribute selectors in browser tests instead of getByRole('combobox') — the bits-ui SelectTriggerState.props derivation does NOT set role=\"combobox\"; it uses aria-haspopup=\"listbox\" + aria-expanded + pointerdown/click handlers. The plan's spec'd getByRole locator was incorrect."
  - "Dispatched synthetic pointerdown + pointerup + click events on the Select.Trigger in the change-action dispatch test — vitest-browser-svelte's Playwright locator.click() does not synthesize the pointer-down sequence that bits-ui's SelectTriggerState.onpointerdown gates the open flow on. The manual pointer sequence faithfully reproduces the user gesture and keeps the dispatch-payload assertion genuine."
  - "Preserved the Phase 12 handleValueChange body verbatim — the `{ ...(action.payload ?? {}), ...surfaceData }` merge is byte-identical to the pre-rewrite shape (line 47-52 of the new SelectInput.svelte), so the country-select node-patch demo cannot regress on payload shape."

patterns-established:
  - "Field.Field wrap for Select leaf: <Field.Field data-invalid={hasError || undefined} class={full_width ? 'col-span-full' : undefined}><Field.Label for={id}>{label}</Field.Label><Select.Root><Select.Trigger id={id} aria-invalid={hasError || undefined}>...</Select.Trigger><Select.Content>...</Select.Content></Select.Root><Field.Description>{description}</Field.Description>(no error)<Field.Error>{error}</Field.Error></Field.Field>"
  - "Backend Select struct parity fields: the Svelte component has always read props.placeholder + props.disabled; Plan 14-03 surfaces them in the Rust builder for typed handler ergonomics. Same shape will land on any primitive where the frontend reads a prop the backend struct forgot to declare."
  - "Browser-test dispatch-contract pattern: vi.mock('$lib/transport/dispatcher') → render component with action prop → synthesize the real user gesture → assert sendAction.mock.calls[0] matches the expected payload shape. The bits-ui-specific pointer-sequence detail is the novel part; the rest mirrors Button/Form patterns."

requirements-completed: [FORM-01]

# Metrics
duration: 11m 22s
completed: 2026-04-17
---

# Phase 14 Plan 03: SelectInput Field Anatomy + Builder Parity Summary

**SelectInput rewritten with shadcn Field.Field anatomy (label/trigger/description/error), backend Select builder gains description + full_width + placeholder + disabled helpers, and the Phase 12 country-select change-action dispatch path is preserved byte-identical — FORM-01 compliance for the second leaf in Wave 1.**

## Performance

- **Duration:** 11 min 22 s
- **Started:** 2026-04-17T22:34:34Z
- **Completed:** 2026-04-17T22:45:56Z
- **Tasks:** 3
- **Files modified:** 4 (2 frontend + 1 backend + 1 gitignore)

## Accomplishments

- `SelectInput.svelte` now follows the D-B1 Shared Leaf Anatomy from `14-UI-SPEC.md` (same pattern Plan 14-02 landed for TextInput): a single `<Field.Field>` wrap with `Field.Label for={id}`, `<Select.Trigger id={id} aria-invalid={…}>`, conditional `<Field.Description>`, and conditional `<Field.Error>`. Attribute-presence semantics (`data-invalid / aria-invalid = hasError || undefined`) neutralize shadcn Pitfall #4.
- `props.description` (D-B3) and `props.full_width` (D-C4) wired end-to-end (backend builder → protocol → Svelte). The `col-span-full` override on the Field.Field wrapper is identical to the TextInput pattern.
- Backend `Select` struct gains four new optional fields: `description` + `full_width` (the D-B3/D-C4 additions) plus `placeholder` + `disabled` (parity fields — the Svelte component already read `props.placeholder` and `props.disabled` before this plan, but the Rust builder never exposed them for typed construction).
- Three new backend unit tests pin the serialization: description + full_width present, placeholder + disabled present, all four omitted when unset. The `required` legacy optional is also asserted as still omitted by default — the new fields don't disturb existing serialization output.
- Phase 12's country-select node-patch demo (D-A6 focus preservation, D-B15 toast lifecycle) is preserved: `handleValueChange` retains the identical `{ ...(action.payload ?? {}), ...surfaceData }` merge, `handleOpenChange` keeps the dirty-tracking mirror-open/close semantics. A browser test pins the dispatch payload shape.
- Field.Label `for={fieldId}` matches the `<Select.Trigger id={fieldId}>` — both resolve to handler-supplied `props.id` or a mount-time `crypto.randomUUID()` fallback (D-B4). Stable across rerenders, unique across two separate component instances.

## Task Commits

1. **Task 1: Extend SelectInput browser tests with Field-anatomy + change-action assertions** — `18d0639` (test)
2. **Task 2: Rewrite SelectInput.svelte with Field.Field anatomy + full_width + stable id** — `66ee1cf` (feat)
3. **Task 3: Extend backend Select builder with description + full_width + placeholder + disabled** — `465be38` (feat)

## SelectInput Before / After

**Before** (`frontend/src/lib/components/form/SelectInput.svelte`, 79 lines):

```svelte
<script lang="ts">
    import * as Select from '$lib/components/ui/select';
    import { Label } from '$lib/components/ui/label';
    import { getAllData, getData, setData } from '$lib/store/data.svelte';
    import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
    import { sendAction } from '$lib/transport/dispatcher';
    import type { ComponentAction } from '$lib/transport/messages';
    import type { Snippet } from 'svelte';

    let { props = {}, bind, action, surface, children }: {
        // ...
        children?: Snippet;
    } = $props();

    let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
    let options = $derived(/* ... */);

    function handleValueChange(newValue: string) {
        if (bind) setData(surface, bind, newValue);
        if (action?.type === 'change' && action.name) {
            // merged payload dispatch — preserved
        }
    }
    function handleOpenChange(open: boolean) { /* markDirty / clearDirty */ }
</script>

<div class="grid w-full gap-2">
    {#if props.label}
        <Label class="font-semibold">{props.label}</Label>
    {/if}
    <Select.Root …>
        <Select.Trigger class="w-full">…</Select.Trigger>
        <Select.Content>…</Select.Content>
    </Select.Root>
</div>
```

**After** (107 lines, +28 net):

```svelte
<script lang="ts">
    import * as Field from '$lib/components/ui/field';
    import * as Select from '$lib/components/ui/select';
    import { getAllData, getData, setData } from '$lib/store/data.svelte';
    import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
    import { sendAction } from '$lib/transport/dispatcher';
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

    let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
    let options = $derived(/* ... */);
    let fieldError = $derived(
        bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
    );
    let hasError = $derived(!!fieldError);

    function handleValueChange(newValue: string) {
        if (bind) setData(surface, bind, newValue);
        // Phase 12 node-patch demo — D-A6. Keep verbatim.
        if (action?.type === 'change' && action.name) {
            const surfaceData = getAllData(surface) ?? {};
            const payload = {
                ...((action.payload as Record<string, unknown>) ?? {}),
                ...surfaceData,
            };
            sendAction(action.name, payload, action.target);
        }
    }
    function handleOpenChange(open: boolean) { /* markDirty / clearDirty preserved */ }
</script>

<Field.Field
    data-invalid={hasError || undefined}
    class={props.full_width ? 'col-span-full' : undefined}
>
    {#if props.label}
        <Field.Label for={fieldId}>{props.label}</Field.Label>
    {/if}
    <Select.Root type="single" {value}
        onValueChange={handleValueChange}
        onOpenChange={handleOpenChange}
        disabled={props.disabled as boolean}>
        <Select.Trigger id={fieldId} class="w-full"
            aria-invalid={hasError || undefined}>
            …selected-value or placeholder…
        </Select.Trigger>
        <Select.Content>{#each options as opt}<Select.Item …/>{/each}</Select.Content>
    </Select.Root>
    {#if props.description && !hasError}
        <Field.Description>{props.description}</Field.Description>
    {/if}
    {#if fieldError}
        <Field.Error>{fieldError}</Field.Error>
    {/if}
</Field.Field>
```

Net diff:

1. Outer `<div class="grid w-full gap-2">` + `<Label class="font-semibold">` replaced with `<Field.Field>` + `<Field.Label for={id}>` — the shadcn recipe.
2. `id={fieldId}` now lives on `<Select.Trigger>` (the interactive element; see deviation #1 re: role), matched to `Field.Label for={fieldId}` for screen-reader/click-to-focus.
3. `aria-invalid={hasError || undefined}` on `<Select.Trigger>` drives shadcn's generated `aria-invalid:ring-destructive aria-invalid:border-destructive` styling on the trigger — no more ad-hoc border override.
4. `props.description` + `props.full_width` consumed per the shared Field anatomy.
5. `children: Snippet` prop removed — SelectInput is a leaf, not a parent.
6. Everything else (`handleValueChange`, `handleOpenChange`, `Select.Content`/`Select.Item` wiring) is byte-identical to the pre-rewrite code — the D-A6 focus-preservation + node-patch demo cannot regress.

## Change-Action Dispatch Payload — Byte-Identical Match

Verification that the Phase 12 country-select demo's dispatch contract is preserved:

```
$ grep -n -A 6 "action?.type === 'change'" frontend/src/lib/components/form/SelectInput.svelte
47:     if (action?.type === 'change' && action.name) {
48:         const surfaceData = getAllData(surface) ?? {};
49:         const payload = {
50:             ...((action.payload as Record<string, unknown>) ?? {}),
51:             ...surfaceData,
52:         };
53:         sendAction(action.name, payload, action.target);
```

Matched against the pre-rewrite source (lines 38-45 of the old SelectInput.svelte, read earlier in this plan): identical spread order `{ ...action.payload, ...surfaceData }`, identical `sendAction(name, payload, target)` signature, identical guard `action?.type === 'change' && action.name`. The browser-test assertion (test 16: "change-action dispatch fires with merged payload on value change") pins this shape with a live dispatch at runtime.

## Backend Select Struct Before / After

**Before** (`backend/crates/marionette/src/builders/standard.rs:55-62`):

```rust
#[derive(ComponentBuilder)]
#[component(type = "select")]
pub struct Select {
    pub label: String,
    pub options: Vec<SelectOption>,
    #[builder(optional)]
    pub required: Option<bool>,
}
```

**After** (+27 lines of new fields, +85 lines with doc comments and tests):

```rust
#[derive(ComponentBuilder)]
#[component(type = "select")]
pub struct Select {
    pub label: String,
    pub options: Vec<SelectOption>,
    #[builder(optional)]
    pub required: Option<bool>,
    /// Backend-authoritative placeholder text rendered inside the trigger
    /// when no value is selected. Phase 14 Plan 03: parity field.
    #[builder(optional)]
    pub placeholder: Option<String>,
    /// Disabled state passthrough. Phase 14 Plan 03: parity field.
    #[builder(optional)]
    pub disabled: Option<bool>,
    /// Helper text rendered via shadcn `Field.Description` (D-B3).
    #[builder(optional)]
    pub description: Option<String>,
    /// Full-row span inside a FieldSet grid (D-C4).
    #[builder(optional)]
    pub full_width: Option<bool>,
}
```

The `ComponentBuilder` derive auto-generates `.description(impl Into<String>)`, `.full_width(bool)`, `.placeholder(impl Into<String>)`, `.disabled(bool)` setters. Existing handler call sites (`handlers/contact.rs` `Select::new("Company", options).id(...).bind(...).build()`, and the country-select demo) continue to compile — all new fields are `Option<...>` with `#[builder(optional)]`.

## Test Count Delta

| File | Before | After | Delta |
|------|--------|-------|-------|
| `frontend/src/lib/components/form/SelectInput.browser-test.ts` | 3 | 16 | +13 |
| `backend/crates/marionette/src/builders/standard.rs::tests` (select coverage) | 1 (filter_select_serializes_with_options) | 4 | +3 |

Plan's acceptance bar was ≥ 6 browser-test `test(...)` blocks; landed with 16 (well above). The three new backend tests exactly match the plan's spec:

- `select_serializes_description_and_full_width`
- `select_serializes_placeholder_and_disabled`
- `select_omits_new_optionals_when_not_set`

The omits-when-unset test also asserts that the pre-existing `required` optional stays omitted by default — an extra guard that the new fields don't disturb existing serialization behavior.

## Verification Commands (all passing as of commit 465be38)

```bash
# Frontend
cd frontend && npx vitest --config vitest-browser.config.ts \
  src/lib/components/form/SelectInput.browser-test.ts --run      # 16/16 green
cd frontend && npx vitest --config vitest-browser.config.ts \
  src/lib/components/form/TextInput.browser-test.ts \
  src/lib/components/form/Form.browser-test.ts --run             # 26/26 green (no regression)
cd frontend && npm run check                                     # 3 pre-existing errors only

# Backend
cd backend && cargo test -p marionette select_                   # 4/4 green
cd backend && cargo test -p marionette                           # 50 + 6 + 3 + 5 = 64 tests, all green
cd backend && cargo build -p crm-demo                            # clean, no regression in CRM handlers

# Change-action dispatch preservation
grep -c "action?.type === 'change'" \
  frontend/src/lib/components/form/SelectInput.svelte            # 1 match
```

## Decisions Made

- **Mount-time `crypto.randomUUID()` fallback captured in a plain `const`, not `$state`.** Same pattern as TextInput (Plan 14-02). Accessed via `$derived((props.id as string) ?? fallbackId)` so handler-supplied `props.id` wins; the fallback is captured once per component instance and is stable across rerenders. Safe per STACK.md (SPA-only, no SSR).
- **Used `[data-slot="select-trigger"]` attribute selectors in browser tests instead of `getByRole('combobox')`.** The plan's spec suggested the role-based locator, but the bits-ui `SelectTriggerState.props` derivation does not set `role="combobox"` — it sets `aria-haspopup="listbox"`, `aria-expanded`, and relies on pointerdown/click handlers. The `data-slot` selector is unambiguous and matches the shadcn-svelte wrapper's authoritative attribute.
- **Synthesized `pointerdown` + `pointerup` + `click` events in the dispatch-contract test.** vitest-browser-svelte's Playwright `locator.click()` does not produce the pointer-down event that bits-ui's `onpointerdown` handler gates the dropdown open flow on. The full pointer sequence is what a real user produces and is what makes the portal mount reliably; without it the dropdown never opens in headless Chromium and the test times out on the `Switzerland` item locator.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Plan's spec'd `getByRole('combobox', { name: 'Country' })` locator would never match**

- **Found during:** Task 2 (first GREEN attempt after the component rewrite).
- **Issue:** The plan's Task 1 `<action>` block suggested `page.getByRole('combobox', { name: 'Country' })` for locating the Select.Trigger. Running this under the new component produced a 15-second timeout — the locator never resolved. Diagnosed by dumping the trigger's attributes in a throwaway debug test: the bits-ui SelectTriggerState (the state class exported as `SelectTriggerState` and used by `select-trigger.svelte`) has a `props = $derived.by(() => ({ ... }))` body that sets `aria-haspopup: "listbox"`, `aria-expanded`, `data-state`, `onpointerdown`, `onkeydown`, `onclick`, `onpointerup` — but it does NOT set `role: "combobox"`. Only the separate `SelectInputState` class (a different combobox-input variant) sets the role. The rendered DOM is `<button data-slot="select-trigger" id="…" aria-haspopup="listbox" aria-expanded="false" type="button">…</button>` — no role attribute.
- **Fix:** Used `[data-slot="select-trigger"]` attribute selectors for every structural assertion (consistent with the existing baseline test `renders select trigger`, which already used the same selector). For the interactive dispatch test, kept the locator-based click on the role-less trigger but also added the synthetic pointer sequence (see deviation #2). No test relies on `getByRole('combobox')`.
- **Files modified:** `frontend/src/lib/components/form/SelectInput.browser-test.ts` (all thirteen new tests use `data-slot` selectors).
- **Verification:** 16/16 tests pass (`npx vitest … SelectInput.browser-test.ts --run`).
- **Committed in:** `66ee1cf` (Task 2 commit — the fix was applied during the Task 1 → Task 2 GREEN iteration).

**2. [Rule 1 — Bug] Plain `locator.click()` on the trigger does not open the dropdown**

- **Found during:** Task 2, same iteration as deviation #1.
- **Issue:** After fixing the locator to use `data-slot`, the dispatch-contract test still timed out because `await locator.click()` on the trigger produced a click event but didn't open the dropdown — the `Switzerland` item never mounted. Diagnosis: the bits-ui `SelectTriggerState.onpointerdown` handler is the actual open-flow gate (see `bits-ui/dist/bits/select/select.svelte.js` line 687-700 in the installed package). `SelectTriggerState.onclick` alone is not sufficient to open the portal; it requires a preceding pointerdown event to set up the gesture state. Playwright's `locator.click()` synthesizes a click event, not the full pointerdown/pointerup/click sequence that a real mouse down-up produces in Chromium.
- **Fix:** Synthesized the full pointer sequence manually before the locator click: `trigger.dispatchEvent(new PointerEvent('pointerdown', { ... }))` + `pointerup` + `trigger.click()`. The portal mounts reliably after this sequence; the `getByText('Switzerland').click()` that follows resolves within the usual retry-wait.
- **Files modified:** `frontend/src/lib/components/form/SelectInput.browser-test.ts` (test 16, the change-action dispatch test).
- **Verification:** 16/16 tests pass; dispatch assertion confirms the payload shape matches the expected `{ contactForm: { name: 'Alice', country: 'CH' } }` merge.
- **Committed in:** `66ee1cf` (Task 2 commit).

**3. [Rule 3 — Blocking] Worktree's `frontend/node_modules` was empty + `.svelte-kit/tsconfig.json` missing**

- **Found during:** Task 1 verification (`npx vitest` reported a Vite tsconfig resolution failure).
- **Issue:** Parallel-executor worktrees start with an empty `node_modules` directory. Vite's tsconfig loader also needs `.svelte-kit/tsconfig.json`, which is generated by `svelte-kit sync` on demand. The first test run failed with "failed to resolve 'extends' in frontend/tsconfig.json".
- **Fix:** Ran `npm ci` once in the worktree (~20 s) to hydrate `node_modules`, then `npx svelte-kit sync` to regenerate the `.svelte-kit` tsconfig. This is a one-shot parallel-worktree hygiene cost, identical to the issue documented in the 14-02 SUMMARY (Issues Encountered).
- **Files modified:** None (neither `node_modules/` nor `.svelte-kit/` is tracked).
- **Verification:** Baseline SelectInput test runs clean, all subsequent test invocations work.
- **Committed in:** N/A — this is tooling state, not source.

**4. [Rule 2 — Missing critical infra] Runtime test artifacts polluting `git status`**

- **Found during:** Task 1 commit.
- **Issue:** Failing browser tests write `.vitest-attachments/` and per-test `__screenshots__/` directories under `frontend/`. Neither is in `frontend/.gitignore`. Running the RED test baseline created untracked directories that would have been committed if the executor had used `git add .` (which the GSD protocol forbids — it requires explicit per-file staging — but belt-and-braces still applies).
- **Fix:** Added `.vitest-attachments/` and `**/__screenshots__/` to `frontend/.gitignore`.
- **Files modified:** `frontend/.gitignore`.
- **Verification:** `git status --short` on a failing-test run no longer lists the two directories.
- **Committed in:** `18d0639` (Task 1 commit, rolled in).

---

**Total deviations:** 4 auto-fixed (2 × Rule 1 test-framework-reality mismatches with the plan's spec'd locator/event code, 1 × Rule 3 worktree hygiene, 1 × Rule 2 gitignore hygiene).
**Impact on plan:** Zero scope creep. All spec'd Select behaviors are asserted (Field.Field anatomy, description, full_width, stable id, aria-invalid, data-invalid, change-action dispatch). The locator/event adjustments are bits-ui-specific reality checks — every test still pins the same semantic contract the plan's spec described. The gitignore + worktree fixes are tooling hygiene, not scope.

## Issues Encountered

- **Vite pre-bundling race on first cold test run.** The first `npx vitest … --run` invocation after `npm ci` showed "Vite unexpectedly reloaded a test" and 3 transient test failures, but the cache-warm second invocation immediately produced a clean 3/3 green (and later 16/16 green). Standard vitest-browser cold-start dance; no action.
- **Svelte-check reports 3 pre-existing errors in `tests/helpers/schema-validator.ts`.** Same 3 errors Plan 14-01 and 14-02 observed (`Cannot find module 'fs' / 'path' / 'url'` — a Node-types config issue). Pre-existing, out of scope per the phase's deferred-items.md. Not introduced by Plan 14-03.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **Wave 1 sibling unblocked:** Plan 14-04 (Checkbox rewrite) can adopt the same Field.Field-wrap pattern + `.description` / `.full_width` / `.placeholder` / `.disabled` backend extensions. The "parity fields" decision (surface props the frontend already reads) is a template for any future primitive that finds itself in the same shape.
- **Wave 2 leaves (14-05 Textarea, 14-06 RadioGroup/Switch):** The bits-ui pointer-sequence test pattern established here applies to any primitive whose `onpointerdown`/`onclick` handlers gate interactivity. RadioGroup and Switch use different bits-ui state classes — confirm per-primitive during those plans.
- **Wave 3 (14-07 FieldSet):** Consumes the `col-span-full` contract already wired end-to-end on TextInput + SelectInput.
- **Wave 4 (14-08 CRM migration):** The existing `Select::new("Company", …)` and `Select::new("Country", …)` call sites in `handlers/contact.rs` continue to compile unchanged. Handlers can now opt into `.description(…)` / `.full_width(…)` / `.placeholder(…)` / `.disabled(…)` when the migration plan lands.

No blockers. No open questions. The Phase 12 country-select demo's dispatch path has test coverage locked in — future refactors will fail `tests/SelectInput.browser-test.ts::change-action dispatch fires with merged payload on value change` if the payload shape regresses.

## Known Stubs

None. Every rendered element has a concrete data source (handler-provided `props.label / description / placeholder / options`, store-provided `value` and `/_errors/{bind}`, dispatcher-supplied `sendAction`). No hardcoded empty props, no "coming soon" text, no TODO markers.

## Self-Check: PASSED

Verified post-SUMMARY:

```
FOUND: frontend/src/lib/components/form/SelectInput.svelte
FOUND: frontend/src/lib/components/form/SelectInput.browser-test.ts
FOUND: backend/crates/marionette/src/builders/standard.rs
FOUND: frontend/.gitignore
FOUND: .planning/phases/14-formscreen-enhancements/14-03-SUMMARY.md
FOUND: commit 18d0639 (Task 1 — RED tests)
FOUND: commit 66ee1cf (Task 2 — SelectInput rewrite, GREEN)
FOUND: commit 465be38 (Task 3 — backend builder + 3 new tests)
```

---

*Phase: 14-formscreen-enhancements*
*Completed: 2026-04-17*
