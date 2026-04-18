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
		// Route by target surface (D-A3 — fixes the hardcoded-'main' bug).
		applyPatch(msg.surface, msg.patch);
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

	// Register hello handler: when server acknowledges connection,
	// initialize the router which sends the initial navigate action.
	// This ensures the navigate action is sent only after the WebSocket
	// is fully open (not dropped due to socket not yet connected).
	let routerInitialized = false;
	registerHandler('hello', () => {
		if (!routerInitialized) {
			routerInitialized = true;
			initRouter(sendAction);
		}
	});

	// Connect WebSocket and route messages through the dispatcher
	connect(wsUrl, handleMessage);

	// E2E / UAT test hooks: expose sendAction + setData on window so Playwright
	// tests and the Chrome UAT driver can dispatch protocol actions and
	// synthesize `/_errors/{bind}` patches programmatically.
	//
	// Phase 15 D-G1 — gate the assignments behind
	// `import.meta.env.DEV`. Vite tree-shakes the entire `if` block at
	// production build time (Rollup's DCE pass recognises the literal
	// `import.meta.env.DEV` constant inlined to `false`), so the hooks
	// disappear from the final bundle. UAT + E2E tests run under
	// `vite dev` where DEV is true, so the hooks remain available for
	// test drivers. Keeping both hook assignments inside the same outer
	// `if` — do NOT gate each hook separately — lets the dead-code
	// elimination strip the whole block.
	if (typeof window !== 'undefined' && import.meta.env.DEV) {
		(window as unknown as { __mrnSendAction: typeof sendAction }).__mrnSendAction =
			sendAction;
		(window as unknown as { __mrnSetData: typeof setData }).__mrnSetData =
			setData;
	}
}

/**
 * Tear down the Marionette runtime.
 */
export function destroyMarionette(): void {
	disconnect();
	destroyRouter();
	resetHandlers();
}
