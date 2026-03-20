/**
 * Optimistic update with snapshot/restore.
 *
 * Before applying an optimistic update, snapshots the current values
 * at affected paths. Can restore (rollback) or confirm (discard snapshot).
 */
import type { PatchOperation } from '$lib/transport/messages.js';
import { getData, setData } from './data.svelte.js';

interface OptimisticEntry {
	surface: string;
	snapshots: Map<string, unknown>;
}

const pending = new Map<string, OptimisticEntry>();

/**
 * Apply an optimistic update: snapshot current values, then apply patches.
 */
export function applyOptimistic(
	correlationId: string,
	surface: string,
	operations: PatchOperation[]
): void {
	const snapshots = new Map<string, unknown>();
	for (const op of operations) {
		snapshots.set(op.path, getData(surface, op.path));
		setData(surface, op.path, op.value);
	}
	pending.set(correlationId, { surface, snapshots });
}

/**
 * Confirm an optimistic update: discard the snapshot (keep the changes).
 */
export function confirmOptimistic(correlationId: string): void {
	pending.delete(correlationId);
}

/**
 * Rollback an optimistic update: restore original values from snapshot.
 */
export function rollbackOptimistic(correlationId: string): void {
	const entry = pending.get(correlationId);
	if (!entry) return; // Unknown ID: no-op

	for (const [path, value] of entry.snapshots) {
		setData(entry.surface, path, value);
	}
	pending.delete(correlationId);
}
