# Feature Research: v1.1 shadcn-svelte Migration

**Project:** Marionette v1.1 — AppShell, Form Layout, DataTable
**Researched:** 2026-04-08
**Confidence:** MEDIUM-HIGH

## Executive Summary

Three high-level organisational components need feature analysis: AppShell, FormScreen, and DataTable. shadcn-svelte's Sidebar component is a near-perfect match for AppShell. TanStack Table is an anti-pattern for SDUI. FormScreen already has most table-stakes features — improvements are mostly visual polish.

---

## AppShell

### Table Stakes

| Feature | Description | shadcn-svelte Component | Complexity |
|---------|-------------|------------------------|------------|
| Collapsible sidebar | Nav area that collapses/expands | Sidebar.Root (collapsible modes: offcanvas, icon, none) | LOW |
| Mobile responsive | Sheet overlay on mobile, sidebar on desktop | Sidebar.Provider (auto-detects mobile) | LOW |
| Header bar | Top area for title, breadcrumbs, user menu | Custom composition within Sidebar.Inset | LOW |
| Content area | Main content region with correct padding | Sidebar.Inset | LOW |
| Footer | Bottom area for status, version info | Sidebar.Footer | LOW |
| Keyboard shortcut | Toggle sidebar with keyboard | Built-in Cmd+B | FREE |
| CSS variable theming | Sidebar-specific color tokens | `--sidebar-*` tokens | LOW |

### Differentiators

| Feature | Description | Complexity | Dependencies |
|---------|-------------|------------|-------------|
| Multiple sidebar variants | offcanvas, icon-only, floating, inset | LOW | Sidebar.Root variant prop |
| Grouped navigation | Collapsible nav groups with labels | LOW | Sidebar.Group / Sidebar.Menu |
| Persistent collapse state | Remember sidebar state across sessions | MEDIUM | Cookie or localStorage |
| Breadcrumb integration | Auto-generated breadcrumbs from nav structure | MEDIUM | Requires nav tree context |

### Anti-Features (Do NOT Build)

| Feature | Why Not |
|---------|---------|
| Server-driven shell layout | Shell is structural, not content — server controls content within it |
| Multiple sidebars | Over-engineering; one sidebar with groups covers all CRM needs |
| Drag-to-resize sidebar | Complexity for minimal value in business apps |

### shadcn-svelte Mapping

The Sidebar composable provides: `Sidebar.Provider`, `Sidebar.Root`, `Sidebar.Header`, `Sidebar.Footer`, `Sidebar.Content`, `Sidebar.Group`, `Sidebar.Menu`, `Sidebar.MenuItem`, `Sidebar.Trigger`, `Sidebar.Inset`

---

## FormScreen

### Table Stakes (Already Implemented in v1.0)

| Feature | Status | Notes |
|---------|--------|-------|
| Field sections with headings | ✓ Built | Via Heading components |
| Multi-column grid layout | ✓ Built | Via Grid component |
| Action bar (save/cancel/delete) | ✓ Built | Via Button components |
| Back navigation | ✓ Built | Via navigate action |
| Dirty tracking | ✓ Built | Data store feature |
| Server-side validation | ✓ Built | Error messages from backend |

### v1.1 Improvements

| Feature | Description | shadcn-svelte Component | Complexity |
|---------|-------------|------------------------|------------|
| Consistent field styling | Label, description, error layout per field | Field.Field / Field.Label / Field.Error / Field.Description | LOW |
| Visual separators | Section dividers between field groups | Separator | LOW |
| Card sections | Grouped fields in card containers | Card | LOW |
| Full-width fields | Fields spanning entire form width | `col-span` CSS utility | LOW |
| Better error display | Inline field errors replacing Flowbite Helper | Field.Error / Alert | LOW |

### Anti-Features (Do NOT Build)

| Feature | Why Not |
|---------|---------|
| Superforms/Formsnap validation | SDUI validates server-side; client-side Zod schemas are redundant |
| Client-side form state management | Data store already handles this via JSON Pointer binding |
| Wizard/multi-step forms | Out of scope for v1.1; simple forms cover CRM needs |

---

## DataTable

### Table Stakes

| Feature | Status | shadcn-svelte Component | Complexity |
|---------|--------|------------------------|------------|
| Column headers with sort indicators | ✓ Built (needs restyle) | Table.Head + lucide-svelte icons | LOW |
| Row rendering with data binding | ✓ Built | Table.Row / Table.Cell | LOW |
| Server-driven sorting | ✓ Built | Action dispatch (keep as-is) | — |
| Server-driven pagination | ✓ Built | Action dispatch (keep as-is) | — |
| Virtual scroll | ✓ Built | Keep existing implementation | — |

### v1.1 Improvements

| Feature | Description | shadcn-svelte Component | Complexity |
|---------|-------------|------------------------|------------|
| Filter bar | Text input + dropdowns above table for server-side filtering | Input + Select + Button | MEDIUM |
| Infinite scroll | IntersectionObserver sentinel for loading more rows | Custom (~30 lines) | LOW |
| Row actions | Per-row action buttons/menu | DropdownMenu | MEDIUM |
| Column visibility toggle | Show/hide columns | DropdownMenu with checkboxes | LOW |
| Empty state | Meaningful display when no data | Custom with illustration | LOW |
| Row count status | "Showing X of Y" footer | Table.Footer | LOW |

### Anti-Features (Do NOT Build)

| Feature | Why Not |
|---------|---------|
| TanStack Table | Client-side sort/filter model contradicts SDUI's server-driven approach |
| Client-side filtering | Server owns data filtering in SDUI |
| Column reordering/resizing | Over-engineering for business app tables |
| Row selection with bulk actions | Out of scope for v1.1 |

---

## Icon Migration

| Flowbite Icon | Lucide Equivalent |
|--------------|-------------------|
| ChevronUpOutline | ChevronUp |
| ChevronDownOutline | ChevronDown |
| All other flowbite-svelte-icons | Corresponding lucide-svelte icons |

---

## Feature Dependencies

```
shadcn-svelte install (foundation)
├── AppShell (Sidebar composable)
│   └── CRM layout migration
├── Leaf component migration (Button, Input, Select, etc.)
│   ├── FormScreen improvements (Field styling, separators)
│   │   └── CRM form migration
│   └── DataTable restyle (Table components)
│       ├── Filter bar
│       ├── Infinite scroll
│       └── CRM table migration
└── Flowbite removal (after all migrations)
```

---

## MVP Recommendations (Build Priority)

1. **Must have:** AppShell with responsive sidebar, DataTable with filter bar and infinite scroll, clean shadcn-svelte styling on all components
2. **Should have:** Field-level error/description styling, card sections in forms, column visibility toggle, empty states
3. **Nice to have:** Persistent sidebar state, row actions dropdown, breadcrumbs, row count status bar

---

## Open Questions

| Question | Impact | When to Resolve |
|----------|--------|-----------------|
| shadcn-svelte Field components without Superforms | Form styling approach | During form migration |
| Exact Sidebar sub-component API for Svelte 5 | AppShell implementation | During AppShell phase |
| Tailwind v4 compatibility with shadcn-svelte | Foundation setup | During installation phase |

---

*Feature research for: Marionette v1.1 — shadcn-svelte + High-Level Components*
*Researched: 2026-04-08*
