# Phase 10: Foundation - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Install shadcn-svelte as the sole component framework, rewrite CSS theming with OKLCH semantic tokens, and remove all Flowbite dependencies. The frontend must build and render without any Flowbite residue. Components get minimal stubs (not full shadcn re-implementations — that's Phase 11).

</domain>

<decisions>
## Implementation Decisions

### shadcn-svelte Initialization
- **D-01:** Use **Default** base style (not New York) — rounded, spacious, standard business app aesthetics
- **D-02:** Border radius set to **0.25rem** — subtle rounding for a professional/technical look
- **D-03:** Use **CSS variables mode** for theming — enables runtime theme switching, standard shadcn approach

### Color Theme
- **D-04:** Primary color: **Zinc/Neutral** — understated, lets content speak, clean professional look
- **D-05:** Dark mode: **Claude's Discretion** — pragmatic approach (define tokens if easy, don't block on toggle wiring)

### Flowbite Removal Strategy
- **D-06:** **Stub-first approach** — replace each Flowbite import with a minimal HTML+Tailwind stub that compiles, then remove Flowbite packages. App stays buildable throughout the process.
- **D-07:** **Drop Flowbite's dark mode** `@custom-variant dark` — shadcn handles dark mode via its own class strategy

### Component Stub Scope
- **D-08:** Stubs are **minimal HTML + Tailwind** — just enough to compile and render something visible. Phase 11 does the real shadcn-svelte primitive wiring.
- **D-09:** Stubs **preserve the full SDUI interface contract** (`surface`, `props`, `bind`, `action`) — backend doesn't change, only rendering internals are stubbed.

### Claude's Discretion
- Dark mode token definition and toggle wiring (D-05) — Claude picks the pragmatic approach based on effort
- Any ordering details within the stub-first removal process

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol & Architecture
- `spec/openapi.yaml` — OpenAPI 3.1 protocol spec defining component types and message formats
- `.planning/codebase/CONVENTIONS.md` — Coding conventions including Svelte 5 component patterns and SDUI interface contract
- `.planning/codebase/STACK.md` — Current tech stack details including Flowbite dependencies

### Requirements
- `.planning/REQUIREMENTS.md` §Foundation — FOUND-01, FOUND-02, FOUND-03 acceptance criteria

### Research
- `.planning/research/STACK.md` — v1.1 stack research (shadcn-svelte, bits-ui, lucide-svelte)
- `.planning/research/PITFALLS.md` — Known pitfalls for Flowbite-to-shadcn migration

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `frontend/src/app.css` — Current theme file to be rewritten (Flowbite plugin + primary color mapping)
- `frontend/src/lib/index.ts` — Single public API barrel file re-exports all components
- `frontend/src/lib/init.ts` — Library initialization entry point
- `frontend/src/lib/registry/defaults.ts` — Component type-to-Svelte-component registry

### Established Patterns
- All renderable components accept `surface: string`, `props: Record<string, unknown>`, `bind?: string`, `action?: ComponentAction`
- Svelte 5 runes (`$state`, `$derived`, `$props()`) used throughout
- Tailwind v4 already active with `@tailwindcss/vite` plugin
- Tabs for indentation, single quotes, 100-char print width (Prettier)

### Integration Points
- 19 files import from `flowbite-svelte` or `flowbite-svelte-icons` — all need stub replacement
- `frontend/package.json` — `flowbite-svelte` and `flowbite-svelte-icons` in dependencies
- `frontend/src/app.css` — `@plugin "flowbite/plugin"`, `@source` directives, `@custom-variant dark`
- `frontend/src/routes/+layout.svelte` — imports Flowbite components

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The key constraint is that stubs must be functional enough that the app compiles and the dev server starts without errors (success criterion 4).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 10-foundation*
*Context gathered: 2026-04-08*
