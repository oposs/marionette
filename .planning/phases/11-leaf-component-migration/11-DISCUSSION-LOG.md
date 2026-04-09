# Phase 11: Leaf Component Migration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 11-leaf-component-migration
**Areas discussed:** shadcn primitive strategy, Toast replacement, Icon migration approach, Test strategy

---

## shadcn Primitive Strategy

### Wrapping approach

| Option | Description | Selected |
|--------|-------------|----------|
| Thin wrapper (Recommended) | SDUI components import shadcn primitives and map props/actions onto them. Minimal custom CSS. | |
| Pass-through with styling | SDUI components become mostly pass-through to shadcn, adding only SDUI-specific logic. Shadcn variants used directly. | ✓ |
| You decide | Claude picks pragmatic approach per component. | |

**User's choice:** Pass-through with styling
**Notes:** None

### Install approach

| Option | Description | Selected |
|--------|-------------|----------|
| Bulk install upfront (Recommended) | Run `npx shadcn-svelte add` once for all needed primitives. | ✓ |
| Incremental per component | Install each shadcn primitive as we migrate the SDUI component that needs it. | |
| You decide | Claude picks based on what flows better. | |

**User's choice:** Bulk install upfront
**Notes:** None

### No-match components

| Option | Description | Selected |
|--------|-------------|----------|
| Keep custom HTML+Tailwind | Components without shadcn equivalents stay as hand-written HTML+Tailwind. | |
| Compose from shadcn parts | Build from shadcn building blocks where possible (NavItem → shadcn Button variant, DataTable → shadcn Table). | ✓ |
| You decide per component | Claude assesses each individually. | |

**User's choice:** Compose from shadcn parts
**Notes:** None

---

## Toast Replacement

| Option | Description | Selected |
|--------|-------------|----------|
| Sonner (Recommended) | svelte-sonner — minimal API, great defaults, recommended by shadcn-svelte docs. | |
| shadcn Toast | Built on Radix Toast primitive via bits-ui. More control, more wiring. | ✓ |
| You decide | Claude picks based on SDUI integration fit. | |

**User's choice:** shadcn Toast
**Notes:** Consistent with the "compose from shadcn parts" approach.

---

## Icon Migration Approach

### Icon lookup

| Option | Description | Selected |
|--------|-------------|----------|
| Dynamic registry (Recommended) | Map string names from server to lucide-svelte components at runtime. | ✓ |
| Static imports only | Each component statically imports specific lucide icons it needs. | |
| You decide | Claude picks based on existing SDUI pattern. | |

**User's choice:** Dynamic registry
**Notes:** None

### Fallback behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Render nothing + console.warn | Silent failure with dev warning, consistent with FallbackComponent. | |
| Fallback placeholder icon | Show a generic icon (e.g., CircleHelp) so something is always visible. | ✓ |
| You decide | Claude picks based on existing error handling patterns. | |

**User's choice:** Fallback placeholder icon
**Notes:** None

---

## Test Strategy

### Existing tests

| Option | Description | Selected |
|--------|-------------|----------|
| Update in-place (Recommended) | Modify existing .browser-test.ts files to match new shadcn markup. | |
| Rewrite from scratch | Delete existing browser tests and write new ones from scratch. | ✓ |
| You decide | Claude assesses per test file. | |

**User's choice:** Rewrite from scratch
**Notes:** None

### Coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Match existing coverage | Only update tests for components that already have tests (6 of 18). | |
| Full coverage | Add browser tests for ALL 18 components. | ✓ |
| You decide | Claude picks based on risk per component. | |

**User's choice:** Full coverage
**Notes:** None

---

## Claude's Discretion

- Specific shadcn primitive mapping per SDUI component
- Icon registry implementation details
- Test assertion granularity per component
- Migration ordering within the phase

## Deferred Ideas

None — discussion stayed within phase scope
