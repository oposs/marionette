/**
 * RED harness for Switch SDUI leaf (scaffolded by Plan 14-01).
 *
 * Switch.svelte does not exist yet — Wave 2 Plan 14-06 creates it. Internal
 * wrap contract per D-B1:
 *   <Field.Field orientation="horizontal" data-invalid>
 *     <Field.Label>{label}</Field.Label>
 *     <Switch bind:checked aria-invalid />
 *     <Field.Error>{err}</Field.Error>
 *   </Field.Field>
 */
import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import Switch from './Switch.svelte';
import { setData, getData, resetStore } from '$lib/store/data.svelte';

beforeEach(() => {
	resetStore('test');
});

test('renders switch with label', async () => {
	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Enable')).toBeVisible();
	await expect.element(screen.getByRole('switch')).toBeInTheDocument();
});

test('reflects bind boolean value (aria-checked true when /enabled=true)', async () => {
	setData('test', '/enabled', true);

	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, bind: '/enabled', surface: 'test' },
	});

	const sw = screen.getByRole('switch').element();
	expect(sw.getAttribute('aria-checked')).toBe('true');
});

test('toggling emits setData on bind', async () => {
	setData('test', '/enabled', false);

	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, bind: '/enabled', surface: 'test' },
	});

	const sw = screen.getByRole('switch').element() as HTMLElement;
	sw.click();
	// Give the click event a tick to flow through.
	await new Promise((r) => setTimeout(r, 50));

	expect(getData('test', '/enabled')).toBe(true);
});

test('data-invalid on wrapper when /_errors/{bind} is set', async () => {
	setData('test', '/_errors/enabled', 'required');

	const screen = await render(Switch, {
		props: { props: { label: 'Enable' }, bind: '/enabled', surface: 'test' },
	});

	await expect.element(screen.getByText('required')).toBeVisible();
	const wrapper = screen.baseElement.querySelector('[data-slot="field"]');
	expect(wrapper?.hasAttribute('data-invalid')).toBe(true);
});
