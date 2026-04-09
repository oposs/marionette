import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import Container from './Container.svelte';

test('renders card variant with shadcn Card', async () => {
	const screen = await render(Container, {
		props: { props: { card: true }, surface: 'test' },
	});

	// Card.Root renders a div with data-slot="card" or card classes
	const card = screen.baseElement.querySelector('[data-slot="card"]');
	expect(card).toBeTruthy();
});

test('renders plain variant without card wrapper', async () => {
	const screen = await render(Container, {
		props: { props: {}, surface: 'test' },
	});

	// No card element when card prop is not set
	const card = screen.baseElement.querySelector('[data-slot="card"]');
	expect(card).toBeFalsy();
});
