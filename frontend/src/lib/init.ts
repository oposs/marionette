/**
 * App initialization: wires store, transport, rendering, and routing together.
 *
 * initMarionette() is the single entry point that a host app calls to
 * start the Marionette runtime. It registers default components, sets up
 * message handlers for all protocol message types, connects the WebSocket,
 * and initializes the router.
 */
import { registerDefaults } from './registry/defaults';
import { registerHandler, handleMessage, sendAction, resetHandlers } from './transport/dispatcher';
import { connect, disconnect } from './transport/websocket.svelte';
import { initRouter, updateUrl, destroyRouter } from './routing/router.svelte';
import { setFullState, applyPatch, setData } from './store/data.svelte';
import { setSurfaceTree } from './store/surfaces.svelte';
import { confirmOptimistic, rollbackOptimistic } from './store/optimistic.svelte';
import type { RenderMessage, PatchMessage, EventMessage, ErrorMessage } from './transport/messages';

/**
 * Initialize the Marionette runtime.
 *
 * @param wsUrl - WebSocket endpoint URL (default: '/ws')
 */
export function initMarionette(wsUrl: string = '/ws'): void {
	// Register built-in components
	registerDefaults();

	// Register protocol message handlers
	registerHandler('render', (raw: unknown) => {
		const msg = raw as RenderMessage;
		// Update data store for this surface
		setFullState(msg.surface, msg.data);
		// Update component tree for this surface
		setSurfaceTree(msg.surface, msg.root, msg.nodes);
		// If render message has a route in data, update the URL
		if (msg.data.route !== undefined) {
			updateUrl(msg.data.route as string);
		}
		// Confirm optimistic update if correlated
		if (msg.id) {
			confirmOptimistic(msg.id);
		}
	});

	registerHandler('patch', (raw: unknown) => {
		const msg = raw as PatchMessage;
		// Apply patch to main surface (protocol lacks surface field on patch messages)
		applyPatch('main', msg.patch);
		// Confirm optimistic update if correlated
		if (msg.id) {
			confirmOptimistic(msg.id);
		}
	});

	registerHandler('event', (raw: unknown) => {
		const msg = raw as EventMessage;
		// Event bus will be implemented in a later plan; log for now
		console.debug('[marionette] event:', msg.name, msg.hint);
	});

	registerHandler('error', (raw: unknown) => {
		const msg = raw as ErrorMessage;
		// Rollback optimistic update if correlated
		if (msg.id) {
			rollbackOptimistic(msg.id);
		}
		// Store errors at /_errors path on main surface
		setData('main', '/_errors', msg.errors);
	});

	// Connect WebSocket and route messages through the dispatcher
	connect(wsUrl, handleMessage);

	// Initialize URL router with sendAction for navigation
	initRouter(sendAction);
}

/**
 * Tear down the Marionette runtime.
 */
export function destroyMarionette(): void {
	disconnect();
	destroyRouter();
	resetHandlers();
}
