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

// Phase 19 Plan 19-01: patch-latency instrumentation hook.
// Consumers: frontend/src/lib/exer02/invariants.svelte.ts (cursor + IME tick
// coordination — 19-RESEARCH.md §Pitfall 5) and
// frontend/src/lib/exer03/perf.svelte.ts (patch latency p95 — 19-RESEARCH.md
// §Pattern 4). The probe is module-local state; installed by calling
// installPatchProbe(fn) and removed by calling installPatchProbe(null).
let patchProbe: ((latencyMs: number) => void) | null = null;

/**
 * Install (or remove) a callback invoked once per patch message with the
 * elapsed time in milliseconds from just-before applyPatch to just-after.
 *
 * Pass `null` to detach. Only one probe is active at a time — re-installing
 * replaces the previous probe.
 */
export function installPatchProbe(fn: ((latencyMs: number) => void) | null): void {
	patchProbe = fn;
}

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
		// Phase 19 Plan 19-01: wrap applyPatch with a performance.now() probe
		// so EXER-02 invariants + EXER-03 perf modules can instrument per-patch
		// latency without duplicating patch-handling code. Probe is opt-in via
		// installPatchProbe(fn); no-op when the probe slot is null (default).
		const t0 = performance.now();
		// Route by target surface (D-A3 — fixes the hardcoded-'main' bug).
		applyPatch(msg.surface, msg.patch);
		const dt = performance.now() - t0;
		if (patchProbe) {
			patchProbe(dt);
		}
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

	// Phase 19 Plan 19-05 UAT harness: import the exerciser instrumentation
	// modules so their auto-arm blocks run in the browser. Without these
	// imports the modules are dead code; Plan 19-02/03/04 shipped them with
	// auto-arm gated on `typeof window !== 'undefined'` + a one-shot flag,
	// so importing here is the canonical activation seam. Dynamic import
	// keeps the modules out of the default bundle when the gallery app is
	// not the active consumer and lets SSR builds tree-shake them.
	//
	// Note (Phase 19-05 UAT finding, deferred to v1.3 seed): EXER-02 autoArm
	// and EXER-03 perf auto-arm locate their targets via
	// `document.getElementById('exer-02-focused-input')` / `#exer-03-perf-ttfp`,
	// but the frontend does not propagate SDUI component ids to DOM element
	// ids. These modules therefore no-op today. Visual screens still render
	// correctly and server-side protocol is verified via probe. The v1.3 seed
	// at .planning/seeds/v1.3-exerciser-instrumentation.md captures the
	// cleanest fix options (data-attribute propagation vs. Svelte-mount hook).
	if (typeof window !== 'undefined') {
		// Surface dynamic-import failures via console.error rather than
		// `void`-discarding the promise — otherwise a bundle/parse error or
		// a throw inside autoArm() silently disables the exerciser
		// instrumentation and UAT only sees "invariants never fire".
		// Matches the init module's error-hygiene style (see handler
		// registrations above).
		import('./exer01/observe.svelte').catch((e) =>
			console.error('[marionette] failed to load exer01/observe', e)
		);
		import('./exer02/invariants.svelte')
			.then((m) => m.autoArm())
			.catch((e) =>
				console.error('[marionette] failed to load/arm exer02/invariants', e)
			);
		import('./exer03/perf.svelte').catch((e) =>
			console.error('[marionette] failed to load exer03/perf', e)
		);
	}

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
