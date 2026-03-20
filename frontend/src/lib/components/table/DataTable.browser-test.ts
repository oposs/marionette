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

test('renders columns and rows from data', async () => {
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
					{ key: 'name', label: 'Name', sortable: true },
					{ key: 'email', label: 'Email', sortable: false },
				],
				totalRows: 2,
			},
			bind: '/contacts',
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Name')).toBeVisible();
	await expect.element(screen.getByText('Email')).toBeVisible();
	await expect.element(screen.getByRole('cell', { name: 'Alice', exact: true })).toBeVisible();
	await expect.element(screen.getByRole('cell', { name: 'Bob', exact: true })).toBeVisible();
});

test('sort dispatches action on sortable column header click', async () => {
	setFullState('test', { contacts: {} });

	const screen = await render(DataTable, {
		props: {
			props: {
				columns: [
					{ key: 'name', label: 'Name', sortable: true },
					{ key: 'email', label: 'Email', sortable: false },
				],
				totalRows: 0,
			},
			bind: '/contacts',
			surface: 'test',
		},
	});

	// Click the sortable 'Name' header
	await screen.getByText('Name').click();

	expect(sendAction).toHaveBeenCalledWith('sort', { column: 'name', direction: 'asc' });
});

test('virtual scroll container has correct total height', async () => {
	setFullState('test', { contacts: {} });

	const screen = await render(DataTable, {
		props: {
			props: {
				columns: [{ key: 'name', label: 'Name' }],
				totalRows: 100,
			},
			bind: '/contacts',
			surface: 'test',
		},
	});

	// DataTable uses ROW_HEIGHT=48, so 100 rows = 4800px total height
	const heightDiv = screen.baseElement.querySelector('[style*="height: 4800px"]');
	expect(heightDiv).toBeTruthy();
});
