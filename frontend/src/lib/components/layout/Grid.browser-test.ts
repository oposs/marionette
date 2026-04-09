import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import Grid from './Grid.svelte';

test('renders grid with inline grid-template-columns style', async () => {
	const screen = await render(Grid, {
		props: { props: { cols: 3 }, surface: 'test' },
	});

	const gridEl = screen.baseElement.querySelector('[style]');
	expect(gridEl).toBeTruthy();
	expect(gridEl!.getAttribute('style')).toContain('grid-template-columns: repeat(3, 1fr)');
});

test('renders flex layout when flex prop is set', async () => {
	const screen = await render(Grid, {
		props: { props: { flex: true }, surface: 'test' },
	});

	const flexEl = screen.baseElement.querySelector('.flex');
	expect(flexEl).toBeTruthy();
	// Flex mode should not have grid-template-columns style
	expect(flexEl!.getAttribute('style')).toBeFalsy();
});
