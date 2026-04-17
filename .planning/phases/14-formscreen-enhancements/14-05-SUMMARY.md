---
phase: 14-formscreen-enhancements
plan: 05
subsystem: ui
tags: [form, field, textarea, new-primitive, backend-builder, shadcn-svelte, svelte5]

# Dependency graph
requires:
  - phase: 14-01
    provides: "shadcn Textarea primitive under $lib/components/ui/textarea/ + Field family + Plan 01 RED Textarea.browser-test.ts stub"
  - phase: 14-02
    provides: "D-B1 Field.Field wrap pattern exemplar (TextInput) — Textarea copies the wrap shape verbatim"
provides:
  - "Textarea.svelte — new SDUI leaf component with internal Field.Field wrap (D-E3)"
  - "'textarea' -> Textarea registry entry in defaults.ts"
  - "Backend Textarea builder struct (label + 6 optional fields) with #[component(type = \"textarea\")]"
  - "Plan 01's RED Textarea.browser-test.ts flipped GREEN (6/6 tests)"
  - "Three Rust serialization tests pinning textarea protocol shape (basic/full/rows_is_u32)"
affects:
  - "Phase 15 (CRM migration) — long-text fields like contact notes can now use Textarea::new(...)"
  - "14-06 (RadioGroup + Switch) — same new-primitive + Field-anatomy pattern template"

# Tech tracking
tech-stack:
  added: []  # no new libraries — Plan 01 installed the shadcn Textarea primitive
  patterns:
    - "shadcn Textarea primitive wrapped by a Marionette SDUI leaf that follows the D-B1 Field.Field anatomy — same template as TextInput/Select/Checkbox rewrites"
    - "rows forwarded verbatim through {...restProps} — the shadcn Textarea primitive does not intercept HTMLTextareaAttributes, so rows={...} reaches the native element without a min-h-* fallback (Open Question OQ4 resolved in favour of direct-pass)"

key-files:
  created:
    - "frontend/src/lib/components/form/Textarea.svelte (84 lines — new SDUI leaf)"
  modified:
    - "frontend/src/lib/components/form/Textarea.browser-test.ts (removed the obsolete @ts-expect-error directive — Plan 01's cue)"
    - "frontend/src/lib/registry/defaults.ts ('textarea' -> Textarea import + registry entry)"
    - "backend/crates/marionette/src/builders/standard.rs (+Textarea struct, +3 unit tests)"

key-decisions:
  - "rows passes through the shadcn Textarea primitive unchanged — the primitive's script spreads {...restProps} onto the native <textarea>, so the plan's conditional 'min-h-{rows*1.5}rem fallback' branch is not needed. OQ4 resolved: direct pass."
  - "handleInput casts currentTarget to HTMLTextAreaElement (not HTMLInputElement) — the only shape difference vs TextInput's handleInput. Everything else — the blur-race safety, the sendAction on action.type === 'blur', the markDirty/clearDirty sequence — is byte-for-byte identical to TextInput for forward-compatibility (same fixes/improvements propagate if the shared pattern is later factored out)."
  - "The backend rows field is u32 rather than u16 — the ComponentBuilder-derived setter uses the declared Rust integer type, and the test pins it with .rows(10u32). u32 was chosen to avoid narrow-int friction at call sites; practically any rows value well below u16 MAX is fine, but tying the type to u16 would force explicit casts in handler code."

patterns-established:
  - "New-primitive leaf template: (1) create $lib/components/form/X.svelte mirroring TextInput's Field.Field structure, (2) register 'x': X in defaults.ts, (3) add #[derive(ComponentBuilder)] #[component(type=\"x\")] struct X in standard.rs, (4) add 3 serialization tests (basic/full/type-pin). Plan 14-05 is now the Template for 14-06 RadioGroup + Switch."

requirements-completed: [FORM-01]

# Metrics
duration: 8min
completed: 2026-04-17
---

# Phase 14 Plan 05: Textarea New Primitive Summary

**New SDUI `Textarea` leaf added: Svelte component with internal Field.Field wrap + registry entry + backend builder struct + 3 serde tests. Plan 01's RED Textarea.browser-test.ts (6 tests) is now GREEN.**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-04-17T01:02:00Z (branch base verified, context loaded)
- **Completed:** 2026-04-17T01:09:30Z (SUMMARY.md written)
- **Tasks:** 2
- **Files changed:** 1 new, 3 modified

## Task Commits

1. **Task 1 — Textarea.svelte + defaults.ts registration:** `6538be2` (feat)
2. **Task 2 — Backend Textarea builder struct + 3 unit tests:** `2c5856a` (feat)

## Textarea.svelte (primary artifact)

The differences vs. TextInput (the canonical Field-wrap reference) are small and structural:

```svelte
<script lang="ts">
    import * as Field from '$lib/components/ui/field';
    import { Textarea as ShadcnTextarea } from '$lib/components/ui/textarea';  // <-- differs
    // ... store / dirty / transport imports identical to TextInput
    const fallbackId = crypto.randomUUID();
    let fieldId = $derived((props.id as string) ?? fallbackId);
    let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
    let fieldError = $derived(bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : '');
    let hasError = $derived(!!fieldError);

    function handleInput(e: Event) {
        if (bind) {
            const target = e.currentTarget as HTMLTextAreaElement;  // <-- differs (TextInput uses HTMLInputElement)
            setData(surface, bind, target.value);
        }
    }
    // handleFocus / handleBlur identical to TextInput
</script>

<Field.Field
    data-invalid={hasError || undefined}
    class={props.full_width ? 'col-span-full' : undefined}
>
    {#if props.label}
        <Field.Label for={fieldId}>{props.label}</Field.Label>
    {/if}
    <ShadcnTextarea
        id={fieldId}
        placeholder={props.placeholder as string}
        rows={(props.rows as number) ?? 4}           <!-- NEW: rows with default 4 per UI-SPEC -->
        required={props.required as boolean}
        disabled={props.disabled as boolean}
        aria-invalid={hasError || undefined}
        {value}
        oninput={handleInput}
        onfocus={handleFocus}
        onblur={handleBlur}
    />
    {#if props.description && !hasError}
        <Field.Description>{props.description}</Field.Description>
    {/if}
    {#if fieldError}
        <Field.Error>{fieldError}</Field.Error>
    {/if}
</Field.Field>
```

Only three lines are semantically distinct from TextInput:

1. `import { Textarea as ShadcnTextarea } from '$lib/components/ui/textarea'` (instead of `Input`).
2. `handleInput`: `e.currentTarget as HTMLTextAreaElement` (instead of `HTMLInputElement`).
3. Control element is `<ShadcnTextarea>` with `rows={(props.rows as number) ?? 4}` (no `type=` attribute — textareas have no type).

All other code — the field-id fallback, the data-store wiring, the blur-race safety, the `data-invalid || undefined` attribute-presence, the `col-span-full` full_width class, and the description/error conditional rendering — is byte-for-byte the same as TextInput. Forward-compatibility: if the shared pattern is later extracted into a common snippet or base component, the inherited changes land uniformly.

## defaults.ts diff

```diff
 import MCheckbox from '../components/form/Checkbox.svelte';
+import Textarea from '../components/form/Textarea.svelte';
 import MButton from '../components/form/Button.svelte';
 ...
 'select': SelectInput,
 'checkbox': MCheckbox,
+'textarea': Textarea,
 'button': MButton,
```

Inserted alphabetically-within-purpose after `'checkbox'` and before `'button'`, matching the existing ordering (form controls grouped together).

## Backend Textarea struct (primary backend artifact)

```rust
#[derive(ComponentBuilder)]
#[component(type = "textarea")]
pub struct Textarea {
    pub label: String,
    #[builder(optional)] pub placeholder: Option<String>,
    /// Visible row count for the native `<textarea>`. Frontend default: 4.
    #[builder(optional)] pub rows: Option<u32>,
    #[builder(optional)] pub required: Option<bool>,
    #[builder(optional)] pub disabled: Option<bool>,
    /// Helper text rendered below the textarea via shadcn Field.Description
    /// (Phase 14 D-B3). Hidden while an `/_errors/{bind}` entry is active.
    #[builder(optional)] pub description: Option<String>,
    /// When `true`, the field's Field.Field wrapper spans every column of
    /// its parent FieldSet grid (Phase 14 D-C4).
    #[builder(optional)] pub full_width: Option<bool>,
}
```

The `ComponentBuilder` derive auto-generates `.placeholder(...)`, `.rows(u32)`, `.required(bool)`, `.disabled(bool)`, `.description(impl Into<String>)`, `.full_width(bool)` setters — handler usage is `Textarea::new("Notes").rows(6).description("Max 500 chars.").full_width(true).build()`.

## Test Count Delta

| File | Before | After | Delta |
|------|--------|-------|-------|
| `frontend/src/lib/components/form/Textarea.browser-test.ts` | 6 RED | 6 GREEN | — (flipped) |
| `backend/crates/marionette/src/builders/standard.rs::tests` (textarea coverage) | 0 | 3 | +3 |
| `backend/crates/marionette` unit test total | 53 | 56 | +3 |

## Verification Evidence

```text
$ cd frontend && npx vitest --config vitest-browser.config.ts \
    src/lib/components/form/Textarea.browser-test.ts --run
Test Files  1 passed (1)
     Tests  6 passed (6)

$ cd frontend && npx vitest --config vitest-browser.config.ts \
    src/lib/components/form/TextInput.browser-test.ts \
    src/lib/components/form/Form.browser-test.ts \
    src/lib/components/form/Checkbox.browser-test.ts \
    src/lib/components/form/SelectInput.browser-test.ts --run
Test Files  4 passed (4)
     Tests  57 passed (57)       # zero regressions in sibling form leaves

$ cd frontend && npm run check
COMPLETED 1060 FILES 3 ERRORS      # only the 3 pre-existing schema-validator.ts
                                    # errors inherited from main (tracked in
                                    # .planning/phases/14-formscreen-enhancements/deferred-items.md)

$ cd backend && cargo test -p marionette textarea
running 3 tests
test builders::standard::tests::textarea_basic_serialization ... ok
test builders::standard::tests::textarea_full_serialization ... ok
test builders::standard::tests::textarea_rows_is_u32 ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured

$ cd backend && cargo test -p marionette --lib
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cd backend && cargo build -p crm-demo
Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.43s

$ grep -n "'textarea': Textarea" frontend/src/lib/registry/defaults.ts
46:            'textarea': Textarea,

$ wc -l frontend/src/lib/components/form/Textarea.svelte
84 frontend/src/lib/components/form/Textarea.svelte   # ≥ 50 line floor
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] `@ts-expect-error` becomes unused after the Svelte sibling lands**

- **Found during:** Task 1 `npm run check` verification.
- **Issue:** Plan 01 scaffolded `Textarea.browser-test.ts` with a `@ts-expect-error` directive on the `import Textarea from './Textarea.svelte'` line. Plan 01's own SUMMARY already forecast this: *"When Wave 2/3 lands the component, the import resolves, the directive becomes unnecessary, and TypeScript will flag it as unused — forcing the downstream wave to remove it."* Plan 05 is the "downstream wave" for Textarea; svelte-check correctly reports `Unused '@ts-expect-error' directive`.
- **Fix:** Removed the three-line directive + comment block. The import now resolves at both type-check time (Svelte component exists) and runtime (registered via defaults.ts). This is the explicit hand-off mechanism baked into Plan 01.
- **Files modified:** `frontend/src/lib/components/form/Textarea.browser-test.ts` (net −3 lines).
- **Verification:** `npm run check` drops from 4 errors to 3 (back to the pre-existing schema-validator baseline).
- **Committed in:** `6538be2` (rolled into Task 1's commit).

### Scope-Bounded "Deviations" That Did NOT Need Fixing

- **OQ4 (rows forwarding):** The plan's `<action>` block warned that if the shadcn Textarea primitive does not forward `rows` to the underlying `<textarea>`, a `min-h-{rows*1.5}rem` inline-style fallback would be needed. Inspection of `frontend/src/lib/components/ui/textarea/textarea.svelte` confirmed the primitive uses `{...restProps}`, which spreads every `HTMLTextareaAttributes` key (including `rows`) onto the native element. The fallback branch is NOT needed. The `rows_is_u32` test and the Textarea.browser-test.ts `rows prop is forwarded to native textarea` test both verify end-to-end forwarding. OQ4 resolved: direct pass.

### Pre-existing, Out of Scope

- `tests/helpers/schema-validator.ts` (3 `Cannot find module 'fs'/'path'/'url'` errors) — logged by Plan 01 in `.planning/phases/14-formscreen-enhancements/deferred-items.md`, pre-existing on `main`. Unrelated to Plan 05.

**Total deviations:** 1 auto-fixed (Rule 3), 0 out-of-scope fixes attempted.

## Decisions Made

- **rows is `u32`, not `u16`:** Declared the backend `rows: Option<u32>` to avoid narrow-int casts at handler call sites. Rows values realistically stay well under `u16::MAX`, but `u32` matches the frontend's `(props.rows as number) ?? 4` cast (JS numbers are 64-bit floats; the backend serializes to a JSON integer and the frontend coerces to a numeric). A dedicated `textarea_rows_is_u32` test pins the JSON shape (`is_u64()`).
- **No rows-fallback `min-h-*` branch:** OQ4 from 14-RESEARCH resolved against the fallback — the shadcn primitive forwards via `{...restProps}` and the browser test `rows prop is forwarded to native textarea` passes without any workaround.
- **Mount-time UUID fallback for field id:** Same D-B4 treatment as TextInput — captured in a plain `const`, surfaced via `$derived` so handler-supplied `props.id` wins. Safe per SPA-only posture (STACK.md).

## Issues Encountered

- **Frontend npm_modules missing in worktree (one-time cost):** `frontend/node_modules/.bin/vitest` was absent at plan start. Ran `npm ci` once to hydrate. This is a parallel-worktree cost already documented in Plan 14-02 SUMMARY — not a Plan 05 concern.
- **Plan's cargo test filter `standard::textarea` matched zero tests** (the actual module path is `builders::standard::tests::textarea_*`). Used `cargo test -p marionette textarea` and `cargo test -p marionette builders::standard::tests::textarea` to verify — both return the expected 3/3 green. Filter-path typo in plan — no code change needed, documenting here for future-plan reference.

## User Setup Required

None — no external service configuration required. Pure Svelte/Rust additions.

## Next Phase Readiness

- **14-06 (RadioGroup + Switch, Wave 2 sibling):** This plan is the second new-primitive template (after TextInput's Wave 0 rewrite pattern). 14-06 follows the exact same sequence: (1) create `.svelte` with Field.Field wrap, (2) register in defaults.ts, (3) add backend struct, (4) add 3 serialization tests. RadioGroup has additional complexity (vec of option objects via a new `RadioOption` Rust struct — similar to existing `SelectOption`); Switch is structurally identical to Textarea minus the input/rows and plus a boolean-typed bind.
- **14-07 (FieldSet, Wave 3):** Unblocked — uses the same Field.Group-based wrap pattern that Form.svelte already has (Plan 14-02).
- **Phase 15 (CRM migration):** A `Textarea::new("Notes")` builder is now callable in `crm-demo` handlers — the `contact` edit form can migrate the plain-text notes field to a proper multi-line control.

No blockers. No open questions.

## Known Stubs

None. Every rendered element has a concrete data source:

- `props.label`, `props.description`, `props.placeholder`, `props.rows`, `props.required`, `props.disabled`, `props.full_width` — all server-authoritative via backend builders.
- `value` — read from store at `getData(surface, bind)`.
- `fieldError` — read from store at `getData(surface, '/_errors' + bind)`.
- `fieldId` — `props.id` (handler-supplied) OR mount-time `crypto.randomUUID()` fallback.

No hardcoded empty props, no "coming soon" text, no TODO markers in the new code.

## Self-Check: PASSED

Verified post-SUMMARY:

```
FOUND: frontend/src/lib/components/form/Textarea.svelte (84 lines)
FOUND: frontend/src/lib/components/form/Textarea.browser-test.ts (@ts-expect-error removed)
FOUND: frontend/src/lib/registry/defaults.ts ('textarea': Textarea registered)
FOUND: backend/crates/marionette/src/builders/standard.rs (Textarea struct + 3 tests)
FOUND: .planning/phases/14-formscreen-enhancements/14-05-SUMMARY.md
FOUND: commit 6538be2 (Task 1 — Svelte + registry)
FOUND: commit 2c5856a (Task 2 — backend builder + tests)
```

---

*Phase: 14-formscreen-enhancements*
*Completed: 2026-04-17*
