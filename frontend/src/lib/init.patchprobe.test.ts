/**
 * Phase 19 Plan 19-01 Task 1 — tests for installPatchProbe + icon registry.
 *
 * initMarionette() registers the real `patch` handler with its
 * installPatchProbe wrapper. We mock the side-effectful parts (websocket
 * connect, router, store writes) so the call is cheap, then dispatch a patch
 * message through the real dispatcher and assert the probe callback fires.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Side-effect mocks: skip real WebSocket + router initialization.
vi.mock('$lib/transport/websocket.svelte', () => ({
	connect: vi.fn(),
	disconnect: vi.fn(),
	send: vi.fn(),
}));
vi.mock('$lib/routing/router.svelte', () => ({
	initRouter: vi.fn(),
	updateUrl: vi.fn(),
	destroyRouter: vi.fn(),
}));
vi.mock('$lib/store/data.svelte', async () => {
	const actual = await vi.importActual<Record<string, unknown>>('$lib/store/data.svelte');
	return {
		...actual,
		applyPatch: vi.fn(),
		setFullState: vi.fn(),
		setData: vi.fn(),
	};
});
vi.mock('$lib/store/surfaces.svelte', () => ({
	setSurfaceTree: vi.fn(),
}));
vi.mock('$lib/store/optimistic.svelte', () => ({
	confirmOptimistic: vi.fn(),
	rollbackOptimistic: vi.fn(),
	applyOptimistic: vi.fn(),
}));
vi.mock('$lib/registry/defaults', () => ({
	registerDefaults: vi.fn(),
}));

import { installPatchProbe, initMarionette, destroyMarionette } from '$lib/init';
import { handleMessage, resetHandlers } from '$lib/transport/dispatcher';
import { applyPatch } from '$lib/store/data.svelte';
import { getIcon } from '$lib/registry/icons';

describe('init.ts — installPatchProbe + icon registry', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetHandlers();
		installPatchProbe(null);
		// Register the real patch handler (and friends) by calling initMarionette.
		// The websocket mock means the connect() call is a no-op.
		initMarionette('/ws');
	});

	it('Test 1: installed probe fires on patch apply with positive finite latencyMs', () => {
		const probe = vi.fn();
		installPatchProbe(probe);

		handleMessage({ type: 'patch', surface: 'content', patch: [] });

		expect(probe).toHaveBeenCalledTimes(1);
		const [dt] = probe.mock.calls[0];
		expect(typeof dt).toBe('number');
		expect(Number.isFinite(dt)).toBe(true);
		expect(dt).toBeGreaterThanOrEqual(0);
		// Real applyPatch (mocked) called with the right surface + patch list.
		expect(applyPatch).toHaveBeenCalledWith('content', []);
	});

	it('Test 2: installPatchProbe(null) silences subsequent patch callbacks', () => {
		const probe = vi.fn();
		installPatchProbe(probe);

		// Detach BEFORE dispatch
		installPatchProbe(null);

		handleMessage({ type: 'patch', surface: 'content', patch: [] });

		expect(probe).not.toHaveBeenCalled();
	});

	it('Test 3: icons registered — activity resolves to a distinct component from the fallback', () => {
		const known = getIcon('activity');
		const unknown = getIcon('this-icon-does-not-exist');
		expect(known).not.toBe(unknown);
	});

	it('Test 3b: all 17 Phase 19 icons resolve to non-fallback components', () => {
		const names = [
			'activity',
			'focus',
			'type',
			'languages',
			'move-horizontal',
			'gauge',
			'timer',
			'cpu',
			'zap',
			'layout-dashboard',
			'layers',
			'triangle-alert',
			'circle-check',
			'circle-x',
			'play',
			'pause',
			'rotate-ccw',
		];
		const fallback = getIcon('this-icon-does-not-exist');
		for (const n of names) {
			const c = getIcon(n);
			expect(c, `icon '${n}' must be registered (not fallback)`).not.toBe(fallback);
		}
	});

	// Teardown so repeated test runs don't leak handlers.
	it.skipIf(false)('Test 4: destroyMarionette() unwires cleanly', () => {
		expect(() => destroyMarionette()).not.toThrow();
	});
});
