# Phase 11: Leaf Component Migration - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Re-implement all 18 existing SDUI components with shadcn-svelte primitives and lucide-svelte icons. Components currently have HTML+Tailwind stubs (from Phase 10) that preserve the SDUI interface contract (`surface`, `props`, `bind`, `action`). This phase replaces those stubs with proper shadcn-svelte primitive wiring. No new component types are added — only existing registrations in `defaults.ts` are migrated.

</domain>

<decisions>
## Implementation Decisions

### shadcn Primitive Strategy
- **D-01:** **Pass-through with styling** — SDUI components become mostly pass-through to shadcn primitives, adding only SDUI-specific logic (bind, action, surface). Shadcn variants used directly.
- **D-02:** **Bulk install upfront** — Install all needed shadcn-svelte primitives in one batch (`npx shadcn-svelte add button input select checkbox dialog ...`) before starting component migration.
- **D-03:** **Compose from shadcn parts** for components without direct shadcn equivalents (SideNav, NavGroup, DataTable, Surface, NodeRenderer). Build from shadcn building blocks where possible (e.g., NavItem could use shadcn Button variant, DataTable could use shadcn Table). More consistency over the whole library.

### Toast Replacement
- **D-04:** **shadcn Toast** (bits-ui Radix Toast primitive) — not Sonner. Requires ToastProvider, ToastViewport, manual stacking. Consistent with the "compose from shadcn parts" approach across all components.

### Icon Migration
- **D-05:** **Dynamic icon registry** — Build a registry that maps string names (from server props) to lucide-svelte components. Server sends icon name, frontend resolves it at runtime. Keeps the SDUI data-driven pattern.
- **D-06:** **Fallback placeholder icon** for unknown icon names — show a generic icon (e.g., lucide `CircleHelp`) so something is always visible. Makes missing icons obvious to users.

### Test Strategy
- **D-07:** **Rewrite browser tests from scratch** — delete existing `.browser-test.ts` files and write new ones targeting shadcn component structure. Clean slate.
- **D-08:** **Full test coverage** — every one of the 18 migrated components gets a browser test. Not just the 6 that previously had tests.

### Claude's Discretion
- Specific shadcn primitive mapping per component (which shadcn component maps to which SDUI component)
- Icon registry implementation details (Map vs object, lazy loading vs eager)
- Test assertion granularity per component
- Migration ordering within the phase

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol & Architecture
- `spec/openapi.yaml` — OpenAPI 3.1 protocol spec defining component types and message formats
- `.planning/codebase/CONVENTIONS.md` — Coding conventions including Svelte 5 component patterns and SDUI interface contract
- `.planning/codebase/STACK.md` — Current tech stack details

### Requirements
- `.planning/REQUIREMENTS.md` §Component Migration — COMP-01 (shadcn primitives), COMP-02 (lucide icons)

### Prior Phase Context
- `.planning/phases/10-foundation/10-CONTEXT.md` — Phase 10 decisions: Default base style, 0.25rem border radius, Zinc theme, CSS variables mode, stub-first approach

### Component Registry
- `frontend/src/lib/registry/defaults.ts` — All 18 SDUI component registrations (the migration scope)

### Research
- `.planning/research/STACK.md` — v1.1 stack research (shadcn-svelte, bits-ui, lucide-svelte)
- `.planning/research/PITFALLS.md` — Known pitfalls for Flowbite-to-shadcn migration

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `frontend/components.json` — shadcn-svelte CLI config with `ui` alias at `$lib/components/ui` (no primitives installed yet)
- `frontend/src/app.css` — OKLCH semantic tokens already configured from Phase 10
- `frontend/src/lib/registry/defaults.ts` — Component registry mapping 18 type strings to Svelte components
- `frontend/src/lib/transport/dispatcher.ts` — `sendAction()` for action dispatch (used by Button, Form, etc.)
- `frontend/src/lib/store/data.svelte.ts` — `getAllData()` for surface data access

### Established Patterns
- All SDUI components accept `surface`, `props`, `bind?`, `action?` — this contract is preserved
- Svelte 5 runes (`$state`, `$derived`, `$props()`) throughout
- Tailwind v4 with `@tailwindcss/vite` plugin
- Tabs for indentation, single quotes, 100-char print width

### Components to Migrate (18 total)
- **core/**: ConnectionBanner, ErrorBoundary, FallbackComponent, LoadingSkeleton, NodeRenderer, Surface
- **feedback/**: ErrorDisplay, Spinner
- **form/**: Button, Checkbox, Form, SelectInput, TextInput
- **layout/**: Container, Grid, Heading, Text
- **nav/**: NavGroup, NavItem, SideNav
- **popup/**: ConfirmDialog, ModalSurface, ToastSurface
- **screen/**: FormScreen, TableScreen
- **table/**: DataTable

### Integration Points
- `$lib/components/ui/` — shadcn primitives will be installed here via CLI
- `frontend/src/lib/index.ts` — barrel file re-exports all components
- `frontend/src/routes/+layout.svelte` — will need Toast provider setup

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Key constraint: every component must render correctly using shadcn-svelte primitives while preserving the SDUI interface contract unchanged.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 11-leaf-component-migration*
*Context gathered: 2026-04-09*
