import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import SelectInput from './SelectInput.svelte';
import { resetStore } from '$lib/store/data.svelte';
import { resetDirty } from '$lib/store/dirty.svelte';

beforeEach(() => {
	resetStore('test');
	resetDirty();
});

test('renders label when provided', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { label: 'Country', options: [] },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Country')).toBeVisible();
});

test('renders trigger with placeholder', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { placeholder: 'Pick one', options: [] },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Pick one')).toBeVisible();
});

test('renders select trigger', async () => {
	const screen = await render(SelectInput, {
		props: {
			props: { options: [{ value: 'a', label: 'Alpha' }] },
			surface: 'test',
		},
	});

	// shadcn Select renders a button as trigger
	const trigger = screen.baseElement.querySelector('[data-slot="select-trigger"]');
	expect(trigger).toBeTruthy();
});
