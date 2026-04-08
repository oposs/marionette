# Architecture Research: v1.1 shadcn-svelte Migration

**Project:** Marionette v1.1 — shadcn-svelte + High-Level Components
**Researched:** 2026-04-08
**Confidence:** HIGH

## Executive Summary

The existing adjacency list model already supports high-level components — FormScreen and TableScreen prove the pattern. Flowbite is isolated to 15 leaf-level files. Migration is a file-by-file swap with zero protocol, store, transport, or backend builder changes. Do NOT adopt TanStack Table — it fights SDUI by pulling sort/filter logic client-side.

---

## Current Architecture Analysis

### How Adjacency List Accommodates High-Level Components

The existing pattern (proven by FormScreen/TableScreen):
- Parent stores child IDs in `props` for slot placement (e.g., `props.fields`, `props.actions`)
- Declares all children in `children[]` for the flat list
- Uses NodeRenderer to render children into specific layout positions

**AppShell follows exactly this pattern:** `props.sidebar`, `props.header`, `props.footer` as slot ID arrays.

### Flowbite Dependency Map

Flowbite is isolated to **15 leaf-level component files**. No Flowbite import touches core infrastructure (registry, store, transport, protocol).

Files grouped by migration complexity:
- **Simple (direct swap):** Button, Spinner, ErrorDisplay
- **Form primitives:** TextInput, Select, Checkbox, DateInput, Textarea
- **Layout:** Container, Grid, Heading, Surface
- **Navigation:** NavItem, Sidebar
- **Overlay:** Modal, Toast, Confirm
- **Data:** DataTable

### Backend Builder Patterns

Two patterns exist:
1. **Derive macro builders** (18 components): Single `(id, Component)` nodes — good for leaf components
2. **Hand-written builders** (FormScreen, TableScreen): Slot-based child placement, `Vec<Node>` output — needed for organisational components

---

## Integration Design

### AppShell Component

**Architecture:** Renders on the sidebar surface but internally contains `<Surface name="main" />`. This simplifies `+layout.svelte` to just `<Surface name="sidebar" />` plus overlays.

**Protocol representation:**
```json
{
  "id": "app-shell",
  "type": "AppShell",
  "props": {
    "sidebar": ["nav-item-1", "nav-item-2"],
    "header": ["header-title", "user-menu"],
    "footer": ["footer-text"]
  },
  "children": ["nav-item-1", "nav-item-2", "header-title", "user-menu", "footer-text"]
}
```

**Frontend:** Uses shadcn-svelte Sidebar composable (Provider/Root/Content/Trigger) with built-in responsive behavior (collapsible on desktop, sheet overlay on mobile).

**Backend:** Hand-written builder following FormScreen/TableScreen pattern. Consider extracting shared helper for ID collection and props building.

### DataTable Enhancement

- Add `filterable` and `row_actions` props to existing derive-macro builder
- **No TanStack Table adoption** — it pulls sort/filter logic client-side, contradicting SDUI's server-driven model
- Keep server-driven sort/filter via actions
- Add infinite scroll via IntersectionObserver sentinel

### FormScreen

- No architectural changes needed
- Existing builder API is good
- Just swap Flowbite leaf components for shadcn equivalents
- Improve layout composition (field grouping, action button placement)

---

## Migration Strategy (Bottom-Up)

1. **Install shadcn-svelte infrastructure:** `npx shadcn-svelte@latest init`, add components, lucide-svelte
2. **Migrate leaf components:** Button/Spinner/ErrorDisplay first, then form primitives, then Container/Form, then DataTable, then nav cluster, then overlay cluster
3. **Migrate +layout.svelte:** AppShell integration
4. **Update app.css:** Remove Flowbite plugin, add shadcn theme in OKLCH
5. **Remove Flowbite dependencies:** Clean break

### What Does NOT Change

- Protocol specification
- Backend builders (existing ones)
- Component registry type strings
- Data store (JSON Pointer binding, dirty tracking)
- WebSocket transport
- Action dispatching

---

## Backend Builder Consolidation

- **Keep:** 18 derive-macro builders for leaf components
- **Add:** AppShell as hand-written builder
- **Consider:** Extract shared helper for ID collection and props building from FormScreen/TableScreen/AppShell

---

## Anti-Patterns to Avoid

| Anti-Pattern | Why | Do Instead |
|-------------|-----|------------|
| TanStack Table in SDUI | Client-side sort/filter contradicts server-driven model | Keep server-driven DataTable with actions |
| Flowbite/shadcn coexistence | CSS conflicts, confusing DX, delays cleanup | Clean break — swap all at once |
| Layout logic in backend props | Backend shouldn't dictate pixel-level layout | Backend declares slots/structure, frontend handles responsive rendering |
| Nested Surface for AppShell slots | Over-complicates the surface model | AppShell is one component that renders into sidebar surface and embeds main surface internally |

---

## Open Questions

| Question | Impact | When to Resolve |
|----------|--------|-----------------|
| Exact shadcn-svelte Sidebar sub-component API | AppShell implementation | During AppShell phase |
| Toast replacement: Sonner vs shadcn Toast | Overlay migration | During component migration phase |
| Surface `layoutClasses` map refactoring | AppShell takes over layout responsibility | During AppShell phase |

---

## Roadmap Implications

Suggested phase ordering:
1. **Infrastructure:** Install shadcn-svelte, configure CSS theme, add lucide-svelte. No visual changes.
2. **Leaf component migration:** Swap all 15 Flowbite-importing files. Can be parallelized. CRM should work after each swap.
3. **AppShell:** New component (frontend + backend builder). Restructure +layout.svelte. Biggest single change.
4. **DataTable/FormScreen enhancements:** Add filtering, infinite scroll, form layout improvements.
5. **Flowbite removal + cleanup:** Remove packages, verify no residual imports.

---

*Architecture research for: Marionette v1.1 — shadcn-svelte + High-Level Components*
*Researched: 2026-04-08*
