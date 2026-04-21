---
title: Live CSS-token editor screen in the gallery app
planted_date: "2026-04-21"
trigger_condition: "Phase E scoping (v1.2 gallery milestone), OR earlier if a designer starts iterating on look-and-feel and wants a dedicated tweak surface"
status: planted
---

# Live `--token` editor (gallery demo app)

## Idea

A dedicated gallery screen with UI controls (sliders, colour pickers, numeric
inputs) bound to shadcn-theme CSS custom properties: `--primary`, `--radius`,
`--sidebar-*`, `--background`, `--foreground`, etc.

Changing a control applies the value to `document.documentElement` via
`style.setProperty(...)`, re-rendering the entire gallery in real time without a
backend round-trip. A "export" button serialises the current token set back to
paste-ready `@theme` block snippets for `app.css`.

## Why planted (not built immediately)

- It's the single highest-leverage feature for "improve look and feel" — every
  other gallery screen benefits from it.
- But it is **not** framework-shaped work: it's pure frontend affordance, sits
  slightly outside the "auto-discoverable component demos" spine of the v1.2
  milestone.
- If the rest of v1.2 (Phases A–D) runs long, this is the obvious thing to defer
  rather than compromising on the auto-discovery rails or the catalog coverage.

## Trigger

Revisit during Phase E scoping in the v1.2 gallery milestone. If A–D fit in
budget, include. If not, keep this seed alive for a v1.3.

## Design sketch (rough, not locked)

- Backend: a single `theme_tokens` handler that returns an SDUI panel with sliders
  for numeric tokens and colour inputs for OKLCH tokens. No state on the server —
  every change is applied client-side.
- Frontend: a new tiny component type `TokenEditor` (or reuse existing input
  widgets with a change-action that sets a CSS var instead of POSTing). Latter is
  preferred — no new component type, just a `target: "--primary"` action hook.
- Export: a read-only textarea rendered with the current token set formatted as a
  pasteable `@theme` / `:root` block.

## Related

- See `.planning/notes/2026-04-21-gallery-demo-architecture.md` (Content shape →
  Theme tools).
