/**
 * RED harness for RadioGroup SDUI leaf (scaffolded by Plan 14-01).
 *
 * RadioGroup.svelte does not exist yet — Wave 2 Plan 14-06 creates it. These
 * tests intentionally fail with import-resolve until then. Internal wrap
 * contract per D-B1:
 *   <Field.Field data-invalid>
 *     <Field.Label>{label}</Field.Label>
 *     <RadioGroup.Root bind:value={value}>
 *       {#each options as opt}
 *         <RadioGroup.Item value={opt.value} id={..}/> <Label>{opt.label}</Label>
 *         {#if opt.description}<span class="text-muted-foreground">…</span>{/if}
 *       {/each}
 *     </RadioGroup.Root>
 *     <Field.Error>{err}</Field.Error>
 *   </Field.Field>
 */
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
// @ts-expect-error - RadioGroup.svelte is scaffolded in Wave 2 (Plan 14-06).
// Remove this directive once that plan lands the concrete component.
import RadioGroup from './RadioGroup.svelte';
import { setData, resetStore } from '$lib/store/data.svelte';

beforeEach(() => {
	resetStore('test');
});

const fruitOptions = [
	{ value: 'a', label: 'Apple' },
	{ value: 'b', label: 'Banana' },
];

test('renders group with legend from props.label as Field.Label', async () => {
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Pick one')).toBeVisible();
});

test('renders each option with a radio input', async () => {
	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			surface: 'test',
		},
	});

	// Bits-ui RadioGroup.Item renders with role="radio"
	const radios = screen.baseElement.querySelectorAll('[role="radio"]');
	expect(radios.length).toBe(2);
});

test('selects option when bind value matches', async () => {
	setData('test', '/choice', 'b');

	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			surface: 'test',
		},
	});

	const radios = screen.baseElement.querySelectorAll('[role="radio"]');
	// Second radio (value=b) should be checked
	expect(radios[1]?.getAttribute('aria-checked')).toBe('true');
});

test('shows Field.Error and data-invalid when /_errors/{bind} is set', async () => {
	setData('test', '/_errors/choice', 'must pick');

	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: fruitOptions },
			bind: '/choice',
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('must pick')).toBeVisible();
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper?.hasAttribute('data-invalid')).toBe(true);
});

test('renders per-option description text adjacent to option label', async () => {
	// A4 assumption check — if shadcn RadioGroup doesn't support per-item
	// description, the rendered DOM is a plain-text span adjacent to label.
	const optsWithDesc = [
		{ value: 'a', label: 'Apple', description: 'red fruit' },
		{ value: 'b', label: 'Banana' },
	];

	const screen = await render(RadioGroup, {
		props: {
			props: { label: 'Pick one', options: optsWithDesc },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('red fruit')).toBeVisible();
});
