# Phase 14: FormScreen Enhancements — Research

**Researched:** 2026-04-17
**Domain:** shadcn-svelte Field family + Marionette SDUI leaf-component composition (Svelte 5 + Tailwind v4)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Area A — FormScreen disposition

- **D-A1:** Retire `FormScreen.svelte` (hard delete). Delete `frontend/src/lib/components/screen/FormScreen.svelte` and `FormScreen.browser-test.ts`. Not registered in `defaults.ts`, no backend builder, no CRM call sites — same orphan situation Phase 13 resolved by deleting `TableScreen.svelte` (13-CONTEXT.md D-A2). No `@deprecated` tombstone — the pre-deployment posture rejects back-compat shims.
- **D-A2:** Handlers compose title + back-button region inline; no DRY helper. Each CRM form handler builds `Container([Heading("Edit Contact"), Button(ArrowLeft, back_action), …FieldSets, …action row])` explicitly.
- **D-A3:** Keep `Form.svelte` as the `<form>` boundary. Stays registered (`'form'`). Emits `<form>`, traps `onsubmit`, dispatches `action` as an `ActionMessage`, renders form-level errors from `/_errors/{form_bind}` as a banner. May tweak class wrapping (e.g., wrap children in a `Field.Group` for consistent spacing) but does not change API surface.

#### Area B — Field integration strategy (per-leaf-component internal wrap)

- **D-B1:** Internal `Field.Field` wrap per leaf component. `TextInput.svelte`, `SelectInput.svelte`, `Checkbox.svelte`, plus the three new primitives (`Textarea`, `RadioGroup`, `Switch`) each render their own `<Field.Field data-invalid={!!err}><Field.Label for={id}>{label}</Field.Label><Input id={id} aria-invalid={!!err} …/><Field.Description>{props.description}</Field.Description><Field.Error>{err}</Field.Error></Field.Field>`. One protocol node per field.
- **D-B2:** Keep `/_errors/{bind}` convention for error data flow. Leaf components read `getData(surface, '/_errors' + bind)`. When non-empty: render `<Field.Error>`, add `data-invalid` to the `Field.Field` wrapper, add `aria-invalid` to the input. Form-level errors (`/_errors/{form_bind}` as an array) continue to render as a banner in `Form.svelte`.
- **D-B3:** Backend builders gain `.description(…)` helper on every field primitive. `TextInput`, `SelectInput`, `Checkbox`, `Textarea`, `RadioGroup`, `Switch` each get `description: Option<String>`. Existing `helperText` prop in TextInput renames to `description` to match shadcn nomenclature — pre-deployment posture, no back-compat alias.
- **D-B4:** Field `id` generated from a stable source. Use the component's protocol `id` (set by backend builder via `.id(...)`) as the HTML `id`. If absent, leaf component falls back to `crypto.randomUUID()` at mount.

#### Area C — FieldSet component + responsive layout

- **D-C1:** New `FieldSet` SDUI component (`field-set`). Registered as `'field-set'`. Backend builder `FieldSet { legend, description, cols }`. Renders `<Field.Set><Field.Legend>{legend}</Field.Legend><Field.Description>{description}</Field.Description><Field.Group class="{gridClasses}">{children}</Field.Group></Field.Set>`.
- **D-C2:** Flat visual style — `Field.Set` + `Field.Separator`, no `Card.Root` wrapping. Between consecutive sibling `FieldSet`s, handler composes a plain `Field.Separator` SDUI node (new, thin) OR parent `Form.svelte` inserts them automatically — planner decides which is cleaner. Preference: explicit nodes.
- **D-C3:** Default auto-responsive grid — 1-col mobile, 2-col desktop. `FieldSet` with no `cols` prop renders `<Field.Group class="grid grid-cols-1 md:grid-cols-2 gap-4">`. Defers container queries (`@container/field-group`) to v2.
- **D-C4:** Two overrides — `FieldSet.cols` and per-field `full_width`. When `FieldSet.cols` is set, replaces auto-responsive with `grid-cols-{cols}`. Each field primitive gains optional `full_width: bool` — when `true`, the field's `Field.Field` gets `col-span-full`.

#### Area D — Action row pattern

- **D-D1:** Recipe pattern for save/cancel — horizontal `Field.Field`. Either (a) a plain `Container` with `class="flex gap-2 justify-end"` and Button children, or (b) a small dedicated SDUI component (e.g., `field-row`) wrapping `<Field.Field orientation="horizontal">{children}</Field.Field>`. Claude's discretion during planning.

#### Area E — Scope: deferred fixes + new primitives

- **D-E1:** Fix TextInput `input_type`/`type` prop mismatch. (Phase 13 13-07 addressed it — verify fix survived into Phase 14 and there's no regression.)
- **D-E2:** Fix NodeRenderer `handleBlur` unmount race. Planner picks between (a) `TextInput.handleBlur` guard or (b) `NodeRenderer.svelte` structural fix. Preference: (b).
- **D-E3:** Install Textarea primitive + SDUI wrapper. `npx shadcn-svelte@latest add textarea`. New SDUI component `textarea`. Backend builder `Textarea` with `placeholder`, `rows`, `required`, `disabled`, `description`, `label`, `full_width`.
- **D-E4:** Install RadioGroup + Switch primitives + SDUI wrappers. `npx shadcn-svelte@latest add radio-group switch`. Backend builders `RadioGroup` (with `options: Vec<{value, label, description?}>`) and `Switch` (boolean toggle).

### Claude's Discretion

- Exact shape of the action row (`Container` with class vs. new `field-row` SDUI component).
- Whether `Field.Separator` between sibling `FieldSet`s is an explicit adjacency-list node or auto-inserted by `Form.svelte` / parent. Preference: explicit node (D-C2 rationale).
- Field `id` fallback strategy (UUID vs derived-from-bind) when handler forgot `.id(...)` — D-B4.
- Blur-race fix location (TextInput vs NodeRenderer) — D-E2 notes preference for NodeRenderer.
- Per-component migration order within the phase (form leaves first, then FieldSet, then new primitives, then CRM smoke migration — or interleaved).
- Specific class utility strings for the auto-responsive grid (e.g., `gap-4` vs `gap-6`; `md:grid-cols-2` vs container queries) as long as D-C3's behavior holds.
- Test granularity per component (browser tests for every leaf vs. a handful of representative ones).

### Deferred Ideas (OUT OF SCOPE)

- Wizard / multi-step forms (FORM-03) — v2.
- Arbitrary per-field col-span / row-span beyond `full_width` (FORM-04) — v2.
- Container-query-based responsive layouts (`@container/field-group`) — v2 polish.
- Persistent form state across reloads / navigation — v2.
- Card-wrapped sections — explicitly rejected (D-C2) in favor of flat `Field.Set` + `Field.Separator`.
- `FormScreen` as a first-class SDUI component — retired (D-A1).
- DRY helper for "heading + back button" — rejected for now (D-A2).
- Validation via Superforms / Formsnap / client-side Zod — out-of-scope per REQUIREMENTS.md §Out of Scope.
- Full CRM form migration — Phase 15 (COMP-03).
- Additional form primitives (Combobox, DatePicker, FileInput) — add when a real CRM screen demands them.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FORM-01 | Form fields display consistent label, description, and error layout using shadcn Field components | D-B1 (per-leaf `Field.Field` wrap) + shadcn Field anatomy verified on shadcn-svelte.com/docs/components/field |
| FORM-02 | Related form fields can be grouped in card sections with visual separators | D-C1 (new `FieldSet` SDUI component) + D-C2 (flat `Field.Set` + `Field.Separator`) + D-C3 (responsive grid default) |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

No `./CLAUDE.md` at the project root. Global `~/.claude/CLAUDE.md` directives that apply:

- **`find /home/oetiker`** must NOT be used — home directory is enormous. Use `Glob`, `cargo metadata`, or targeted paths instead.
- The user's username is `oetiker`.

Project memory highlights relevant to Phase 14:

- **Don't hand-roll UI design** — adopt framework recipes (shadcn-svelte etc.) over hand-rolled UI even if smaller; mismatches are design problems to solve, not reasons to abandon the framework. **Phase 14 fully embraces the shadcn Field recipe verbatim.**
- **Options need reasoning + check framework first** — every option gets pros/cons/rationale; check shadcn-svelte recipes before inventing custom designs. **Pre-checked: the Field recipe covers every FORM-01/02 need.**
- **Pre-deployment posture** — no deployed base yet; fix root causes, no migration shims. **Applies to `helperText→description` rename, `FormScreen` deletion, `input_type` alignment.**
- **Chrome MCP for UAT** — drive human-verify checkpoints with claude-in-chrome tools instead of handing a walkthrough to the user.
- **shadcnSvelteSearchTool hangs** — never finishes. Use `shadcnSvelteListTool` / `shadcnSvelteGetTool` or WebFetch instead.
- **Svelte MCP server MUST be used** whenever svelte development is involved. [VERIFIED: `.mcp.json` registers `svelte`, `shadcn-svelte`, `rust-docs`]

## Summary

Phase 14 adopts the shadcn-svelte `Field` family (`Field.Field`, `Field.Label`, `Field.Description`, `Field.Error`, `Field.Set`, `Field.Legend`, `Field.Group`, `Field.Separator`) as the canonical layout primitive for every form leaf. Each existing leaf (`TextInput`, `SelectInput`, `Checkbox`) is rewritten internally to render its own `<Field.Field>` wrapper while preserving the SDUI contract (`surface`/`props`/`bind`/`action`). Three new primitives (`Textarea`, `RadioGroup`, `Switch`) follow the same internal-wrap pattern. A new `FieldSet` SDUI component (`field-set`) renders the semantic `<Field.Set>`/`<Field.Group>` grouping with an auto-responsive 1-col-mobile / 2-col-desktop grid by default. The orphan `FormScreen.svelte` is hard-deleted (same precedent as `TableScreen` in Phase 13). Two Phase-13-deferred bugs close out inside Phase 14: (1) verifying the `input_type` alignment hasn't regressed; (2) fixing the `NodeRenderer.get bind` undefined race on `TextInput` blur by guarding the destructured prop access inside the `{#if node}` branch.

The shadcn-svelte primitives required are all installed via `npx shadcn-svelte@latest add field textarea radio-group switch` and generated under `frontend/src/lib/components/ui/{field,textarea,radio-group,switch}/`. `bits-ui@2.17.3` is already installed and satisfies every primitive's peer dep. Tailwind v4 is already configured with OKLCH semantic tokens (Phase 10), so no theming work is required — all shadcn-generated class utilities (`text-destructive`, `text-muted-foreground`, `border-destructive`, `aria-invalid:*`) resolve to the existing token set.

**Primary recommendation:** Install the four primitives first (single Wave 0 task), then rewrite the three existing leaves + add the three new ones in parallel under a strict "internal Field wrap, SDUI contract unchanged" spec, then add the `FieldSet` and `FieldSeparator` SDUI components, then smoke-migrate the `contact.rs` edit form to validate every new primitive end-to-end, then close out the two deferred bugs. Spec/protocol additions (new component types + `description`/`full_width` props on existing types) land incrementally as each leaf is touched — no "big-bang" schema merge.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `shadcn-svelte` (CLI) | `1.2.7` | Primitive code-gen for `field`, `textarea`, `radio-group`, `switch` | [VERIFIED: `npx shadcn-svelte@latest --version` → `1.2.7`] Official Svelte port of shadcn/ui; every Marionette UI component already flows through it. |
| `bits-ui` | `^2.17.3` | Headless primitives (RadioGroup.Root, Switch.Root, Label, etc.) | [VERIFIED: `frontend/package.json`] shadcn-svelte generates bits-ui compositions; already installed. |
| `@lucide/svelte` | `^1.8.0` | Icons (already used in `FormScreen.svelte` for `ArrowLeft`) | [VERIFIED: `frontend/package.json`] |
| `tailwindcss` | `^4.2.0` | Utility CSS (grid / gap / destructive / muted-foreground) | [VERIFIED: `frontend/package.json`] Tailwind v4 with OKLCH tokens; already wired. |
| `svelte` | `^5.53.0` | Runes (`$state` / `$derived` / `$props`) for leaf components | [VERIFIED: `frontend/package.json`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `vitest-browser-svelte` | `^2.1.0` | Browser component tests (real Chromium, `expect.element()`) | Every new/rewritten leaf component — regression suite per Phase 14's `*.browser-test.ts` tree. |
| `@playwright/test` | `^1.58.2` | E2E validation of the contact form smoke path | Final phase gate: one E2E check per FORM-01/02 + one for the `input_type=password` fix. |
| `tailwind-merge` / `clsx` | `^3.5.0` / `^2.1.1` | `cn()` helper used by every generated shadcn primitive | Already standard — no new integration. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Native `<label>` + `<p>` + `<span>` ad-hoc | `Field.Field` + `Field.Label` + `Field.Description` + `Field.Error` | Rejected — project memory mandates framework recipes over hand-rolled UI. Field gives us `role="group"`, accessible `data-invalid` semantics, and the shadcn design system "for free." |
| Formsnap / Superforms (client-side Zod) | `/_errors/{bind}` server-driven validation | Rejected — REQUIREMENTS.md §Out of Scope and CONTEXT.md D-B2. Server-side validation is canonical. |
| Card.Root wrapping for grouped fields | `Field.Set` + `Field.Separator` | Rejected in CONTEXT.md D-C2 — flat visual style matches shadcn recipe; cards felt visually heavy. |
| Container queries (`@container/field-group`) for responsive grid | Viewport breakpoint (`md:grid-cols-2`) | Deferred to v2 (CONTEXT.md D-C3). Container queries are the "correct" long-term answer but viewport `md:` is good enough for v1.1 and avoids surprise breakpoints inside sidebars. |
| Auto-insert `Field.Separator` between sibling FieldSets in `Form.svelte` | Explicit `FieldSeparator` SDUI node in adjacency list | Preference (D-C2): explicit nodes make the protocol self-describing and keep node-patching granular. Both are valid; planner decides. |

**Installation:**

```bash
# Run from frontend/
npx shadcn-svelte@latest add field textarea radio-group switch
```

This generates:
- `frontend/src/lib/components/ui/field/` (`field.svelte`, `index.ts`) — Field.Field, Field.Label, Field.Description, Field.Error, Field.Set, Field.Legend, Field.Group, Field.Separator, Field.Content
- `frontend/src/lib/components/ui/textarea/` (`textarea.svelte`, `index.ts`)
- `frontend/src/lib/components/ui/radio-group/` (`radio-group.svelte`, `radio-group-item.svelte`, `index.ts`)
- `frontend/src/lib/components/ui/switch/` (`switch.svelte`, `index.ts`)

No `package.json` changes are expected (bits-ui already installed). [CITED: https://shadcn-svelte.com/docs/components/field install command]

**Version verification:**

- `shadcn-svelte` CLI resolves to `1.2.7` as of 2026-04-17. [VERIFIED: `npx shadcn-svelte@latest --version`]
- `bits-ui` installed at `^2.17.3` (latest `2.18.0` on npm). [VERIFIED: `npm view bits-ui version`]
- `@lucide/svelte` installed at `^1.8.0`. [VERIFIED]

## Architecture Patterns

### Recommended Project Structure

```
frontend/src/lib/components/
├── form/
│   ├── Form.svelte              # existing — <form> boundary (stays)
│   ├── TextInput.svelte         # REWRITTEN — internal Field.Field wrap
│   ├── SelectInput.svelte       # REWRITTEN — internal Field.Field wrap
│   ├── Checkbox.svelte          # REWRITTEN — internal Field.Field wrap
│   ├── Textarea.svelte          # NEW — new primitive, internal Field.Field wrap
│   ├── RadioGroup.svelte        # NEW — new primitive, internal Field.Field wrap
│   ├── Switch.svelte            # NEW — new primitive, internal Field.Field wrap
│   ├── FieldSet.svelte          # NEW — structural (Field.Set + Field.Group)
│   ├── FieldSeparator.svelte    # NEW (D-C2 preferred) — thin Field.Separator node
│   ├── FieldRow.svelte          # NEW OPTIONAL (D-D1 discretion) — horizontal action row
│   ├── Button.svelte            # existing — unchanged
│   └── *.browser-test.ts        # one per component
├── ui/
│   ├── field/                   # NEW — shadcn-generated
│   ├── textarea/                # NEW — shadcn-generated
│   ├── radio-group/             # NEW — shadcn-generated
│   ├── switch/                  # NEW — shadcn-generated
│   ├── input/, label/, select/, checkbox/, separator/, ...  # existing
├── core/
│   └── NodeRenderer.svelte      # EDITED — guard `get bind` unmount race
├── screen/
│   # FormScreen.svelte DELETED (hard delete, D-A1)
│   # FormScreen.browser-test.ts DELETED
└── registry/
    └── defaults.ts              # EDITED — register 'field-set', 'textarea', 'radio-group', 'switch', maybe 'field-row', 'field-separator'
```

```
backend/crates/marionette/src/builders/
└── standard.rs
    # EXTENDED:
    #   TextInput: add `description`, `full_width`
    #   Select:    add `description`, `full_width`
    #   Checkbox:  add `description`, `full_width`
    #   Form:      no prop changes
    # NEW STRUCTS:
    #   FieldSet { legend, description, cols }
    #   Textarea { label, placeholder, rows, required, disabled, description, full_width }
    #   RadioGroup { label, options: Vec<RadioOption>, description, full_width }
    #   Switch { label, disabled, description, full_width }
    #   FieldSeparator {} (if explicit-node path chosen)
    #   FieldRow {} (if dedicated-action-row path chosen)
```

### Pattern 1: Internal Field.Field Wrap per Leaf Component

**What:** Every form-leaf Svelte component renders its own shadcn `Field.Field` wrapper with `Field.Label`, the control, `Field.Description`, and `Field.Error`. The SDUI contract (`surface`/`props`/`bind`/`action`) is unchanged, so one protocol node continues to produce one form field.
**When to use:** For all six form leaves (`TextInput`, `SelectInput`, `Checkbox`, `Textarea`, `RadioGroup`, `Switch`).

**Example (new `TextInput.svelte` shape):**

```svelte
<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { getData, setData } from '$lib/store/data.svelte';
	import { markDirty, clearDirty } from '$lib/store/dirty.svelte';
	import { sendAction } from '$lib/transport/dispatcher';
	import type { ComponentAction } from '$lib/transport/messages';

	let { props = {}, bind, action, surface }: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();

	// Stable id — prefer handler-supplied, fall back to UUID (D-B4).
	const fallbackId = crypto.randomUUID();
	let fieldId = $derived((props.id as string) ?? fallbackId);

	let value = $derived(bind ? ((getData(surface, bind) as string) ?? '') : '');
	let fieldError = $derived(
		bind ? ((getData(surface, '/_errors' + bind) as string) ?? '') : ''
	);
	let hasError = $derived(!!fieldError);

	function handleInput(e: Event) { /* unchanged from current TextInput */ }
	function handleFocus() { /* unchanged */ }
	function handleBlur() { /* unchanged (D-E2 fix may live in NodeRenderer instead) */ }
</script>

<Field.Field data-invalid={hasError || undefined}>
	{#if props.label}
		<Field.Label for={fieldId}>{props.label}</Field.Label>
	{/if}
	<Input
		id={fieldId}
		type={(props.input_type as string) ?? 'text'}
		placeholder={props.placeholder as string}
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

Key points:

- `data-invalid` and `aria-invalid` use `value || undefined` so the attribute is omitted (not `data-invalid="false"`) when there's no error — required by shadcn Field's CSS selector conventions. [CITED: https://shadcn-svelte.com/docs/components/field]
- `Input` already has built-in `aria-invalid:*` styling (ring-destructive / border-destructive) via its generated Tailwind classes — no extra `class` override required. [VERIFIED: `frontend/src/lib/components/ui/input/input.svelte` line 41]
- `Field.Description` is suppressed while an error is active (shadcn recipe pattern — error replaces description).
- `props.input_type` (snake_case) is the backend-authoritative key. [VERIFIED: `backend/crates/marionette/src/builders/standard.rs` line 32 — `pub input_type: Option<String>`]

### Pattern 2: FieldSet Structural Component

**What:** A new SDUI component type `field-set` that renders `<Field.Set>` + `<Field.Legend>` + `<Field.Description>` + `<Field.Group>` with a responsive grid class string. Children resolve via `NodeRenderer` (same adjacency-list pattern as `Container`, `Form`, `Grid`).
**When to use:** To group related form fields with an accessible `<fieldset>`/`<legend>` semantic anchor and a 1-col-mobile / 2-col-desktop layout.

**Example:**

```svelte
<!-- frontend/src/lib/components/form/FieldSet.svelte -->
<script lang="ts">
	import * as Field from '$lib/components/ui/field';
	import type { Snippet } from 'svelte';

	let { props = {}, surface, children }: {
		props: Record<string, unknown>;
		surface: string;
		children?: Snippet;
	} = $props();

	let cols = $derived(props.cols as number | undefined);
	// D-C3 default: 1-col mobile, 2-col desktop.
	// D-C4 override: explicit `cols` uses fixed grid (no breakpoint stack).
	// Use inline style for dynamic cols — Tailwind v4 JIT can't resolve `grid-cols-{n}` (PITFALLS #12).
	let gridClass = $derived(cols ? 'grid gap-4' : 'grid grid-cols-1 md:grid-cols-2 gap-4');
	let gridStyle = $derived(cols ? `grid-template-columns: repeat(${cols}, minmax(0, 1fr))` : undefined);
</script>

<Field.Set>
	{#if props.legend}
		<Field.Legend>{props.legend}</Field.Legend>
	{/if}
	{#if props.description}
		<Field.Description>{props.description}</Field.Description>
	{/if}
	<Field.Group class={gridClass} style={gridStyle}>
		{@render children?.()}
	</Field.Group>
</Field.Set>
```

**Backend builder (Rust):**

```rust
#[derive(ComponentBuilder)]
#[component(type = "field-set")]
pub struct FieldSet {
    #[builder(optional)] pub legend: Option<String>,
    #[builder(optional)] pub description: Option<String>,
    #[builder(optional)] pub cols: Option<u8>, // None = auto-responsive
}
```

### Pattern 3: Per-Field `full_width` Override

**What:** Any form-field primitive can set `props.full_width = true` to span all columns within its parent `FieldSet`. Implemented by adding a `class="col-span-full"` to the leaf's `Field.Field` wrapper.
**When to use:** Long textareas, multi-line addresses, descriptions.

**Example (inside a leaf):**

```svelte
<Field.Field data-invalid={hasError || undefined} class={props.full_width ? 'col-span-full' : undefined}>
```

### Pattern 4: Form-Level Errors via Data Store

**What:** Unchanged from the existing codebase. `Form.svelte` reads `getData(surface, '/_errors' + bind)` where `bind` is a form-level path (e.g., `/formErrors`). When the result is a non-empty array, render a banner above `{@render children?.()}`. Per-field errors are handled inside each leaf.
**When to use:** Anywhere — server rejects a submit with cross-field errors ("passwords don't match", "email already exists").

### Anti-Patterns to Avoid

- **Hand-rolling the Field anatomy with plain `<label>` + `<p>` + `<div>`.** The shadcn Field recipe handles the `data-invalid` cascade, `aria-describedby` wiring, and `role="group"` semantics that are easy to get wrong. Project memory mandates recipes over hand-rolled UI.
- **Wrapping a `FieldSet` around the top-level form.** `Form.svelte` is the `<form>` boundary; FieldSets go inside. Composing `<form><fieldset>…</fieldset></form>` is correct HTML; `<fieldset><form>…</form></fieldset>` is not.
- **Relying on `props.type` instead of `props.input_type`.** Backend serializes snake_case. The Phase 13 fix stands — don't reintroduce a `props.type ?? props.input_type` fallback chain (pre-deployment posture: no back-compat shims).
- **Mutating `helperText` back in as a backwards-compat alias.** CONTEXT.md D-B3 explicitly forbids. Rename to `description`, update every call site, done.
- **Driving the responsive grid via `@media` in a style tag or a `<Grid>` wrapper.** Use `Field.Group` with Tailwind utilities — stays inside the recipe and keeps the class utilities JIT-tractable.
- **Auto-inserting `Field.Separator` based on a hidden `Form.svelte` heuristic.** If auto-insertion is chosen over explicit nodes, the logic must be documented and the adjacency list must still be a faithful representation of the rendered tree. Preferred path: explicit `field-separator` SDUI node (D-C2 rationale — "self-describing protocol").

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Label + input + description + error layout | Raw HTML with ad-hoc Tailwind classes | `<Field.Field>` + `<Field.Label>` + `<Field.Description>` + `<Field.Error>` | shadcn recipe handles `data-invalid` cascade, spacing, typography, dark-mode variants. |
| Grouped fields with legend | Raw `<fieldset>`/`<legend>` + flex/grid divs | `<Field.Set>` + `<Field.Legend>` + `<Field.Group>` | Semantic HTML + shadcn styling + integrated `data-invalid` inheritance. |
| Vertical separator between sections | `<hr class="my-4 border-border">` | `<Field.Separator />` | Uses shadcn's separator primitive with correct theme tokens; can hold inline content (e.g., "Or"). |
| Horizontal action row (save/cancel) | `<div class="flex gap-2 justify-end">` (plain) | `<Field.Field orientation="horizontal">…</Field.Field>` | Optional but recipe-faithful — matches shadcn's canonical submit row pattern. Plain Container is acceptable per D-D1. |
| Radio group | Plain `<input type="radio">` with manual name grouping | shadcn `RadioGroup.Root` + `RadioGroup.Item` (from `bits-ui`) | bits-ui handles keyboard navigation, focus trap, and roving-tabindex. |
| Toggle / switch | Checkbox with CSS trickery | shadcn `Switch` (bits-ui) | Correct ARIA (`role="switch"`), keyboard handling, animated thumb. |
| Textarea resize + styling | `<textarea>` with ad-hoc classes | shadcn `Textarea` | Consistent with `Input` styling (border/ring/destructive states, dark-mode variants). |
| Dynamic `grid-cols-N` from a prop | `class="grid-cols-{cols}"` (broken in Tailwind v4) | Inline `style="grid-template-columns: repeat({cols}, minmax(0, 1fr))"` | Tailwind v4 can't JIT dynamic class names; PITFALLS.md #12. [VERIFIED: Existing `FormScreen.svelte` line 91 uses this exact pattern.] |
| Stable field `id` | Mount-time `Math.random()` | Handler-supplied `.id(...)` with `crypto.randomUUID()` fallback | `Field.Label for={id}` accessibility + Playwright selector stability. |

**Key insight:** Every primitive Phase 14 needs already exists in the shadcn-svelte registry. Hand-rolling is strictly redundant — and the project memory explicitly mandates adopting framework recipes. Total new Svelte component code for the six leaves should be <500 lines; the Field family absorbs the layout/spacing/a11y concerns.

## Runtime State Inventory

Not applicable — Phase 14 is an additive feature + rewrite phase, not a rename / migration. No persisted keys, no OS-registered state, no env vars with names that change. The only "stored" state touched is the SDUI data-store at runtime (per-surface), and the protocol `description` key simply becomes the authoritative source for helper text going forward (replacing `helperText`). Backend handlers must be updated in the same phase (`.description(...)` vs `.helper_text(...)`), but since no backend handler currently uses `.helper_text(...)` (the only `helperText` references are `.planning/phases/10-foundation/...` historical docs and the `TextInput.svelte` frontend fallback branch) [VERIFIED: `grep -rn "helperText\|helper_text" .` — 8 matches, all in planning docs or `TextInput.svelte:71-72`], the rename is effectively a no-op on existing CRM handler code.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — SDUI data store is ephemeral per-connection; no `helperText` keys persisted. | None. |
| Live service config | None — no external service references a form-prop name. | None. |
| OS-registered state | None. | None. |
| Secrets/env vars | None. | None. |
| Build artifacts | None — no generated types to clean. The `frontend/src/lib/transport/messages.ts` is hand-maintained; extend in-phase. | None (add new types as tasks land). |

## Common Pitfalls

### Pitfall 1: Dynamic Tailwind classes from a runtime prop

**What goes wrong:** Writing `class="grid-cols-{cols}"` where `cols` is a runtime value produces a missing class at build time because Tailwind v4's JIT can't resolve interpolated names.
**Why it happens:** JIT scans source for literal class strings; interpolated values are invisible.
**How to avoid:** Use an inline `style="grid-template-columns: repeat({cols}, minmax(0, 1fr))"` and a static `grid gap-4` class. The existing `FormScreen.svelte` already does this correctly (line 91) — mirror the pattern in `FieldSet.svelte`.
**Warning signs:** A `cols=3` FieldSet renders in one column in the browser; `class` attribute in dev-tools shows `grid-cols-3` with no matching CSS rule.
**Source:** `.planning/research/PITFALLS.md` #12 [VERIFIED]

### Pitfall 2: `NodeRenderer.get bind` TypeError on TextInput blur

**What goes wrong:** `TypeError: Cannot read properties of undefined (reading 'bind')` fires asynchronously when a `TextInput` loses focus while its parent tree is being patched (e.g., country-select → node-patch replaces sibling fields; or filter input blurs while DataTable re-renders).
**Why it happens:** `TextInput.handleBlur` calls `clearDirty` → `setData` → surface re-renders → `NodeRenderer` children unmount → Svelte compiler's generated accessor for the destructured `{bind}` prop reads `node.bind` on a now-undefined `node`.
**How to avoid:** Guard the destructuring in `NodeRenderer.svelte` — move the destructure **inside** the `{#if node}` branch (preferred per D-E2) so Svelte's compiler generates a nullish-safe accessor. Alternative: no-op `TextInput.handleBlur` when `bind === undefined`. Preference is the structural fix.
**Warning signs:** Console spam `TypeError: Cannot read properties of undefined (reading 'bind')` with stack trace through `NodeRenderer.svelte` compiled line ~116. Caught by `ErrorBoundary`, user-invisible but noisy.
**Source:** `.planning/phases/13-datatable-enhancements/deferred-items.md` §"NodeRenderer `get bind` undefined" [VERIFIED]

### Pitfall 3: `props.type` vs `props.input_type` drift

**What goes wrong:** Password fields render as `<input type="text">` because the Svelte component reads `props.type` while the backend serializes `input_type`.
**Why it happens:** Field names drifted between the initial leaf rewrite (Phase 10) and the backend builder's `input_type` field name (snake_case Rust convention). Phase 13 Plan 07 fixed it by aligning the Svelte side to `props.input_type`.
**How to avoid:** Only read `(props.input_type as string) ?? 'text'` — no fallback chain. Phase 14 must preserve this (D-E1). Guard with a browser test that sets `input_type: 'password'` and asserts `<input type="password">`.
**Warning signs:** Login page password field renders plaintext; any other field with an explicit `input_type` ignores it.
**Source:** `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` and Phase 13-07 SUMMARY [VERIFIED]

### Pitfall 4: Field.Field without `data-invalid` omitted (not empty-string)

**What goes wrong:** shadcn Field's CSS selectors use `[data-invalid]` (attribute presence), not `[data-invalid="true"]`. Writing `data-invalid={hasError}` (where `hasError` is a boolean) renders `data-invalid="false"` in the DOM — which is presence-positive and triggers the error style.
**Why it happens:** Svelte attribute bindings serialize booleans to strings.
**How to avoid:** Use `data-invalid={hasError || undefined}` so the attribute is omitted entirely when `hasError === false`. Same pattern for `aria-invalid`.
**Warning signs:** All fields render with red borders even when there's no error.
**Source:** [CITED: shadcn-svelte.com/docs/components/field validation pattern]

### Pitfall 5: Mounting `<form>` inside `<fieldset>`

**What goes wrong:** HTML disallows nesting a `<form>` inside a `<fieldset>` — browsers ignore the form, submission breaks silently.
**Why it happens:** Adjacency-list composition makes it easy to accidentally wrap `Form` inside `FieldSet` if the handler composes `Container([FieldSet([Form([…])])])`.
**How to avoid:** `Form` is the outermost interactive container. Handlers compose `Container([Heading, Button, Form([FieldSet([…fields]), FieldSeparator, FieldSet([…]), …actions])])`. Document in `spec/PROTOCOL.md`.
**Warning signs:** Submit button does nothing; Playwright E2E `form.onSubmit` never fires.

### Pitfall 6: `crypto.randomUUID()` SSR unsafe

**What goes wrong:** SvelteKit SSR can pre-render a component once at build time, baking the fallback UUID into the static HTML — every subsequent hydration uses a different UUID and breaks `Field.Label for={id}` matching.
**Why it happens:** `crypto.randomUUID()` runs during server render, then again on client hydration.
**How to avoid:** Marionette is an SPA (`@sveltejs/adapter-static` + SPA fallback [VERIFIED: STACK.md]) so components never SSR. A mount-time UUID is safe. Still — document the assumption inline so a future SSR switch flags this.
**Warning signs:** Label clicks don't focus the input; E2E selectors keyed on `for` stop matching.

### Pitfall 7: `shadcnSvelteSearchTool` hangs

**What goes wrong:** The shadcn-svelte MCP search tool never returns. Time wasted waiting.
**Why it happens:** Known issue in the MCP implementation.
**How to avoid:** Use `shadcnSvelteListTool` and `shadcnSvelteGetTool` for MCP access, or WebFetch `https://shadcn-svelte.com/docs/components/{field,textarea,radio-group,switch}` for docs.
**Source:** Project memory `feedback_shadcn_svelte_search_broken.md`

### Pitfall 8: Pre-existing popup browser-test failures

**What goes wrong:** Five popup browser-tests (`ConfirmDialog.browser-test.ts` ×4, `ToastSurface.browser-test.ts` ×1) already fail before Phase 14 starts.
**Why it happens:** Tailwind layout classes (`flex`, `hidden`) are no-ops in the browser-test harness because `src/app.css` isn't imported by `vitest-browser.config.ts`.
**How to avoid:** Don't try to fix these in Phase 14 — they're out of scope. Baseline the suite at phase start so Phase 14's additions aren't blamed for existing failures. Phase 14 browser tests should follow the same harness — if a Field primitive needs theme tokens to render, inline critical styles in the test or (better) follow existing passing tests' approach (rely on structural assertions, not visual ones).
**Source:** `.planning/phases/13-datatable-enhancements/deferred-items.md`

### Pitfall 9: 76-86 pre-existing clippy pedantic warnings in crm-demo

**What goes wrong:** `cargo clippy -p crm-demo -- -D warnings` exits non-zero due to unrelated toolchain drift.
**Why it happens:** Rust 1.93 introduced new pedantic lints that the pre-existing `crm-demo` code triggers.
**How to avoid:** Don't run clippy with `-D warnings` against `crm-demo` as a phase gate. Scope clippy to `marionette` + `marionette-protocol` (both clean). Alternatively, accept the baseline and verify Phase 14's additions don't increase the count.
**Source:** `.planning/phases/13-datatable-enhancements/deferred-items.md` and Phase 12 deferred-items. Blocker logged on STATE.md.

### Pitfall 10: `helperText → description` rename without migrating every call site

**What goes wrong:** TextInput now reads `props.description`; old CRM handlers still emit `props.helperText`; helper text silently disappears.
**Why it happens:** D-B3 rename. Pre-deployment posture = no back-compat alias.
**How to avoid:** Grep all Rust handler code for `helper_text` / `helperText` before landing the rename. [VERIFIED: current `grep` finds zero uses in backend handler code — only the `TextInput.svelte:71` fallback branch and historical planning docs. Safe to rename without side-effects.]
**Warning signs:** Form field description text missing after a CRM deploy.

## Code Examples

Verified patterns from official sources.

### Basic vertical Field (Label + Input + Description + Error)

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/field -->
<Field.Field data-invalid>
  <Field.Label for="email">Email</Field.Label>
  <Input id="email" type="email" aria-invalid />
  <Field.Description>Optional helper text.</Field.Description>
  <Field.Error>Enter a valid email address.</Field.Error>
</Field.Field>
```

### Field.Set with Legend and Field.Group

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/field -->
<Field.Set>
  <Field.Legend>Profile</Field.Legend>
  <Field.Description>This appears on invoices and emails.</Field.Description>
  <Field.Group>
    <Field.Field>
      <Field.Label for="name">Full name</Field.Label>
      <Input id="name" placeholder="Evil Rabbit" />
    </Field.Field>
  </Field.Group>
</Field.Set>
```

### Horizontal Field.Field for action row

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/field -->
<Field.Field orientation="horizontal">
  <Button type="submit">Submit</Button>
  <Button variant="outline" type="button">Cancel</Button>
</Field.Field>
```

### Separator between sibling FieldSets

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/field -->
<Field.Set>…</Field.Set>
<Field.Separator />
<Field.Set>…</Field.Set>
```

### Switch wired to Label

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/switch -->
<div class="flex items-center space-x-2">
  <Switch id="airplane-mode" />
  <Label for="airplane-mode">Airplane Mode</Label>
</div>
```

### RadioGroup with Label per item

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/radio-group -->
<RadioGroup.Root value="option-one">
  <div class="flex items-center space-x-2">
    <RadioGroup.Item value="option-one" id="option-one" />
    <Label for="option-one">Option One</Label>
  </div>
</RadioGroup.Root>
```

### Textarea (minimal)

```svelte
<!-- Source: https://shadcn-svelte.com/docs/components/textarea -->
<script lang="ts">
  import { Textarea } from "$lib/components/ui/textarea/index.js";
</script>
<Textarea placeholder="Type your message here." />
```

### Marionette-style Rust builder for FieldSet (new)

```rust
// Source: backend/crates/marionette/src/builders/standard.rs pattern
#[derive(ComponentBuilder)]
#[component(type = "field-set")]
pub struct FieldSet {
    #[builder(optional)] pub legend: Option<String>,
    #[builder(optional)] pub description: Option<String>,
    #[builder(optional)] pub cols: Option<u8>,
}

// Usage in a handler:
let contact_info = FieldSet::new()
    .id("contact-info-set")
    .legend("Contact Info")
    .children(vec![name_input, email_input, phone_input, title_input])
    .build_tree();
```

### Marionette-style Rust builder for Textarea (new)

```rust
#[derive(ComponentBuilder)]
#[component(type = "textarea")]
pub struct Textarea {
    pub label: String,
    #[builder(optional)] pub placeholder: Option<String>,
    #[builder(optional)] pub rows: Option<u32>,
    #[builder(optional)] pub required: Option<bool>,
    #[builder(optional)] pub disabled: Option<bool>,
    #[builder(optional)] pub description: Option<String>,
    #[builder(optional)] pub full_width: Option<bool>,
}
```

### NodeRenderer unmount-race guard (D-E2)

```svelte
<!-- Current NodeRenderer.svelte (fragile): -->
<script lang="ts">
  let { nodeId, nodes, surface }: { … } = $props();
  let node = $derived(nodes[nodeId]);
  let ResolvedComponent = $derived(node ? getComponent(node.type) : undefined);
</script>

{#if node}
  {#if !node.visible || getData(surface, node.visible)}
    <ErrorBoundary>
      {#if ResolvedComponent}
        <ResolvedComponent
          props={node.props ?? {}}
          bind={node.bind}       <!-- ← destructure compiled as getter; reads node.bind -->
          action={node.action}
          {surface}
        >
          …
        </ResolvedComponent>
      {/if}
    </ErrorBoundary>
  {/if}
{/if}

<!-- Fixed (D-E2 preferred): derive individual locals inside the {#if node} -->
{#if node}
  {@const props = node.props ?? {}}
  {@const bind = node.bind}
  {@const action = node.action}
  {#if !node.visible || getData(surface, node.visible)}
    <ErrorBoundary>
      {#if ResolvedComponent}
        <ResolvedComponent {props} {bind} {action} {surface}>
          …
        </ResolvedComponent>
      {/if}
    </ErrorBoundary>
  {/if}
{/if}
```

The `{@const}` inside the `{#if node}` branch narrows Svelte's reactive graph so the compiled getter never reads off a torn-down `node`. Verify with browser test + targeted E2E replay of the Phase 13 deferred reproduction (type in filter input, blur).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flowbite-svelte form primitives | shadcn-svelte Field family | Phase 10–11 (Foundation / Leaf migration) | Done. Phase 14 finalizes the Field adoption. |
| Ad-hoc `<Label>` + `<Input>` + `<p class="text-xs">` per leaf | `<Field.Field>` + `<Field.Label>` + `<Field.Description>` + `<Field.Error>` | Phase 14 (this phase) | Consistent label/description/error anatomy, `data-invalid` cascade, a11y semantics for free. |
| `<Card.Root>` wrapping each form section | Flat `<Field.Set>` + `<Field.Separator>` | Phase 14 (D-C2) | Less visual weight, matches shadcn Field recipe verbatim. |
| `FormScreen.svelte` mega-component with sections/actions/nodes props | Inline handler composition + `FieldSet` structural SDUI | Phase 14 (D-A1) | Protocol stays adjacency-list-flat; handlers compose explicitly. |
| `props.helperText` | `props.description` | Phase 14 (D-B3) | Aligns with shadcn Field nomenclature. |
| Viewport-based `md:grid-cols-2` | Container-query `@container/field-group` | v2 (deferred) | Sidebar-aware layouts without JS; not needed for v1.1. |

**Deprecated/outdated:**

- `FormScreen.svelte` and `FormScreen.browser-test.ts` — deleted (D-A1).
- `props.helperText` — renamed to `props.description` (D-B3).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `vitest@4.1` (unit) + `vitest-browser-svelte@2.1` (browser, Chromium) + `@playwright/test@1.58` (E2E) |
| Config files | `frontend/vite.config.ts` (unit), `frontend/vitest-browser.config.ts` (browser), `frontend/playwright.config.ts` + `frontend/playwright.e2e.config.ts` (E2E), `backend/Cargo.toml` (Rust) |
| Quick run command (frontend unit) | `cd frontend && npm test` |
| Quick run command (frontend browser) | `cd frontend && npx vitest --config vitest-browser.config.ts --run` |
| Quick run command (backend) | `cd backend && cargo test -p marionette` |
| Typecheck command (frontend) | `cd frontend && npm run check` (svelte-check, not raw tsc) |
| Full suite command | `cd frontend && npm test && npx vitest --config vitest-browser.config.ts --run && npx playwright test && cd ../backend && cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FORM-01 | `Field.Field` wraps `Label + Input + Description + Error` per leaf | browser | `npx vitest --config vitest-browser.config.ts src/lib/components/form/TextInput.browser-test.ts --run` | ✅ (needs new assertions for Field.Field markup) |
| FORM-01 | `data-invalid` attribute present on error | browser | Same TextInput harness, new test case | ✅ (needs expansion) |
| FORM-01 | `aria-invalid` attribute on input when error present | browser | Same TextInput harness | ✅ (needs expansion) |
| FORM-01 | `description` prop renders as `<Field.Description>` | browser | Same TextInput harness, new test case | ✅ (needs expansion) |
| FORM-01 | Label clicks focus the input (correct `for`/`id`) | browser | Same TextInput harness | ✅ (needs expansion) |
| FORM-01 | Works for all input types (text, select, checkbox, textarea, radio, switch) | browser | One browser-test file per leaf | ❌ (Textarea/RadioGroup/Switch) Wave 0 scaffolds |
| FORM-02 | `FieldSet` renders `Field.Set + Field.Legend + Field.Group` | browser | `FieldSet.browser-test.ts` | ❌ Wave 0 |
| FORM-02 | `FieldSet` default renders 1-col mobile / 2-col desktop | browser + visual snapshot | `FieldSet.browser-test.ts` + Playwright `tests/visual/form.spec.ts` | ❌ Wave 0 (browser) + ✅ (visual — extend existing) |
| FORM-02 | `FieldSet.cols=N` forces fixed N columns | browser | `FieldSet.browser-test.ts` | ❌ Wave 0 |
| FORM-02 | Per-field `full_width` spans all columns | browser | Leaf browser-test + FieldSet integration | ❌ Wave 0 |
| FORM-02 | Sibling FieldSets separated by `Field.Separator` | browser + visual | `FieldSet.browser-test.ts` + visual | ❌ Wave 0 |
| D-A1 | `FormScreen.svelte` no longer exists | backend/smoke | `grep -r "FormScreen" frontend/src backend/crates` → zero matches | n/a (grep-based) |
| D-A1 | `FormScreen` is not in registry | unit | Registry unit test | ✅ (if exists) or grep verification |
| D-E1 | `input_type="password"` renders `<input type="password">` | browser | `TextInput.browser-test.ts` (existing Phase-13-07 test) | ✅ (must stay green) |
| D-E2 | `TextInput` blur during parent patch does not throw | browser + E2E | New browser test + Playwright reproduction | ❌ Wave 0 (browser); extend `tests/e2e/contact-edit.spec.ts` |
| D-E3 | `Textarea` renders with placeholder, rows, description, error | browser | `Textarea.browser-test.ts` | ❌ Wave 0 |
| D-E4 | `RadioGroup` renders options, selection, error | browser | `RadioGroup.browser-test.ts` | ❌ Wave 0 |
| D-E4 | `Switch` toggles state, renders label, error | browser | `Switch.browser-test.ts` | ❌ Wave 0 |
| Backend | Every new builder serializes expected JSON | unit | `cd backend && cargo test -p marionette standard` | ✅ (extend `tests` module in `standard.rs`) |
| Integration | contact-edit form submits successfully via WebSocket | E2E | `npx playwright test --config playwright.e2e.config.ts tests/e2e/contact-edit.spec.ts` | ✅ (extend existing) |
| Visual regression | Form screen screenshot matches baseline | visual | `npx playwright test tests/visual/form.spec.ts` | ✅ (update baseline after Field rewrite) |

### Sampling Rate

- **Per task commit:** Quick unit + browser run for the touched component — `npx vitest --config vitest-browser.config.ts <specific-file> --run` + `cargo test -p marionette <specific-test>`.
- **Per wave merge:** Full frontend browser suite + `cargo test -p marionette -p marionette-protocol` + `npm run check`.
- **Phase gate:** Full suite (unit + browser + E2E + visual + backend tests + `npm run check`), plus a Chrome-MCP-driven UAT of the contact-edit form covering: field render, label-click focus, error state (submit empty), grouped layout responsive stack at 375px vs 1024px, action row placement, textarea typing, radio selection, switch toggle, password input type, blur-during-patch no-console-error.

### Wave 0 Gaps

- [ ] `frontend/src/lib/components/form/FieldSet.browser-test.ts` — covers FORM-02 (legend, group, default grid, cols override, separator handling).
- [ ] `frontend/src/lib/components/form/Textarea.browser-test.ts` — covers D-E3.
- [ ] `frontend/src/lib/components/form/RadioGroup.browser-test.ts` — covers D-E4 radio.
- [ ] `frontend/src/lib/components/form/Switch.browser-test.ts` — covers D-E4 switch.
- [ ] (Optional, if explicit node chosen) `frontend/src/lib/components/form/FieldSeparator.browser-test.ts`.
- [ ] (Optional, if explicit node chosen) `frontend/src/lib/components/form/FieldRow.browser-test.ts`.
- [ ] `frontend/tests/e2e/form-blur-race.spec.ts` — Playwright reproduction of the Phase 13 deferred blur race against a page that patches during blur (contact edit form is suitable).
- [ ] `frontend/src/lib/components/core/NodeRenderer.browser-test.ts` — expand to test unmount race guard (or add inline to an existing file).
- [ ] Expand `TextInput.browser-test.ts` / `SelectInput.browser-test.ts` / `Checkbox.browser-test.ts` with Field anatomy assertions (`Field.Field` wrapper present, `data-invalid` toggles with error, `description` rendered).
- [ ] Extend `backend/crates/marionette/src/builders/standard.rs` `#[cfg(test)] mod tests` with serialization tests for `FieldSet`, `Textarea`, `RadioGroup`, `Switch`, and the new `description`/`full_width` optional props.
- [ ] Update `frontend/tests/visual/form.spec.ts` baselines after the Field rewrite. The visual test already exists [VERIFIED: `.planning/codebase/TESTING.md`] — baselines regenerated with `--update-snapshots`, then reviewed.

Framework install: not required (all Vitest/Playwright/Cargo infrastructure is already in place).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `npx` (from Node) | Running `shadcn-svelte@latest add` | ✓ | Node 25+ | — |
| `shadcn-svelte` CLI | Primitive code-gen (`add field textarea radio-group switch`) | ✓ | 1.2.7 (on-demand via npx) | — |
| `bits-ui` | Peer dep for RadioGroup, Switch, Label | ✓ | 2.17.3 | — |
| Chromium (Playwright) | Browser tests + E2E + visual | ✓ (installed via `playwright` dep) | — | — |
| Rust toolchain | Backend builders + tests | ✓ | 1.93+ | — |
| Internet access (one-shot, during primitive install) | Fetch shadcn component source | ✓ | — | Have someone with access run the commands and commit the generated files. |
| Chrome (for Chrome-MCP UAT) | Human-verify via claude-in-chrome | ✓ | — | Fall back to manual UAT only if MCP browser is unavailable. |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** none.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | indirect | TextInput `input_type=password` must render `<input type="password">` (D-E1). Unchanged from baseline; verified by browser test. No password handling logic in Phase 14. |
| V3 Session Management | no | No session touchpoints. |
| V4 Access Control | no | Form components render whatever the backend sends; access control is a handler concern, not a component concern. |
| V5 Input Validation | yes | Per-field validation via `/_errors/{bind}` (server-side). Phase 14 preserves this — the `<Field.Error>` pattern is purely presentational. Form-level errors via `/_errors/{form_bind}` array. [VERIFIED: code already in `TextInput.svelte` line 25-27 and `Form.svelte` line 21-23.] |
| V6 Cryptography | no | No crypto. `crypto.randomUUID()` is used only as a fallback HTML id (not a secret). |
| V14 Config | yes | New SDUI components add new protocol types. The spec's `component.yaml` already uses `additionalProperties: true` on `Component.props`, so new props land permissively — but `spec/schemas/data.yaml` SHOULD gain structured descriptors for `FieldSet`, `Textarea`, etc. (same pattern Phase 13 used for `DataTable`). No breaking change. |

### Known Threat Patterns for Svelte + SDUI stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via unescaped `{@html}` in description/error/legend | Tampering | Do not use `{@html}` anywhere in the new primitives. Svelte auto-escapes `{expression}` interpolations. [VERIFIED: existing TextInput/SelectInput/Form use `{...}` interpolation, never `{@html}`.] |
| Form spoofing (submit to unintended action) | Spoofing | `Form.svelte` dispatches only via `sendAction(action.name, …)` where `action` is a protocol-provided `ComponentAction`. Handler-authoritative. |
| Input value tampering via DOM | Tampering | Server revalidates on submit; `/_errors/{bind}` is the canonical validation surface. Client-side validation is not trusted. |
| Console-error information disclosure via unmount race | Information Disclosure | The `NodeRenderer.get bind` error is a benign `TypeError` caught by `ErrorBoundary`; no sensitive data leaks. D-E2 fix suppresses it regardless. |
| Third-party registry supply-chain risk (npx shadcn-svelte) | Tampering | `shadcn-svelte@latest` is resolved from the public npm registry (huntabyte-maintained). Generated source lands in git and is reviewable at PR time. No runtime dependency on the CLI after install. |

Phase 14 adds no new authentication, authorization, session, or crypto surface. Security posture is unchanged from Phase 13.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | [ASSUMED] `shadcn-svelte@latest add` command emits components compatible with the existing Tailwind v4 theme tokens (`text-destructive`, `border-destructive`, `text-muted-foreground`, `bg-background`). | Standard Stack / Installation | Low. If class tokens drift, the shadcn init would already have caught it in Phase 10. Verified indirectly by the fact that every other shadcn primitive (`input`, `select`, `checkbox`, `card`, `separator`) works correctly in the codebase. Worst case: tweak classes post-install. |
| A2 | [ASSUMED] `@container/field-group` container queries are supported in the user's target browsers (recent Chrome/Firefox/Safari). | Architecture (D-C3 alternative) | Very low — container queries are CSS-spec-stable; Phase 14 doesn't use them anyway (deferred to v2). |
| A3 | [ASSUMED] `Field.Field` with `orientation="horizontal"` for the action row renders Save/Cancel right-aligned or aligned-to-content by default. Exact class utility string needed inside the Field.Field to match the existing `FormScreen.svelte` visual (right-aligned, gap-2). | Code Examples / Pattern 2 | Low. If the default orientation doesn't right-align, add `class="justify-end"` or a wrapping Container. Verify in first browser test. |
| A4 | [ASSUMED] `RadioGroup.Item` accepts a `description` slot or prop for per-option descriptions (for RadioGroup with `options: Vec<{value, label, description?}>` per CONTEXT.md D-E4). | Rust builder (RadioGroup) | Medium — shadcn RadioGroup docs didn't verify per-item descriptions. Fallback: render description as plain Muted text next to the label, or drop `description` from `RadioOption` for v1.1. Planner should check via `mcp__shadcn-svelte__shadcnSvelteGetTool` for `radio-group` during plan or wave 0. |
| A5 | [ASSUMED] Retaining `Field.Group` as the descendant of `FieldSet` won't visually double-indent when the enclosing `Form.svelte` is also tweaked to wrap children in a `Field.Group` (per CONTEXT.md D-A3). | Architecture | Low. If Form.svelte wraps children in `Field.Group` and each FieldSet is itself a `Field.Group`, there may be redundant gaps. Easy fix: keep Form.svelte simple and let FieldSets own grouping. Planner should reconcile in the Form.svelte rewrite task. |
| A6 | [ASSUMED] Existing `frontend/tests/visual/form.spec.ts` Playwright baselines will fail after the Field rewrite and need an `--update-snapshots` pass with human review. Visual diff is expected and acceptable. | Validation Architecture | Low — standard behavior. Planner includes a baseline-refresh task in the final wave. |
| A7 | [ASSUMED] The `Field.Separator` primitive accepts no children and renders a thin horizontal rule styled to match the existing `Separator` in `ui/separator/`. If it's a distinct visual token in the Field recipe, CSS fallbacks may differ. | Architecture (D-C2) | Low. Both look identical in the shadcn recipe screenshots; can adjust class overrides after install. |
| A8 | [ASSUMED] The `description` prop (for Textarea/RadioGroup/Switch/FieldSet) is optional at the backend — omission produces no `Field.Description` element in the DOM. Matches existing `helperText` behavior. | Architecture / Patterns | None — explicit in D-B3. |

**User confirmation needed before planning locks:** none of the above are critical enough to block planning. A4 is the most likely to need a post-install verification task. The rest are ergonomic details the planner resolves in Wave 0 or early task implementations.

## Open Questions

1. **Explicit `FieldSeparator` node vs. auto-insertion in `Form.svelte`.**
   - What we know: CONTEXT.md D-C2 expresses a preference for explicit nodes. Auto-insertion is the "magic" alternative.
   - What's unclear: whether the planner finds the explicit-node path's verbosity acceptable (every handler writes `FieldSeparator::new().build()` between sibling FieldSets).
   - Recommendation: **go explicit**. Matches Phase 14's "self-describing protocol" posture and keeps node-patching granular. Add a shorthand Rust helper `FieldSet::with_separator()` if verbosity becomes a problem.

2. **Action row: plain `Container` vs. dedicated `FieldRow`/`field-row` SDUI component.**
   - What we know: D-D1 is explicitly Claude's discretion. Both pass the visual target.
   - What's unclear: whether the `Field.Field orientation="horizontal"` gives us enough styling wins over `<Container class="flex gap-2 justify-end">` to justify a new component type.
   - Recommendation: **plain `Container`** for Phase 14. It's zero-new-surface, handlers already use `Container` for everything. Promote to `field-row` in Phase 15 if a real pattern emerges across ≥3 CRM screens.

3. **NodeRenderer unmount-race fix: `{@const}` inside `{#if node}` vs. fully restructured `$derived` chain.**
   - What we know: The bug is a compiled getter reading `node.bind` on a torn-down node. Preferred fix is structural (D-E2 preference b).
   - What's unclear: whether `{@const}` captures a stable snapshot or is re-evaluated on every dependency tick (Svelte 5 semantics — need to verify via Svelte MCP or official docs). An alternative is a guarded `$derived.by(() => node ? { props, bind, action } : null)`.
   - Recommendation: prototype both in Wave 0; keep the one whose browser-test repro is green and doesn't rerender more than needed. Svelte MCP's `suggest-svelte-code` tool would be a good verification step.

4. **Textarea `rows` prop — passthrough to `<textarea rows={n}>` or use min-height utility class?**
   - What we know: shadcn `Textarea` docs don't spell out whether `rows` is supported directly.
   - What's unclear: whether the shadcn Textarea wrapper forwards arbitrary HTML attributes.
   - Recommendation: verify during installation — `rows` is a native `<textarea>` attribute, shadcn primitives almost always forward `restProps` (as `Input` does). If not forwarded, use `min-h-[{rows * 1.5}rem]` equivalent.

5. **`FieldSet` legend vs. description typography.**
   - What we know: shadcn Field docs show both can appear in a FieldSet.
   - What's unclear: whether they should both be used in the same FieldSet or one-or-the-other.
   - Recommendation: allow both in the Rust builder; document in the backend builder docstring that `legend` is the section title and `description` is optional sub-text.

## Sources

### Primary (HIGH confidence)

- [`shadcn-svelte` — Field component](https://shadcn-svelte.com/docs/components/field) — anatomy, orientation variants, validation pattern (`data-invalid` + `aria-invalid`), Field.Field/Label/Description/Error, Field.Set/Legend/Group/Separator/Content. Fetched 2026-04-17.
- [`shadcn-svelte` — Textarea](https://shadcn-svelte.com/docs/components/textarea) — install command, minimal usage.
- [`shadcn-svelte` — RadioGroup](https://shadcn-svelte.com/docs/components/radio-group) — install, Root/Item pattern, Label pairing.
- [`shadcn-svelte` — Switch](https://shadcn-svelte.com/docs/components/switch) — install, Label pairing, bits-ui base.
- `backend/crates/marionette/src/builders/standard.rs` — canonical builder macro pattern (`ComponentBuilder`), existing TextInput/Select/Checkbox/Form struct definitions, `#[cfg(test)] mod tests` patterns.
- `frontend/src/lib/components/form/*.svelte` (existing leaves) — current SDUI contract and error-store convention.
- `frontend/src/lib/components/core/NodeRenderer.svelte` — the prop-destructuring pattern that causes the blur race.
- `frontend/src/lib/components/screen/FormScreen.svelte` — the orphan to retire; also the reference implementation for the dynamic-grid-cols inline-style workaround.
- `.planning/phases/14-formscreen-enhancements/14-CONTEXT.md` — all locked decisions.
- `.planning/REQUIREMENTS.md` — FORM-01, FORM-02, §Out of Scope (Superforms/Formsnap).
- `.planning/codebase/CONVENTIONS.md` — naming, formatting, imports, Svelte 5 runes, error handling, comment style.
- `.planning/codebase/STACK.md` — verified versions, SPA mode (adapter-static), npm/cargo toolchain.
- `.planning/codebase/TESTING.md` — vitest + vitest-browser-svelte + Playwright patterns.
- `.planning/research/PITFALLS.md` — #12 dynamic Tailwind classes, #7 screen component pattern, #11 toast, #5 CSS conflicts (resolved).
- `.planning/phases/13-datatable-enhancements/deferred-items.md` — §NodeRenderer `get bind` undefined (D-E2 source).
- `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` — §TextInput `input_type` (D-E1 source).
- `.mcp.json` — registered MCP servers (`svelte`, `shadcn-svelte`, `rust-docs`).

### Secondary (MEDIUM confidence)

- [`ui.shadcn.com` — Field (React/Radix port)](https://ui.shadcn.com/docs/components/radix/field) — used to cross-verify the full Field anatomy (FieldContent, FieldTitle, "label" vs "legend" FieldLegend variants). Translation between React and Svelte versions is 1:1 for props and structure.
- [Medium — "Shadcn/ui React Series Part 32: Field"](https://medium.com/@rivainasution/shadcn-ui-react-series-part-32-field-structuring-form-intent-cdd7917feac6) — Feb 2026 article confirming `orientation="responsive"` with `@container/field-group` class. Third-party; used for corroboration only.
- [shadcn.io — Responsive Layout patterns](https://www.shadcn.io/patterns/field-layouts-3) — shows `orientation="responsive"` is viewport/breakpoint-agnostic via container queries.

### Tertiary (LOW confidence)

- WebSearch results for "shadcn-svelte Field component import namespace 2026 example" — used only to surface corroborating sources; each finding was then verified against the primary docs.

## Metadata

**Confidence breakdown:**

- User Constraints / Phase Requirements: HIGH — verbatim from CONTEXT.md and REQUIREMENTS.md.
- Standard Stack: HIGH — every package version verified against `package.json` or `npm view`.
- Architecture patterns: HIGH — Field anatomy corroborated across shadcn-svelte docs + shadcn/ui docs + third-party examples; SDUI contract already proven in Phase 11.
- Don't Hand-Roll list: HIGH — every entry cross-references either official docs or the project memory.
- Common Pitfalls: HIGH — #1 from `.planning/research/PITFALLS.md`, #2 and #3 from Phase 12/13 deferred-items, #4 from official shadcn docs (validation section), #10 from direct grep verification.
- Code Examples: HIGH — verbatim from shadcn-svelte.com/docs/components/* pages.
- State of the Art: HIGH — tied to Phase 10/11/12/13 history in-repo.
- Validation Architecture: HIGH — framework names/versions verified against `frontend/package.json` + `.planning/codebase/TESTING.md`.
- Security Domain: MEDIUM-HIGH — straightforward analysis; Phase 14 doesn't add new security surface, and all five ASVS categories were evaluated.
- Assumptions Log: 8 items, all low-risk and flagged for planner attention on A4 (RadioGroup per-option description).

**Research date:** 2026-04-17
**Valid until:** 2026-05-17 (30 days) — shadcn-svelte minor releases are stable; if the Field component's API changed between now and the planning session, re-fetch https://shadcn-svelte.com/docs/components/field and diff.

Sources:
- [shadcn-svelte Field docs](https://shadcn-svelte.com/docs/components/field)
- [shadcn-svelte Textarea docs](https://shadcn-svelte.com/docs/components/textarea)
- [shadcn-svelte RadioGroup docs](https://shadcn-svelte.com/docs/components/radio-group)
- [shadcn-svelte Switch docs](https://shadcn-svelte.com/docs/components/switch)
- [shadcn/ui Field (React reference)](https://ui.shadcn.com/docs/components/radix/field)
- [shadcn.io responsive Field layouts](https://www.shadcn.io/patterns/field-layouts-3)
