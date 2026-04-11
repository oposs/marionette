import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import SvelteVirtualSmoke from './SvelteVirtualSmoke.svelte';

/**
 * Phase 13 Wave 0 smoke test — proves that @tanstack/svelte-virtual
 * (store-based adapter or our virtual-core-direct fallback) renders
 * windowed rows correctly under Svelte 5.
 *
 * Contract:
 *  - mounts with count=100, estimateSize=40
 *  - row-0 must be present and visible
 *  - row-99 must NOT be in the DOM (proof that virtualization is actually
 *    windowing, not rendering all 100 rows)
 *  - total inner height must be >= 4000px (100 * 40)
 */
test('store-based svelte-virtual renders virtualItems on mount', async () => {
	const screen = render(SvelteVirtualSmoke);

	// row-0 must render in the first visible window
	await expect.element(screen.getByTestId('row-0')).toBeVisible();

	// row-99 must NOT be in the DOM (virtualization proves itself)
	const farRow = screen.container.querySelector('[data-testid="row-99"]');
	expect(farRow).toBeNull();

	// Inner container must report total virtualized size
	const inner = screen.container.querySelector('[data-testid="inner"]') as HTMLElement | null;
	expect(inner).not.toBeNull();
	const heightStyle = inner!.style.height;
	const h = Number(heightStyle.replace('px', ''));
	expect(h).toBeGreaterThanOrEqual(4000);
});
