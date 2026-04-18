---
phase: 14-formscreen-enhancements
reviewed: 2026-04-17T00:00:00Z
depth: standard
files_reviewed: 44
files_reviewed_list:
  - backend/crates/crm-demo/src/handlers/contact.rs
  - backend/crates/marionette/src/builders/standard.rs
  - frontend/src/lib/components/core/NodeRenderer.browser-test.ts
  - frontend/src/lib/components/core/NodeRenderer.svelte
  - frontend/src/lib/components/form/Checkbox.browser-test.ts
  - frontend/src/lib/components/form/Checkbox.svelte
  - frontend/src/lib/components/form/FieldSeparator.browser-test.ts
  - frontend/src/lib/components/form/FieldSeparator.svelte
  - frontend/src/lib/components/form/FieldSet.browser-test.ts
  - frontend/src/lib/components/form/FieldSet.svelte
  - frontend/src/lib/components/form/Form.browser-test.ts
  - frontend/src/lib/components/form/Form.svelte
  - frontend/src/lib/components/form/RadioGroup.browser-test.ts
  - frontend/src/lib/components/form/RadioGroup.svelte
  - frontend/src/lib/components/form/SelectInput.browser-test.ts
  - frontend/src/lib/components/form/SelectInput.svelte
  - frontend/src/lib/components/form/Switch.browser-test.ts
  - frontend/src/lib/components/form/Switch.svelte
  - frontend/src/lib/components/form/TextInput.browser-test.ts
  - frontend/src/lib/components/form/TextInput.svelte
  - frontend/src/lib/components/form/Textarea.browser-test.ts
  - frontend/src/lib/components/form/Textarea.svelte
  - frontend/src/lib/components/ui/field/field-content.svelte
  - frontend/src/lib/components/ui/field/field-description.svelte
  - frontend/src/lib/components/ui/field/field-error.svelte
  - frontend/src/lib/components/ui/field/field-group.svelte
  - frontend/src/lib/components/ui/field/field-label.svelte
  - frontend/src/lib/components/ui/field/field-legend.svelte
  - frontend/src/lib/components/ui/field/field-separator.svelte
  - frontend/src/lib/components/ui/field/field-set.svelte
  - frontend/src/lib/components/ui/field/field-title.svelte
  - frontend/src/lib/components/ui/field/field.svelte
  - frontend/src/lib/components/ui/field/index.ts
  - frontend/src/lib/components/ui/radio-group/index.ts
  - frontend/src/lib/components/ui/radio-group/radio-group-item.svelte
  - frontend/src/lib/components/ui/radio-group/radio-group.svelte
  - frontend/src/lib/components/ui/switch/index.ts
  - frontend/src/lib/components/ui/switch/switch.svelte
  - frontend/src/lib/components/ui/textarea/index.ts
  - frontend/src/lib/components/ui/textarea/textarea.svelte
  - frontend/src/lib/init.ts
  - frontend/src/lib/registry/defaults.ts
  - frontend/tests/e2e/contact-edit.spec.ts
  - frontend/tests/uat/playwright.uat.config.ts
  - frontend/tests/uat/uat-driver.spec.ts
  - frontend/tests/visual/form.spec.ts
  - frontend/vitest-browser.config.ts
  - spec/PROTOCOL.md
  - spec/schemas/data.yaml
findings:
  critical: 0
  warning: 4
  info: 9
  total: 13
status: issues_found
---

# Phase 14: Code Review Report

**Reviewed:** 2026-04-17
**Depth:** standard
**Files Reviewed:** 44 (Rust backend + Svelte form primitives + shadcn vendor UI + specs + tests + init hook)
**Status:** issues_found

## Summary

Phase 14 introduces a shadcn Field.Field anatomy across all form leaves (TextInput, SelectInput, Checkbox, Textarea, RadioGroup, Switch), adds structural FieldSet + FieldSeparator primitives, and migrates the crm-demo contact form to the canonical form-screen composition pattern. The Rust builder layer gains `description` / `full_width` props plus the new `Textarea`, `RadioGroup`, `Switch`, `FieldSet`, `FieldSeparator` builders with matching serialization tests. Frontend behavior is pinned by a strong Vitest browser suite (including the D-E2 unmount-race structural contract) and Playwright E2E / UAT drivers.

Overall code quality is high: the Field anatomy is applied consistently across all leaves, `data-invalid` / `aria-invalid` follow the attribute-presence contract documented in 14-UI-SPEC Pitfall #4, the D-E2 unmount-race fix uses `{@const}` bindings inside `{#if node}` and is pinned by both a behavioural and a structural test, and the backend builder tests cover every new optional field both when set and when omitted. The backend contact-form handler composes FieldSets via the `build_tree()` / `build_with_children()` split and correctly accumulates descendants into the render map.

The issues below are mostly latent concerns and documentation inconsistencies rather than live bugs; the most impactful is the native `<form>` submit path in `Form.svelte` sending an empty payload (WR-01), which becomes a problem only if a handler ever ships a Form with its own `action` (it is currently only wired via nested Buttons).

---

## Warnings

### WR-01: `Form.svelte` native submit dispatches an empty payload

**File:** `frontend/src/lib/components/form/Form.svelte:26-31`

**Issue:** `handleSubmit` dispatches `sendAction(action.name ?? 'submit', {}, action.target)` with an empty payload. Every other bound control in Phase 14 (`Button.svelte`, `SelectInput.svelte`) follows the Phase 12 pattern of `{ ...(action.payload ?? {}), ...getAllData(surface) }` so the backend receives the current surface data. As a result:

- If a caller ever attaches `action` directly to `Form` (e.g., to support Enter-to-submit), the server receives `{}` and handlers like `handle_contact_save` will reject it with `BadPayload("Contact name is required")` even when the form is fully filled in.
- The existing `prevents default submit and dispatches action` browser test asserts the current empty-payload shape (`expect(sendAction).toHaveBeenCalledWith('save-form', {}, undefined)`), so the contract is *locked* to the buggy shape by the test suite. This is especially concerning because the test target argument is `undefined` even though `action.target` is passed through — the test happens to match because neither was provided.

The contact-form flow works today because the save is dispatched via `Button` (which does merge surface data), not via the `<form>` onsubmit. But any screen that sets `action` on a `Form` — or any user who presses Enter inside a text input in such a form — will silently send the wrong payload.

**Fix:**

```svelte
// Form.svelte
<script lang="ts">
    import { getAllData, getData } from '$lib/store/data.svelte';
    // ...
    function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        if (action) {
            const surfaceData = getAllData(surface) ?? {};
            const payload = {
                ...((action.payload as Record<string, unknown>) ?? {}),
                ...surfaceData,
            };
            sendAction(action.name ?? 'submit', payload, action.target);
        }
    }
</script>
```

And update `Form.browser-test.ts` to assert the merged payload shape (mirroring the SelectInput `change-action dispatch fires with merged payload on value change` test).

---

### WR-02: PROTOCOL.md documents two mutually-exclusive validation-error shapes

**File:** `spec/PROTOCOL.md:804-819` (legacy) vs. `spec/PROTOCOL.md:593-600` (Phase 14)

**Issue:** The "Error Handling → Validation Errors as Data" section (lines 804–819) shows validation errors as an array of `{path, message}` objects patched to `/contactForm/errors`. The newer "Form Components → Validation semantics" section (lines 593–600) specifies the authoritative Phase 14 contract: `/_errors/{bind}` holds a `string` for per-field errors and a `string[]` for form-level errors. These two shapes are incompatible: the older example is pre-Phase-14 and no longer reflects the implementation.

Implementers reading the older section first will build the wrong shape; the Svelte form leaves only read the new shape (`getData(surface, '/_errors' + bind) as string`). Since this is the authoritative protocol spec, the inconsistency is a real documentation bug — it will teach callers the wrong contract.

**Fix:** Replace the legacy Validation Errors example (PROTOCOL.md lines 804–819) with an example matching the Phase 14 shape, e.g.:

```yaml
# Server patches per-field validation errors into the same data store:
type: patch
surface: content
patch:
  - op: set
    path: "/_errors/contactForm/email"
    value: "Invalid email address"
  - op: set
    path: "/_errors/contactForm"
    value: []  # clear form-level banner on the same patch
```

Add a cross-reference back to "Form Components → Validation semantics" so future edits can't drift again.

---

### WR-03: `SelectInput.handleOpenChange` double-writes on selection + close

**File:** `frontend/src/lib/components/form/SelectInput.svelte:37-67`

**Issue:** When a user picks a value, bits-ui fires `onValueChange(newValue)` immediately followed by `onOpenChange(false)`. The current wiring calls `setData(...)` in `handleValueChange` AND `clearDirty(bind, (op) => setData(surface, op.path, op.value))` in `handleOpenChange(false)`. If the dirty queue buffered an earlier optimistic write while open, `clearDirty` will replay it on close, potentially overwriting the value just committed by `handleValueChange`.

For a fresh open→pick flow (no prior buffered op) the result is idempotent. But if the user opens the dropdown, hovers through options (each potentially buffering a dirty op depending on `markDirty` semantics), then picks a value, the buffered op can overwrite the final selection.

This is a latent race, not an observable bug today — but it mirrors the kind of D-E2 unmount-race that Plan 14-01 fixed, and the mitigation is cheap. Unlike TextInput where focus/blur bracket a buffered string being typed, Select has no equivalent "typed but uncommitted" state; the `markDirty` on open serves only to mark the bind as touched.

**Fix:** Either drop the `markDirty`/`clearDirty` pair for Select (it writes atomically on selection, there is nothing to buffer), or make `clearDirty` a no-op when the buffered op's value matches the current store value. The simplest correct change is to remove the pair entirely:

```svelte
function handleOpenChange(_open: boolean) {
    // no-op — Select writes atomically on valueChange; no buffered state to flush
}
// Or simply drop the onOpenChange prop on <Select.Root>.
```

If dirty tracking on Select is actually needed for another consumer, replace the unconditional replay with a guarded one:

```svelte
function handleOpenChange(open: boolean) {
    if (!bind) return;
    if (open) markDirty(bind);
    else clearDirty(bind); // do NOT re-apply buffered ops
}
```

---

### WR-04: `TextInput` / `Textarea` `handleBlur` uses `bind!` assertion after destructive pattern

**File:** `frontend/src/lib/components/form/TextInput.svelte:45-56` and `frontend/src/lib/components/form/Textarea.svelte:45-56`

**Issue:** Inside `handleBlur`, the bind-gated block uses `getData(surface, bind!)` with a non-null assertion even though `bind` was just narrowed by the enclosing `if (bind)`. This is benign today, but the non-null assertion fights TypeScript's narrowing and hides a real footgun: if the `action?.type === 'blur'` branch ever gets hoisted out of the `if (bind)` block during a refactor, the `bind!` will silently dereference undefined. The canonical `Button.svelte` handler uses `getAllData(surface)` and doesn't rely on `bind` there; `SelectInput.handleValueChange` gates the change-action dispatch explicitly on `if (action?.type === 'change' && action.name)` without reaching into `bind`.

Additionally the `handleBlur` dispatch uses `action.name ?? action.type` as the action name. Per the same reasoning as `Button.svelte` line 46-48 ("Do NOT fall back to action.type here: type is a protocol classifier, not a backend action name"), falling back to `action.type` here (which is always `'blur'`) means a mis-configured action with `type: 'blur'` and no `name` will dispatch an action literally called `"blur"`, silently creating a handler-not-found on the backend.

**Fix:** Drop the non-null assertion (the narrowing is already correct) and drop the `action.type` fallback so mis-configured actions fail loudly:

```svelte
function handleBlur() {
    if (!bind) return;
    clearDirty(bind, (op) => setData(surface, op.path, op.value));
    if (action?.type === 'blur' && action.name) {
        sendAction(action.name, { value: getData(surface, bind) }, action.target);
    }
}
```

Apply the same fix to `Textarea.svelte:45-56`.

---

## Info

### IN-01: `handle_contact_country_change` builds a `Component` literal without helper

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:1577-1584`

**Issue:** The toast node is constructed via a hand-rolled `Component { r#type: "button".into(), props: Some(Value::Object(toast_props)), ... }` rather than `Button::new(&toast_label).action(...).build().1`. This bypasses the builder layer's type-safety (e.g., a typo in the `"button"` string is only caught at runtime on the frontend). The documented reason ("Button is the one that dispatches click actions; Heading ignores action") is correct, but the Button builder exists and would satisfy that requirement.

**Fix:** Replace the literal with the Button builder (the surrounding code already imports `Button`):

```rust
let (toast_id, toast_component) = Button::new(&toast_label)
    .id("toast-country-change")
    .action(ComponentAction::click("dismiss_toast"))
    .build();
```

### IN-02: `__mrnSetData` test hook is exposed in production builds

**File:** `frontend/src/lib/init.ts:92-102`

**Issue:** Both `__mrnSendAction` and `__mrnSetData` are unconditionally attached to `window`. The code comment documents this as a "narrow, intentional test-only surface" with the rationale that anything an attacker can do via the hook they could already do via a crafted WebSocket message. That's broadly correct, but it removes one useful defense-in-depth property: an XSS payload that runs in the victim's browser can now call `__mrnSetData('content', '/_errors/contactForm/name', 'Please contact support@evil.example')` to phish via in-app error messages without needing to speak WebSocket framing. This is an incremental, not critical, erosion.

Given the pre-deployment posture, this is acceptable as a Plan 14-08 scaffold per the phase notes. For production, gate behind a build flag such as `if (import.meta.env.DEV) { ... }` or a feature flag — Phase 15 should wire real server-side per-field errors and delete this hook.

### IN-03: PROTOCOL.md §Error Handling claims validation errors "are data patches" but lacks `/_errors/{bind}` shape

**File:** `spec/PROTOCOL.md:789-833`

**Issue:** The "When to Use Which" table points readers to "Validation error as data patch" but the section above (referenced in WR-02) uses the old shape. Readers comparing the Error Handling section to the Form Components section get two different answers. Info-level because WR-02 already covers the primary fix; this is a cross-reference omission.

**Fix:** Add a "See also: Form Components → Validation semantics" note in the Error Handling section after WR-02 is resolved.

### IN-04: `contact.rs` has a dead binding to silence `search_term` unused-variable warning

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:256`

**Issue:** `let _ = &search_term;` is a no-op that exists only to keep the unused-variable lint quiet after the pre-Phase-13 post-filter was removed (documented in the surrounding comment). The comment adequately explains *why*, but the cleaner fix is to drop the `let search_term = ...` block above: `search_term` is only used in the `if let Some(ref q) = search_term` pattern immediately above. Since that pattern owns the value, the variable can go out of scope naturally.

**Fix:** Remove the `let _ = &search_term;` line and inline the search-term trimming into the conditional:

```rust
if let Some(q) = params
    .search
    .as_deref()
    .map(str::trim)
    .filter(|s| !s.is_empty())
{
    condition = condition.add(
        Condition::any()
            .add(contact::Column::ContactName.contains(q))
            .add(contact::Column::ContactEmail.contains(q)),
    );
}
```

### IN-05: Email validation in `handle_contact_save` is trivially weak

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:1061-1065`

**Issue:** `if !data.email.contains('@')` accepts `"@"`, `"a@"`, `"@b"`, `"a@@b"`. This is widely-known, not a new Phase 14 issue, and calls out as V-05-level in 14-VALIDATION.md. Noting for completeness since the handler was touched in Plan 14-08.

**Fix:** Either use the `email_address` crate (`EmailAddress::from_str(&data.email)`) or at minimum require both non-empty local-part and domain: `data.email.split_once('@').is_some_and(|(l, d)| !l.is_empty() && !d.is_empty())`.

### IN-06: `find_or_create_tag` has a TOCTOU race

**File:** `backend/crates/crm-demo/src/handlers/contact.rs:1281-1306`

**Issue:** Between the `find()` at line 1288 and `insert()` at line 1301, a concurrent request can insert the same tag name, causing the second insert to fail with a UNIQUE constraint violation that bubbles up as `ActionError::Internal`. The caller (`handle_contact_tag_save`) already handles UNIQUE-violation from the subsequent `contact_tag` link insert, but not from the tag create itself. Demo-scale, single-user; flagged for completeness.

**Fix:** Catch the UNIQUE violation on the tag insert and re-query by name:

```rust
match new_tag.insert(db).await {
    Ok(t) => Ok(t.tag_id),
    Err(e) if e.to_string().contains("UNIQUE") => {
        let existing = tag::Entity::find()
            .filter(tag::Column::TagName.eq(trimmed))
            .one(db).await
            .map_err(|e| ActionError::Internal(e.to_string()))?
            .ok_or_else(|| ActionError::Internal("Tag race: existing not found".into()))?;
        Ok(existing.tag_id)
    }
    Err(e) => Err(ActionError::Internal(e.to_string())),
}
```

### IN-07: `FieldSeparator.svelte` declares but never uses `bind`, `action`, `surface`

**File:** `frontend/src/lib/components/form/FieldSeparator.svelte:9-19`

**Issue:** The component destructures `bind`, `action`, `surface` from `$props()` but none are referenced. The comment documents this as intentional ("SDUI contract declares all four standard props for NodeRenderer invocation uniformity"), which is correct — NodeRenderer passes them unconditionally — but Svelte 5 + TypeScript will still flag `action` and `surface` as unused. The `props = {}` default and the unused bindings create compiler warnings in strict mode.

**Fix:** Either prefix with underscore (`_bind`, `_action`, `_surface`) to signal deliberate non-use, or add `// eslint-disable-next-line` directives. The `props` parameter is genuinely unused and can be omitted from destructuring entirely since FieldSeparator has no props.

### IN-08: `Form.svelte` `{#each formErrors as error}` has no key

**File:** `frontend/src/lib/components/form/Form.svelte:39-41`

**Issue:** Unkeyed `{#each}` in Svelte 5 still works but can produce incorrect DOM reuse when the array shifts (e.g., server patches `["error A", "error B"]` to `["error B"]`). In Svelte 5 the default keying is by index, so dropping the first error will re-render element 0 with "error B" text while DOM element 1 gets torn down — functionally correct here (text-only nodes) but wasteful and divergent from Svelte's keyed-list idiom.

**Fix:** Either key by index explicitly or (simpler) by identity since errors are strings:

```svelte
{#each formErrors as error, i (i)}
    <p class="text-sm">{error}</p>
{/each}
```

### IN-09: UAT driver swallows `sendAction` result and UAT-03b assertion is unconditional pass

**File:** `frontend/tests/uat/uat-driver.spec.ts:366-414`

**Issue:** `UAT-03b End-to-end submit with invalid payload surfaces server error` writes `passed: true // UAT-03b is informational` into the evidence JSON regardless of what actually happened. The `serverErrors` lookup uses `window.__mrnGetData`, which is never exposed by `init.ts` (only `__mrnSendAction` and `__mrnSetData` are). So `serverErrors` is always `null` and the "informational" disposition hides the fact that the evidence is empty. If the intent is to document the error propagation manually, say so in the comment; otherwise expose `__mrnGetData` and assert on the returned error.

**Fix:** Either expose `__mrnGetData` in `init.ts` (same safety argument as `__mrnSetData`) and turn UAT-03b into a real assertion, or drop the unused `serverErrors` field and make the test inspect the DOM's form-level error banner directly (see `Form.browser-test.ts` for the banner's `bg-destructive/10` selector pattern).

---

_Reviewed: 2026-04-17_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
