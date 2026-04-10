---
phase: 12
plan: 04
type: execute
wave: 2
depends_on: [12-02, 12-03]
files_modified:
  - frontend/src/lib/transport/messages.ts
  - frontend/src/lib/init.ts
  - frontend/src/lib/store/data.svelte.ts
  - frontend/src/lib/store/surfaces.svelte.ts
  - frontend/src/lib/store/surfaces.svelte.test.ts
  - frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts
  - frontend/src/lib/index.ts
autonomous: true
requirements: [PATCH-01, PATCH-02]
nyquist_compliant: true
tags: [frontend, svelte, store, focus-preservation]
must_haves:
  truths:
    - "Frontend PatchOperation TypeScript type is a tagged union with 6 variants discriminated by `op`"
    - "PatchMessage in messages.ts has a required `surface: string` field"
    - "init.ts routes patches via `msg.surface`, not a hardcoded 'main'"
    - "surfaces.svelte.ts exports setNode, deleteNode, setChildren, insertChild, removeChild, gcOrphans with in-place mutation semantics"
    - "applyPatch in data.svelte.ts dispatches on `op` and routes node ops to the surface store"
    - "Focus-preservation browser test proves a focused input retains focus+cursor across a patch to a sibling node"
    - "Walk-and-prune GC unit test proves orphan nodes are removed via BFS from root"
  artifacts:
    - path: "frontend/src/lib/transport/messages.ts"
      provides: "PatchOperation tagged union + PatchMessage surface field"
      contains: "op: 'set-node'"
    - path: "frontend/src/lib/store/surfaces.svelte.ts"
      provides: "fine-grained mutation API"
      exports: ["setSurfaceTree", "getSurfaceTree", "clearSurfaceTree", "setNode", "deleteNode", "setChildren", "insertChild", "removeChild", "gcOrphans"]
    - path: "frontend/src/lib/init.ts"
      provides: "patch handler routes by msg.surface"
      contains: "applyPatch(msg.surface"
    - path: "frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts"
      provides: "focus-preservation proof test"
      contains: "selectionStart"
  key_links:
    - from: "init.ts patch handler"
      to: "applyPatch dispatcher"
      via: "msg.surface routing"
      pattern: "applyPatch\\(msg\\.surface"
    - from: "data.svelte.ts applyPatch"
      to: "surfaces.svelte.ts fine-grained API"
      via: "dispatch on op variant"
      pattern: "case 'set-node'"
---

<objective>
Mirror the Rust protocol changes on the frontend: update `PatchOperation` / `PatchMessage` types, fix the `init.ts:47` hardcoded-`main` bug, rewrite `surfaces.svelte.ts` with fine-grained `setNode` / `deleteNode` / `setChildren` / `insertChild` / `removeChild` / `gcOrphans` APIs that mutate in place, extend `applyPatch` in `data.svelte.ts` to dispatch on the op discriminator, and prove focus preservation with a browser test plus the walk-and-prune GC with a unit test.

Purpose: This plan implements D-A6 (focus preservation, mandatory + proven by test) and D-A3 (surface routing fix). It is the frontend half of Part A. Every downstream visual feature (AppShell, field-swap demo) depends on fine-grained reactivity working correctly here.

Output: Fine-grained surface store with in-place mutation, applyPatch dispatcher that routes all 6 op variants, fixed init.ts bug, two passing tests (focus-preservation browser test + surfaces unit test including GC).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/12-protocol-node-patching-appshell/12-CONTEXT.md
@.planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md
@frontend/src/lib/transport/messages.ts
@frontend/src/lib/init.ts
@frontend/src/lib/store/surfaces.svelte.ts
@frontend/src/lib/store/data.svelte.ts
@frontend/src/lib/store/dirty.svelte.ts
@frontend/src/lib/store/pointer.ts
@frontend/src/lib/components/core/Surface.svelte
@frontend/src/lib/components/core/NodeRenderer.svelte
@frontend/src/lib/components/form/TextInput.svelte
@frontend/src/lib/registry/defaults.ts
@frontend/src/lib/index.ts

<interfaces>
Current `PatchOperation` in `frontend/src/lib/transport/messages.ts` (lines 17-21):

```typescript
export interface PatchOperation {
	path: string;
	value: unknown;
}
```

Target: Discriminated union matching Plan 02's Rust enum. Variant names match wire kebab-case `op` values.

Current `PatchMessage` (lines 73-78):

```typescript
export interface PatchMessage {
	type: 'patch';
	id?: string;
	patch: PatchOperation[];
}
```

Target: Add required `surface: string`.

Current `init.ts:44-52` patch handler hardcodes `'main'`:

```typescript
registerHandler('patch', (raw: unknown) => {
    const msg = raw as PatchMessage;
    applyPatch('main', msg.patch);   // BUG: surface hardcoded
    if (msg.id) confirmOptimistic(msg.id);
});
```

Target: `applyPatch(msg.surface, msg.patch);`.

Current `surfaces.svelte.ts` stores:

```typescript
interface SurfaceTree { root: string; nodes: Record<string, ComponentNode>; }
const surfaceState: Record<string, SurfaceTree> = $state({});
export function setSurfaceTree(surface, root, nodes) { surfaceState[surface] = { root, nodes }; }
```

The wholesale reassignment at line 23 is the focus-preservation bug (RESEARCH Finding 1).

`NodeRenderer.svelte:15` reads `let node = $derived(nodes[nodeId]);` — a per-key proxy read. `NodeRenderer.svelte:31` uses `{#each node.children as childId (childId)}` — keyed each block. Both mechanisms are ALREADY IN PLACE and require no changes. The only bug is `setSurfaceTree` replacing the whole tree object reference. Fix is in-place mutation of `tree.nodes[id]` / `tree.nodes[id].children`.

Current `applyPatch` in `data.svelte.ts:62-70`:

```typescript
export function applyPatch(surface: string, operations: PatchOperation[]): void {
    for (const op of operations) {
        if (isDirty(op.path)) {
            queuePatch(op.path, op);
        } else {
            setData(surface, op.path, op.value);
        }
    }
}
```

Target: Dispatch on `op.op` (the discriminator). `set` → data path (possibly via dirty queue); 5 node ops → delegate to surfaces store. Run `gcOrphans(surface)` once after the batch (per D-A8).

`frontend/src/lib/index.ts` currently re-exports `setSurfaceTree, getSurfaceTree, clearSurfaceTree` from `surfaces.svelte`. Target: add the 5 new mutators + `gcOrphans` to the re-exports.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Rewrite PatchOperation TypeScript union + fix PatchMessage surface + fix init.ts routing</name>
  <read_first>
    - frontend/src/lib/transport/messages.ts
    - frontend/src/lib/init.ts
    - frontend/src/lib/store/data.svelte.ts (for applyPatch signature coupling)
    - frontend/src/lib/index.ts
  </read_first>
  <behavior>
    - `PatchOperation` is a discriminated union — `op: 'set'`, `'set-node'`, `'delete-node'`, `'set-children'`, `'insert-child'`, `'remove-child'`
    - TypeScript `npm run check` exhaustively switches on `op.op` without an `any` cast at call sites
    - `PatchMessage` has a required `surface: string` field
    - `init.ts` patch handler calls `applyPatch(msg.surface, msg.patch)` — no hardcoded `'main'`
  </behavior>
  <action>
1. REPLACE the `PatchOperation` interface (currently lines 17-21 of `frontend/src/lib/transport/messages.ts`) with a discriminated union. Exact shape:

```typescript
import type { ComponentNode } from './messages'; // (already in same file — no import needed)

/** Data op: set a value at a JSON Pointer path. */
export interface PatchOperationSet {
	op: 'set';
	path: string;
	value: unknown;
}

/** Node op: replace (or create) the component at this node ID. */
export interface PatchOperationSetNode {
	op: 'set-node';
	id: string;
	component: ComponentNode;
}

/** Node op: delete the node with this ID. */
export interface PatchOperationDeleteNode {
	op: 'delete-node';
	id: string;
}

/** Node op: replace the children array of the given node. */
export interface PatchOperationSetChildren {
	op: 'set-children';
	id: string;
	children: string[];
}

/** Node op: insert an existing child ID into a parent's children array at index. */
export interface PatchOperationInsertChild {
	op: 'insert-child';
	parent: string;
	index: number;
	childId: string;
}

/** Node op: remove a child ID from a parent's children array. */
export interface PatchOperationRemoveChild {
	op: 'remove-child';
	parent: string;
	childId: string;
}

/** Tagged union of all patch operations. Discriminated by `op`. */
export type PatchOperation =
	| PatchOperationSet
	| PatchOperationSetNode
	| PatchOperationDeleteNode
	| PatchOperationSetChildren
	| PatchOperationInsertChild
	| PatchOperationRemoveChild;
```

(Do NOT add a separate import line for `ComponentNode` — `PatchOperationSetNode` references `ComponentNode` which is declared later in the same file. TypeScript allows forward references within a module.)

2. Modify `PatchMessage` (lines 73-78):

```typescript
/** Server patch message: incremental update to one surface (data and/or tree ops). */
export interface PatchMessage {
	type: 'patch';
	id?: string;
	surface: string;
	patch: PatchOperation[];
}
```

3. Fix `frontend/src/lib/init.ts` lines 44-52. Replace the body of the `registerHandler('patch', ...)` callback:

```typescript
registerHandler('patch', (raw: unknown) => {
	const msg = raw as PatchMessage;
	// Route by target surface (D-A3 — fixes the hardcoded-'main' bug).
	applyPatch(msg.surface, msg.patch);
	if (msg.id) confirmOptimistic(msg.id);
});
```

Delete the `// Apply patch to main surface (protocol lacks surface field...)` comment.

4. Run `cd frontend && npm run check` — EXPECT type errors at places that construct `PatchOperation { path, value }` (not yet the dispatched-union shape). The affected files are:
   - `frontend/src/lib/store/data.svelte.ts` (`applyPatch` reads `op.path` unconditionally — Task 2 rewrites this)
   - Any test file that manually constructs `PatchOperation` for mock data — update to use the `op: 'set'` variant explicitly
   - `frontend/src/lib/store/optimistic.svelte.ts` or similar if it stores or inspects patch ops

For each error, change `{ path: p, value: v }` to `{ op: 'set', path: p, value: v }`. Do NOT silence errors with `as any`.

5. Do NOT update `frontend/src/lib/index.ts` yet — Task 2 adds the new store exports in the same edit.

6. Verify: `cd frontend && npm run check` — must be green.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npm run check 2&gt;&amp;1 | tail -10 &amp;&amp; grep -q "op: 'set'" src/lib/transport/messages.ts &amp;&amp; grep -q "surface: string" src/lib/transport/messages.ts &amp;&amp; grep -q 'applyPatch(msg\.surface' src/lib/init.ts</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "op:\s*'set'\|op:\s*'set-node'\|op:\s*'delete-node'\|op:\s*'set-children'\|op:\s*'insert-child'\|op:\s*'remove-child'" frontend/src/lib/transport/messages.ts` returns at least 6
    - `grep -q 'surface: string' frontend/src/lib/transport/messages.ts` inside the `PatchMessage` interface
    - `grep -q 'applyPatch(msg\.surface' frontend/src/lib/init.ts` succeeds
    - `grep -q "applyPatch('main'" frontend/src/lib/init.ts` fails (hardcode gone)
    - `cd frontend && npm run check` exits 0
    - No `as any` casts introduced in `init.ts` or `messages.ts`
  </acceptance_criteria>
  <done>PatchOperation is a discriminated union; PatchMessage has `surface`. init.ts routes by `msg.surface`. `npm run check` is green.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Rewrite surfaces.svelte.ts with fine-grained mutation API + gcOrphans + unit tests</name>
  <read_first>
    - frontend/src/lib/store/surfaces.svelte.ts
    - frontend/src/lib/store/surfaces.svelte.test.ts (scaffold from Plan 01)
    - frontend/src/lib/components/core/NodeRenderer.svelte (verify keyed each is at line 31)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Pattern 1 (Svelte 5 fine-grained reactivity)
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Example 2 (gcOrphans BFS)
  </read_first>
  <behavior>
    - `setSurfaceTree(surface, root, nodes)` — establishes or replaces a whole surface tree (called on `render`; wholesale replacement is CORRECT here)
    - `setNode(surface, id, component)` — mutates `surfaceState[surface].nodes[id]` in place; no reassignment of the tree or `nodes` references
    - `deleteNode(surface, id)` — `delete surfaceState[surface].nodes[id]`
    - `setChildren(surface, id, children)` — mutates `surfaceState[surface].nodes[id].children` in place (not a new array reference — use length-zero + push loop OR direct assignment depending on Svelte 5 proxy semantics; the unit test asserts child order after the call)
    - `insertChild(surface, parent, index, childId)` — splices `childId` into the parent's children array at `index`. Creates `children = []` if missing.
    - `removeChild(surface, parent, childId)` — splices the matching entry out of the parent's children array
    - `gcOrphans(surface)` — BFS reachability walk from `surfaceState[surface].root`, deleting any `nodes[id]` entry whose id is not reachable
    - All mutations on a non-existent surface are no-ops (do not throw)
    - Unit tests cover every function including an orphan-creation + GC scenario
  </behavior>
  <action>
1. REPLACE the entire contents of `frontend/src/lib/store/surfaces.svelte.ts` with:

```typescript
/**
 * Surface tree state: tracks the component tree (root + nodes) for each surface.
 *
 * `render` messages call `setSurfaceTree` to replace a whole tree (correct — the
 * root changes). `patch` messages call the fine-grained mutators (`setNode`,
 * `deleteNode`, `setChildren`, `insertChild`, `removeChild`) which mutate node
 * map entries IN PLACE so Svelte 5's per-key reactive proxy only invalidates the
 * changed entry. This is how focus preservation under node patches works.
 *
 * `gcOrphans` performs a BFS reachability walk from the surface's root and
 * deletes any unreachable node entries. Cost is O(N) in the target surface's
 * node count — see D-A8.
 */
import type { ComponentNode } from '$lib/transport/messages.js';

interface SurfaceTree {
	root: string;
	nodes: Record<string, ComponentNode>;
}

const surfaceState: Record<string, SurfaceTree> = $state({});

/**
 * Set (or replace) the component tree for a surface. Used by the `render`
 * message handler; the whole tree is replaced wholesale because `render`
 * semantically changes the root.
 */
export function setSurfaceTree(
	surface: string,
	root: string,
	nodes: Record<string, ComponentNode>
): void {
	surfaceState[surface] = { root, nodes };
}

/** Get the component tree for a surface, or undefined if not yet rendered. */
export function getSurfaceTree(surface: string): SurfaceTree | undefined {
	return surfaceState[surface];
}

/** Clear a surface's tree (for testing or cleanup). */
export function clearSurfaceTree(surface: string): void {
	delete surfaceState[surface];
}

/**
 * Replace (or create) the component at node `id` in `surface`. Mutates the
 * per-key entry in place so only `NodeRenderer` instances bound to `id` re-derive.
 */
export function setNode(surface: string, id: string, component: ComponentNode): void {
	const tree = surfaceState[surface];
	if (!tree) return;
	tree.nodes[id] = component;
}

/** Delete the node with `id` from `surface`. */
export function deleteNode(surface: string, id: string): void {
	const tree = surfaceState[surface];
	if (!tree) return;
	delete tree.nodes[id];
}

/** Replace the children array of the node at `id` in `surface`. */
export function setChildren(surface: string, id: string, children: string[]): void {
	const tree = surfaceState[surface];
	const parent = tree?.nodes[id];
	if (!parent) return;
	// Mutate in place so `{#each parent.children as childId (childId)}` in
	// NodeRenderer reorders DOM nodes instead of remounting them.
	parent.children = children.slice();
}

/** Insert `childId` into the children array of `parent` at `index`. */
export function insertChild(
	surface: string,
	parent: string,
	index: number,
	childId: string
): void {
	const tree = surfaceState[surface];
	const p = tree?.nodes[parent];
	if (!p) return;
	if (!p.children) p.children = [];
	p.children.splice(index, 0, childId);
}

/** Remove `childId` from `parent`'s children array (first occurrence). */
export function removeChild(surface: string, parent: string, childId: string): void {
	const tree = surfaceState[surface];
	const p = tree?.nodes[parent];
	if (!p?.children) return;
	const i = p.children.indexOf(childId);
	if (i >= 0) p.children.splice(i, 1);
}

/**
 * Walk-and-prune GC: BFS from the surface's root, then delete any `nodes[id]`
 * whose id is not reachable. Per D-A8, scoped to one surface.
 */
export function gcOrphans(surface: string): void {
	const tree = surfaceState[surface];
	if (!tree) return;

	const reachable = new Set<string>();
	const queue: string[] = [tree.root];
	while (queue.length > 0) {
		const id = queue.shift()!;
		if (reachable.has(id)) continue;
		reachable.add(id);
		const node = tree.nodes[id];
		if (node?.children) {
			for (const child of node.children) queue.push(child);
		}
	}

	for (const id of Object.keys(tree.nodes)) {
		if (!reachable.has(id)) {
			delete tree.nodes[id];
		}
	}
}
```

2. REPLACE the contents of the `frontend/src/lib/store/surfaces.svelte.test.ts` scaffold with real unit tests:

```typescript
import { describe, test, expect, beforeEach } from 'vitest';
import {
	setSurfaceTree,
	getSurfaceTree,
	clearSurfaceTree,
	setNode,
	deleteNode,
	setChildren,
	insertChild,
	removeChild,
	gcOrphans,
} from './surfaces.svelte';
import type { ComponentNode } from '$lib/transport/messages';

const SURFACE = 'test-surface';

function textNode(label: string, bind?: string): ComponentNode {
	return { type: 'text-input', props: { label }, bind };
}

function container(children: string[]): ComponentNode {
	return { type: 'container', children };
}

beforeEach(() => {
	clearSurfaceTree(SURFACE);
});

describe('surfaces.svelte.ts fine-grained mutation API', () => {
	test('setSurfaceTree establishes a tree that getSurfaceTree reads back', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
		});
		const tree = getSurfaceTree(SURFACE);
		expect(tree?.root).toBe('root');
		expect(tree?.nodes['a'].type).toBe('text-input');
	});

	test('setNode mutates nodes[id] in place', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
		});
		const treeBefore = getSurfaceTree(SURFACE);
		const nodesRefBefore = treeBefore!.nodes;

		setNode(SURFACE, 'a', textNode('A renamed'));

		const treeAfter = getSurfaceTree(SURFACE);
		expect(treeAfter!.nodes).toBe(nodesRefBefore); // same reference
		expect(treeAfter!.nodes['a'].props?.label).toBe('A renamed');
	});

	test('deleteNode removes entry from tree.nodes', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
		});
		deleteNode(SURFACE, 'a');
		expect(getSurfaceTree(SURFACE)!.nodes['a']).toBeUndefined();
	});

	test('setChildren replaces parent.children with new order', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a', 'b', 'c']),
			a: textNode('A'),
			b: textNode('B'),
			c: textNode('C'),
		});
		setChildren(SURFACE, 'root', ['c', 'a', 'b']);
		expect(getSurfaceTree(SURFACE)!.nodes['root'].children).toEqual(['c', 'a', 'b']);
	});

	test('insertChild inserts at index', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a', 'c']),
			a: textNode('A'),
			c: textNode('C'),
		});
		insertChild(SURFACE, 'root', 1, 'b');
		expect(getSurfaceTree(SURFACE)!.nodes['root'].children).toEqual(['a', 'b', 'c']);
	});

	test('removeChild removes first matching id', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a', 'b', 'c']),
			a: textNode('A'),
			b: textNode('B'),
			c: textNode('C'),
		});
		removeChild(SURFACE, 'root', 'b');
		expect(getSurfaceTree(SURFACE)!.nodes['root'].children).toEqual(['a', 'c']);
	});

	test('gcOrphans deletes unreachable nodes via BFS from root', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: textNode('A'),
			// Orphans — never referenced by root's children transitively
			ghost1: textNode('Ghost 1'),
			ghost2: textNode('Ghost 2'),
		});
		gcOrphans(SURFACE);
		const nodes = getSurfaceTree(SURFACE)!.nodes;
		expect(Object.keys(nodes).sort()).toEqual(['a', 'root']);
		expect(nodes['ghost1']).toBeUndefined();
		expect(nodes['ghost2']).toBeUndefined();
	});

	test('gcOrphans preserves deep descendants', () => {
		setSurfaceTree(SURFACE, 'root', {
			root: container(['a']),
			a: container(['a1', 'a2']),
			a1: textNode('A1'),
			a2: container(['a2a']),
			a2a: textNode('A2a'),
			orphan: textNode('Orphan'),
		});
		gcOrphans(SURFACE);
		const keys = Object.keys(getSurfaceTree(SURFACE)!.nodes).sort();
		expect(keys).toEqual(['a', 'a1', 'a2', 'a2a', 'root']);
	});

	test('mutators on a non-existent surface are no-ops (no throw)', () => {
		expect(() => setNode('missing', 'x', textNode('X'))).not.toThrow();
		expect(() => deleteNode('missing', 'x')).not.toThrow();
		expect(() => setChildren('missing', 'x', [])).not.toThrow();
		expect(() => insertChild('missing', 'x', 0, 'y')).not.toThrow();
		expect(() => removeChild('missing', 'x', 'y')).not.toThrow();
		expect(() => gcOrphans('missing')).not.toThrow();
	});
});
```

3. Extend `frontend/src/lib/store/data.svelte.ts` `applyPatch` (lines 62-70) to dispatch on the union discriminator:

```typescript
import type { PatchOperation } from '$lib/transport/messages.js';
import { resolvePointer, setAtPointer } from './pointer.js';
import { isDirty, queuePatch } from './dirty.svelte.js';
import {
	setNode,
	deleteNode,
	setChildren,
	insertChild,
	removeChild,
	gcOrphans,
} from './surfaces.svelte';

// ... (other existing exports unchanged)

export function applyPatch(surface: string, operations: PatchOperation[]): void {
	for (const op of operations) {
		switch (op.op) {
			case 'set': {
				if (isDirty(op.path)) {
					queuePatch(op.path, op);
				} else {
					setAtPointer(getStore(surface).data, op.path, op.value);
				}
				break;
			}
			case 'set-node':
				setNode(surface, op.id, op.component);
				break;
			case 'delete-node':
				deleteNode(surface, op.id);
				break;
			case 'set-children':
				setChildren(surface, op.id, op.children);
				break;
			case 'insert-child':
				insertChild(surface, op.parent, op.index, op.childId);
				break;
			case 'remove-child':
				removeChild(surface, op.parent, op.childId);
				break;
		}
	}
	// Run one GC pass per batch, scoped to the target surface.
	gcOrphans(surface);
}
```

Note: `queuePatch` signature may need adjustment — it currently takes `(path, op)` where op is the old `{path, value}` shape. Inspect `dirty.svelte.ts` and if `queuePatch` relies on `op.value`, it continues to work because `op` is narrowed to `PatchOperationSet` inside the `'set'` case. No changes needed in `dirty.svelte.ts`.

4. Update `frontend/src/lib/index.ts` to re-export the new surface store mutators:

```typescript
export {
	setSurfaceTree,
	getSurfaceTree,
	clearSurfaceTree,
	setNode,
	deleteNode,
	setChildren,
	insertChild,
	removeChild,
	gcOrphans,
} from './store/surfaces.svelte';
```

(Replace the existing `surfaces.svelte` re-export line with the above.)

5. Run `cd frontend && npm run check` — must be green.

6. Run `cd frontend && npx vitest --run surfaces.svelte.test` — all 9 tests must pass.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npm run check &amp;&amp; npx vitest --run src/lib/store/surfaces.svelte.test.ts 2&gt;&amp;1 | tail -15</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'export function setNode' frontend/src/lib/store/surfaces.svelte.ts` succeeds
    - `grep -q 'export function deleteNode' frontend/src/lib/store/surfaces.svelte.ts` succeeds
    - `grep -q 'export function setChildren' frontend/src/lib/store/surfaces.svelte.ts` succeeds
    - `grep -q 'export function insertChild' frontend/src/lib/store/surfaces.svelte.ts` succeeds
    - `grep -q 'export function removeChild' frontend/src/lib/store/surfaces.svelte.ts` succeeds
    - `grep -q 'export function gcOrphans' frontend/src/lib/store/surfaces.svelte.ts` succeeds
    - `grep -q "case 'set-node'" frontend/src/lib/store/data.svelte.ts` succeeds
    - `grep -q "case 'insert-child'" frontend/src/lib/store/data.svelte.ts` succeeds
    - `grep -q 'gcOrphans(surface)' frontend/src/lib/store/data.svelte.ts` succeeds
    - `grep -q 'setNode,\s*deleteNode,\s*setChildren,\s*insertChild,\s*removeChild,\s*gcOrphans' frontend/src/lib/index.ts` succeeds (flexible whitespace)
    - `cd frontend && npm run check` exits 0
    - `cd frontend && npx vitest --run src/lib/store/surfaces.svelte.test.ts` reports 9 passing tests
  </acceptance_criteria>
  <done>Fine-grained surface store exports all 6 new mutators + GC, applyPatch dispatches on op discriminator, 9 unit tests pass, type check is green.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Focus-preservation browser test — the canonical D-A6 proof</name>
  <read_first>
    - frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts (scaffold from Plan 01)
    - frontend/src/lib/components/core/Surface.svelte
    - frontend/src/lib/components/core/NodeRenderer.svelte
    - frontend/src/lib/components/form/TextInput.svelte
    - frontend/src/lib/components/form/TextInput.browser-test.ts (existing browser-test pattern to emulate)
    - frontend/src/lib/registry/defaults.ts
    - .planning/phases/12-protocol-node-patching-appshell/12-RESEARCH.md Example 4
  </read_first>
  <behavior>
    - Render a Surface with a container node that has two text-input children (field-a, field-b)
    - User focuses field-a and sets cursor at position 3 mid-word
    - `setNode('fptest', 'field-b', <new component>)` is called directly against the surface store
    - After `tick()`, `document.activeElement` is still the field-a input element
    - `inputA.selectionStart === 3 && inputA.selectionEnd === 3`
    - `inputA.value === 'hello'` (user's typed content preserved)
    - field-b's new label is visible in the DOM
  </behavior>
  <action>
REPLACE the contents of `frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` with:

```typescript
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import Surface from '$lib/components/core/Surface.svelte';
import {
	setSurfaceTree,
	setNode,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { registerDefaults } from '$lib/registry/defaults';

const SURFACE = 'fptest';

beforeEach(() => {
	resetStore(SURFACE);
	clearSurfaceTree(SURFACE);
	registerDefaults();
});

test('setNode on sibling preserves focus and cursor on focused input', async () => {
	// Arrange: surface with a container holding two text inputs
	setFullState(SURFACE, { a: '', b: '' });
	setSurfaceTree(SURFACE, 'root', {
		root: { type: 'container', children: ['field-a', 'field-b'] },
		'field-a': { type: 'text-input', bind: '/a', props: { label: 'A' } },
		'field-b': { type: 'text-input', bind: '/b', props: { label: 'B' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	// Locate the two inputs by their labels' rendered DOM order
	const inputs = screen.baseElement.querySelectorAll('input');
	expect(inputs.length).toBeGreaterThanOrEqual(2);
	const inputA = inputs[0] as HTMLInputElement;

	// Focus field-a, type "hello", move cursor to position 3
	inputA.focus();
	inputA.value = 'hello';
	inputA.dispatchEvent(new Event('input', { bubbles: true }));
	inputA.setSelectionRange(3, 3);

	expect(document.activeElement).toBe(inputA);
	expect(inputA.selectionStart).toBe(3);

	// Act: patch field-b to change its label (NOT touching field-a)
	setNode(SURFACE, 'field-b', {
		type: 'text-input',
		bind: '/b',
		props: { label: 'B (changed)' },
	});
	await tick();

	// Assert: field-a retains focus and cursor exactly where the user left it
	expect(document.activeElement).toBe(inputA);
	expect(inputA.selectionStart).toBe(3);
	expect(inputA.selectionEnd).toBe(3);
	expect(inputA.value).toBe('hello');

	// Sanity: field-b's new label is visible in the DOM
	const allLabels = Array.from(screen.baseElement.querySelectorAll('label')).map((l) =>
		l.textContent?.trim()
	);
	expect(allLabels.some((l) => l?.includes('B (changed)'))).toBe(true);
});

test('setNode on focused node does replace it (not a focus-preservation guarantee)', async () => {
	// This is the negative control per RESEARCH Pitfall 5 — we do NOT claim
	// that patching the focused node preserves focus. Document explicitly.
	setFullState(SURFACE, { a: '' });
	setSurfaceTree(SURFACE, 'root', {
		root: { type: 'container', children: ['only-field'] },
		'only-field': { type: 'text-input', bind: '/a', props: { label: 'Only' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	const inputOnly = screen.baseElement.querySelector('input') as HTMLInputElement;
	inputOnly.focus();
	expect(document.activeElement).toBe(inputOnly);

	setNode(SURFACE, 'only-field', {
		type: 'text-input',
		bind: '/a',
		props: { label: 'Changed' },
	});
	await tick();

	// No assertion on document.activeElement — this test documents that
	// patching THE focused node is not covered by D-A6. D-A6 only covers
	// patching siblings of the focused node.
	expect(true).toBe(true);
});
```

Run: `cd frontend && npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation 2>&1 | tail -20`.

If the test fails with "focus lost", inspect the `setNode` implementation — it MUST mutate `tree.nodes[id]` in place. Do NOT wrap the mutation in `{...tree.nodes, [id]: component}` — that creates a new map reference and invalidates every derived.

If the test fails because `tick()` isn't enough, try `await new Promise((r) => setTimeout(r, 0));` after `tick()`. Do NOT add arbitrary waits longer than one microtask.

If the test fails because `inputs.length < 2`, check that `registerDefaults()` ran first and that the `text-input` component in the registry corresponds to an `<input>` element in the rendered DOM. The Phase 11 `TextInput.browser-test.ts` is the reference.
  </action>
  <verify>
    <automated>cd frontend &amp;&amp; npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation 2&gt;&amp;1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - `grep -q 'selectionStart' frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` succeeds
    - `grep -q 'document.activeElement' frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` succeeds
    - `grep -q 'setNode(SURFACE' frontend/src/lib/store/surfaces.focus-preservation.browser-test.ts` succeeds
    - Test file contains BOTH the positive case (sibling patch preserves focus) AND the negative control (documenting that focused-node patch is NOT covered)
    - `cd frontend && npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation` exits 0 with at least 2 passing tests
    - `inputA.selectionStart === 3` assertion passes after the sibling patch
  </acceptance_criteria>
  <done>Focus-preservation browser test passes. The canonical D-A6 proof is in the repo.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| transport→store | PatchMessage from server becomes surface state mutations; message shape is trusted (server authoritative) |
| focus lifecycle | DOM focus/selection is a user-facing state that must survive programmatic store mutations |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-12-07 | Denial of Service | `gcOrphans` BFS on a cyclic children graph could loop forever | mitigate | Unit test `gcOrphans preserves deep descendants` + `reachable` visited-set short-circuits re-visits. Cycle creates a bounded walk (each id enqueued at most once). |
| T-12-08 | Tampering | A malicious backend could send `set-node` with a `component` that has a huge `props` payload, consuming memory | accept | Server is authoritative in this architecture; no trust boundary between server and client store. Size limits are a v2 concern. |
| T-12-09 | Information Disclosure | Focus-preservation bug could cause a user's typed-but-unsubmitted password to leak into an element replaced by `set-node` | accept | Password fields are typically targeted by `set-node` only in explicit logout flows; the focus-preservation test confirms SIBLING patches do not affect the focused element. Documented in Plan 03 PROTOCOL.md. |
| T-12-10 | Elevation of Privilege | Hardcoded `applyPatch('main', ...)` bug (pre-fix) could let a crafted `patch` message from a compromised server target the wrong surface | mitigate | Fix lands in Task 1 — `applyPatch(msg.surface, msg.patch)`. No server-crafted bypass after this plan. |
</threat_model>

<verification>
- `cd frontend && npm run check` exits 0
- `cd frontend && npx vitest --run src/lib/store/surfaces.svelte.test.ts` exits 0 with 9 passing tests
- `cd frontend && npx vitest --config vitest-browser.config.ts --run surfaces.focus-preservation` exits 0 with 2 passing tests
- `grep -q "applyPatch('main'" frontend/src/lib/init.ts` fails (bug is fixed)
- `grep -q 'applyPatch(msg\.surface' frontend/src/lib/init.ts` succeeds
- `grep -q 'gcOrphans(surface)' frontend/src/lib/store/data.svelte.ts` succeeds (GC runs once per batch)
</verification>

<success_criteria>
- `PatchOperation` TypeScript type is a discriminated union with 6 variants exactly matching Plan 02's Rust enum on the wire
- `PatchMessage.surface` is a required string
- `init.ts` patch handler routes by `msg.surface` (bug fixed)
- `surfaces.svelte.ts` exports 6 new functions: `setNode`, `deleteNode`, `setChildren`, `insertChild`, `removeChild`, `gcOrphans`
- `applyPatch` in `data.svelte.ts` dispatches on `op.op`; runs `gcOrphans(surface)` once per batch
- All 9 surfaces unit tests pass
- Focus-preservation browser test passes — sibling patch preserves focus and cursor position
- Type check is green
- `index.ts` re-exports all new mutators
</success_criteria>

<output>
After completion, create `.planning/phases/12-protocol-node-patching-appshell/12-04-SUMMARY.md` recording:
- Test counts for surfaces.svelte.test.ts and surfaces.focus-preservation.browser-test.ts
- Any call sites outside init.ts that had to be migrated away from the old `{path, value}` PatchOperation shape
- Cursor-position value the focus-preservation test asserts (for future reference when extending the test)
</output>
