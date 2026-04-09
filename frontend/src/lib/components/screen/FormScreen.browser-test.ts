import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import FormScreen from './FormScreen.svelte';

// Mock dispatcher and data store
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

vi.mock('$lib/store/data.svelte', () => ({
	getData: vi.fn(() => null),
	getAllData: vi.fn(() => ({})),
	setData: vi.fn(),
}));

vi.mock('$lib/registry/registry', () => ({
	getComponent: vi.fn(() => undefined),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders title', async () => {
	const screen = await render(FormScreen, {
		props: { props: { title: 'Edit Contact' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Edit Contact')).toBeVisible();
});

test('renders back button when backAction provided', async () => {
	const screen = await render(FormScreen, {
		props: {
			props: {
				title: 'Edit Contact',
				back_action: { name: 'go-back', type: 'click' },
			},
			surface: 'test',
		},
	});

	const backButton = screen.getByRole('button', { name: 'Go back' });
	await expect.element(backButton).toBeVisible();

	// Verify ArrowLeft SVG icon is rendered inside back button
	const el = backButton.element() as HTMLButtonElement;
	expect(el.querySelector('svg')).toBeTruthy();
});

test('renders form element', async () => {
	const screen = await render(FormScreen, {
		props: { props: { title: 'New Contact' }, surface: 'test' },
	});

	const form = screen.baseElement.querySelector('form');
	expect(form).toBeTruthy();
});

test('dispatches back action on back button click', async () => {
	const screen = await render(FormScreen, {
		props: {
			props: {
				title: 'Edit Contact',
				back_action: { name: 'go-back', type: 'click', target: 'main' },
			},
			surface: 'test',
		},
	});

	await screen.getByRole('button', { name: 'Go back' }).click();

	expect(sendAction).toHaveBeenCalledWith('go-back', {}, 'main');
});

test('renders Card sections with Separator between them', async () => {
	const screen = await render(FormScreen, {
		props: {
			props: {
				title: 'Edit',
				sections: [
					{ title: 'Personal', fields: [] },
					{ title: 'Address', fields: [] },
				],
			},
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Personal')).toBeVisible();
	await expect.element(screen.getByText('Address')).toBeVisible();

	// Card.Root renders with data-slot="card"
	const cards = screen.baseElement.querySelectorAll('[data-slot="card"]');
	expect(cards.length).toBe(2);

	// Separator between sections
	const separators = screen.baseElement.querySelectorAll('[data-slot="separator"]');
	expect(separators.length).toBeGreaterThanOrEqual(1);
});
