# Phase 10: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-08
**Phase:** 10-foundation
**Areas discussed:** shadcn-svelte init choices, Color theme design, Flowbite removal strategy, Component stub scope

---

## shadcn-svelte Init Choices

### Base Style

| Option | Description | Selected |
|--------|-------------|----------|
| Default (Recommended) | Rounded corners, softer feel, slightly more spacious — typical for modern business apps | ✓ |
| New York | Sharper corners, tighter spacing, more compact — denser information display | |

**User's choice:** Default
**Notes:** None

### Border Radius

| Option | Description | Selected |
|--------|-------------|----------|
| 0.5rem (Recommended) | Standard rounded — balanced, works well for forms and cards | |
| 0.75rem | More rounded — softer, friendlier feel | |
| 0.25rem | Subtle rounding — more technical/professional look | ✓ |

**User's choice:** 0.25rem
**Notes:** User opted for the more professional/technical look despite the recommended 0.5rem

### CSS Variables Mode

| Option | Description | Selected |
|--------|-------------|----------|
| CSS variables (Recommended) | Colors defined as CSS custom properties — enables runtime theme switching, easier dark mode | ✓ |
| Inline Tailwind colors | Colors baked into Tailwind classes directly — simpler but no runtime theme switching | |

**User's choice:** CSS variables
**Notes:** None

---

## Color Theme Design

### Primary Color Hue

| Option | Description | Selected |
|--------|-------------|----------|
| Blue (current) | Keep the existing blue primary — professional, familiar, matches current Flowbite mapping | |
| Zinc/Neutral (Recommended) | shadcn default — understated, lets content speak, very clean professional look | ✓ |
| Slate | Cool gray-blue neutral — slightly warmer than zinc, still professional | |

**User's choice:** Zinc/Neutral
**Notes:** None

### Dark Mode Support

| Option | Description | Selected |
|--------|-------------|----------|
| Light only (Recommended) | Ship light theme first — define dark tokens but don't wire up toggle | |
| Both light and dark | Wire up both themes with a toggle — more work but complete from the start | |
| You decide | Claude picks the pragmatic approach | ✓ |

**User's choice:** You decide (Claude's discretion)
**Notes:** None

---

## Flowbite Removal Strategy

### Removal Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Stub first, then remove (Recommended) | Replace each Flowbite import with a minimal stub, then remove packages. App stays buildable. | ✓ |
| Remove packages first | Delete Flowbite from package.json first, then fix all 19 files. Faster but breaks build. | |
| You decide | Claude picks the safest approach | |

**User's choice:** Stub first, then remove
**Notes:** None

### Dark Mode CSS Variant

| Option | Description | Selected |
|--------|-------------|----------|
| Drop it (Recommended) | Remove `@custom-variant dark` — shadcn handles dark mode its own way | ✓ |
| Keep for now | Preserve as compatibility bridge until Phase 11 | |

**User's choice:** Drop it
**Notes:** None

---

## Component Stub Scope

### Stub Fidelity

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal HTML stubs (Recommended) | Plain HTML + Tailwind that compiles and renders something visible. Phase 11 does real implementation. | ✓ |
| Basic shadcn equivalents | Already wire up shadcn-svelte primitives with basic props. More work now. | |
| You decide | Claude picks based on cleanest phase boundary | |

**User's choice:** Minimal HTML stubs
**Notes:** None

### Interface Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve interface (Recommended) | Keep full SDUI contract — backend doesn't change, only rendering internals | ✓ |
| Simplify temporarily | Drop unused props temporarily — Phase 11 restores them | |

**User's choice:** Preserve interface
**Notes:** None

---

## Claude's Discretion

- Dark mode: Claude decides pragmatic approach (define tokens if easy, don't block on toggle)

## Deferred Ideas

None — discussion stayed within phase scope
