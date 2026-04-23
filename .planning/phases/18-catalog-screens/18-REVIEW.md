---
phase: 18-catalog-screens
reviewed: 2026-04-23T18:55:28Z
depth: standard
files_reviewed: 28
files_reviewed_list:
  - backend/crates/marionette/src/builders/button.rs
  - backend/crates/marionette/src/builders/container.rs
  - backend/crates/gallery-demo/src/lib.rs
  - backend/crates/gallery-demo/src/fixtures.rs
  - backend/crates/gallery-demo/src/catalog/mod.rs
  - backend/crates/gallery-demo/src/catalog/buttons.rs
  - backend/crates/gallery-demo/src/catalog/forms.rs
  - backend/crates/gallery-demo/src/catalog/data_table.rs
  - backend/crates/gallery-demo/src/catalog/feedback.rs
  - backend/crates/gallery-demo/src/catalog/typography.rs
  - backend/crates/gallery-demo/src/handlers/mod.rs
  - backend/crates/gallery-demo/src/handlers/show.rs
  - backend/crates/gallery-demo/src/handlers/catalog_forms.rs
  - backend/crates/gallery-demo/src/handlers/fetch_rows.rs
  - frontend/src/app.css
  - frontend/src/lib/components/form/Button.svelte
  - frontend/src/lib/components/form/Button.browser-test.ts
  - frontend/src/lib/components/form/SelectInput.svelte
  - frontend/src/lib/components/form/SelectInput.browser-test.ts
  - frontend/src/lib/components/form/Checkbox.svelte
  - frontend/src/lib/components/form/Checkbox.browser-test.ts
  - frontend/src/lib/components/form/Switch.svelte
  - frontend/src/lib/components/form/Switch.browser-test.ts
  - frontend/src/lib/components/form/RadioGroup.svelte
  - frontend/src/lib/components/form/RadioGroup.browser-test.ts
  - frontend/src/lib/components/layout/Container.svelte
  - frontend/src/lib/components/layout/Container.browser-test.ts
findings:
  critical: 0
  warning: 0
  info: 4
  total: 4
status: advisory
---

# Phase 18: Code Review Report

**Reviewed:** 2026-04-23T18:55:28Z
**Depth:** standard
**Files Reviewed:** 28
**Status:** advisory (4 Info findings; no Critical or Warning issues)

## Summary

Phase 18 ships five catalog screens (CAT-01..CAT-05) plus framework polish (Button props, blur dispatch, Container icon affordance). Code quality is high across all 28 files: consistent patterns, thorough tests, defensive payload extraction, deterministic fixtures, and stable id conventions. Clippy pedantic passes for new code; the pre-existing 31 marionette-crate clippy errors at base are correctly logged in `deferred-items.md` as out-of-scope.

No Critical or Warning findings. Four Info items are flagged for future polish — none block shipping this phase. All are consistent with the "gallery/demo surface, not production" posture documented in the phase context.

Security posture: appropriate for a localhost-only developer gallery binary. The `fetch-rows` source-dispatch correctly rejects unknown sources with `BadPayload` and saturating-arithmetic bounds pagination at the 500-row cap. Every validator handler uses `serde_json::Value::as_str` / `as_bool` with safe `unwrap_or_default()` fallbacks — no panic paths on malformed payloads. All `AuthRequirement::None` gates are appropriate for the no-auth, no-DB gallery binary. No hardcoded secrets, no `eval` usage, no shell interpolation, no SQL (SeaORM `MockDatabase` only used in tests).

## Info

### IN-01: Button.svelte destructures `bind` and `children` props that are never used

**File:** `frontend/src/lib/components/form/Button.svelte:10-22`

**Issue:** The component signature declares `bind?: string` (line 12) and `children?: Snippet` (line 15) in `$props()`, but neither is referenced in the script or template. The template renders `{#if props.label}{props.label}{/if}` and the icon/spinner — there is no `{@render children?.()}` call, and `bind` is never read. This is dead prop surface: callers passing `bind` or child snippets would see them silently discarded.

This is a pre-Phase 18 artifact that Phase 18 Plan 01 did not introduce — the destructuring was already present and was carried through the rewire. Not a scope-boundary violation (the file was heavily rewritten in Plan 01), so flagging as observed during review.

**Fix:** Either remove the unused destructured fields, or if future catalog usage is anticipated (e.g., Button with child icon in a compound layout), add a brief comment explaining the contract. Minimal change:
```svelte
let {
    props = {},
    action,
    surface,
}: {
    props: Record<string, unknown>;
    action?: ComponentAction;
    surface: string;
} = $props();
```

### IN-02: CAT-02 Checkbox / Switch / Textarea interactive fields mention "Required" in description but do not set `.required(true)` on the builder

**File:** `backend/crates/gallery-demo/src/catalog/forms.rs:325-332, 371-378, 485-492`

**Issue:** Three of the six CAT-02 interactive fields have descriptions that begin with "Required." — the Checkbox (`"I agree to the terms"`), Switch (`"Enable notifications"`), and Textarea (`"Bio (min. 20 characters)"`). Their builder chains do NOT call `.required(true)`, whereas the Select Card (line 281) and Radio Card (line 441) interactive fields DO call `.required(true)`. The inconsistency is visible to handler authors reading the demo code.

Server-driven validation fires on blur regardless of this flag (the handler enforces the rule), so there is no functional bug. But a reader of the demo code might conclude the flag is optional when it's actually just inconsistent within the catalog itself.

**Fix:** Either add `.required(true)` to the Checkbox, Switch, and Textarea interactive-field builders for consistency with Select/Radio, or remove `.required(true)` from Select/Radio to match — whichever better aligns with the design intent. A one-line comment noting "required-indicator is visual (description copy); validation is server-driven on blur" would also document the choice.

### IN-03: `input_display_label` silently returns `"Unknown"` on drift

**File:** `backend/crates/gallery-demo/src/catalog/forms.rs:180-190`

**Issue:** The helper maps stem strings to display labels via a `match` with a wildcard fallthrough returning `"Unknown"`. If a future refactor renames a stem (e.g., "text" → "text-input") without updating the caller in `assemble_card`, the Card heading would silently read "Unknown" instead of failing loudly. The function is only called from `assemble_card` in this file, and all six callers pass the six known stems.

Not a current bug — every existing caller is correct — but the silent fallback pattern means drift would produce a cosmetic regression that Chrome MCP UAT would have to catch visually.

**Fix:** Replace the wildcard with `unreachable!("unknown catalog-forms input stem: {stem}")` so the compiler/runtime fails on drift instead of silently rendering "Unknown". Since the function is called deterministically at registration time, the panic would surface during `cargo test` rather than at user-facing runtime.

### IN-04: `seed_for_key` `#[allow(clippy::match_same_arms)]` applies to the whole match, not just the zero-state arms

**File:** `backend/crates/gallery-demo/src/handlers/show.rs:56-210`

**Issue:** Per Plan 18-04 deviation notes, `#[allow(clippy::match_same_arms)]` was applied at the match-expression level (line 61) because an arm-level allow is not supported. The intent is to suppress the lint for the explicit zero-state catalog arms (`"catalog-buttons"`, `"catalog-typography"` → `serde_json::json!({})`) which duplicate the wildcard `_ => serde_json::json!({})`. However, the allow now silences any FUTURE match_same_arms issues across the ~150-line match block — including arms that might shouldn't share bodies. A reviewer adding a new arm that accidentally duplicates another's logic won't see the lint warning.

**Fix:** Tighten the scope. Option A: extract the zero-state catalog arms into a helper (`fn catalog_zero_seed() -> serde_json::Value { serde_json::json!({}) }`) and call it from each arm — now the arms are not literally identical, so the lint doesn't fire and the allow can be removed. Option B: fold the zero-state arms into the wildcard and document the known keys in a comment above the match. The current in-code comment already lists the intent; a sentinel helper preserves the documentation without the blanket-allow footprint.

---

_Reviewed: 2026-04-23T18:55:28Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
