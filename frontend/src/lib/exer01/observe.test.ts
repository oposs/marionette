// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock sendAction so we can inspect the call. `svelte`'s `getContext` is also
// mocked to avoid the "lifecycle_outside_component" runtime error — we don't
// exercise a real Svelte component tree here, just the pure probe function.
const sentinel = vi.fn();
vi.mock('$lib/transport/dispatcher', () => ({ sendAction: sentinel }));
vi.mock('svelte', async () => {
	const actual = await vi.importActual<typeof import('svelte')>('svelte');
	return {
		...actual,
		getContext: () => undefined
	};
});

describe('probeNestability', () => {
	beforeEach(() => {
		sentinel.mockReset();
		(window as unknown as { __mrnExer01OuterSidebar?: unknown }).__mrnExer01OuterSidebar =
			undefined;
		// Remove any inner-wrap nodes a previous test might have injected.
		document.body.innerHTML = '';
	});

	it('sends a 4-dimension report payload', async () => {
		// Seed an outer sidebar state handle
		(window as unknown as { __mrnExer01OuterSidebar?: unknown }).__mrnExer01OuterSidebar = {
			open: false
		};
		// Add the inner-wrap element so computed-style lookup is non-null
		const el = document.createElement('div');
		el.id = 'exer-01-inner-wrap';
		document.body.appendChild(el);

		const { probeNestability } = await import('./observe.svelte');
		await probeNestability();

		expect(sentinel).toHaveBeenCalledTimes(1);
		const [action, payload] = sentinel.mock.calls[0];
		expect(action).toBe('gallery-demo/exer-01/report');
		const p = payload as Record<string, { state: string; details: string }>;
		for (const key of [
			'provider-context',
			'mobile-sheet',
			'keyboard-shortcuts',
			'sidebar-tokens'
		]) {
			expect(p).toHaveProperty(key);
			expect(p[key]).toHaveProperty('state');
			expect(p[key]).toHaveProperty('details');
			expect(['PASS', 'FAIL', 'WARN']).toContain(p[key].state);
		}
	});
});
