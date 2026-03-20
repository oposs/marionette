/**
 * Surface tree state: tracks the component tree (root + nodes) for each surface.
 *
 * When a render message arrives, the init module calls setSurfaceTree to
 * update the tree for the target surface. Surface.svelte reads this via
 * getSurfaceTree to know what to render.
 */
import type { ComponentNode } from '$lib/transport/messages.js';

interface SurfaceTree {
	root: string;
	nodes: Record<string, ComponentNode>;
}

const surfaceState: Record<string, SurfaceTree> = $state({});

/** Set the component tree for a surface. */
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
