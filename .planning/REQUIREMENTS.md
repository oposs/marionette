# Requirements: Marionette v1.1

**Defined:** 2026-04-08
**Core Value:** Clean, well-specified SDUI protocol enabling rapid business app development where backend developers control UI

## v1.1 Requirements

Requirements for shadcn-svelte migration and high-level components. Each maps to roadmap phases.

### Foundation

- [ ] **FOUND-01**: shadcn-svelte CLI initialized with bits-ui, lucide-svelte, tw-animate-css, clsx+tailwind-merge dependencies installed
- [ ] **FOUND-02**: app.css rewritten with OKLCH semantic color tokens and shadcn theme system (no Flowbite plugin)
- [ ] **FOUND-03**: All Flowbite packages (flowbite-svelte, flowbite-svelte-icons, flowbite plugin) removed with zero residual imports

### AppShell

- [ ] **SHELL-01**: AppShell component renders a collapsible sidebar on desktop and sheet overlay on mobile using shadcn Sidebar composable
- [ ] **SHELL-02**: AppShell provides header and footer areas for title/user menu and status/version info
- [ ] **SHELL-03**: AppShell uses CSS variable theming (`--sidebar-*` tokens) for consistent styling
- [ ] **SHELL-04**: Backend hand-written AppShell builder follows FormScreen/TableScreen pattern with slot-based child references

### DataTable

- [ ] **TABLE-01**: DataTable displays a filter bar with text input and dropdowns that dispatch server-side filter actions
- [ ] **TABLE-02**: DataTable supports infinite scroll via IntersectionObserver sentinel for progressive server-side data loading
- [ ] **TABLE-03**: User can show/hide DataTable columns via a column visibility toggle

### FormScreen

- [ ] **FORM-01**: Form fields display consistent label, description, and error layout using shadcn Field components
- [ ] **FORM-02**: Related form fields can be grouped in card sections with visual separators

### Component Migration

- [ ] **COMP-01**: All existing SDUI components (Button, TextInput, Select, Checkbox, etc.) re-implemented with shadcn-svelte primitives
- [ ] **COMP-02**: All icons migrated from flowbite-svelte-icons to lucide-svelte
- [ ] **COMP-03**: CRM demo screens fully functional with new component implementations

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### AppShell

- **SHELL-05**: Persistent sidebar collapse state across sessions (cookie or localStorage)
- **SHELL-06**: Auto-generated breadcrumbs from navigation structure
- **SHELL-07**: Multiple sidebar variants (floating, inset)

### DataTable

- **TABLE-04**: Row actions via per-row dropdown menu
- **TABLE-05**: Row count status bar ("Showing X of Y")
- **TABLE-06**: Empty state display with illustration when no data
- **TABLE-07**: Row selection with bulk actions

### FormScreen

- **FORM-03**: Wizard/multi-step form support
- **FORM-04**: Full-width field support (`col-span` across entire form)

## Out of Scope

| Feature | Reason |
|---------|--------|
| TanStack Table adoption | Client-side sort/filter contradicts SDUI's server-driven model |
| Superforms/Formsnap validation | SDUI validates server-side; client-side Zod schemas are redundant |
| TanStack Virtual | Pre-1.0, Svelte 5 incompatibility (GitHub #866); custom virtual scroll works |
| Server-driven shell layout | Shell is structural, not content — server controls content within it |
| Multiple sidebars | Over-engineering; one sidebar with groups covers all business app needs |
| Drag-to-resize sidebar | Complexity for minimal value in business apps |
| Gradual migration (dual Flowbite+shadcn) | CSS conflicts, confusing DX — clean break required |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | — | Pending |
| FOUND-02 | — | Pending |
| FOUND-03 | — | Pending |
| SHELL-01 | — | Pending |
| SHELL-02 | — | Pending |
| SHELL-03 | — | Pending |
| SHELL-04 | — | Pending |
| TABLE-01 | — | Pending |
| TABLE-02 | — | Pending |
| TABLE-03 | — | Pending |
| FORM-01 | — | Pending |
| FORM-02 | — | Pending |
| COMP-01 | — | Pending |
| COMP-02 | — | Pending |
| COMP-03 | — | Pending |

**Coverage:**
- v1.1 requirements: 15 total
- Mapped to phases: 0
- Unmapped: 15 ⚠️

---
*Requirements defined: 2026-04-08*
*Last updated: 2026-04-08 after initial definition*
