/**
 * RED harness for FieldSet SDUI structural component (scaffolded by Plan 14-01).
 *
 * FieldSet.svelte does not exist yet — Wave 3 Plan 14-07 creates it.
 * Render contract per D-C1:
 *   <Field.Set>
 *     <Field.Legend>{legend}</Field.Legend>
 *     <Field.Description>{description}</Field.Description>
 *     <Field.Group class="{auto-responsive grid classes}" style="..">
 *       {@render children()}
 *     </Field.Group>
 *   </Field.Set>
 *
 * Default grid: `grid-cols-1 md:grid-cols-2` (D-C3).
 * `cols` override: inline `grid-template-columns: repeat(N, minmax(0, 1fr))`
 * per Pitfall #1 (Tailwind v4 cannot JIT dynamic class names).
 */
import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
// @ts-expect-error - FieldSet.svelte is scaffolded in Wave 3 (Plan 14-07).
// Remove this directive once that plan lands the concrete component.
import FieldSet from './FieldSet.svelte';
import { createRawSnippet } from 'svelte';

test('renders Field.Legend when props.legend is set', async () => {
	const screen = await render(FieldSet, {
		props: { props: { legend: 'Contact' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Contact')).toBeVisible();
	// Field.Legend renders as <legend> inside a <fieldset>.
	const legend = screen.baseElement.querySelector('legend');
	expect(legend?.textContent).toContain('Contact');
});

test("default grid class includes 'grid-cols-1' and 'md:grid-cols-2'", async () => {
	const screen = await render(FieldSet, {
		props: { props: {}, surface: 'test' },
	});

	const group = screen.baseElement.querySelector('[data-slot="field-group"]');
	expect(group?.className).toContain('grid-cols-1');
	expect(group?.className).toContain('md:grid-cols-2');
});

test('cols prop produces inline grid-template-columns style', async () => {
	const screen = await render(FieldSet, {
		props: { props: { cols: 3 }, surface: 'test' },
	});

	const group = screen.baseElement.querySelector('[data-slot="field-group"]') as HTMLElement;
	expect(group?.getAttribute('style') ?? '').toContain('repeat(3, minmax(0, 1fr))');
});

test('renders children via Snippet', async () => {
	const childSnippet = createRawSnippet(() => ({
		render: () => `<span data-testid="child">inside</span>`,
	}));

	const screen = await render(FieldSet, {
		props: { props: {}, surface: 'test', children: childSnippet },
	});

	const group = screen.baseElement.querySelector('[data-slot="field-group"]');
	expect(group?.textContent).toContain('inside');
});

test('renders Field.Description when props.description set', async () => {
	const screen = await render(FieldSet, {
		props: {
			props: { legend: 'Contact', description: 'Basic identity fields.' },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Basic identity fields.')).toBeVisible();
});
