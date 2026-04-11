import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import TextInput from './TextInput.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
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

// -----------------------------------------------------------------------------
// D-H4a: props.input_type (backend-authoritative, snake_case) — Phase 13 Plan 07
//
// The backend serializes TextInput's input type via `props.input_type`
// (see builders/standard.rs TextInput.input_type field). Prior to Phase 13
// the Svelte component incorrectly read `props.type`, so password fields
// rendered as `<input type="text">`. Pre-deployment posture: no back-compat
// fallback to `props.type` — the only authoritative source is `input_type`.
// -----------------------------------------------------------------------------

test('defaults to type="text" when no input_type set', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Name' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('text');
});

test('reads props.input_type (backend-authoritative) — password field', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Password', input_type: 'password' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('password');
});

test('reads props.input_type for email', async () => {
	const screen = await render(TextInput, {
		props: { props: { label: 'Email', input_type: 'email' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('email');
});

test('ignores legacy props.type (no backward-compat fallback per pre-deployment posture)', async () => {
	// Pre-deployment: there is no deployed base shipping props.type.
	// If a caller mistakenly passes props.type, it is silently ignored and
	// the input falls back to the default 'text'. This documents the
	// no-compat-shim posture and guards against accidental reintroduction.
	const screen = await render(TextInput, {
		props: { props: { label: 'Legacy', type: 'password' }, surface: 'test' },
	});
	const input = screen.baseElement.querySelector('input') as HTMLInputElement;
	expect(input.getAttribute('type')).toBe('text');
});
