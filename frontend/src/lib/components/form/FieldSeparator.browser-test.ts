/**
 * RED harness for FieldSeparator SDUI leaf (scaffolded by Plan 14-01).
 *
 * FieldSeparator.svelte does not exist yet — Wave 3 Plan 14-07 creates it
 * (per D-C2 preference: explicit separator nodes in the adjacency list).
 * Render contract: a bare `<Field.Separator />` — no props, no children.
 */
import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import FieldSeparator from './FieldSeparator.svelte';

test('renders a single Field.Separator element', async () => {
	const screen = await render(FieldSeparator, {
		props: { props: {}, surface: 'test' },
	});

	// shadcn field-separator renders a wrapping div with data-slot and a
	// bits-ui Separator child that exposes role="separator".
	const sep = screen.baseElement.querySelector(
		'[role="separator"], [data-slot="field-separator"]'
	);
	expect(sep).toBeTruthy();
});

test('takes no props without throwing', async () => {
	// Smoke: rendering with an empty props object must not throw.
	await render(FieldSeparator, {
		props: { props: {}, surface: 'test' },
	});
});
