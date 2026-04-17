# Phase 14: FormScreen Enhancements - Context

**Gathered:** 2026-04-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 14 gives Marionette forms consistent shadcn-svelte `Field.*` styling (label + description + error), semantic `Field.Set` grouping with visual separators, and an auto-responsive default layout (1 column on mobile, 2 columns on desktop). Delivers FORM-01 + FORM-02.

Concretely, Phase 14:

1. **Retires the orphan `FormScreen.svelte`** (hard delete — mirrors Phase 13's TableScreen retirement). The file exists but is not registered in `defaults.ts`, has no backend builder, and is not used by any CRM handler.
2. **Wraps each leaf form component internally with `Field.Field`** — `TextInput`, `SelectInput`, `Checkbox`, plus three new primitives (`Textarea`, `RadioGroup`, `Switch`) — so each renders `<Field.Field data-invalid><Field.Label><Input aria-invalid><Field.Description><Field.Error></Field.Field>` as a single protocol node. Keeps the existing `/_errors/{bind}` data-store convention for validation state.
3. **Adds a new first-class `FieldSet` SDUI component** (`field-set`) that renders `<Field.Set><Field.Legend>…<Field.Group>{children}</Field.Group></Field.Set>`, with an auto-responsive CSS grid default (1-col mobile / 2-col desktop) and overrides (`cols`, `full_width`). Consecutive sibling `FieldSet`s are visually separated by `Field.Separator`.
4. **Keeps `Form.svelte` as the `<form>` boundary** — still emits native `<form>` submit, still renders form-level errors from `/_errors/{form_bind}` as a banner above children.
5. **Introduces an action-row pattern** for save/cancel buttons using the shadcn Field recipe (`Field.Field orientation="horizontal"` wrapping the buttons). Exact shape (new SDUI component vs. plain `Container` with utility classes) is Claude's discretion during planning.
6. **Closes two carried-over bugs**: TextInput's `props.type` vs `props.input_type` mismatch (password fields currently render as text) and the `NodeRenderer.bind undefined` console noise when a TextInput unmounts mid-blur.

**What this phase is NOT:**

- NOT wizard / multi-step forms (FORM-03, v2).
- NOT full-width / arbitrary column-span layouts beyond the `cols` + `full_width` knobs (FORM-04, v2).
- NOT `FormScreen` as a first-class SDUI component (explicitly retired — handlers compose inline).
- NOT a DRY helper for the `Heading + back Button` title region (handlers compose it inline; Phase 15 can revisit if the repetition becomes painful).
- NOT Superforms / Formsnap / client-side Zod validation (server-side validation via `/_errors` stays canonical — `REQUIREMENTS.md §Out of Scope`).
- NOT CRM screen-by-screen migration to the new shapes — Phase 15 owns that. Phase 14 migrates the minimum needed to exercise each new primitive end-to-end and to prove the `input_type` fix + blur fix on a real screen.
- NOT persistence of any form state across reloads.
- NOT form-level layout presets (card-wrapped sections, multi-step, etc.).

</domain>

<decisions>
## Implementation Decisions

### Area A — FormScreen disposition

- **D-A1: Retire `FormScreen.svelte` (hard delete).** Delete `frontend/src/lib/components/screen/FormScreen.svelte` and `FormScreen.browser-test.ts`. Not registered in `defaults.ts`, no backend builder, no CRM call sites — same orphan situation Phase 13 resolved by deleting `TableScreen.svelte` (13-CONTEXT.md D-A2). No `@deprecated` tombstone — the pre-deployment posture rejects back-compat shims.
- **D-A2: Handlers compose title + back-button region inline; no DRY helper.** Each CRM form handler builds `Container([Heading("Edit Contact"), Button(ArrowLeft, back_action), …FieldSets, …action row])` explicitly. Matches how list handlers work post-Phase 13. Phase 15 can introduce a Rust-side helper if the repetition becomes painful across ≥3 screens.
- **D-A3: Keep `Form.svelte` as the `<form>` boundary.** Stays registered (`'form'`). Responsibilities unchanged at the protocol level: emit `<form>`, trap `onsubmit`, dispatch `action` as an `ActionMessage`, render form-level errors from `/_errors/{form_bind}` as a banner. Phase 14 may tweak class wrapping (e.g., wrap children in a `Field.Group` for consistent spacing) but does not change its API surface.

### Area B — Field integration strategy (per-leaf-component internal wrap)

- **D-B1: Internal `Field.Field` wrap per leaf component.** `TextInput.svelte`, `SelectInput.svelte`, `Checkbox.svelte`, plus the three new primitives (`Textarea`, `RadioGroup`, `Switch`) each render their own `<Field.Field data-invalid={!!err}><Field.Label for={id}>{label}</Field.Label><Input id={id} aria-invalid={!!err} …/><Field.Description>{props.description}</Field.Description><Field.Error>{err}</Field.Error></Field.Field>`. One protocol node per field. Matches Phase 11 D-01 ("pass-through with styling") and keeps handler ergonomics identical (`TextInput::new(label).bind(path).build()`).
- **D-B2: Keep `/_errors/{bind}` convention for error data flow.** Leaf components read `getData(surface, '/_errors' + bind)` (already present in `TextInput.svelte:26`). When non-empty: render `<Field.Error>`, add `data-invalid` to the `Field.Field` wrapper, add `aria-invalid` to the input. No protocol change — the data-store path convention Phase 11 uses stays canonical. Form-level errors (`/_errors/{form_bind}` as an array) continue to render as a banner in `Form.svelte`.
- **D-B3: Backend builders gain `.description(…)` helper on every field primitive.** `TextInput`, `SelectInput`, `Checkbox`, `Textarea`, `RadioGroup`, `Switch` each get `description: Option<String>` (or equivalent) in their `#[derive(ComponentBuilder)]` struct + a fluent helper. Maps to `props.description`. Enables the "helper text under field" UX shadcn Field renders via `<Field.Description>`. Existing `helperText` prop in TextInput (`TextInput.svelte:71`) renames to `description` to match shadcn nomenclature — pre-deployment posture, no back-compat alias.
- **D-B4: Field `id` generated from a stable source.** `Field.Label for={id}` and `<Input id={id}>` must match for accessibility. Use the component's protocol `id` (set by backend builder via `.id(...)`) as the HTML `id`. If absent, leaf component falls back to `crypto.randomUUID()` at mount. Documented so handler authors know to set `.id(...)` on every form field for best a11y and for Playwright selectors.

### Area C — FieldSet component + responsive layout

- **D-C1: New `FieldSet` SDUI component (`field-set`).** Registered in `frontend/src/lib/registry/defaults.ts` as `'field-set'`. Backend builder in `backend/crates/marionette/src/builders/standard.rs`:
  ```rust
  #[derive(ComponentBuilder)]
  #[component(type = "field-set")]
  pub struct FieldSet {
      #[builder(optional)] pub legend: Option<String>,
      #[builder(optional)] pub description: Option<String>,
      #[builder(optional)] pub cols: Option<u8>,   // force-override; default auto
  }
  ```
  Renders `<Field.Set><Field.Legend>{legend}</Field.Legend><Field.Description>{description}</Field.Description><Field.Group class="{gridClasses}">{children}</Field.Group></Field.Set>`. Children are resolved through `NodeRenderer` (adjacency-list), same as every other structural component.
- **D-C2: Flat visual style — `Field.Set` + `Field.Separator`, no `Card.Root` wrapping.** Follows the shadcn Field recipe visually. No shadowed cards. Between consecutive sibling `FieldSet`s, the handler composes a plain `Field.Separator` SDUI node (new, thin — renders `<Field.Separator />`) OR the parent `Form.svelte` inserts them automatically between direct-child `field-set` nodes — planner decides which is cleaner. Preference: explicit nodes in the adjacency list so the protocol is self-describing and node-patching stays granular.
- **D-C3: Default auto-responsive grid — 1-col mobile, 2-col desktop.** `FieldSet` with no `cols` prop renders `<Field.Group class="grid grid-cols-1 md:grid-cols-2 gap-4">`. On viewports `<768px`, children stack; on `≥768px`, they flow in 2 columns. Zero handler config required. Defers to Phase 15 / v2 whether to switch to container queries (`@container/field-group`) for sidebar-aware layouts; viewport-based `md:` is good enough for v1.1.
- **D-C4: Two overrides — `FieldSet.cols` and per-field `full_width`.** When `FieldSet.cols` is set, it replaces the auto-responsive pattern with `grid-cols-{cols}` (no breakpoint stack). Each field primitive (`TextInput`, `SelectInput`, etc.) gains an optional `full_width: bool` prop — when `true`, the field's `Field.Field` gets `col-span-full` and spans the entire row. Covers both "force single column" (FieldSet.cols=1) and "this particular field is a long textarea" (Textarea.full_width=true) without over-engineering responsive rules.

### Area D — Action row pattern

- **D-D1: Recipe pattern for save/cancel — horizontal `Field.Field`.** Buttons at the end of a form live in a horizontal row matching the shadcn Field recipe's final example. Concretely: either (a) a plain `Container` with `class="flex gap-2 justify-end"` and Button children, or (b) a small dedicated SDUI component (e.g., `field-row`) wrapping `<Field.Field orientation="horizontal">{children}</Field.Field>`. **Claude's discretion** during planning — both reach the same visual outcome. No dedicated `ActionBar` with top-border (the old FormScreen pattern is retired). No protocol-level mandate about where actions must go; handlers include the action row as the last child of the form.

### Area E — Scope: deferred fixes + new primitives

- **D-E1: Fix TextInput `input_type`/`type` prop mismatch.** `TextInput.svelte:59` currently reads `props.input_type ?? 'text'` but historically read `props.type` and the Phase 12 commit drifted. Aligns to `props.input_type` (the backend-serialized key from `builders::standard::TextInput.input_type`). Verify login password field renders `<input type="password">` after the fix via an E2E check or browser test. Phase 12 deferred-items (§"TextInput `input_type` -> `type` prop mismatch") explicitly assigns this to Phase 13 or Phase 14; Phase 13 didn't take it. (Phase 13 13-07 SUMMARY in fact addressed it — verify the fix survived into Phase 14 and there's no regression.)
- **D-E2: Fix NodeRenderer handleBlur unmount race.** `TextInput.svelte:43 handleBlur` triggers `clearDirty → setData → tree re-render → NodeRenderer children unmount → destructured `bind` accessor reads `node.bind` on undefined → `TypeError: Cannot read properties of undefined (reading 'bind')`. Caught by `ErrorBoundary` at render-time, user-invisible, but noisy in the console. Guard at one of: (a) `TextInput.handleBlur` — no-op when `bind === undefined` or the surface is mid-patch, (b) `NodeRenderer.svelte:15` — bail from the `$derived` when `node` is `undefined` OR destructure props only inside the `{#if node}` branch so Svelte's compiler generates safer accessors. Planner picks — leaf preference is (b) since it's more structural and avoids fragility in every form component's `handleBlur`. See Phase 13 deferred-items §"NodeRenderer `get bind` undefined on TextInput blur".
- **D-E3: Install Textarea primitive + SDUI wrapper.** Run `npx shadcn-svelte@latest add textarea` to install the primitive under `frontend/src/lib/components/ui/textarea/`. New SDUI component `textarea` in `components/form/Textarea.svelte` — same internal Field.Field wrap as TextInput (D-B1). Backend builder `Textarea` in `builders/standard.rs` with `placeholder`, `rows`, `required`, `disabled`, `description`, `label`, plus `full_width`. Registers in `defaults.ts` as `'textarea'`. Enables long-text form fields for Phase 15 CRM migration (contact notes, etc.).
- **D-E4: Install RadioGroup + Switch primitives + SDUI wrappers.** Run `npx shadcn-svelte@latest add radio-group switch`. Two new SDUI components: `radio-group` and `switch`. Both follow D-B1 (internal Field wrap). Backend builders: `RadioGroup` (with `options: Vec<{value, label, description?}>`) and `Switch` (boolean toggle, mirroring Checkbox ergonomics). Enables the Field recipe's full range so Phase 15 + future screens can reach for the right primitive without requesting one mid-milestone.
- **D-E5: Claude's Discretion.** Within Phase 14:
  - Exact shape of the action row (`Container` with class vs. new `field-row` SDUI component).
  - Whether `Field.Separator` between sibling `FieldSet`s is an explicit adjacency-list node or auto-inserted by `Form.svelte` / parent. Preference: explicit node (see D-C2 rationale).
  - Field `id` fallback strategy (UUID vs derived-from-bind) when handler forgot `.id(...)` — D-B4.
  - Blur-race fix location (TextInput vs NodeRenderer) — D-E2 notes preference for NodeRenderer.
  - Per-component migration order within the phase (form leaves first, then FieldSet, then new primitives, then CRM smoke migration — or interleaved).
  - Specific class utility strings for the auto-responsive grid (e.g., `gap-4` vs `gap-6`; `md:grid-cols-2` vs container queries) as long as D-C3's behavior holds.
  - Test granularity per component (browser tests for every leaf vs. a handful of representative ones).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project state and prior phases

- `.planning/REQUIREMENTS.md` §FormScreen — FORM-01 (shadcn Field), FORM-02 (card/set sections); §v2 — FORM-03, FORM-04 deferrals.
- `.planning/PROJECT.md` — v1.1 milestone goal; "Form component: structured layout with action buttons, special widgets" is what Phase 14 delivers.
- `.planning/ROADMAP.md` Phase 14 — goal, depends-on (Phase 12), success criteria.
- `.planning/phases/11-leaf-component-migration/11-CONTEXT.md` — Phase 11 leaf-component decisions (D-01 pass-through, D-03 compose from shadcn parts) that Phase 14 inherits.
- `.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md` — node-patching + surface-store semantics that make per-field Field wrapping safe under live patching.
- `.planning/phases/12-protocol-node-patching-appshell/deferred-items.md` §"TextInput `input_type` -> `type` prop mismatch" — D-E1 source.
- `.planning/phases/13-datatable-enhancements/13-CONTEXT.md` D-A2 — the TableScreen retirement precedent applied to FormScreen here (D-A1).
- `.planning/phases/13-datatable-enhancements/deferred-items.md` §"NodeRenderer `get bind` undefined on TextInput blur" — D-E2 source.

### Protocol specs that Phase 14 mutates

- `spec/PROTOCOL.md` — component type registry. Add `field-set`, `textarea`, `radio-group`, `switch` (and possibly `field-row`, `field-separator`) with their props and children semantics.
- `spec/schemas/data.yaml` — component schema. Add schema entries for the new component types and for new optional props on existing ones (`description` on field components, `full_width`).
- `spec/openapi.yaml` — verify if regeneration is needed after schema additions.
- `spec/schemas/message.yaml` — no changes expected (form submit uses existing `ActionMessage`; field-level errors flow via existing data patches).

### Frontend code that Phase 14 rewrites, adds, or deletes

- **Rewritten** (internal Field.Field wrap per D-B1):
  - `frontend/src/lib/components/form/TextInput.svelte`
  - `frontend/src/lib/components/form/SelectInput.svelte`
  - `frontend/src/lib/components/form/Checkbox.svelte`
  - `frontend/src/lib/components/form/Form.svelte` (minor — wrap children in `Field.Group`, keep rest)
- **New** (per D-C1 and D-E3/D-E4):
  - `frontend/src/lib/components/form/FieldSet.svelte`
  - `frontend/src/lib/components/form/Textarea.svelte`
  - `frontend/src/lib/components/form/RadioGroup.svelte`
  - `frontend/src/lib/components/form/Switch.svelte`
  - `frontend/src/lib/components/form/FieldSeparator.svelte` (if D-C2 preference is kept)
  - Possibly `frontend/src/lib/components/form/FieldRow.svelte` (per D-D1 discretion)
  - `frontend/src/lib/components/ui/field/*` (from `shadcn-svelte add field`)
  - `frontend/src/lib/components/ui/textarea/*` (from `shadcn-svelte add textarea`)
  - `frontend/src/lib/components/ui/radio-group/*` (from `shadcn-svelte add radio-group`)
  - `frontend/src/lib/components/ui/switch/*` (from `shadcn-svelte add switch`)
- **Deleted** (per D-A1):
  - `frontend/src/lib/components/screen/FormScreen.svelte`
  - `frontend/src/lib/components/screen/FormScreen.browser-test.ts`
- **Edited** (per D-E2 and new registrations):
  - `frontend/src/lib/components/core/NodeRenderer.svelte` — guard unmount race.
  - `frontend/src/lib/registry/defaults.ts` — register new components, de-register nothing (FormScreen was never registered).

### Backend code that Phase 14 extends

- `backend/crates/marionette/src/builders/standard.rs` — new structs (`FieldSet`, `Textarea`, `RadioGroup`, `Switch`, possibly `FieldRow`, `FieldSeparator`) using `#[derive(ComponentBuilder)]`; extend existing `TextInput`, `Select`, `Checkbox`, `Form` with `description` and/or `full_width` as appropriate.
- `backend/crates/crm-demo/src/handlers/contact.rs` — migrate contact edit form to the new shape (the canonical exercise — already the most complex form handler). Cover all new primitives at least once (textarea for notes? radio-group or switch if a natural fit; otherwise leave those exercised by a dedicated demo screen).
- `backend/crates/crm-demo/src/handlers/user.rs` — small form; migrate opportunistically.
- `backend/crates/crm-demo/src/handlers/company.rs` — form migration.
- Other handlers with inline forms (tag-add, note-add in `contact.rs`) — Phase 14 migrates only what's needed to exercise the new primitives end-to-end; Phase 15 does the sweep.

### External library docs (research-phase reading)

- https://shadcn-svelte.com/docs/components/field — Field.Field / Field.Label / Field.Description / Field.Error / Field.Set / Field.Legend / Field.Group / Field.Separator anatomy, accessibility notes, orientation variants, validation pattern (`data-invalid` + `aria-invalid`).
- https://shadcn-svelte.com/docs/components/textarea — textarea primitive.
- https://shadcn-svelte.com/docs/components/radio-group — radio group primitive.
- https://shadcn-svelte.com/docs/components/switch — switch primitive.
- https://shadcn-svelte.com/docs/dark-mode — not Phase 14 scope but relevant context for class utilities.

### Codebase intel

- `.planning/codebase/CONVENTIONS.md` — Svelte 5 component patterns, tabs-for-indent, 100-char line width.
- `.planning/codebase/STACK.md` — current stack (shadcn-svelte CLI installed, Tailwind v4).
- `.planning/codebase/TESTING.md` — browser-test patterns.
- `.planning/research/PITFALLS.md` — known Flowbite-to-shadcn pitfalls (may carry forward to Field adoption).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`frontend/src/lib/components/ui/{input,select,checkbox,label,button,card,separator}/`** — all shadcn primitives Phase 14 composes are already installed (Phase 10/11). Phase 14 only adds `field`, `textarea`, `radio-group`, `switch`.
- **`frontend/src/lib/components/form/{TextInput,SelectInput,Checkbox,Form,Button}.svelte`** — current leaf components preserve the SDUI interface contract (`surface`, `props`, `bind?`, `action?`). Phase 14 rewrites their render output but keeps the contract.
- **`frontend/src/lib/store/data.svelte.ts` `getData(surface, bind)`** — how leaf components read their value AND their error from `/_errors/{bind}`. Unchanged.
- **`frontend/src/lib/store/dirty.svelte.ts` `markDirty` / `clearDirty`** — dirty-tracking hooks fired on focus/blur. Unchanged.
- **`frontend/src/lib/transport/dispatcher.ts` `sendAction`** — action dispatch (already returns action id post-Phase 13). Unchanged.
- **`backend/crates/marionette/src/builders/standard.rs` `#[derive(ComponentBuilder)]`** — the macro pattern every new builder (`FieldSet`, `Textarea`, etc.) reuses verbatim. `Container`, `Grid`, `Form`, `Heading`, `Button` call sites illustrate the fluent builder style.
- **`frontend/src/lib/components/core/NodeRenderer.svelte`** — recursive adjacency-list renderer. Unchanged by Phase 14's protocol additions (new component types just resolve via the registry); only the blur-race guard (D-E2) touches it.

### Established Patterns

- **Adjacency-list composition** — FieldSet follows the same pattern as Container / AppShell: `props` for layout knobs, `children` (node IDs) for contents. No special protocol machinery.
- **Internal wrap with SDUI contract preserved** — Phase 11 D-01 sets the precedent; Phase 14's TextInput/SelectInput/Checkbox rewrites follow the same shape (leaf component does its own shadcn composition; backend props are unchanged or only additively extended).
- **Node-patch granularity** — Phase 12's surface store preserves input focus across patches to sibling nodes (D-A6). FieldSet is a plain container in that model, so a patch that replaces one field's node doesn't unmount its siblings.
- **Form-level errors via data store** — `/_errors/{form_bind}` renders as a banner in `Form.svelte`; per-field errors via `/_errors/{field_bind}` render as `Field.Error` in each leaf component. Same backend pattern, different render targets.
- **Pre-deployment posture** — no back-compat shims for the `helperText → description` rename (D-B3), for the deletion of FormScreen (D-A1), or for the `props.type` → `props.input_type` alignment (D-E1). Consistent with 13-CONTEXT and project memory.

### Integration Points

- **`frontend/src/lib/registry/defaults.ts`** — register `'field-set'`, `'textarea'`, `'radio-group'`, `'switch'` (and any of `'field-row'`, `'field-separator'` the planner chooses to add).
- **`spec/schemas/data.yaml`** — add schema for each new component's props; add `description` (string) and `full_width` (bool) to existing form-field components; align `input_type` vs `type` doc.
- **CRM handlers with forms** — `backend/crates/crm-demo/src/handlers/{contact,user,company}.rs` and the tag/note inline forms inside `contact.rs`. Phase 14 migrates the contact edit form (canonical exercise) + one demo surface that exercises Textarea/RadioGroup/Switch. Phase 15 sweeps the rest.
- **Protocol version** — Phase 14 is additive; no version bump required. `HelloMessage.version` stays `"1.1.0"` (Phase 12 bump).

</code_context>

<specifics>
## Specific Ideas

- **"Hand-rolling UI is explicitly off the table"** — Phase 13's rationale carries forward. Phase 14 adopts the shadcn Field recipe verbatim for anatomy, separator placement, and the `data-invalid` + `aria-invalid` validation convention.
- **"We want flexibility without overdefining"** — (user, Area 3 discussion). The default auto-responsive grid (1-col mobile / 2-col desktop) delivers a good out-of-the-box result. Explicit overrides (`FieldSet.cols`, per-field `full_width`) exist for deliberate deviations. Nothing beyond that in v1.1.
- **FieldSet default layout is the **zero-config** default.** Handlers that just write `FieldSet::new().legend("Contact").children([name, email, phone, title]).build()` get a 2x2 grid on desktop and a stacked column on mobile without any prop gymnastics. This matches the "professional screens out of the box" milestone promise.
- **Every form field gets a stable `.id(...)`.** The Phase 14 rewrite's `Field.Label for={id}` depends on it for a11y and for Playwright selectors. Handler code should always set it explicitly (matches Phase 13's `'contact-form-name'`-style convention already in `handlers/contact.rs:490` and friends). Fallback to a mount-time UUID only for the rare case a handler omits it.
- **Keep the native `<form>` wrapper.** Enter-to-submit still works because the form element is preserved. Phase 14 doesn't chase custom keyboard handling.

</specifics>

<deferred>
## Deferred Ideas

- **Wizard / multi-step forms** (FORM-03) — v2.
- **Arbitrary per-field col-span / row-span beyond `full_width`** (FORM-04) — v2.
- **Container-query-based responsive layouts (`@container/field-group`)** — v2 polish. Viewport-based `md:` breakpoints suffice for v1.1.
- **Persistent form state across reloads / navigation** — v2. SDUI is stateless, the server re-renders form state each navigation.
- **Card-wrapped sections** — considered and explicitly rejected (D-C2) in favor of flat `Field.Set` + `Field.Separator`. Can revisit if a real screen demands visual heft.
- **`FormScreen` as a first-class SDUI component** — retired (D-A1). If Phase 15 reveals the inline-composition pattern produces unbearable repetition across ≥3 form handlers, a Rust-side `form_shell()` helper can be introduced without reviving a frontend wrapper.
- **DRY helper for "heading + back button"** — rejected for now (D-A2). Revisit in Phase 15 if the repetition proves annoying across CRM handlers.
- **Validation via Superforms / Formsnap / client-side Zod** — out-of-scope per REQUIREMENTS.md §Out of Scope. Phase 14 stays with the server-side `/_errors` pattern.
- **Full CRM form migration** — Phase 15 (COMP-03). Phase 14 migrates only the minimum needed to smoke-test each new primitive and to regression-check the two carried-over bug fixes.
- **Additional form primitives (Combobox, DatePicker, FileInput)** — not in Field recipe's core set; add when a real CRM screen demands it.

</deferred>

---

*Phase: 14-formscreen-enhancements*
*Context gathered: 2026-04-17*
