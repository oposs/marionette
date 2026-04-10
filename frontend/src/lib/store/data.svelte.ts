/**
 * Reactive data store with JSON Pointer get/set/patch.
 *
 * Uses Svelte 5 $state rune for reactivity. Each surface has its own
 * data namespace. Components bind to paths within the store.
 */
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

const surfaces: Record<string, { data: Record<string, unknown> }> = $state({});

/**
 * Get (or create) the store for a surface.
 */
export function getStore(surface: string): { data: Record<string, unknown> } {
	if (!surfaces[surface]) {
		surfaces[surface] = { data: {} };
	}
	return surfaces[surface];
}

/**
 * Get all data for a surface as a plain object.
 */
export function getAllData(surface: string): Record<string, unknown> {
	return getStore(surface).data;
}

/**
 * Read a value at a JSON Pointer path within a surface's data.
 */
export function getData(surface: string, pointer: string): unknown {
	return resolvePointer(getStore(surface).data, pointer);
}

/**
 * Set a value at a JSON Pointer path within a surface's data.
 */
export function setData(surface: string, pointer: string, value: unknown): void {
	setAtPointer(getStore(surface).data, pointer, value);
}

/**
 * Replace the entire data for a surface.
 * Mutates the existing object (required for $state reactivity).
 */
export function setFullState(surface: string, data: Record<string, unknown>): void {
	const store = getStore(surface);
	// Clear all existing keys
	for (const key of Object.keys(store.data)) {
		delete store.data[key];
	}
	// Assign new keys
	Object.assign(store.data, data);
}

/**
 * Apply an array of patch operations to a surface's data.
 *
 * Dispatches on the `op` discriminator:
 * - `set` — data op; routes through the dirty queue when the target path is
 *   currently being edited (preserves user input per Phase 12 D-A6).
 * - `set-node` / `delete-node` / `set-children` / `insert-child` /
 *   `remove-child` — node-tree ops, delegated to `surfaces.svelte.ts` which
 *   mutates the per-key proxy entries in place (required for focus
 *   preservation under sibling patches — D-A6).
 *
 * After the batch is applied, runs a single `gcOrphans(surface)` pass (D-A8)
 * so unreachable nodes created by the patch sequence are pruned once rather
 * than after every individual op.
 */
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
	// Run one GC pass per batch, scoped to the target surface (D-A8).
	gcOrphans(surface);
}

/**
 * Reset (clear) a surface's store.
 */
export function resetStore(surface: string): void {
	delete surfaces[surface];
}
