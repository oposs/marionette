import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import TableScreen from './TableScreen.svelte';

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
	const screen = await render(TableScreen, {
		props: { props: { title: 'Contacts' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Contacts')).toBeVisible();
});

test('renders toolbar area with action buttons', async () => {
	const screen = await render(TableScreen, {
		props: {
			props: {
				title: 'Contacts',
				toolbar: [
					{ name: 'new-contact', type: 'primary', label: 'New Contact' },
				],
			},
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('New Contact')).toBeVisible();
});

test('dispatches toolbar action on click', async () => {
	const screen = await render(TableScreen, {
		props: {
			props: {
				title: 'Companies',
				toolbar: [
					{ name: 'new-company', type: 'primary', label: 'New', target: 'main' },
				],
			},
			surface: 'test',
		},
	});

	await screen.getByText('New').click();

	expect(sendAction).toHaveBeenCalledWith(
		'new-company',
		expect.any(Object),
		'main',
	);
});

test('renders mobile filter toggle when filters provided', async () => {
	const screen = await render(TableScreen, {
		props: {
			props: {
				title: 'Contacts',
				filters: [{ id: 'f1', label: 'Name' }],
				nodes: {},
			},
			surface: 'test',
		},
	});

	// Mobile filter toggle button should exist (hidden on md+ via CSS)
	await expect.element(screen.getByText('Show Filters')).toBeInTheDocument();
});
