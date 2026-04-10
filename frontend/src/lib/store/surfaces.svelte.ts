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
 *
 * Visited-set short-circuit ensures cyclic children graphs produce a bounded
 * walk (each id enqueued at most once) — see threat T-12-07.
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
