import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import DataTable from './DataTable.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';

// Mock sendAction for sort and row-click assertions
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	resetStore('test');
	vi.clearAllMocks();
});

test('renders table with columns', async () => {
	const screen = await render(DataTable, {
		props: {
			props: {
				columns: [
					{ key: 'name', label: 'Name', sortable: true },
					{ key: 'email', label: 'Email', sortable: false },
				],
			},
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Name')).toBeVisible();
	await expect.element(screen.getByText('Email')).toBeVisible();
});

test('renders rows from bound data', async () => {
	setFullState('test', {
		contacts: {
			row1: { id: '1', name: 'Alice', email: 'alice@example.com' },
			row2: { id: '2', name: 'Bob', email: 'bob@example.com' },
		},
	});

	const screen = await render(DataTable, {
		props: {
			props: {
				columns: [
					{ key: 'name', label: 'Name' },
					{ key: 'email', label: 'Email' },
				],
			},
			bind: '/contacts',
			surface: 'test',
		},
	});

	await expect.element(screen.getByRole('cell', { name: 'Alice', exact: true })).toBeVisible();
	await expect.element(screen.getByRole('cell', { name: 'Bob', exact: true })).toBeVisible();
});

test('dispatches sort action on header click', async () => {
	const screen = await render(DataTable, {
		props: {
			props: {
				columns: [
					{ key: 'name', label: 'Name', sortable: true },
					{ key: 'email', label: 'Email', sortable: false },
				],
			},
			surface: 'test',
		},
	});

	await screen.getByText('Name').click();

	expect(sendAction).toHaveBeenCalledWith('sort', { column: 'name', direction: 'asc' });
});

test('dispatches select-row on row click', async () => {
	setFullState('test', {
		contacts: {
			row1: { id: '1', name: 'Alice' },
		},
	});

	const screen = await render(DataTable, {
		props: {
			props: {
				columns: [{ key: 'name', label: 'Name' }],
			},
			bind: '/contacts',
			action: { type: 'select', name: 'select-row' },
			surface: 'test',
		},
	});

	await screen.getByText('Alice').click();

	expect(sendAction).toHaveBeenCalledWith('select-row', { id: '1' }, undefined);
});
