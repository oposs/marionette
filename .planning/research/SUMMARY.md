# Research Summary: v1.1 shadcn-svelte Migration

**Project:** Marionette v1.1 — shadcn-svelte + High-Level Components
**Researched:** 2026-04-08
**Confidence:** HIGH

## Executive Summary

Marionette v1.1 is a targeted migration of the frontend component layer from Flowbite to shadcn-svelte, plus three high-level organisational components: AppShell (responsive sidebar shell), enhanced FormScreen, and enhanced DataTable. The backend Rust stack, WebSocket transport, protocol, and data store are completely unchanged. Flowbite is isolated to 15 leaf-level files — no core infrastructure is touched — making this tractable as a clean break.

---

## Stack Additions

| Add | Remove |
|-----|--------|
| shadcn-svelte ^1.2.4 (CLI) | flowbite-svelte |
| bits-ui ^2.16 | flowbite-svelte-icons |
| @lucide/svelte | flowbite Tailwind plugin |
| tw-animate-css | |
| clsx + tailwind-merge | |

app.css requires full rewrite: OKLCH semantic tokens, `@import 'tw-animate-css'`, no Tailwind plugins.

## Feature Table Stakes

| Component | Must Have | Should Have | Do NOT Build |
|-----------|-----------|-------------|-------------|
| AppShell | Collapsible sidebar, mobile sheet overlay, header/footer, CSS tokens | Persistent collapse state, nav groups | Server-driven shell layout, multiple sidebars |
| DataTable | Filter bar (server-driven), infinite scroll, sort indicators | Column visibility, empty states, row count | TanStack Table, client-side filtering |
| FormScreen | Consistent field styling (label/error/description) | Card sections, visual separators | Superforms/Formsnap, wizard forms |

## Architecture Key Points

- Flowbite isolated to exactly **15 leaf files** — zero protocol/store/transport changes
- AppShell follows FormScreen/TableScreen pattern: `props.sidebar_items`, `props.header_items` reference child IDs
- AppShell owns shadcn `Sidebar.Provider`; Surfaces render inside it
- 18 existing derive-macro builders untouched; one new hand-written AppShell builder added
- **Do NOT adopt TanStack Table** — client-side sort/filter contradicts SDUI's server-driven model
- Keep custom virtual scroll (~90 lines) — TanStack Virtual has Svelte 5 issues (GitHub #866)

## Top 5 Watch-Out Pitfalls

| # | Pitfall | Prevention | Phase |
|---|---------|------------|-------|
| 1 | CSS variable conflict during swap | Remove Flowbite in same commit as shadcn install | Foundation |
| 2 | Sidebar.Provider context vs Surface model | AppShell owns Provider; bits-ui >= 1.3.5 | AppShell |
| 3 | TanStack Virtual Svelte 5 incompatibility | Keep custom virtual scroll | DataTable |
| 4 | fetchedRanges stale after sort/filter | Reset Set on parameter change | DataTable |
| 5 | Dynamic Tailwind classes don't work | Use `style=` attribute for dynamic grids | Forms |

## Suggested Phase Structure (6 phases)

1. **Foundation** — Install shadcn-svelte, rewrite app.css, swap dependencies, remove Flowbite entirely
2. **Leaf Component Migration** — Swap all 15 Flowbite-importing files to shadcn-svelte primitives
3. **AppShell** — New hand-written Rust builder + frontend using shadcn Sidebar composable
4. **DataTable Enhancements** — Filter bar, infinite scroll, empty state, column visibility
5. **FormScreen Enhancements** — Field-level styling, card sections, visual separators
6. **CRM Migration + Cleanup** — Update all CRM screens, verify zero Flowbite references

## Research Gaps (Resolve During Planning)

- Exact shadcn Sidebar sub-component API for Svelte 5 (Phase 3)
- Toast replacement: Sonner vs shadcn Toast (Phase 2)
- Field components without Superforms (Phase 5)
- IntersectionObserver + virtual scroll integration (Phase 4)

---

*Research summary for: Marionette v1.1 — shadcn-svelte + High-Level Components*
*Synthesized: 2026-04-08*
