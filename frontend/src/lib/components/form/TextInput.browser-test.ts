import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import TextInput from './TextInput.svelte';
import { setFullState, getData, resetStore } from '$lib/store/data.svelte';
import { resetDirty } from '$lib/store/dirty.svelte';

beforeEach(() => {
	resetStore('test');
	resetDirty();
});

test('renders label when provided', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Email')).toBeVisible();
});

test('renders input element', async () => {
	const screen = await render(TextInput, {
		props: { props: {}, surface: 'test' },
	});

	const input = screen.baseElement.querySelector('input');
	expect(input).toBeTruthy();
});

test('renders error state', async () => {
	setFullState('test', { _errors: { email: 'Email is required' } });

	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/email', surface: 'test' },
	});

	await expect.element(screen.getByText('Email is required')).toBeVisible();
	// Error message should use destructive text
	const errorEl = screen.getByText('Email is required').element();
	expect(errorEl.className).toContain('text-destructive');
});

test('renders placeholder', async () => {
	const screen = await render(TextInput, {
		props: { props: { placeholder: 'Enter email' }, surface: 'test' },
	});

	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.placeholder).toBe('Enter email');
});
