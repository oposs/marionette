/**
 * Dirty field tracking with pending patch queue.
 *
 * While a field is dirty (user is editing), incoming server patches
 * for that path are queued. On clearDirty, queued patches are applied.
 */
import type { PatchOperation } from '$lib/transport/messages.js';

const dirtyPaths = new Set<string>();
const pendingPatches = new Map<string, PatchOperation[]>();

/**
 * Mark a path as dirty (user is editing this field).
 */
export function markDirty(path: string): void {
	dirtyPaths.add(path);
}

/**
 * Clear dirty state for a path and apply any queued patches.
 * The applyFn callback is called for each queued patch operation.
 */
export function clearDirty(path: string, applyFn: (op: PatchOperation) => void): void {
	dirtyPaths.delete(path);
	const queued = pendingPatches.get(path);
	if (queued) {
		for (const op of queued) {
			applyFn(op);
		}
		pendingPatches.delete(path);
	}
}

/**
 * Check if a path (or any parent path) is currently dirty.
 */
export function isDirty(path: string): boolean {
	for (const dirty of dirtyPaths) {
		if (path === dirty || path.startsWith(dirty + '/')) {
			return true;
		}
	}
	return false;
}

/**
 * Queue a patch operation for a dirty path.
 * Will be applied when clearDirty is called.
 */
export function queuePatch(path: string, op: PatchOperation): void {
	let queue = pendingPatches.get(path);
	if (!queue) {
		queue = [];
		pendingPatches.set(path, queue);
	}
	queue.push(op);
}

/**
 * Reset all dirty state (for testing or reconnection).
 */
export function resetDirty(): void {
	dirtyPaths.clear();
	pendingPatches.clear();
}
