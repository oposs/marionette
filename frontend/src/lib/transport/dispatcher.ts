// Message dispatcher: routes incoming messages by type, sends actions

import { send } from './websocket.svelte';
import { applyOptimistic } from '$lib/store/optimistic.svelte';
import type { ActionMessage, PatchOperation } from './messages';

type MessageHandler = (msg: unknown) => void;

let handlers: Record<string, MessageHandler> = {};

/**
 * Register a handler for a given message type.
 */
export function registerHandler(type: string, handler: MessageHandler): void {
  handlers[type] = handler;
}

/**
 * Route an incoming message to its registered handler.
 * Logs a warning for unhandled message types.
 */
export function handleMessage(raw: unknown): void {
  const msg = raw as Record<string, unknown>;
  const type = msg.type as string;
  const handler = handlers[type];
  if (handler) {
    handler(msg);
  } else {
    console.warn('Unhandled message type:', type);
  }
}

/**
 * Send an action message to the backend.
 * Generates a correlation ID via crypto.randomUUID().
 * If optimistic patch is provided, applies it immediately before sending.
 */
export function sendAction(
  name: string,
  payload?: Record<string, unknown>,
  source?: string,
  optimistic?: { patch: PatchOperation[] }
): void {
  const id = crypto.randomUUID();

  const msg: ActionMessage = {
    type: 'action',
    id,
    name
  };

  if (payload !== undefined) {
    msg.payload = payload;
  }

  if (source !== undefined) {
    msg.source = source;
  }

  if (optimistic) {
    msg.optimistic = optimistic;
    // Apply optimistic update locally before sending
    applyOptimistic(id, '', optimistic.patch);
  }

  send(msg);
}

/**
 * Clear all registered handlers (for testing).
 */
export function resetHandlers(): void {
  handlers = {};
}
