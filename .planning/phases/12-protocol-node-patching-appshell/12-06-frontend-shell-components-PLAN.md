---
phase: 12
plan: 06
type: execute
wave: 3
depends_on: [12-04]
files_modified:
  - frontend/src/lib/components/shell/AppShell.svelte
  - frontend/src/lib/components/shell/AppShell.browser-test.ts
  - frontend/src/lib/components/core/SurfaceMount.svelte
  - frontend/src/lib/components/core/SurfaceMount.browser-test.ts
  - frontend/src/lib/registry/defaults.ts
  - frontend/src/lib/index.ts
  - frontend/src/routes/+layout.svelte
  - frontend/src/lib/components/core/ConnectionBanner.svelte
  - frontend/src/lib/components/core/ConnectionBanner.browser-test.ts
autonomous: true
requirements: [SHELL-01, SHELL-02, SHELL-03, SHELL-04]
nyquist_compliant: true
tags: [frontend, svelte, shell, shadcn]
must_haves:
  truths:
    - "AppShell.svelte renders shadcn Sidebar.Provider + Sidebar.Root + Sidebar.Content + Sidebar.Inset + Sidebar.Trigger"
    - "AppShell resolves slot node IDs from props.sidebarNodeId/headerNodeId/footerNodeId/mainNodeId/popupsNodeId/toastsNodeId and renders each via NodeRenderer"
    - "SurfaceMount.svelte renders <Surface name={props.name}/>"
    - "registry/defaults.ts registers 'app-shell' and 'surface-mount' component types"
    - "routes/+layout.svelte contains ONLY <Surface name='main'/> (plus app.css import and children render)"
    - "ConnectionBanner.svelte and ConnectionBanner.browser-test.ts are deleted"
    - "lib/index.ts no longer exports ConnectionBanner"
  artifacts:
    - path: "frontend/src/lib/components/shell/AppShell.svelte"
      provides: "AppShell Svelte implementation"
      contains: "Sidebar.Provider"
    - path: "frontend/src/lib/components/core/SurfaceMount.svelte"
      provides: "SurfaceMount Svelte implementation"
      contains: "<Surface name={props.name"
    - path: "frontend/src/lib/registry/defaults.ts"
      provides: "app-shell and surface-mount registered"
      contains: "'app-shell'"
    - path: "frontend/src/routes/+layout.svelte"
      provides: "collapsed single-surface root"
      contains: "Surface name=\"main\""
  key_links:
    - from: "AppShell.svelte"
      to: "NodeRenderer via slot IDs from props"
      via: "$derived props.sidebarNodeId"
      pattern: "props\\.sidebarNodeId"
    - from: "registry/defaults.ts"
      to: "AppShell + SurfaceMount imports"
      via: "registerAll map"
      pattern: "'app-shell':\\s*AppShell"
---

<objective>
Implement the frontend Svelte components for the sub-surface architecture: `AppShell.svelte` (using shadcn Sidebar primitives), `SurfaceMount.svelte` (trivial recursive Surface mount), register both in the component registry, collapse `routes/+layout.svelte` to a single `<Surface name="main"/>`, and retire `ConnectionBanner.svelte`. Implements D-B1, D-B2, D-B8, D-B9, D-B10.

Purpose: Part B visual layer. Depends on Plan 04 for fine-grained reactivity in the surface store (SurfaceMount needs `getSurfaceTree` to respect per-key reactivity so sub-surface rendering is stable).

Output: Working AppShell + SurfaceMount components registered and usable. Top-level layout collapses. Browser tests assert slot rendering + recursive surface mount.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@frontend/src/lib/components/shell/AppShell.svelte
@frontend/src/lib/components/core/SurfaceMount.svelte
@frontend/src/lib/components/core/Surface.svelte
@frontend/src/lib/components/core/NodeRenderer.svelte
@frontend/src/lib/components/ui/sidebar/
@frontend/src/lib/registry/defaults.ts
@frontend/src/lib/index.ts
@frontend/src/routes/+layout.svelte
@frontend/src/lib/components/core/ConnectionBanner.svelte
@frontend/src/lib/store/surfaces.svelte.ts

<interfaces>
Shadcn Sidebar primitives installed in Plan 01 live at `frontend/src/lib/components/ui/sidebar/index.ts`. The standard export shape is `import * as Sidebar from '$lib/components/ui/sidebar';` — then `<Sidebar.Provider>`, `<Sidebar.Root>`, `<Sidebar.Content>`, `<Sidebar.Inset>`, `<Sidebar.Trigger>`. Exact sub-component names come from the shadcn-svelte registry output; if any differ (e.g., `Sidebar.Header` vs `Sidebar.SidebarHeader`), follow whatever `index.ts` actually exports. Do NOT hand-roll Sidebar styling.

`NodeRenderer.svelte` (verified) accepts `{ nodeId: string, nodes: Record<string, ComponentNode>, surface: string }`.

`Surface.svelte` (verified) reads `getSurfaceTree(name)` via `$derived`. Recursion is safe (RESEARCH Pattern 2): `<SurfaceMount>` → `<Surface name={props.name}/>` → `<NodeRenderer .../>` with no mount loops.

All SDUI components accept `props, bind?, action?, surface, children?` per the CONVENTIONS.md contract. Both AppShell and SurfaceMount honor this — `children?` is unused (slot children live at the top of the adjacency list, referenced by IDs in props).

Current `routes/+layout.svelte` (verified):
```svelte
<script>
	import '../app.css';
	import { ConnectionBanner, Surface } from '$lib';
	let { children } = $props();
</script>

<ConnectionBanner />
<div class="flex h-screen">
	<Surface name="sidebar" />
	<Surface name="main" />
</div>
<Surface name="modal" />
<Surface name="toast" />
{@render children()}
```

Target:
```svelte
<script>
	import '../app.css';
	import { Surface } from '$lib';
	let { children } = $props();
</script>

<Surface name="main" />
{@render children()}
```

`registry/defaults.ts` `registerAll` block (verified) currently registers 18 types. This plan adds two more: `'app-shell': AppShell` and `'surface-mount': SurfaceMount`.

`lib/index.ts` re-exports `ConnectionBanner` (verified line 6). After deletion, that export line must go.

Phase 11 D-04 picked shadcn Toast and a `ToastSurface.svelte` is registered under `'toast'` in `registry/defaults.ts:46`. Do NOT delete it — it stays registered for the `toasts` sub-surface's content. The `ConnectionBanner` retirement does NOT affect toasts.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement SurfaceMount.svelte + register + browser test</name>
  <read_first>
    - frontend/src/lib/components/core/SurfaceMount.svelte (scaffold from Plan 01)
    - frontend/src/lib/components/core/SurfaceMount.browser-test.ts (scaffold from Plan 01)
    - frontend/src/lib/components/core/Surface.svelte
    - frontend/src/lib/components/core/LoadingSkeleton.svelte
    - frontend/src/lib/store/surfaces.svelte.ts
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pattern 2 (recursion safety)
  </read_first>
  <behavior>
    - `<SurfaceMount props={{ name: 'content' }} surface="main" />` mounts a nested `<Surface name="content"/>`
    - If surface `content` is not yet rendered, the inner Surface shows its LoadingSkeleton
    - After `setSurfaceTree('content', 'root', { root: ... })`, the inner Surface renders the content tree
    - Browser test asserts that the mounted sub-surface text appears in the DOM after a `setSurfaceTree` call
  </behavior>
  <action>
1. REPLACE the scaffold contents of `frontend/src/lib/components/core/SurfaceMount.svelte` with:

```svelte
<script lang="ts">
	import Surface from './Surface.svelte';
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

	// `surface` (parent) is intentionally unused — SurfaceMount is a pure
	// redirection to the sub-surface identified by `props.name`.
	void bind;
	void action;
	void surface;

	let name = $derived((props.name as string | undefined) ?? '');
</script>

{#if name}
	<Surface name={name} />
{/if}
```

2. REPLACE the scaffold contents of `frontend/src/lib/components/core/SurfaceMount.browser-test.ts` with real tests:

```typescript
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import SurfaceMount from './SurfaceMount.svelte';
import {
	setSurfaceTree,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { registerDefaults } from '$lib/registry/defaults';

const CHILD_SURFACE = 'child-test';

beforeEach(() => {
	resetStore(CHILD_SURFACE);
	clearSurfaceTree(CHILD_SURFACE);
	registerDefaults();
});

test('SurfaceMount with props.name mounts the named sub-surface', async () => {
	// Arrange: pre-populate the child sub-surface before mount
	setFullState(CHILD_SURFACE, {});
	setSurfaceTree(CHILD_SURFACE, 'root', {
		root: { type: 'heading', props: { text: 'Child Content Rendered' } },
	});

	const screen = await render(SurfaceMount, {
		props: {
			props: { name: CHILD_SURFACE },
			surface: 'main',
		},
	});
	await tick();

	expect(screen.baseElement.textContent).toContain('Child Content Rendered');
});

test('SurfaceMount with a surface that has no tree shows LoadingSkeleton', async () => {
	const screen = await render(SurfaceMount, {
		props: {
			props: { name: 'not-rendered-yet' },
			surface: 'main',
		},
	});
	await tick();

	// Surface.svelte renders LoadingSkeleton when its tree is undefined.
	// The skeleton is visible; assert nothing else crashed.
	expect(screen.baseElement.querySelector('[data-surface="not-rendered-yet"]')).not.toBeNull();
});
```

3. Add `SurfaceMount` to `frontend/src/lib/registry/defaults.ts`:
   - Add import: `import SurfaceMount from '../components/core/SurfaceMount.svelte';`
   - Add entry to the `registerAll` map: `'surface-mount': SurfaceMount,`

4. Run `cd frontend && npm run check` — must be green.

5. Run `cd frontend && npx vitest --config vitest-browser.config.ts --run SurfaceMount.browser-test 2>&1 | tail -15` — 2 tests must pass.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npm run check &amp;&amp; npx vitest --config vitest-browser.config.ts --run SurfaceMount.browser-test 2&gt;&amp;1 | tail -15</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q '<Surface name={name}' frontend/src/lib/components/core/SurfaceMount.svelte` succeeds (or equivalent literal `<Surface name={name} />`)
    - `grep -q "'surface-mount': SurfaceMount" frontend/src/lib/registry/defaults.ts` succeeds
    - `grep -q "import SurfaceMount from" frontend/src/lib/registry/defaults.ts` succeeds
    - `cd frontend && npm run check` exits 0
    - SurfaceMount browser test exits 0 with both tests passing
    - `grep -q 'test.todo' frontend/src/lib/components/core/SurfaceMount.browser-test.ts` returns no match (scaffold replaced with real tests)
  </acceptance_criteria>
  <done>SurfaceMount renders a nested Surface, is registered as 'surface-mount', and has 2 passing browser tests.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement AppShell.svelte with shadcn Sidebar composition + browser tests</name>
  <read_first>
    - frontend/src/lib/components/shell/AppShell.svelte (scaffold from Plan 01)
    - frontend/src/lib/components/shell/AppShell.browser-test.ts (scaffold from Plan 01)
    - frontend/src/lib/components/ui/sidebar/index.ts (the exact exports shadcn generated)
    - frontend/src/lib/components/core/NodeRenderer.svelte
    - frontend/src/lib/store/surfaces.svelte.ts
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Example 3 (AppShell composition)
  </read_first>
  <behavior>
    - `<AppShell props={{sidebarNodeId: 's1', headerNodeId: 'h1', footerNodeId: 'f1', mainNodeId: 'm1'}} surface="main" />` renders:
      - `Sidebar.Provider` wrapping the whole tree
      - `Sidebar.Root` → `Sidebar.Content` containing `<NodeRenderer nodeId="s1" .../>`
      - `Sidebar.Inset` containing a flex column with `<header>` (including `Sidebar.Trigger` and header slot render) → `<main>` (main slot render) → `<footer>` (footer slot render)
      - `popupsNodeId` and `toastsNodeId` NodeRenderers rendered as siblings OR inside Inset (RESEARCH Example 3 note: Dialog/Toast use portals so placement does not matter for DOM destination; choose inside Inset)
    - Undefined slot IDs gracefully skip their NodeRenderer
    - Browser test: mount an AppShell whose tree has a `heading` in each slot, assert all slot texts are visible in the DOM
  </behavior>
  <action>
1. Inspect the actual shadcn Sidebar export surface:
   ```bash
   cat frontend/src/lib/components/ui/sidebar/index.ts
   ```
   Note the named exports. Common names are `Provider`, `Root`, `Content`, `Inset`, `Trigger` — but verify against the installed file because minor registry updates may rename them.

2. REPLACE the scaffold contents of `frontend/src/lib/components/shell/AppShell.svelte` with (adjust `Sidebar.*` member names to match the actual exports from step 1):

```svelte
<script lang="ts">
	import type { ComponentAction } from '$lib/transport/messages';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
	import { getSurfaceTree } from '$lib/store/surfaces.svelte';

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

	void bind;
	void action;

	let sidebarId = $derived(props.sidebarNodeId as string | undefined);
	let headerId = $derived(props.headerNodeId as string | undefined);
	let footerId = $derived(props.footerNodeId as string | undefined);
	let mainId = $derived(props.mainNodeId as string | undefined);
	let popupsId = $derived(props.popupsNodeId as string | undefined);
	let toastsId = $derived(props.toastsNodeId as string | undefined);

	// The shell lives IN `surface` — look up slot children from this surface's tree.
	let tree = $derived(getSurfaceTree(surface));
	let nodes = $derived(tree?.nodes ?? {});
</script>

<Sidebar.Provider>
	<Sidebar.Root collapsible="offcanvas">
		<Sidebar.Content>
			{#if sidebarId && nodes[sidebarId]}
				<NodeRenderer nodeId={sidebarId} {nodes} {surface} />
			{/if}
		</Sidebar.Content>
	</Sidebar.Root>
	<Sidebar.Inset>
		<div class="flex min-h-screen flex-col">
			<header class="flex items-center gap-2 border-b bg-background px-4 py-2">
				<Sidebar.Trigger />
				{#if headerId && nodes[headerId]}
					<div class="flex flex-1 items-center justify-between">
						<NodeRenderer nodeId={headerId} {nodes} {surface} />
					</div>
				{/if}
			</header>
			<main class="flex-1 overflow-auto bg-background">
				{#if mainId && nodes[mainId]}
					<NodeRenderer nodeId={mainId} {nodes} {surface} />
				{/if}
			</main>
			<footer class="border-t bg-background px-4 py-2 text-xs text-muted-foreground">
				{#if footerId && nodes[footerId]}
					<NodeRenderer nodeId={footerId} {nodes} {surface} />
				{/if}
			</footer>
			{#if popupsId && nodes[popupsId]}
				<NodeRenderer nodeId={popupsId} {nodes} {surface} />
			{/if}
			{#if toastsId && nodes[toastsId]}
				<NodeRenderer nodeId={toastsId} {nodes} {surface} />
			{/if}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
```

**IMPORTANT:** If `cat frontend/src/lib/components/ui/sidebar/index.ts` shows that the exports do not expose `Provider`/`Root`/`Content`/`Inset`/`Trigger` as named members (e.g., they might be exported individually as `SidebarProvider`/`SidebarRoot`/etc.), use the **svelte MCP server** (`mcp__svelte__*` tools if available, otherwise WebFetch of `https://www.shadcn-svelte.com/docs/components/sidebar`) to look up the current canonical usage pattern, and adapt the imports and JSX accordingly. Keep the overall structure (Provider wraps, Root+Content holds sidebar, Inset holds the flex column with header/main/footer/popups/toasts).

3. REPLACE the scaffold contents of `frontend/src/lib/components/shell/AppShell.browser-test.ts`:

```typescript
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import Surface from '$lib/components/core/Surface.svelte';
import {
	setSurfaceTree,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { registerDefaults } from '$lib/registry/defaults';

const SURFACE = 'appshell-test';

beforeEach(() => {
	resetStore(SURFACE);
	clearSurfaceTree(SURFACE);
	registerDefaults();
});

test('AppShell renders all slot contents via NodeRenderer', async () => {
	setFullState(SURFACE, {});
	setSurfaceTree(SURFACE, 'shell-root', {
		'shell-root': {
			type: 'app-shell',
			props: {
				sidebarNodeId: 'side-1',
				headerNodeId: 'head-1',
				footerNodeId: 'foot-1',
				mainNodeId: 'main-1',
			},
		},
		'side-1': { type: 'heading', props: { text: 'SidebarContent' } },
		'head-1': { type: 'heading', props: { text: 'HeaderContent' } },
		'foot-1': { type: 'heading', props: { text: 'FooterContent' } },
		'main-1': { type: 'heading', props: { text: 'MainContent' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	expect(screen.baseElement.textContent).toContain('SidebarContent');
	expect(screen.baseElement.textContent).toContain('HeaderContent');
	expect(screen.baseElement.textContent).toContain('FooterContent');
	expect(screen.baseElement.textContent).toContain('MainContent');
});

test('AppShell with missing slots renders without crashing', async () => {
	setFullState(SURFACE, {});
	setSurfaceTree(SURFACE, 'shell-root', {
		'shell-root': {
			type: 'app-shell',
			props: {
				mainNodeId: 'main-1',
			},
		},
		'main-1': { type: 'heading', props: { text: 'LoneMain' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	expect(screen.baseElement.textContent).toContain('LoneMain');
});

test('AppShell header includes the Sidebar.Trigger (mobile hamburger)', async () => {
	setFullState(SURFACE, {});
	setSurfaceTree(SURFACE, 'shell-root', {
		'shell-root': {
			type: 'app-shell',
			props: { mainNodeId: 'main-1' },
		},
		'main-1': { type: 'heading', props: { text: 'X' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	// Sidebar.Trigger renders as a button with aria-controls or aria-label.
	// Accept any of the common shadcn-svelte markers.
	const triggerCandidates = screen.baseElement.querySelectorAll(
		'button[data-sidebar="trigger"], button[aria-label*="sidebar" i], button[aria-controls*="sidebar" i]'
	);
	expect(triggerCandidates.length).toBeGreaterThanOrEqual(1);
});
```

4. Add `AppShell` to `frontend/src/lib/registry/defaults.ts`:
   - Import: `import AppShell from '../components/shell/AppShell.svelte';`
   - Register: `'app-shell': AppShell,`

5. Run `cd frontend && npm run check` — must be green.

6. Run `cd frontend && npx vitest --config vitest-browser.config.ts --run AppShell.browser-test 2>&1 | tail -20` — all 3 tests must pass.

7. After AppShell tests are green, confirm the svelte MCP server (if available in this session) flags no issues: if `mcp__svelte__*` tools exist, call the validator tool on `AppShell.svelte` and fix any reported issues, then re-run the browser test.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npm run check &amp;&amp; npx vitest --config vitest-browser.config.ts --run AppShell.browser-test 2&gt;&amp;1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'import \* as Sidebar' frontend/src/lib/components/shell/AppShell.svelte` succeeds (or equivalent named imports from the ui/sidebar module)
    - `grep -q 'Sidebar.Provider\|SidebarProvider' frontend/src/lib/components/shell/AppShell.svelte` succeeds
    - `grep -q 'Sidebar.Trigger\|SidebarTrigger' frontend/src/lib/components/shell/AppShell.svelte` succeeds
    - `grep -q 'NodeRenderer' frontend/src/lib/components/shell/AppShell.svelte` succeeds (at least once)
    - `grep -q 'props.sidebarNodeId' frontend/src/lib/components/shell/AppShell.svelte` succeeds
    - `grep -q "'app-shell': AppShell" frontend/src/lib/registry/defaults.ts` succeeds
    - `grep -q "import AppShell from '../components/shell/AppShell.svelte'" frontend/src/lib/registry/defaults.ts` succeeds
    - `cd frontend && npm run check` exits 0
    - AppShell browser test exits 0 with all 3 tests passing
  </acceptance_criteria>
  <done>AppShell.svelte renders all slots via NodeRenderer inside shadcn Sidebar primitives. Registered in defaults.ts. All 3 browser tests pass.</done>
</task>

<task type="auto">
  <name>Task 3: Collapse routes/+layout.svelte and retire ConnectionBanner</name>
  <read_first>
    - frontend/src/routes/+layout.svelte
    - frontend/src/lib/index.ts
    - frontend/src/lib/components/core/ConnectionBanner.svelte
    - frontend/src/lib/components/core/ConnectionBanner.browser-test.ts (if it exists)
  </read_first>
  <action>
1. REPLACE the entire contents of `frontend/src/routes/+layout.svelte` with:

```svelte
<script>
	import '../app.css';
	import { Surface } from '$lib';
	let { children } = $props();
</script>

<Surface name="main" />
{@render children()}
```

Do NOT preserve the old `<ConnectionBanner />`, the `<div class="flex h-screen">`, or the `<Surface name="sidebar"/>` / `<Surface name="modal"/>` / `<Surface name="toast"/>` mounts — the sub-surface architecture routes them through AppShell's `surface-mount` nodes now.

2. Delete `frontend/src/lib/components/core/ConnectionBanner.svelte`:
   ```bash
   rm frontend/src/lib/components/core/ConnectionBanner.svelte
   ```

3. Delete the browser test if it exists:
   ```bash
   rm -f frontend/src/lib/components/core/ConnectionBanner.browser-test.ts
   ```

4. In `frontend/src/lib/index.ts`, delete the line `export { default as ConnectionBanner } from './components/core/ConnectionBanner.svelte';` (currently line 6). The remaining core exports (`Surface`, `NodeRenderer`) stay.

5. Remove the `sidebar:` entry from the `layoutClasses` map in `frontend/src/lib/components/core/Surface.svelte` (lines 14-19). After the edit the map contains only `main:`, `modal:`, `toast:` entries. The `main:` entry stays — it is used by the remaining `<Surface name="main"/>` mount in `+layout.svelte`. The `modal:` and `toast:` entries are now dead code because those surfaces are only mounted via SurfaceMount inside AppShell (which renders `<Surface name={...} />` without the top-level layout class). DELETE the `modal:` and `toast:` entries too — they are unused. Resulting map:

```typescript
const layoutClasses: Record<string, string> = {
	main: 'bg-background p-6 overflow-y-auto min-w-[320px] flex-1',
};
```

Note: the `bg-sidebar` rename from Plan 01 Task 2 still applies — but since we are deleting the `sidebar:` entry entirely, the already-renamed line disappears with it. That is fine.

6. Grep for any other references to `ConnectionBanner`:
   ```bash
   grep -rn 'ConnectionBanner' frontend/src/
   ```
   Must return zero hits after the edits.

7. Run `cd frontend && npm run check` — must be green.
8. Run `cd frontend && npx vitest --run` — all existing unit tests must still pass (the deletion of ConnectionBanner removes its tests, but no other tests should reference it).
9. Run `cd frontend && npm run build` — the SvelteKit build must still succeed (this validates the `+layout.svelte` change at the framework level).
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; ! test -f src/lib/components/core/ConnectionBanner.svelte &amp;&amp; ! grep -rn 'ConnectionBanner' src/ &amp;&amp; grep -q 'Surface name="main"' src/routes/+layout.svelte &amp;&amp; ! grep -q 'ConnectionBanner' src/lib/index.ts &amp;&amp; npm run check</automated>
  </verify>
  <acceptance_criteria>
    - `frontend/src/lib/components/core/ConnectionBanner.svelte` does NOT exist
    - `frontend/src/lib/components/core/ConnectionBanner.browser-test.ts` does NOT exist
    - `grep -rn 'ConnectionBanner' frontend/src/` returns zero lines
    - `frontend/src/routes/+layout.svelte` contains the literal `<Surface name="main" />` and nothing else in terms of Surface mounts
    - `frontend/src/routes/+layout.svelte` does NOT contain `<ConnectionBanner` or `<Surface name="sidebar"` or `<Surface name="modal"` or `<Surface name="toast"`
    - `frontend/src/lib/components/core/Surface.svelte` `layoutClasses` map contains only the `main:` key (no `sidebar`, `modal`, `toast` entries)
    - `grep -q "export.*ConnectionBanner" frontend/src/lib/index.ts` returns no match
    - `cd frontend && npm run check` exits 0
    - `cd frontend && npm run build` exits 0
    - `cd frontend && npx vitest --run` exits 0 (no test references a deleted component)
  </acceptance_criteria>
  <done>Top-level layout collapsed to single Surface mount. ConnectionBanner deleted cleanly with zero residual references. Build and tests green.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Surface store → DOM | AppShell reads `getSurfaceTree(surface).nodes[id]` and passes to NodeRenderer; trust boundary is the store itself |
| AppShell props → NodeRenderer | Slot IDs come from server-sent tree props, treated as opaque identifiers |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-13 | Information Disclosure | A malicious server could inject a `<script>` via a `heading` component's `text` prop in the header slot | accept | Svelte's default escaping (curly expressions are HTML-escaped) prevents XSS. We do NOT use `{@html}` anywhere in AppShell / SurfaceMount / NodeRenderer. This is the same posture as every other SDUI component in the project. |
| T-12-14 | Denial of Service | Recursive `surface-mount` inside `surface-mount` inside `surface-mount` could stack-overflow | mitigate | `Surface.svelte` does not re-trigger on its own render; recursion is finite because each SurfaceMount terminates at a Surface with its own independent tree. Documented in RESEARCH Pattern 2. If a handler accidentally creates a cycle (surface A mounts surface B mounts surface A), the result is visual confusion, not crash — each surface renders its own LoadingSkeleton until content arrives. |
| T-12-15 | Tampering | User clicks `Sidebar.Trigger` which toggles local state — no server round-trip | accept | Sidebar state is UI-local per Phase 12 scope (SHELL-05 defers persistence to v2). No data tampered. |
</threat_model>

<verification>
- `cd frontend && npm run check` exits 0
- `cd frontend && npm run build` exits 0
- AppShell browser test: 3 passing
- SurfaceMount browser test: 2 passing
- `grep -rn 'ConnectionBanner' frontend/src/` returns zero lines
- `grep -q 'Surface name="main"' frontend/src/routes/+layout.svelte` succeeds
- `grep -c 'Surface name=' frontend/src/routes/+layout.svelte` returns 1 (exactly one Surface mount at top level)
- `grep -q "'app-shell': AppShell" frontend/src/lib/registry/defaults.ts`
- `grep -q "'surface-mount': SurfaceMount" frontend/src/lib/registry/defaults.ts`
</verification>

<success_criteria>
- AppShell.svelte wires shadcn Sidebar.Provider/Root/Content/Inset/Trigger (or equivalent sub-component names from the installed registry) and resolves slot IDs from props
- SurfaceMount.svelte renders `<Surface name={props.name}/>`
- Both components registered in `registry/defaults.ts` under `'app-shell'` and `'surface-mount'`
- `routes/+layout.svelte` collapsed to a single `<Surface name="main"/>`
- `ConnectionBanner.svelte` and its browser test deleted
- `lib/index.ts` no longer exports ConnectionBanner
- `Surface.svelte` layoutClasses map simplified to only `main:`
- All 5 browser tests (2 SurfaceMount + 3 AppShell) pass
- `npm run check` and `npm run build` green
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-06-SUMMARY.md` recording:
- Exact Sidebar sub-component names used (Provider/Root/Content/Inset/Trigger vs. SidebarProvider/etc.)
- Whether the svelte MCP server was consulted and what it reported
- Count of files deleted (ConnectionBanner + browser test + empty directories)
- Whether the `npm run build` succeeded on first try or required adjustments
</output>
