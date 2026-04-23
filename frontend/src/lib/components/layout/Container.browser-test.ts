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

test('renders lucide icon when props.icon is set (CAT-05 Plan 18-08 Task 0)', async () => {
	const screen = await render(Container, {
		props: { props: { icon: 'plus' }, surface: 'test' },
	});

	// getIcon('plus') returns the Lucide Plus component — rendered as an <svg>
	// with aria-hidden="true" per CAT-05 contract (sibling text label is the
	// a11y affordance).
	const svg = screen.baseElement.querySelector('svg');
	expect(svg).toBeTruthy();
	expect(svg?.getAttribute('aria-hidden')).toBe('true');
});

test('renders no icon when props.icon is absent (regression guard)', async () => {
	const screen = await render(Container, {
		props: { props: {}, surface: 'test' },
	});

	// Absent icon prop MUST not render any svg in the Container's subtree.
	const svg = screen.baseElement.querySelector('svg');
	expect(svg).toBeFalsy();
});
