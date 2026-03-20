import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import TextInput from './TextInput.svelte';
import { setFullState, getData, resetStore } from '$lib/store/data.svelte';
import { isDirty, resetDirty } from '$lib/store/dirty.svelte';

beforeEach(() => {
	resetStore('test');
	resetDirty();
});

test('renders input with label from props', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Email')).toBeVisible();
	// Input element should exist
	const input = screen.baseElement.querySelector('input');
	expect(input).toBeTruthy();
});

test('binds value from store', async () => {
	setFullState('test', { user: { email: 'test@example.com' } });

	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/user/email', surface: 'test' },
	});

	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.value).toBe('test@example.com');
});

test('updates store on input', async () => {
	setFullState('test', { user: { email: '' } });

	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/user/email', surface: 'test' },
	});

	const input = screen.getByRole('textbox');
	await input.fill('new@example.com');

	expect(getData('test', '/user/email')).toBe('new@example.com');
});

test('marks dirty on focus, clears on blur', async () => {
	setFullState('test', { user: { email: '' } });

	const screen = await render(TextInput, {
		props: { props: { label: 'Email' }, bind: '/user/email', surface: 'test' },
	});

	const input = screen.getByRole('textbox');

	// Focus input -> verify dirty
	await input.click();
	expect(isDirty('/user/email')).toBe(true);

	// Blur by clicking outside the input
	await screen.baseElement.querySelector('div')!.dispatchEvent(
		new FocusEvent('focusin', { bubbles: true })
	);
	// Trigger blur on the input
	(screen.baseElement.querySelector('input') as HTMLInputElement).blur();

	// After blur, dirty should be cleared
	// Allow a tick for the event handler
	await new Promise((r) => setTimeout(r, 50));
	expect(isDirty('/user/email')).toBe(false);
});
