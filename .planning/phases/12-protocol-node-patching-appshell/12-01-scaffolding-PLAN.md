---
phase: 12
plan: 01
type: execute
wave: 0
depends_on: []
files_modified:
  - frontend/src/app.css
  - frontend/src/lib/components/nav/SideNav.svelte
  - frontend/src/lib/components/core/Surface.svelte
  - frontend/src/lib/components/ui/sidebar/
  - frontend/src/lib/components/ui/sonner/
  - frontend/src/lib/components/shell/AppShell.svelte
  - frontend/src/lib/components/shell/AppShell.browser-test.ts
  - frontend/src/lib/components/core/SurfaceMount.svelte
  - frontend/src/lib/components/core/SurfaceMount.browser-test.ts
  - frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts
  - frontend/src/lib/store/surfaces.svelte.test.ts
  - backend/crates/marionette/src/builders/app_shell.rs
  - backend/crates/marionette/src/builders/mod.rs
  - frontend/tests/e2e/shell-nav.spec.ts
  - frontend/tests/e2e/node-patch-focus.spec.ts
autonomous: true
requirements: [SHELL-03]
nyquist_compliant: true
tags: [scaffolding, shadcn, css-tokens]
must_haves:
  truths:
    - "shadcn-svelte Sidebar primitive is installed at frontend/src/lib/components/ui/sidebar/"
    - "Toast primitive (Sonner or shadcn Toast) is installed at frontend/src/lib/components/ui/"
    - "CSS token --sidebar (no suffix) exists in app.css in both :root and .dark; --sidebar-background is gone"
    - "No source file references bg-sidebar-background any more"
    - "Empty scaffold files exist so Wave 1+ tests can compile against placeholders"
  artifacts:
    - path: "frontend/src/app.css"
      provides: "renamed --sidebar token"
      contains: "--sidebar:"
    - path: "frontend/src/lib/components/ui/sidebar/index.ts"
      provides: "shadcn Sidebar primitives"
    - path: "frontend/src/lib/components/shell/AppShell.svelte"
      provides: "AppShell component scaffold"
    - path: "frontend/src/lib/components/core/SurfaceMount.svelte"
      provides: "SurfaceMount component scaffold"
    - path: "backend/crates/marionette/src/builders/app_shell.rs"
      provides: "AppShell builder scaffold"
  key_links:
    - from: "frontend/src/lib/components/nav/SideNav.svelte"
      to: "app.css --sidebar token"
      via: "bg-sidebar class"
      pattern: "bg-sidebar(?![-\\w])"
---

<objective>
Install the shadcn-svelte Sidebar and toast primitives, rename the CSS sidebar tokens to the canonical shadcn names, and create empty scaffold files for every net-new source and test file the phase needs. This unblocks Waves 1+ so every task can see the file it is about to edit.

Purpose: Waves 1+ edit these files concurrently; if Wave 0 does not create the scaffolds, Nyquist validation (which requires `<automated>` commands to target files that already exist) blocks planning. Also closes the `--sidebar-*` token-name trap from RESEARCH.md Pitfall 1 before AppShell composition work.

Output: Installed shadcn Sidebar + Toast primitives, renamed CSS tokens with zero `--sidebar-background` residue, and empty scaffold files for all AppShell / SurfaceMount / focus-preservation / builder / E2E test targets.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@.planning/phases/12-protocol-node-patching-appshell/12-VALIDATION.md
@frontend/src/app.css
@frontend/src/lib/components/nav/SideNav.svelte
@frontend/src/lib/components/core/Surface.svelte
@.planning/codebase/CONVENTIONS.md

<interfaces>
Current wrong token names in `frontend/src/app.css` `:root` block (lines 29-36):

```css
--sidebar-background: oklch(0.985 0 0);
--sidebar-foreground: oklch(0.141 0.005 285.823);
--sidebar-primary: oklch(0.21 0.006 285.885);
--sidebar-primary-foreground: oklch(0.985 0 0);
--sidebar-accent: oklch(0.967 0.001 286.375);
--sidebar-accent-foreground: oklch(0.21 0.006 285.885);
--sidebar-border: oklch(0.92 0.004 286.32);
--sidebar-ring: oklch(0.705 0.015 286.067);
```

Target names per shadcn-svelte Sidebar registry (same values, one rename only):

```css
--sidebar: oklch(0.985 0 0);                          /* was --sidebar-background */
--sidebar-foreground: oklch(0.141 0.005 285.823);     /* unchanged */
--sidebar-primary: oklch(0.21 0.006 285.885);         /* unchanged */
--sidebar-primary-foreground: oklch(0.985 0 0);       /* unchanged */
--sidebar-accent: oklch(0.967 0.001 286.375);         /* unchanged */
--sidebar-accent-foreground: oklch(0.21 0.006 285.885); /* unchanged */
--sidebar-border: oklch(0.92 0.004 286.32);           /* unchanged */
--sidebar-ring: oklch(0.705 0.015 286.067);           /* unchanged */
```

`@theme inline` mapping (lines 87-94): rename one line only.

```css
--color-sidebar: var(--sidebar);        /* was --color-sidebar-background: var(--sidebar-background) */
```

Current `bg-sidebar-background` usages found by grep (must all become `bg-sidebar`):
- `frontend/src/lib/components/nav/SideNav.svelte:20` — `class="flex flex-col h-full bg-sidebar-background"`
- `frontend/src/lib/components/core/Surface.svelte:16` — `sidebar: 'bg-sidebar-background border-r border-sidebar-border p-4 overflow-y-auto w-64 shrink-0'`

`NavItem.svelte:48` uses `bg-sidebar-accent` — that token name is already canonical, do NOT touch it.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Install shadcn Sidebar + toast primitive and verify components.json registry</name>
  <read_first>
    - frontend/components.json
    - frontend/package.json
    - frontend/src/lib/components/ui/ (directory listing — confirm what is already installed)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md §Standard Stack and Pitfall 1
  </read_first>
  <action>
1. `cd frontend && ls src/lib/components/ui/` — confirm the current inventory is exactly: badge, button, card, checkbox, dialog, input, label, select, separator, skeleton, table. Note any discrepancies.
2. Run `cd frontend && pnpm dlx shadcn-svelte@latest add sidebar -y` (fallback to `npx shadcn-svelte@latest add sidebar -y` if `pnpm dlx` is unavailable). This creates `frontend/src/lib/components/ui/sidebar/` with the standard sub-modules (`index.ts` plus the block's `.svelte` files — exact names come from the registry).
3. Check for toast primitive: `ls src/lib/components/ui/ | grep -iE "toast|sonner"`.
   - If nothing matches, install sonner: `cd frontend && pnpm dlx shadcn-svelte@latest add sonner -y`. (Phase 11 D-04 picked shadcn Toast; the current shadcn-svelte registry ships sonner as the toast block, so this honors the decision.)
   - If it exists, skip the install.
4. If the installer modified `frontend/src/app.css` by adding or rewriting `--sidebar-*` tokens, inspect the diff. Keep the installer's canonical names but preserve existing OKLCH values from the pre-install file. Duplicate token declarations must be collapsed manually in Task 2.
5. Do NOT delete `ConnectionBanner.svelte` yet (that is Plan 06's responsibility).
6. Run `cd frontend && pnpm install` if the installer added new dependencies.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; ls src/lib/components/ui/sidebar/ &gt;/dev/null &amp;&amp; ls src/lib/components/ui/ | grep -iE 'sonner|toast' &gt;/dev/null</automated>
  </verify>
  <acceptance_criteria>
    - `ls frontend/src/lib/components/ui/sidebar/` exits 0 and lists at least `index.ts` plus one `.svelte` file
    - `ls frontend/src/lib/components/ui/ | grep -iE 'sonner|toast'` exits 0
    - `frontend/package.json` is valid JSON; `pnpm install` (or `npm install`) exits 0 if rerun
    - `frontend/components.json` still exists and is valid JSON
    - `cd frontend && npm run check` exits 0 (pre-existing warnings tolerated, but no new errors)
  </acceptance_criteria>
  <done>Sidebar primitive directory exists under `frontend/src/lib/components/ui/sidebar/` with shadcn-svelte-generated exports. A toast primitive exists under `frontend/src/lib/components/ui/` (sonner or toast subdir). Type-check is green.</done>
</task>

<task type="auto">
  <name>Task 2: Rename --sidebar-background to --sidebar in app.css and audit class usages</name>
  <read_first>
    - frontend/src/app.css
    - frontend/src/lib/components/nav/SideNav.svelte
    - frontend/src/lib/components/core/Surface.svelte
    - frontend/src/lib/components/nav/NavItem.svelte (confirm bg-sidebar-accent stays unchanged)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pitfall 1
  </read_first>
  <action>
1. In `frontend/src/app.css`:
   - In the `:root` block (around lines 29-36), rename `--sidebar-background: oklch(0.985 0 0);` to `--sidebar: oklch(0.985 0 0);`. All other `--sidebar-*` tokens (foreground, primary, primary-foreground, accent, accent-foreground, border, ring) stay unchanged.
   - In the `.dark` block (around lines 58-65), rename `--sidebar-background: oklch(0.21 0.006 285.885);` to `--sidebar: oklch(0.21 0.006 285.885);`. Other tokens unchanged.
   - In the `@theme inline { ... }` block (around lines 87-94), rename `--color-sidebar-background: var(--sidebar-background);` to `--color-sidebar: var(--sidebar);`. Other `--color-sidebar-*` lines unchanged.
   - If Task 1's installer already added shadcn Sidebar tokens, deduplicate so exactly one declaration per token remains in each block. Prefer the existing OKLCH values (keep the repo's Zinc palette).
2. In `frontend/src/lib/components/nav/SideNav.svelte` line 20, replace `bg-sidebar-background` with `bg-sidebar` in the `<nav>` element's `class` attribute. The full attribute must read `class="flex flex-col h-full bg-sidebar"`. Leave the rest of the markup alone.
3. In `frontend/src/lib/components/core/Surface.svelte` line 16, replace `bg-sidebar-background` with `bg-sidebar` in the `sidebar:` layout class string. Full line becomes: `sidebar: 'bg-sidebar border-r border-sidebar-border p-4 overflow-y-auto w-64 shrink-0',`. Leave the rest of the layout map alone. (Note: this code is removed in Plan 06 when `+layout.svelte` stops mounting a top-level `sidebar` surface, but the rename must happen now so Tailwind builds keep working in the interim.)
4. Run `cd frontend && grep -rn 'sidebar-background' src/` — must return ZERO hits (`bg-sidebar-background`, `--sidebar-background`, `--color-sidebar-background` all gone).
5. Do NOT change `bg-sidebar-accent` / `bg-sidebar-primary` / `bg-sidebar-border` — those token names are already canonical in `NavItem.svelte` and are unaffected.
6. `cd frontend && npm run check` — must be green.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; ! grep -rn 'sidebar-background' src/ &amp;&amp; grep -q '^\s*--sidebar:' src/app.css &amp;&amp; grep -q 'bg-sidebar"' src/lib/components/nav/SideNav.svelte</automated>
  </verify>
  <acceptance_criteria>
    - `grep -rn 'sidebar-background' frontend/src/` returns zero lines (exit 1)
    - `grep -n '^[[:space:]]*--sidebar:' frontend/src/app.css` returns at least 2 lines (one each for `:root` and `.dark`)
    - `grep -n '^[[:space:]]*--color-sidebar:' frontend/src/app.css` returns 1 line inside the `@theme inline` block
    - `frontend/src/lib/components/nav/SideNav.svelte` `<nav>` element contains the literal class `bg-sidebar` but NOT `bg-sidebar-background`
    - `frontend/src/lib/components/core/Surface.svelte` layout map `sidebar:` entry contains `bg-sidebar border-r border-sidebar-border` exactly, no `-background` suffix
    - `cd frontend && npm run check` exits 0
  </acceptance_criteria>
  <done>All references to `--sidebar-background` / `bg-sidebar-background` are gone. Token renamed to shadcn canonical name. Type check is green.</done>
</task>

<task type="auto">
  <name>Task 3: Create empty scaffold files for all Wave 1+ targets (tests, components, builder)</name>
  <read_first>
    - .planning/phases/12-protocol-node-patching-appshell/12-VALIDATION.md §Wave 0 Requirements
    - backend/crates/marionette/src/builders/mod.rs
    - frontend/src/lib/components/core/Surface.svelte (so the SurfaceMount scaffold props match Surface's expected signature)
  </read_first>
  <action>
Create the following files as MINIMAL compilable placeholders. Every file MUST either compile as-is or use `test.todo()` / `test.skip()` so test runners stay green. NO real implementation — that lands in later plans.

1. **`frontend/src/lib/components/shell/AppShell.svelte`** (new directory `shell/`):
```svelte
<script lang="ts">
	import type { ComponentAction } from '$lib/transport/messages';
	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();
	// Scaffold — real implementation in Plan 06.
	void props;
	void bind;
	void action;
	void surface;
</script>

<div data-component="app-shell">AppShell scaffold — implemented in Plan 06</div>
```

2. **`frontend/src/lib/components/shell/AppShell.browser-test.ts`**:
```typescript
import { test } from 'vitest';

test.todo('AppShell renders sidebar/header/footer/main from slot node IDs — implemented in Plan 06');
test.todo('AppShell Sidebar.Trigger is visible in header — Plan 06');
test.todo('AppShell mobile viewport collapses sidebar into sheet — Plan 06');
```

3. **`frontend/src/lib/components/core/SurfaceMount.svelte`**:
```svelte
<script lang="ts">
	import type { ComponentAction } from '$lib/transport/messages';
	let {
		props = {},
		bind,
		action,
		surface,
	}: {
		props: Record<string, unknown>;
		bind?: string;
		action?: ComponentAction;
		surface: string;
	} = $props();
	// Scaffold — real implementation in Plan 06.
	void props;
	void bind;
	void action;
	void surface;
</script>

<div data-component="surface-mount">SurfaceMount scaffold — implemented in Plan 06</div>
```

4. **`frontend/src/lib/components/core/SurfaceMount.browser-test.ts`**:
```typescript
import { test } from 'vitest';

test.todo('SurfaceMount with name=content mounts the content sub-surface — Plan 06');
test.todo('SurfaceMount with unknown name shows LoadingSkeleton — Plan 06');
```

5. **`frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts`**:
```typescript
import { test } from 'vitest';

test.todo('patch to sibling node preserves focus and cursor on focused input — Plan 04');
```

6. **`frontend/src/lib/store/surfaces.svelte.test.ts`**:
```typescript
import { test } from 'vitest';

test.todo('setNode mutates nodes[id] in place — Plan 04');
test.todo('deleteNode removes node from tree.nodes — Plan 04');
test.todo('setChildren replaces children array in place — Plan 04');
test.todo('insertChild inserts at index — Plan 04');
test.todo('removeChild removes matching childId — Plan 04');
test.todo('gcOrphans removes unreachable nodes via BFS from root — Plan 04');
```

7. **`backend/crates/marionette/src/builders/app_shell.rs`**:
```rust
//! Hand-written AppShell builder — scaffold.
//! Real implementation lands in Plan 05.

#![allow(dead_code, unused_imports)]

use marionette_protocol::Component;

/// Placeholder. Real `AppShell::new()` builder in Plan 05.
pub struct AppShell;

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_compiles() {
        // Real tests land in Plan 05.
    }
}
```

8. **Update `backend/crates/marionette/src/builders/mod.rs`** to add `pub mod app_shell;` and `pub use app_shell::*;`. New file contents:
```rust
pub mod node;
pub mod standard;
pub mod app_shell;

pub use node::*;
pub use standard::*;
pub use app_shell::*;
```

9. **`frontend/tests/e2e/shell-nav.spec.ts`**:
```typescript
import { test } from '@playwright/test';

test.skip('CRM nav renders inside AppShell and clicking items updates content sub-surface — Plan 08', () => {});
```

10. **`frontend/tests/e2e/node-patch-focus.spec.ts`**:
```typescript
import { test } from '@playwright/test';

test.skip('country-select change swaps sibling form fields with preserved focus — Plan 08', () => {});
```

Do NOT create a new `surfaces.svelte.ts` (Plan 04 REWRITES the existing file) and do NOT create new `data.rs` / `messages.rs` / `standard.rs` files — those are modified in place in Plans 02 and 05.
  </action>
  <verify>
    <automated>test -f frontend/src/lib/components/shell/AppShell.svelte &amp;&amp; test -f frontend/src/lib/components/shell/AppShell.browser-test.ts &amp;&amp; test -f frontend/src/lib/components/core/SurfaceMount.svelte &amp;&amp; test -f frontend/src/lib/components/core/SurfaceMount.browser-test.ts &amp;&amp; test -f frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts &amp;&amp; test -f frontend/src/lib/store/surfaces.svelte.test.ts &amp;&amp; test -f backend/crates/marionette/src/builders/app_shell.rs &amp;&amp; test -f frontend/tests/e2e/shell-nav.spec.ts &amp;&amp; test -f frontend/tests/e2e/node-patch-focus.spec.ts &amp;&amp; cd backend &amp;&amp; cargo check -p marionette</automated>
  </verify>
  <acceptance_criteria>
    - All 9 scaffold files exist at the listed absolute paths
    - `backend/crates/marionette/src/builders/mod.rs` contains the literal lines `pub mod app_shell;` and `pub use app_shell::*;`
    - `cd backend && cargo check -p marionette` exits 0 (scaffold compiles)
    - `cd frontend && npm run check` exits 0 (placeholder Svelte files type-check)
    - `grep -q 'test.todo' frontend/src/lib/store/surfaces.svelte.test.ts` succeeds
    - No real implementation leaked: `grep -l 'Sidebar.Provider\|setNode\|insertChild' frontend/src/lib/components/shell/AppShell.svelte frontend/src/lib/components/core/SurfaceMount.svelte` returns zero matches (that code lives in Plans 04/05/06)
  </acceptance_criteria>
  <done>All scaffold files exist and compile. Backend and frontend type checks pass. No real logic in any scaffold (`test.todo` / `test.skip` / `void` discarded props only).</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries
No user-affecting surface in this plan — install + file rename + empty scaffolds only. No new data paths, no new auth checks, no external I/O beyond `pnpm dlx shadcn-svelte@latest add` (reads from npm registry). Per threat_model_gate, pure infrastructure plans may use a single-line rationale.

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-01 | Tampering | `pnpm dlx shadcn-svelte@latest add` pulls from public registry | accept | Upstream is the same registry the project already uses for all shadcn components; no new supply-chain risk vs. Phases 10/11 |
</threat_model>

<verification>
- `ls frontend/src/lib/components/ui/sidebar/` succeeds with the shadcn-generated files present
- `grep -rn 'sidebar-background' frontend/src/` returns zero hits
- `cd backend && cargo check -p marionette` exits 0
- `cd frontend && npm run check` exits 0
- All 9 scaffold files from Task 3 exist
</verification>

<success_criteria>
- Shadcn Sidebar primitive installed and available under `$lib/components/ui/sidebar`
- Toast primitive (sonner or shadcn toast) installed and available under `$lib/components/ui/`
- `--sidebar-background` token renamed to `--sidebar` in both `:root`, `.dark`, and `@theme inline` blocks of `app.css`
- All `bg-sidebar-background` class usages replaced with `bg-sidebar`
- Empty scaffold files exist for AppShell, SurfaceMount, focus-preservation test, surfaces unit test, AppShell builder, shell-nav E2E, node-patch-focus E2E
- `cargo check -p marionette` and `npm run check` both exit 0
- No real implementation leaked into scaffolds
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-01-SUMMARY.md` recording:
- Which shadcn primitives were added (with version from components.json)
- Confirmed list of files created under `frontend/src/lib/components/ui/sidebar/`
- Actual toast primitive chosen (sonner vs. toast) and rationale if it differed from Phase 11 D-04
- `grep -c` counts before/after the token rename
- Any installer-added CSS that had to be reconciled manually
</output>
