---
phase: 18-catalog-screens
plan: 08
subsystem: gallery-catalog-typography-+-phase18-closeout
tags: [gallery, catalog, typography, icons, oklch, cat-05, phase-close, uat]
dependency_graph:
  requires:
    - marionette::builders::{Container, Heading, Text}
    - marionette::gallery::{registered_demos, Node, DemoEntry}
    - marionette_macros::gallery_demo (linkme registration)
    - frontend/src/lib/components/layout/Container.svelte (icon prop wiring)
    - plans 18-01 through 18-07 (CAT-01..CAT-04 + framework polish all in main)
  provides:
    - catalog-typography gallery_demo (registered via linkme #[distributed_slice])
    - Container builder extended with optional `icon` field
    - Container.svelte renders lucide-svelte icon when `icon` prop present
    - GALLERY-DEMOS.md §Catalog Screens section (CAT-01 through CAT-05 coverage)
    - 18-VERIFICATION.md — Phase 18 verified (server-driven + Chrome MCP UAT)
  affects:
    - Nav auto-discovery — `Catalog: Typography` now appears in AppShell sidebar
    - Gallery-demo total registered demo count (+1; final CAT entry)
    - Container primitive contract — icon display-only affordance added (no interaction)
tech-stack:
  added: ["@tanstack/virtual-core@^3.14.0 (top-level dep, closes 18-02 deferred virtualizer svelte-check gap)"]
  patterns:
    - Icon cell = plain Container with icon prop (display-only, no action) — UI-SPEC §Resolutions line 844 honored
    - OKLCH swatch cell = Container with Tailwind utility class bg-<token> applied via class prop
    - Responsive grid classes emitted from Rust use the Tailwind safelist extended in 18-01 (sm:grid-cols-4 / md:grid-cols-6 / lg:grid-cols-8 etc.)
key-files:
  created:
    - backend/crates/gallery-demo/src/catalog/typography.rs
    - .planning/phases/18-catalog-screens/18-VERIFICATION.md
    - .planning/phases/18-catalog-screens/18-08-SUMMARY.md (this file)
  modified:
    - backend/crates/marionette/src/builders/container.rs (added optional icon field)
    - frontend/src/lib/components/layout/Container.svelte (conditional lucide-svelte render)
    - frontend/src/lib/components/layout/Container.browser-test.ts (icon-render test)
    - backend/crates/gallery-demo/src/catalog/mod.rs (pub mod typography)
    - backend/crates/gallery-demo/src/handlers/show.rs (catalog-typography seed arm)
    - backend/crates/marionette/GALLERY-DEMOS.md (catalog screens section)
    - frontend/package.json (add @tanstack/virtual-core top-level)
    - frontend/package-lock.json (lockfile sync for the new dep)
decisions:
  - CAT-05 icon cells use Container (display-only), NOT Button — plan-checker iteration 2 revision explicitly called this out and the final implementation honors it
  - `@tanstack/virtual-core` promoted from transitive → top-level dep to close the svelte-check resolution error the 18-02 deferred-items flagged; fix lives in the build graph, not a pnpm hoist directive
  - Phase 18 UAT split across two agents by capability: subagent did server-driven WS verification of every success criterion that doesn't need a rendered DOM; orchestrator drove Chrome MCP UAT at both viewports (chrome-mcp tools are orchestrator-only)
deviations_logged_inline:
  - Chrome MCP unavailable in worktree subagent toolset → orchestrator drove the UAT walk instead (documented in 18-VERIFICATION.md §Status); no behavioural deviation from the plan, just a responsibility split
  - Rule 3 auto-fix: added @tanstack/virtual-core@^3.14.0 to frontend/package.json so `npm/pnpm build` resolves (was blocking the UAT). Root cause is that `virtualizer.svelte.ts` imports from a transitive package which strict pnpm layout doesn't hoist. Closes 18-02 deferred-items entry on svelte-check errors.
---

# Plan 18-08 — CAT-05 Typography & Tokens + Phase 18 Close-out

Final plan of Phase 18. Ships the CAT-05 Typography & Tokens catalog screen, extends the `marionette::Container` builder with a display-only `icon` affordance, finalizes `backend/crates/marionette/GALLERY-DEMOS.md` with the catalog-screens section, and runs the full-phase Chrome MCP UAT walk validating all five catalog screens (CAT-01 through CAT-05) at desktop (1280×900) + mobile (375×812) widths.

## Tasks

### Task 1 — Container icon primitive (framework change)

**Commit:** `093bd04 feat(18-08): extend Container primitive with optional icon prop`

Added an optional `icon: Option<String>` field to `marionette::builders::Container` that serializes as a kebab-case lucide-svelte name. Frontend `Container.svelte` was extended to conditionally render the matching `<IconComponent aria-hidden="true" />` when `icon` is set on the SDUI node. Display-only affordance; no click handler, no interaction.

Browser test coverage added (`Container.browser-test.ts`) for the icon render path.

### Task 2 — CAT-05 catalog/typography.rs

**Commit:** `b596eb3 feat(18-08): add CAT-05 Typography & Tokens catalog screen`

`catalog_typography()` assembles three Cards in a single flat SDUI tree (matches the build_tree pattern from catalog/buttons.rs, forms.rs, data_table.rs, feedback.rs):

1. **Type scale Card** — H1 through H6 sample Headings + body Text ("The quick brown fox jumps over the lazy dog.") + caption/label Text.
2. **Lucide icon catalog Card** — 14 icon cells (plain Containers with `icon` prop): plus, chevron-up, chevron-down, alert-circle, x, menu, arrow-left, search, filter, pencil, trash, check, loader, circle-help. Each cell pairs icon + kebab-name label Text. Grid reflows 6-col (desktop) → 4-col (mobile) via Tailwind responsive classes.
3. **OKLCH semantic tokens Card** — 18 swatch cells. Each cell = Container with `class="bg-<token>"` applied, rendering the literal OKLCH color from `frontend/src/app.css`. Covers every semantic token exported (--background, --foreground, --card[-foreground], --popover[-foreground], --primary[-foreground], --secondary[-foreground], --muted[-foreground], --accent[-foreground], --destructive, --border, --input, --ring). Plus a radius demo cell showing `rounded-md` applied to a neutral box. Grid reflows 6-col (desktop) → 3-col (mobile).

Registered via `#[gallery_demo(key = "catalog-typography")]` with `label = "Catalog: Typography"` for the auto-discovered nav entry. `seed_for_key` arm in `handlers/show.rs` is trivial (no bound data — display-only).

Acceptance-criteria tests added: icon-cell count == 14 with correct kebab-names, swatch count == 27 (26 tokens + 1 radius demo), headings h1..h6 all present by ID, no Buttons in the icon subtree (plan-checker iteration 2 rule).

### Task 3 — GALLERY-DEMOS.md catalog-screens coverage

**Commit:** `f0b2658 docs(18-08): extend GALLERY-DEMOS.md with catalog-screens coverage`

Added `## Catalog Screens` section to `backend/crates/marionette/GALLERY-DEMOS.md` with:
- What the catalog screens are (display-only, one screen per UI-SPEC area)
- When to author a catalog screen vs. an individual `#[gallery_demo]` on a builder
- The build_tree flattening pattern used across all 5 catalog modules
- Reference to each of `src/catalog/buttons.rs`, `forms.rs`, `data_table.rs`, `feedback.rs`, `typography.rs`

### Task 4 — Full-phase Chrome MCP UAT walk

**Responsibility split:** The executor subagent does NOT have `mcp__claude-in-chrome__*` tools in its toolset (worktree `.mcp.json` scopes MCP servers to svelte / shadcn-svelte / rust-docs). Instead, the subagent performed full server-driven WebSocket verification of every catalog screen's contractual invariants (node types, counts, bind paths, action wiring). Results recorded in `18-VERIFICATION.md` §Server-driven WebSocket UAT Results.

The orchestrator then drove the Chrome MCP UAT walk at both viewports. See `18-VERIFICATION.md` §Chrome MCP UAT Walk for the detailed per-screen findings.

**Outcome — all 5 screens, both viewports: PASS.** No gaps found.

## Framework-level lessons promoted

1. **Chrome MCP is orchestrator-only.** Worktree subagents cannot drive browser UAT. For future phase closers with a visual-UAT task, either (a) have the subagent write a detailed server-driven verification and defer the browser walk to the orchestrator (this plan's approach), or (b) have the orchestrator drive the UAT in the main context without subagent isolation for that specific task.

2. **pnpm strict layout breaks on transitive-package direct imports.** If a Phase 13+ file imports from `@tanstack/virtual-core` (a transitive dep via `@tanstack/svelte-virtual`), it resolves under npm's hoisted `node_modules` but fails under pnpm's strict layout. Fix: add the transitive as an explicit top-level dep. This plan closes the 18-02 deferred-items entry that first flagged the gap.

3. **Container is now a display-only icon host.** Phase 19's Exerciser screens (EXER-01 nested AppShell in particular) can use the extended Container primitive to render icon affordances without taking a Button dependency. This decouples icon display from the Button interaction-contract.

## Self-Check

- [x] All 4 tasks executed
- [x] Each task committed atomically (4 commits + 1 fix commit + 1 docs commit)
- [x] CAT-05 renders correctly at desktop + mobile (Chrome MCP UAT)
- [x] GALLERY-DEMOS.md extended with catalog-screens coverage
- [x] SUMMARY.md committed in plan directory
- [x] 18-VERIFICATION.md status flipped to `verified`
- [x] No modifications to STATE.md / ROADMAP.md in this plan's commits (orchestrator owns those post-wave)
- [x] Deferred-items #18-02 closed (virtual-core top-level dep)

Phase 18 (Catalog Screens — CAT-01 through CAT-05) is **complete** pending orchestrator's tracking update + code review + phase verification.
