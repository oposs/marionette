// Optimistic update stub -- full implementation in Plan 01
// This provides the applyOptimistic function that dispatcher.ts needs

import type { PatchOperation } from '$lib/transport/messages';

export function applyOptimistic(
  _correlationId: string,
  _surface: string,
  _operations: PatchOperation[]
): void {
  // Stub: will be implemented in Plan 01 (data store plan)
}
