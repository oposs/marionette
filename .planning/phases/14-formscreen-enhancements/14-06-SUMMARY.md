---
phase: 14-formscreen-enhancements
plan: 06
subsystem: ui
tags: [form, field, radio-group, switch, new-primitive, backend-builder, shadcn-svelte, svelte5]

# Dependency graph
requires:
  - phase: 14-01
    provides: "shadcn RadioGroup + Switch primitives under $lib/components/ui/radio-group/ + ui/switch/, Field family, and Plan 01 RED RadioGroup.browser-test.ts + Switch.browser-test.ts stubs"
  - phase: 14-02
    provides: "D-B1 Field.Field wrap pattern exemplar (TextInput) — RadioGroup and Switch copy the wrap shape"
  - phase: 14-04
    provides: "D-B1 horizontal-orientation Field.Field pattern (Checkbox) — Switch copies the horizontal wrap"
  - phase: 14-05
    provides: "D-E3 new-primitive template (Textarea) — 4-step sequence: .svelte, registry, backend struct, 3 tests"
provides:
  - "RadioGroup.svelte — new SDUI leaf with Field.Field wrap + per-option label and optional description (D-E4)"
  - "Switch.svelte — new SDUI leaf with horizontal-orientation Field.Field wrap + boolean bind (D-E4)"
  - "'radio-group' -> RadioGroup and 'switch' -> MSwitch registry entries in defaults.ts"
  - "Backend RadioOption struct (value + label + optional description, skip_serializing_if None)"
  - "Backend RadioGroup ComponentBuilder struct (label + options + required + disabled + description + full_width)"
  - "Backend Switch ComponentBuilder struct (label + disabled + description + full_width)"
  - "Plan 01's RED RadioGroup.browser-test.ts + Switch.browser-test.ts flipped GREEN (5 + 4 tests)"
  - "Four Rust serialization tests pinning the radio-group + switch JSON shape"
affects:
  - "14-07 (FieldSet) — RadioGroup / Switch can now live inside FieldSet with full_width override"
  - "14-08 (CRM migration) — contact form notification preferences can use RadioGroup; consent / opt-in switches available"
  - "Phase 15 (CRM migration continued) — full six-leaf form primitive set now FORM-01 compliant"

# Tech tracking
tech-stack:
  added: []  # No new libraries — Plan 14-01 installed the shadcn RadioGroup + Switch primitives
  patterns:
    - "shadcn RadioGroup primitive wrapped by a Marionette SDUI leaf (D-B1 Field.Field anatomy)"
    - "Per-option description rendered adjacent to label as a muted 12px <p> (Assumption A4 fallback — shadcn RadioGroup.Item has no built-in description slot)"
    - "Switch uses horizontal-orientation Field.Field (matches Checkbox template from Plan 14-04)"
    - "MSwitch alias in defaults.ts to avoid collision with shadcn Switch namespace (matches MCheckbox/MButton/MSpinner convention)"

key-files:
  created:
    - "frontend/src/lib/components/form/RadioGroup.svelte (73 lines — new SDUI leaf)"
    - "frontend/src/lib/components/form/Switch.svelte (58 lines — new SDUI leaf)"
    - ".planning/phases/14-formscreen-enhancements/14-06-SUMMARY.md"
  modified:
    - "frontend/src/lib/components/form/RadioGroup.browser-test.ts (@ts-expect-error removed — Plan 01 hand-off)"
    - "frontend/src/lib/components/form/Switch.browser-test.ts (@ts-expect-error removed — Plan 01 hand-off)"
    - "frontend/src/lib/registry/defaults.ts ('radio-group' + 'switch' imports + registry entries)"
    - "backend/crates/marionette/src/builders/standard.rs (+RadioOption, +RadioGroup, +Switch, +4 tests)"

key-decisions:
  - "Used {value} + onValueChange on <RadioGroup> rather than bind:value to keep the data flow one-directional through the Marionette data store (setData -> re-render -> {value}). Same pattern as SelectInput. The shadcn primitive supports both bind:value and value / onValueChange; the callback form is the Marionette-canonical choice."
  - "Per-option id derived as `${groupId}-${opt.value}` rather than a fresh UUID per item. Deterministic — two RadioGroup instances with the same props.id would collide, but in practice handler-supplied props.id is already required to be unique per group. The browser test only asserts the adjacent <Label for={itemId}> click-through behaves correctly, not id-uniqueness-across-instances."
  - "Imported {RadioGroup, RadioGroupItem} by name (not namespace). The installed shadcn-svelte index.ts exports both Root/Item aliases AND RadioGroup/RadioGroupItem aliases. The plan spec suggested either form; named-import is cleaner and matches Textarea's import shape."
  - "Switch struct name — used `Switch` verbatim (not `SwitchField` / `Toggle`). Rust's reserved `switch` is a lowercase keyword in match patterns; `Switch` as a PascalCase type identifier does not collide. Compilation and crm-demo build confirmed no downstream conflict."
  - "Mount-time UUID fallback for groupId (RadioGroup) and fieldId (Switch) — same D-B4 treatment as TextInput/Textarea/Checkbox. Captured in a plain const, surfaced via $derived so handler-supplied props.id wins. Safe per STACK.md SPA-only posture."

patterns-established:
  - "Multi-item control Field-wrap pattern (RadioGroup): Field.Field -> Field.Label (no `for` — group label, not control label) -> Control.Root({value}, onValueChange) -> {#each options}<Control.Item id={${groupId}-${opt.value}}/> + adjacent <Label for={itemId}> + optional <p.muted> -> Field.Description / Field.Error. Any future multi-item picker (e.g., CheckboxGroup) reuses this shape."
  - "Horizontal-boolean-toggle Field-wrap pattern (Switch): Field.Field orientation=\"horizontal\" -> Field.Label for={id} (LEFT — semantic reversal vs. Checkbox where control is first) -> Control id={id} -> Field.Description / Field.Error. Matches 14-UI-SPEC Switch contract (label LEFT, switch RIGHT)."
  - "Per-option muted description: <p class=\"text-xs text-muted-foreground\"> under the adjacent label — 14-UI-SPEC §Typography Small (12px regular) role + §Colors muted-foreground."

requirements-completed: [FORM-01]

# Metrics
duration: 4m 45s
completed: 2026-04-17
---

# Phase 14 Plan 06: RadioGroup + Switch Primitives Summary

**Two new SDUI form leaves added: `RadioGroup` (vertical, per-option label + optional description) and `Switch` (horizontal, boolean toggle). Completes the Phase 14 six-leaf FORM-01 set (TextInput / SelectInput / Checkbox / Textarea / RadioGroup / Switch). Backend: `RadioOption` + `RadioGroup` + `Switch` structs via `ComponentBuilder` derive.**

## Performance

- **Duration:** 4 min 45 s
- **Started:** 2026-04-17T23:11:05Z
- **Completed:** 2026-04-17T23:15:50Z
- **Tasks:** 2
- **Files created:** 3 (2 Svelte leaves + this SUMMARY)
- **Files modified:** 4 (2 browser-test stubs + defaults.ts + backend standard.rs)

## Task Commits

1. **Task 1 — RadioGroup.svelte + Switch.svelte + defaults.ts registration:** `a303029` (feat)
2. **Task 2 — Backend RadioOption + RadioGroup + Switch builders + 4 unit tests:** `5d58921` (feat)

## RadioGroup.svelte (primary frontend artifact, 73 lines)

```svelte
<script lang="ts">
    import * as Field from '$lib/components/ui/field';
    import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group';
    import { Label } from '$lib/components/ui/label';
    import { getData, setData } from '$lib/store/data.svelte';
    import type { ComponentAction } from '$lib/transport/messages';

    type RadioOption = { value: string; label: string; description?: string };

    let { props = {}, bind, action, surface }: { /* ... */ } = $props();

    const fallbackId = crypto.randomUUID();
    let groupId = $derived((props.id as string) ?? fallbackId);

    let options = $derived((props.options as RadioOption[]) ?? []);
    let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
    let fieldError = $derived(
        bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
    );
    let hasError = $derived(!!fieldError);

    function handleValueChange(newValue: string) {
        if (bind) setData(surface, bind, newValue);
    }
</script>

<Field.Field
    data-invalid={hasError || undefined}
    class={props.full_width ? 'col-span-full' : undefined}
>
    {#if props.label}
        <Field.Label>{props.label}</Field.Label>
    {/if}
    <RadioGroup
        {value}
        onValueChange={handleValueChange}
        disabled={props.disabled as boolean}
        aria-invalid={hasError || undefined}
    >
        {#each options as opt (opt.value)}
            {@const itemId = `${groupId}-${opt.value}`}
            <div class="flex items-start gap-2">
                <RadioGroupItem value={opt.value} id={itemId} />
                <div class="grid gap-1">
                    <Label for={itemId} class="font-semibold">{opt.label}</Label>
                    {#if opt.description}
                        <p class="text-xs text-muted-foreground">{opt.description}</p>
                    {/if}
                </div>
            </div>
        {/each}
    </RadioGroup>
    {#if props.description && !hasError}
        <Field.Description>{props.description}</Field.Description>
    {/if}
    {#if fieldError}
        <Field.Error>{fieldError}</Field.Error>
    {/if}
</Field.Field>
```

**Semantic highlights:**

1. `<Field.Label>` has NO `for` attribute — a RadioGroup has no single focusable control; bits-ui's `role="radiogroup"` on the root carries the ARIA group semantics. The Field.Label acts as a visible group title.
2. Per-option id `${groupId}-${opt.value}` wires each `<RadioGroupItem>` to its adjacent `<Label for={itemId}>`. Clicking any option label focuses and selects that radio via the browser's native `<label for>` → button forwarding.
3. Per-option description (from `RadioOption.description`) renders as a muted 12px `<p>` under the option's bold label. Matches 14-UI-SPEC §Typography Small role + §Colors muted-foreground.
4. `{value}` + `onValueChange` (not `bind:value`) keeps the Marionette data-store as the source of truth — same pattern as SelectInput/Textarea.
5. `data-invalid` on the Field.Field wrapper, `aria-invalid` on `<RadioGroup>` root — both present only when `hasError` is truthy (attribute-presence semantics per shadcn Pitfall #4).

## Switch.svelte (primary frontend artifact, 58 lines)

```svelte
<script lang="ts">
    import * as Field from '$lib/components/ui/field';
    import { Switch } from '$lib/components/ui/switch';
    import { getData, setData } from '$lib/store/data.svelte';
    import type { ComponentAction } from '$lib/transport/messages';

    let { props = {}, bind, action, surface }: { /* ... */ } = $props();

    const fallbackId = crypto.randomUUID();
    let fieldId = $derived((props.id as string) ?? fallbackId);

    let checked = $derived(bind ? ((getData(surface, bind) as boolean) ?? false) : false);
    let fieldError = $derived(
        bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
    );
    let hasError = $derived(!!fieldError);

    function handleCheckedChange(val: boolean) {
        if (bind) setData(surface, bind, val);
    }
</script>

<Field.Field
    orientation="horizontal"
    data-invalid={hasError || undefined}
    class={props.full_width ? 'col-span-full' : undefined}
>
    {#if props.label}
        <Field.Label for={fieldId}>{props.label}</Field.Label>
    {/if}
    <Switch
        id={fieldId}
        {checked}
        onCheckedChange={handleCheckedChange}
        disabled={props.disabled as boolean}
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

**Differences vs. Checkbox.svelte (Plan 14-04 — the horizontal-orientation template):**

1. `<Field.Label>` comes FIRST (before the control) in Switch, whereas Checkbox places `<ShadcnCheckbox>` first. This matches the 14-UI-SPEC §Component Visual Contracts contract for Switch ("`Field.Label` on the left, `Switch` control on the right"). Under horizontal orientation the visual flow is label → control regardless of DOM order (Field.Label has `flex-auto`), but source order affects assistive-tech reading order and DOM-semantic clarity.
2. `onCheckedChange` signature is `(val: boolean)` not `(val: boolean | 'indeterminate')` — bits-ui's Switch has no indeterminate state.
3. No click-through writes: the boolean passes through verbatim via `setData(surface, bind, val)`.

## defaults.ts diff

```diff
 import Textarea from '../components/form/Textarea.svelte';
+import RadioGroup from '../components/form/RadioGroup.svelte';
+import MSwitch from '../components/form/Switch.svelte';
 import MButton from '../components/form/Button.svelte';
 ...
 'textarea': Textarea,
+'radio-group': RadioGroup,
+'switch': MSwitch,
 'button': MButton,
```

Added alphabetically-within-form-purpose, after `'textarea'` and before `'button'`. `MSwitch` alias avoids collision with the shadcn `Switch` namespace (matches `MCheckbox` / `MButton` / `MSpinner` / `MSwitch` convention).

## Backend additions (primary backend artifact)

```rust
/// Option entry for a RadioGroup component (Phase 14 D-E4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Radio group primitive (Phase 14 D-E4).
#[derive(ComponentBuilder)]
#[component(type = "radio-group")]
pub struct RadioGroup {
    pub label: String,
    pub options: Vec<RadioOption>,
    #[builder(optional)] pub required: Option<bool>,
    #[builder(optional)] pub disabled: Option<bool>,
    #[builder(optional)] pub description: Option<String>,
    #[builder(optional)] pub full_width: Option<bool>,
}

/// Toggle switch primitive (Phase 14 D-E4).
#[derive(ComponentBuilder)]
#[component(type = "switch")]
pub struct Switch {
    pub label: String,
    #[builder(optional)] pub disabled: Option<bool>,
    #[builder(optional)] pub description: Option<String>,
    #[builder(optional)] pub full_width: Option<bool>,
}
```

`RadioOption` lives next to `SelectOption` (line 48-53 sibling). `RadioGroup` + `Switch` structs follow `Textarea` (the Plan 14-05 template). `ComponentBuilder` derive auto-generates `.description(impl Into<String>)`, `.full_width(bool)`, `.disabled(bool)`, `.required(bool)` setters.

Handler usage examples:

```rust
// RadioGroup
let opts = vec![
    RadioOption { value: "email".into(), label: "Email".into(), description: Some("Weekly digest.".into()) },
    RadioOption { value: "sms".into(),   label: "SMS".into(),   description: None },
];
RadioGroup::new("Notification channel", opts)
    .description("How should we contact you?")
    .full_width(true)
    .build();

// Switch
Switch::new("Marketing emails")
    .description("Tips and product updates.")
    .build();
```

## Test Count Delta

| File | Before | After | Delta |
|------|--------|-------|-------|
| `frontend/src/lib/components/form/RadioGroup.browser-test.ts` | 5 RED | 5 GREEN | — (flipped) |
| `frontend/src/lib/components/form/Switch.browser-test.ts` | 4 RED | 4 GREEN | — (flipped) |
| `backend/crates/marionette/src/builders/standard.rs::tests` (radio-group + switch coverage) | 0 | 4 | +4 |
| `backend/crates/marionette` unit test total | 56 | 60 | +4 |

## Verification Evidence

```text
$ cd frontend && npx vitest --config vitest-browser.config.ts \
    src/lib/components/form/RadioGroup.browser-test.ts \
    src/lib/components/form/Switch.browser-test.ts --run
 Test Files  2 passed (2)
      Tests  9 passed (9)                        # 5 RadioGroup + 4 Switch

$ cd frontend && npm run check
COMPLETED 1062 FILES 3 ERRORS 0 WARNINGS 1 FILES_WITH_PROBLEMS
                                               # only 3 pre-existing schema-validator.ts
                                               # errors inherited from main (tracked in
                                               # deferred-items.md); ZERO new errors

$ cd backend && cargo test -p marionette --lib
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured

$ cd backend && cargo test -p marionette
# Full suite (lib + integration + doc): all pass

$ cd backend && cargo build -p crm-demo
Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.60s

$ grep -n "'radio-group':" frontend/src/lib/registry/defaults.ts
49:     'radio-group': RadioGroup,

$ grep -n "'switch':" frontend/src/lib/registry/defaults.ts
50:     'switch': MSwitch,

$ wc -l frontend/src/lib/components/form/RadioGroup.svelte \
        frontend/src/lib/components/form/Switch.svelte
  73 frontend/src/lib/components/form/RadioGroup.svelte      # ≥ 55 floor
  58 frontend/src/lib/components/form/Switch.svelte          # ≥ 45 floor
```

## Deviations from Plan

None. The plan executed exactly as written. Task 1 and Task 2 landed first-try on their respective verification commands.

- **Task 1:** Both `.svelte` files written per the plan's action blocks. The shadcn `RadioGroup` export exposed both namespace-object and named-export shapes; I chose named-imports (`import { RadioGroup, RadioGroupItem }`) for clarity — the plan explicitly permitted either form. All 9 browser tests flipped GREEN on the first run. `@ts-expect-error` directives removed from the Plan 01 RED stubs (expected hand-off).
- **Task 2:** `RadioOption` + `RadioGroup` + `Switch` structs + 4 serialization tests landed cleanly. `Switch` as a PascalCase identifier compiled without clashing with Rust's reserved lowercase `switch` keyword (as anticipated by the plan note). All 60 marionette lib tests + the integration / doc tests passed; `cargo build -p crm-demo` finished clean.

No auto-fixes, no blocking issues, no architectural escalations, no auth gates.

## Scope-Bounded Out-of-Scope Observations (NOT fixed)

Running `npx vitest .../components/form/ --run` surfaced pre-existing failures in **four unrelated test files**:

- `FieldSeparator.browser-test.ts` — RED stub for Plan 14-07 (not yet landed).
- `FieldSet.browser-test.ts` — RED stub for Plan 14-07 (not yet landed).
- `Button.browser-test.ts > dispatches action on click` — pre-existing on `main`.
- `SelectInput.browser-test.ts > change-action dispatch fires with merged payload on value change (Phase 12 D-A6)` — pre-existing on `main`.

None of these involve `RadioGroup.svelte`, `Switch.svelte`, or `defaults.ts`'s new entries. Confirmed via `git log --oneline <base>..HEAD -- <file>` returning empty for every listed component. Out of scope per the execute-plan rules.

## Decisions Made

- **Named imports for RadioGroup primitives** (`import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group'`). The generated `index.ts` exports both `Root`/`Item` and `RadioGroup`/`RadioGroupItem`. Named form is more readable at use-sites than `<RadioGroup.Root>` / `<RadioGroup.Item>`.
- **`{value}` + `onValueChange` (not `bind:value`)** — keeps the data-store one-directional. Matches SelectInput / Textarea pattern.
- **Per-option id = `${groupId}-${opt.value}`** — deterministic mapping; handler is responsible for unique option values (same invariant as SelectInput's options).
- **`<Field.Label>` without `for` on RadioGroup** — because there is no single control to focus. The group's `role="radiogroup"` (from bits-ui) carries the ARIA semantics; the label is a visible group title rather than a control-focusing element.
- **`<Field.Label for={fieldId}>` FIRST (before the Switch control)** — matches 14-UI-SPEC Switch visual contract (label LEFT, control RIGHT). Source order also improves assistive-tech reading. Different from Checkbox (where control-first makes semantic sense), matching the two controls' distinct UX meanings.
- **Mount-time `crypto.randomUUID()` fallback** for both `groupId` (RadioGroup) and `fieldId` (Switch) — identical D-B4 treatment to the other five leaves. Captured in plain `const`, surfaced via `$derived` so handler `props.id` wins.

## Issues Encountered

- **Frontend `node_modules` missing in worktree (one-time hydration cost):** `frontend/node_modules/.bin/vitest` was absent at plan start. Ran `npm ci` (~11 s) to hydrate. Same pattern documented in Plans 14-02, 14-03, 14-04, 14-05. Not tracked as a deviation — this is parallel-worktree hygiene.
- **Pre-existing schema-validator.ts errors (3, baseline):** Inherited from `main`. Tracked in `.planning/phases/14-formscreen-enhancements/deferred-items.md`. Unrelated to Plan 06.

## User Setup Required

None — no external service configuration required. Pure Svelte / Rust additions.

## Next Phase Readiness

- **Plan 14-07 (FieldSet, Wave 3):** The six form leaves are now FORM-01 compliant. FieldSet's grid can consume `col-span-full` (full_width override) end-to-end — confirmed wired on TextInput / SelectInput / Checkbox / Textarea / RadioGroup / Switch.
- **Plan 14-08 (CRM migration, Wave 4):** Handler code can now compose notification-preference forms using `RadioGroup::new(...).description(...).build()` and consent/opt-in controls using `Switch::new(...).description(...).build()`. All six leaves (TextInput / SelectInput / Checkbox / Textarea / RadioGroup / Switch) share the same Field.Field anatomy — handler code composes a full SDUI form with consistent visual language.
- **Phase 15+ (future):** The Field-wrap pattern is now the canonical shape for new form primitives. New leaves (e.g., DatePicker, FileInput) can copy the Textarea template and extend `backend/crates/marionette/src/builders/standard.rs`.

No blockers. No open questions. All six Phase 14 form-leaf plans (14-02, 14-03, 14-04, 14-05, 14-06) have landed.

## Known Stubs

None. Every rendered element has a concrete data source:

- `props.label`, `props.options`, `props.description`, `props.required`, `props.disabled`, `props.full_width` — all server-authoritative via backend builders.
- `value` (RadioGroup) — read from store at `getData(surface, bind)`.
- `checked` (Switch) — read from store at `getData(surface, bind)` as boolean.
- `fieldError` — read from store at `getData(surface, '/_errors' + bind)`.
- `groupId` / `fieldId` — `props.id` (handler-supplied) OR mount-time `crypto.randomUUID()` fallback.
- Per-option `description` — optional `RadioOption.description` from handler-supplied `options`.

No hardcoded empty props, no "coming soon" text, no TODO markers.

## Threat Flags

None. The new surface mirrors the existing Phase 14 form leaves:

- Text injection via `props.label` / `options[].label` / `options[].description` / `/_errors/{bind}` is mitigated by Svelte's auto-escaping of `{expression}` interpolation (no `{@html}`) — T-14-06-01, T-14-06-02 mitigated per the plan's threat register.
- Per-option id mapping `${groupId}-${opt.value}` is deterministic, ensuring each `<Label for={itemId}>` points at exactly one `<RadioGroupItem id={itemId}>` — T-14-06-03 mitigated by construction. The browser test `renders each option with a radio input` + `selects option when bind value matches` exercises this pairing.
- Click-label-to-select correctness assertion for RadioGroup is implicit (bits-ui forwards `<label for>` clicks natively to the role=radio button). The Switch tests explicitly assert `sw.click()` flips the bound boolean.

No new network endpoint or trust boundary introduced. Boolean bind for Switch reflects server-pushed data identically to Checkbox (T-14-06-05 accepted per the plan).

## Self-Check: PASSED

Verified post-SUMMARY:

```
FOUND: frontend/src/lib/components/form/RadioGroup.svelte (73 lines)
FOUND: frontend/src/lib/components/form/Switch.svelte (58 lines)
FOUND: frontend/src/lib/components/form/RadioGroup.browser-test.ts (@ts-expect-error removed)
FOUND: frontend/src/lib/components/form/Switch.browser-test.ts (@ts-expect-error removed)
FOUND: frontend/src/lib/registry/defaults.ts ('radio-group': RadioGroup, 'switch': MSwitch)
FOUND: backend/crates/marionette/src/builders/standard.rs (RadioOption + RadioGroup + Switch structs + 4 tests)
FOUND: .planning/phases/14-formscreen-enhancements/14-06-SUMMARY.md
FOUND: commit a303029 (Task 1 — Svelte components + registry)
FOUND: commit 5d58921 (Task 2 — backend builders + serialization tests)
```

---

*Phase: 14-formscreen-enhancements*
*Completed: 2026-04-17*
