# Phase 14: FormScreen Enhancements - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-17
**Phase:** 14-formscreen-enhancements
**Areas discussed:** FormScreen disposition, Field integration strategy, Section/group protocol shape, Action row pattern, Scope (deferred fixes + new primitives)

---

## Gray Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| FormScreen disposition | Retire orphan vs. rewrite as first-class screen vs. builder-helper only. | ✓ |
| Field integration strategy | Internal wrap per leaf component vs. external FormField wrapper. | ✓ |
| Section/group protocol shape | FieldSet SDUI component vs. sections prop vs. Card.Root wrapping. | ✓ |
| Scope: deferred bug fixes + new primitives | Fold deferred items + install Textarea / RadioGroup / Switch. | ✓ |

---

## Area 1 — FormScreen disposition

### Q1a — What should happen to FormScreen.svelte?

| Option | Description | Selected |
|--------|-------------|----------|
| Retire it | Hard delete FormScreen.svelte + browser-test. Handlers compose inline, mirroring Phase 13's TableScreen retirement. | ✓ |
| Rewrite as first-class screen | Register in defaults.ts + add backend builder with sections prop. | |
| Convert to composition helper only | Delete frontend wrapper, keep a Rust-side form_screen() helper. | |

**User's choice:** Retire it (recommended).
**Notes:** Aligns with Phase 13 D-A2 precedent and the "no hand-rolled UI" memory.

### Q1b — Do handlers build title + back-button region inline, or through a reusable pattern?

| Option | Description | Selected |
|--------|-------------|----------|
| Inline, per-handler composition | Each handler builds Container([Heading, Button(ArrowLeft, back)]) explicitly. | ✓ |
| Rust builder helper: form_header(title, back_action) | Backend helper returns the heading + back-button subtree. | |

**User's choice:** Inline composition.
**Notes:** Matches post-Phase 13 list-handler style; Phase 15 can DRY up if ≥3 screens prove the repetition is painful.

---

## Area 2 — Field integration strategy

### Q2a — Where does the Field.Field/Label/Description/Error scaffolding live?

| Option | Description | Selected |
|--------|-------------|----------|
| Inside each leaf component | TextInput, SelectInput, Checkbox render their own Field.Field internally. One protocol node per field. | ✓ |
| External FormField wrapper | New FormField component wraps a single leaf input child. Two protocol nodes per field. | |

**User's choice:** Internal wrap (recommended).
**Notes:** Matches Phase 11 D-01 "pass-through with styling"; keeps handler ergonomics (`TextInput::new().bind().build()`) identical.

### Q2b — How does the error state flow from data store to Field?

| Option | Description | Selected |
|--------|-------------|----------|
| Keep /_errors/{bind} convention | Leaf component reads getData(surface, '/_errors' + bind). No protocol change. | ✓ |
| Explicit error prop | Backend adds props.error: Option<String> on each field. | |

**User's choice:** Keep /_errors convention.
**Notes:** Zero protocol change; matches existing TextInput.svelte:26.

---

## Area 3 — Section/group protocol shape

### Q3a — How do handlers express grouped sections?

| Option | Description | Selected |
|--------|-------------|----------|
| FieldSet SDUI component | New 'field-set' component; pure adjacency-list composition. | ✓ |
| Sections prop on Container/Form | Extend with sections: [{legend, field_ids[]}] prop. | |
| Card.Root wrap (visual-only) | Each section inside Card.Root. | |

**User's choice:** FieldSet SDUI component (recommended).
**Notes:** Parallels Phase 13's Container([Heading, Buttons, DataTable]) composition style.

### Q3b — Visual style for section boundaries — Field.Set or Card.Root?

| Option | Description | Selected |
|--------|-------------|----------|
| Field.Set with Field.Separator between sets | Flat visual rhythm, no shadowed cards. | ✓ |
| Card.Root wrapping Field.Set | Heavier visual weight. | |
| Backend chooses via FieldSet variant prop | Backend picks per screen. | |

**User's choice:** Field.Set with Field.Separator (recommended).
**Notes:** Uses the shadcn Field recipe verbatim.

### Q3c — Column layout within a FieldSet (initial framing)

| Option | Description | Selected |
|--------|-------------|----------|
| Field orientation + nested grids (recipe pattern) | Recipe uses inline grid-cols-N divs inside Field.Group. YAGNI. | |
| FieldSet columns prop | Simple N-column layouts. | |
| Per-field span prop | Most flexible, probably overkill. | |
| Defer entirely | Single column only in v1.1. | |

**User's choice:** (rejected — asked for clarification)
**Notes:** User redirected: "looking for a solution which has a default mode that renders well on desktop (with multi column) and on mobile with single column, all automatically without special configuration ... the configuration should come into play when we want something special, to override the sensible default behavior." Question reformulated.

### Q3c-v2 — Auto-responsive default — which shape fits best?

| Option | Description | Selected |
|--------|-------------|----------|
| FieldSet auto-grid: 1-col mobile, 2-col desktop | grid-cols-1 md:grid-cols-2. Zero handler config. Overrides: FieldSet.cols + per-field full_width. | ✓ |
| FieldSet auto-grid with container queries | @container/field-group for sidebar-aware layouts. | |
| Keep it simple: stacked-by-default, opt-in multi-column | No auto-responsive without explicit opt-in. | |

**User's choice:** FieldSet auto-grid with viewport breakpoint (recommended).
**Notes:** Defers container queries to v2.

---

## Area 4 — Scope: deferred bug fixes + new primitives

### Q4a — Which deferred items / new primitives land in Phase 14?

| Option | Description | Selected |
|--------|-------------|----------|
| TextInput input_type/type fix | Phase 12 deferred: password fields render as text. | ✓ |
| NodeRenderer handleBlur unmount race fix | Phase 13 deferred: noisy console exception on unmount. | ✓ |
| Install Textarea primitive | Field examples use it; Phase 15 CRM will want it. | ✓ |
| Install RadioGroup + Switch primitives | Complete Field-recipe primitive coverage. | ✓ |

**User's choice:** All four folded in.
**Notes:** Brings in all deferred form-adjacent items while the form layer is open.

### Q4b — Deletion sweep policy for retired FormScreen

| Option | Description | Selected |
|--------|-------------|----------|
| Hard delete FormScreen + its test | Remove orphan outright. | ✓ |
| Keep FormScreen.svelte, mark @deprecated | Leave tombstoned dead code. | |

**User's choice:** Hard delete (recommended).
**Notes:** Consistent with pre-deployment posture.

---

## Area 5 — Form.svelte + action row (wrap-up)

### Q5a — What happens to Form.svelte?

| Option | Description | Selected |
|--------|-------------|----------|
| Keep Form.svelte as the <form> boundary | Emit <form>, dispatch submit, render form-level error banner. Minimal change. | ✓ |
| Retire Form.svelte too | Handlers dispatch via button action; separate FormErrors component. | |

**User's choice:** Keep Form.svelte (recommended).
**Notes:** Preserves native Enter-to-submit behavior. API unchanged.

### Q5b — Action bar pattern (save/cancel buttons)

| Option | Description | Selected |
|--------|-------------|----------|
| Recipe pattern: Field.Field orientation=horizontal | New field-row component OR plain Container with flex classes. | ✓ |
| Dedicated ActionBar component with top border | Heavier separator. | |
| Claude's discretion — pick best shape during planning | Planner decides. | |

**User's choice:** Recipe pattern (recommended). Exact shape (new SDUI component vs. plain Container) is Claude's discretion during planning.
**Notes:** Matches shadcn Field recipe's own end-of-form layout.

---

## Closing

**Q6 — Ready for context or explore more?**

| Option | Description | Selected |
|--------|-------------|----------|
| Ready for context | Write CONTEXT.md and commit. | ✓ |
| Explore more | Name another area. | |

---

## Claude's Discretion (consolidated)

- Exact action-row shape (`Container` with class vs. new `field-row` SDUI component).
- Whether `Field.Separator` is an explicit adjacency-list node or auto-inserted between sibling `FieldSet`s.
- Field `id` fallback when handler forgot `.id(...)` — UUID vs. derived-from-bind.
- Blur-race fix location — TextInput vs. NodeRenderer (leaf preference: NodeRenderer).
- Per-component migration order within the phase.
- Specific utility classes for the auto-responsive grid (gap, breakpoint choices) as long as D-C3 behavior holds.
- Test granularity per component (every leaf vs. a representative subset).

## Deferred Ideas (consolidated)

- Wizard / multi-step forms (FORM-03, v2).
- Arbitrary per-field col-span / row-span (FORM-04, v2).
- Container-query-based responsive layouts (v2 polish).
- Persistent form state across reloads / navigation (v2).
- Card-wrapped sections (rejected — D-C2).
- FormScreen as a first-class SDUI component (retired — D-A1; revisit as a Rust helper in Phase 15 only if repetition proves painful).
- DRY helper for heading + back-button (rejected for now — D-A2).
- Client-side validation (out-of-scope per REQUIREMENTS.md).
- Full CRM form migration (Phase 15 COMP-03).
- Additional form primitives (Combobox, DatePicker, FileInput) — add on demand.
