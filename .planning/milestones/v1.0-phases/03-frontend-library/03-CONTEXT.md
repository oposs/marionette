# Phase 3: Frontend Library - Context

**Gathered:** 2026-03-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the complete Marionette Svelte 5 library: reactive data store with JSON Pointer binding, component registry with dynamic rendering from adjacency lists, WebSocket message handling, multi-surface renderer, and the full component vocabulary (navigation, forms, layout, tables, popups, feedback). Includes comprehensive tests (unit, browser component, visual regression). The library lives in `frontend/src/lib/` and is consumed by the CRM demo in `frontend/src/routes/`.

</domain>

<decisions>
## Implementation Decisions

### Data store
- Single Svelte 5 `$state()` store holding all protocol data
- JSON Pointer paths resolve into the store via get/set helpers
- Components bind via derived signals from the store
- Patch operations update the store reactively

### Dirty field tracking
- Focus-based: mark field dirty on focus, clean on blur
- While dirty, skip incoming server patches to that path — queue them as pending
- On blur, apply pending patches for that path
- Simple, predictable, per CONCEPT.md recommendation

### Optimistic updates
- Snapshot affected paths before applying optimistic patch
- If server responds with error, restore snapshot (simple undo)
- No event-sourcing complexity — snapshot + restore is sufficient

### URL routing
- Action-driven routing — SvelteKit's router is bypassed for SDUI
- Backend render messages include a `route` field
- Frontend updates URL via `history.pushState` to match
- Browser back/forward send navigation actions to backend
- Initial page load: frontend connects WebSocket, sends navigate action with current URL

### Component registry
- Static registry map: plain object mapping type strings to Svelte component constructors
- Registered at app init, extensible via `register(type, component)` function
- Unknown types render a visible fallback component (red border, type name, props dump in dev)

### Component rendering
- Recursive `<NodeRenderer>` component traverses the adjacency list
- Looks up type in registry, passes props/bind/action, recursively renders children by ID
- Each surface has its own root node and independent component tree

### Multi-surface rendering
- Named `<Surface>` components in the root layout: main, sidebar, modal, toast
- Each surface renders its own component tree from render messages targeting that surface
- Modal and toast surfaces render as overlays

### Component vocabulary — Flowbite integration
- Interactive/styled controls: thin wrappers around Flowbite Svelte components
- Marionette adds data binding + action dispatch, Flowbite handles visual styling
- Container components (form, grid, container): use Flowbite Card/Section wrappers for consistent visual treatment

### Data table
- Virtual scrolling: appears as if entire table is present, rows load progressively as user scrolls
- Backend provides total row count; frontend requests rows in chunks as scroll position changes
- Server-side sort: sort action goes to backend, backend re-sends data in new order, frontend resets virtual scroller
- No client-side pagination — the virtual scroll replaces traditional pagination

### Claude's Discretion
- Virtual scroll chunk size and prefetch strategy
- Exact WebSocket reconnection backoff parameters (within exponential backoff spec)
- Component prop type definitions (TypeScript interfaces)
- Error boundary implementation around individual components
- Toast auto-dismiss timing and animation
- Loading skeleton design for surfaces

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Protocol specification (authoritative)
- `spec/PROTOCOL.md` — Authoritative protocol manual: message types, data binding, keyed collections, transport
- `spec/openapi.yaml` — OpenAPI 3.1 entry point with all schema references
- `spec/schemas/message.yaml` — All 6 message type schemas (hello, render, patch, action, event, error)
- `spec/schemas/component.yaml` — Component adjacency list node structure
- `spec/schemas/data.yaml` — PatchOperation, KeyedCollection, ValidationError schemas
- `spec/schemas/common.yaml` — Surface, JsonPointer, MessageId types

### Project definition
- `CONCEPT.md` — Original vision document (superseded by PROTOCOL.md but useful for motivation)
- `TOOLING.md` — Tech stack decisions, testing frameworks (Vitest, vitest-browser-svelte, Playwright)
- `.planning/REQUIREMENTS.md` — FRONT-01 through FRONT-27 requirements

### Prior phases
- `.planning/phases/01-project-infrastructure/01-CONTEXT.md` — SvelteKit structure, src/lib/ vs src/routes/ separation
- `.planning/phases/02-protocol-specification/02-CONTEXT.md` — WebSocket-only transport, message envelope, optimistic updates

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `frontend/src/lib/index.ts` — Library entry point (stub, ready for exports)
- `frontend/src/routes/+layout.svelte` — Root layout (imports app.css with Tailwind/Flowbite)
- `frontend/src/routes/+page.svelte` — Demo home page (placeholder)
- `frontend/src/app.css` — Tailwind v4 + Flowbite plugin configured
- Flowbite Svelte 1.31.0 already installed as dependency

### Established Patterns
- SvelteKit with adapter-static (SPA fallback to index.html)
- Vite proxy: `/api/*` → localhost:3001, `/ws` → ws://localhost:3001
- ESLint flat config + Prettier for code quality
- Vitest configured with placeholder test

### Integration Points
- `frontend/src/lib/` — all library code goes here (publishable via svelte-package)
- `frontend/src/routes/` — CRM demo app consumes library via `$lib` imports
- WebSocket connects to `/ws` (proxied to backend in dev, direct in production)
- `spec/schemas/` — TypeScript types should match these JSON schemas

</code_context>

<specifics>
## Specific Ideas

- Virtual scroll table gives a desktop-app feel where the whole dataset appears present
- Thin Flowbite wrappers keep components visually consistent without reinventing styling
- Visible fallback for unknown types makes development/debugging much easier
- Focus-based dirty tracking is simple and predictable — no complex keystroke detection

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-frontend-library*
*Context gathered: 2026-03-19*
