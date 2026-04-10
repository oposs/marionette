/**
 * Reactive data store with JSON Pointer get/set/patch.
 *
 * Uses Svelte 5 $state rune for reactivity. Each surface has its own
 * data namespace. Components bind to paths within the store.
 */
import type { PatchOperation } from '$lib/transport/messages.js';
import { resolvePointer, setAtPointer } from './pointer.js';
import { isDirty, queuePatch } from './dirty.svelte.js';

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
 * Skips patches to dirty paths (queues them instead).
 *
 * Task 2 extends this to dispatch on the `op` discriminator and route node
 * ops (`set-node`, `delete-node`, `set-children`, `insert-child`,
 * `remove-child`) to the surface tree store.
 */
export function applyPatch(surface: string, operations: PatchOperation[]): void {
	for (const op of operations) {
		if (op.op !== 'set') continue;
		if (isDirty(op.path)) {
			queuePatch(op.path, op);
		} else {
			setData(surface, op.path, op.value);
		}
	}
}

/**
 * Reset (clear) a surface's store.
 */
export function resetStore(surface: string): void {
	delete surfaces[surface];
}
