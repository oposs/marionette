import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import Spinner from './Spinner.svelte';

test('renders Loader2 icon with animate-spin', async () => {
	const screen = await render(Spinner, {
		props: { props: {}, surface: 'test' },
	});

	const svg = screen.baseElement.querySelector('svg');
	expect(svg).toBeTruthy();
	expect(svg!.classList.contains('animate-spin')).toBe(true);
});

test('uses text-primary color', async () => {
	const screen = await render(Spinner, {
		props: { props: {}, surface: 'test' },
	});

	const svg = screen.baseElement.querySelector('svg');
	expect(svg).toBeTruthy();
	expect(svg!.classList.contains('text-primary')).toBe(true);
});
