# Stack Research: v1.1 shadcn-svelte Migration

**Project:** Marionette v1.1 — shadcn-svelte + High-Level Components
**Researched:** 2026-04-08
**Confidence:** HIGH

## Executive Summary

This research covers stack changes needed for the Flowbite → shadcn-svelte migration and the addition of high-level organisational components (AppShell, Form, DataTable with infinite scroll/filtering). The backend Rust stack is unchanged. Frontend changes are significant but well-documented.

---

## Stack Changes

### Add

| Package | Version | Purpose | Rationale | Confidence |
|---------|---------|---------|-----------|------------|
| shadcn-svelte (CLI) | ^1.2.4 | Component scaffolding | Copies component source into project; Svelte 5 + Tailwind v4 native | HIGH |
| bits-ui | ^2.16 | Runtime primitives | Installed automatically by shadcn-svelte; headless accessible components | HIGH |
| tw-animate-css | latest | Animation utilities | Required by shadcn-svelte theming | HIGH |
| @lucide/svelte | latest | Icons | Replaces flowbite-svelte-icons; shadcn-svelte default icon set | HIGH |
| @tanstack/svelte-table | ^8.21 | DataTable engine | shadcn-svelte provides adapter helpers; headless table with sort/filter/pagination | MEDIUM |
| clsx + tailwind-merge | latest | Class utilities | Standard shadcn-svelte pattern for conditional classes (`cn()` helper) | HIGH |

### Remove

| Package | Reason |
|---------|--------|
| flowbite-svelte | Replaced by shadcn-svelte components |
| flowbite-svelte-icons | Replaced by @lucide/svelte |
| flowbite (Tailwind plugin) | No longer needed; shadcn-svelte uses CSS-native theming |

### Keep (No Changes)

| Package | Notes |
|---------|-------|
| svelte ^5.x | Already compatible with shadcn-svelte |
| tailwindcss ^4.x | shadcn-svelte uses CSS-native theming, no plugins needed |
| vite | No changes needed |
| All Rust backend crates | No backend stack changes for this milestone |

---

## CSS/Theming Migration

**app.css requires a full rewrite:**

1. **Remove:** `@plugin "flowbite/plugin"` and `@source` directives pointing to Flowbite
2. **Add:** `@import 'tw-animate-css'` for animation utilities
3. **Add:** OKLCH semantic color tokens:
   - `--background`, `--foreground` (base)
   - `--primary`, `--primary-foreground` (brand)
   - `--muted`, `--muted-foreground` (subdued)
   - `--card`, `--popover`, `--border`, `--input`, `--ring` (surfaces)
   - `--destructive`, `--accent`, `--secondary` (variants)
   - `--sidebar-*` tokens for AppShell
4. **Add:** `@theme inline` directive mapping CSS vars to Tailwind classes
5. **No Tailwind plugins needed** — everything is CSS-native in v4

---

## Key Component Mappings

### AppShell → shadcn-svelte Sidebar

shadcn-svelte's **Sidebar component** maps directly to the AppShell requirement:
- Composable: Provider / Root / Header / Content / Footer / Trigger
- Built-in responsive behavior: collapsible on desktop, sheet overlay on mobile
- Dedicated CSS tokens (`--sidebar-*`)
- Supports multiple variants: sidebar, floating, inset

### DataTable → TanStack Table + shadcn-svelte

- TanStack Table v8 provides headless table logic (sort, filter, pagination, column visibility)
- shadcn-svelte provides adapter helpers bridging TanStack v8 to Svelte 5
- Watch for TanStack v9 which will have native Svelte 5 support

### Infinite Scroll → IntersectionObserver

- Use native `IntersectionObserver` browser API — do NOT add a library
- ~30-line sentinel component pattern is standard
- Use `rootMargin: "0px 0px 200px 0px"` for early triggering
- SDUI pagination model means backend controls data volume; virtual scrolling likely unnecessary

### Forms → shadcn-svelte primitives

- shadcn-svelte provides Input, Select, Checkbox, RadioGroup, Switch, Textarea, Label
- Formsnap (shadcn-svelte's form library) may be unnecessary — SDUI validates server-side
- Focus on layout composition (action button placement, field grouping, responsive grid)

---

## Integration Notes

- **shadcn-svelte is copy-paste, not a dependency:** Components are scaffolded into `$lib/components/ui/` and can be customized freely. This means SDUI wrappers can import and compose them directly.
- **bits-ui is the runtime dependency:** Headless accessible primitives that shadcn-svelte builds on. Installed automatically by the CLI.
- **No dual-dependency period:** Clean break — remove Flowbite entirely, replace with shadcn-svelte from the start.

---

## Open Questions

| Question | Impact | When to Resolve |
|----------|--------|-----------------|
| How does Formsnap interact with SDUI's server-side validation? | May add unnecessary client-side complexity | During Form component design |
| TanStack Table v8 adapter pattern for Svelte 5 — needs hands-on validation | DataTable implementation approach | During DataTable phase |
| Whether virtual scrolling is needed or if SDUI pagination makes it moot | Scope of DataTable infinite scroll | During DataTable phase |

---

## Roadmap Implications

1. **Foundation first:** Flowbite-to-shadcn swap (app.css rewrite, dependency swap, `npx shadcn-svelte@latest init`)
2. **Component migration:** Re-implement existing 20+ SDUI components using shadcn-svelte primitives
3. **High-level components:** AppShell (via Sidebar), enhanced Form layout, enhanced DataTable with TanStack + infinite scroll
4. **CRM migration:** Update all CRM screens to use new components

---

*Stack research for: Marionette v1.1 — shadcn-svelte + High-Level Components*
*Researched: 2026-04-08*
