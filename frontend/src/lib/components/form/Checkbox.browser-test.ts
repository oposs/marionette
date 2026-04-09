import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import Checkbox from './Checkbox.svelte';
import { resetStore } from '$lib/store/data.svelte';

beforeEach(() => {
	resetStore('test');
});

test('renders with label', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Active' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Active')).toBeVisible();
});

test('renders checkbox element', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Active' }, surface: 'test' },
	});

	// shadcn checkbox renders a button with role="checkbox"
	const checkbox = screen.getByRole('checkbox');
	await expect.element(checkbox).toBeVisible();
});

test('renders disabled state', async () => {
	const screen = await render(Checkbox, {
		props: { props: { label: 'Active', disabled: true }, surface: 'test' },
	});

	const checkbox = screen.getByRole('checkbox');
	await expect.element(checkbox).toBeDisabled();
});
