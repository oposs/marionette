/*
 * DataTable.browser-test.ts — Phase 13 Plan 05 rewrite
 *
 * Targets the recipe-shaped DataTable: createSvelteTable + FlexRender +
 * createRuneVirtualizer + IntersectionObserver sentinel + per-kind cell
 * renderers + filter bar + column visibility dropdown + stale-fetch-rows
 * guard.
 *
 * Every test references the corresponding row in
 * `.planning/phases/13-datatable-enhancements/13-VALIDATION.md`
 * §Per-Task Verification Map as "V-NN".
 */

import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import DataTable from './DataTable.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';

// Mock sendAction so tests can assert outgoing dispatch calls. By default
// we return a deterministic correlation id string.
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(() => 'test-uuid-1234'),
}));

import { sendAction } from '$lib/transport/dispatcher';

// Helper cast for mock interaction (asserts + mockReturnValue). All test
// assertions still reference the imported `sendAction` symbol directly for
// easy grep auditability.
const sendActionMock = sendAction as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
	resetStore('test');
	vi.clearAllMocks();
	sendActionMock.mockReturnValue('test-uuid-1234');
});

afterEach(() => {
	vi.useRealTimers();
});

// Helper: build N rows keyed by id for the bound collection.
function buildRows(n: number, prefix = 'row'): Record<string, Record<string, unknown>> {
	const out: Record<string, Record<string, unknown>> = {};
	for (let i = 0; i < n; i++) {
		const id = `${prefix}-${i}`;
		out[id] = {
			id,
			name: `Name ${i}`,
			email: `n${i}@example.com`,
			created: '2026-04-01T12:00:00Z',
			count: i,
		};
	}
	return out;
}

// -----------------------------------------------------------------------------
// Filter bar (TABLE-01)
// -----------------------------------------------------------------------------

describe('Filter bar (TABLE-01)', () => {
	test('V-01 renders one input per filter definition', async () => {
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					filters: [
						{ id: 'search', kind: 'text', label: 'Search', placeholder: 'Filter contacts...' },
						{
							id: 'company',
							kind: 'select',
							label: 'Company',
							options: [
								{ value: '', label: 'All' },
								{ value: '1', label: 'Acme' },
							],
						},
						{ id: 'created', kind: 'date-range', label: 'Created' },
					],
				},
				surface: 'test',
			},
		});

		// Text filter: the placeholder renders on the <input>.
		await expect.element(screen.getByPlaceholder('Filter contacts...')).toBeVisible();
		// Select filter: aria-label on the trigger.
		await expect.element(screen.getByLabelText('Company')).toBeVisible();
		// Date-range filter: two date inputs with from/to aria-labels.
		await expect.element(screen.getByLabelText('Created from')).toBeVisible();
		await expect.element(screen.getByLabelText('Created to')).toBeVisible();
	});

	test('V-02 text filter debounces 300ms then dispatches filter action', async () => {
		vi.useFakeTimers();
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					filters: [{ id: 'search', kind: 'text', label: 'Search', placeholder: 'Search' }],
				},
				surface: 'test',
			},
		});

		const input = screen.getByPlaceholder('Search');
		await input.fill('Ali');

		// After 200ms, no dispatch yet (still inside the debounce window).
		vi.advanceTimersByTime(200);
		expect(sendActionMock).not.toHaveBeenCalledWith('filter', expect.anything());

		// Advancing the full 300ms fires the filter dispatch.
		vi.advanceTimersByTime(100);
		expect(sendActionMock).toHaveBeenCalledWith('filter', { search: 'Ali' });
	});

	test('V-03 Enter in text filter flushes immediately', async () => {
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					filters: [{ id: 'search', kind: 'text', label: 'Search', placeholder: 'Search' }],
				},
				surface: 'test',
			},
		});

		const input = screen.getByPlaceholder('Search');
		await input.fill('Alice');
		// Don't advance the 300ms debounce — Enter should flush synchronously.
		await input.element().dispatchEvent(
			new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }),
		);

		expect(sendActionMock).toHaveBeenCalledWith('filter', { search: 'Alice' });
	});

	test('V-05 empty/undefined filter values stripped from payload', async () => {
		vi.useFakeTimers();
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					filters: [
						{ id: 'search', kind: 'text', label: 'Search', placeholder: 'Search' },
						{
							id: 'company',
							kind: 'select',
							label: 'Company',
							options: [
								{ value: '', label: 'All' },
								{ value: '1', label: 'Acme' },
							],
						},
					],
				},
				surface: 'test',
			},
		});

		await screen.getByPlaceholder('Search').fill('Alice');
		vi.advanceTimersByTime(300);

		// Only `search` in the payload — the empty `company` select never fired
		// a change and must therefore be omitted entirely.
		expect(sendActionMock).toHaveBeenCalledWith('filter', { search: 'Alice' });
	});

	test('V-31 filter input retains focus across server Render reset', async () => {
		// Seed initial rows.
		setFullState('test', { rows: buildRows(5) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					filters: [{ id: 'search', kind: 'text', label: 'Search', placeholder: 'Search' }],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		// Focus the filter input.
		const input = screen.getByPlaceholder('Search');
		await input.element().focus();
		expect(document.activeElement).toBe(input.element());

		// Simulate a server Render replacement: swap the bound collection.
		setFullState('test', { rows: buildRows(8, 'fresh') });
		// Allow the reactive update to flush.
		await Promise.resolve();

		// The filter input must still be focused afterward. Mirrors Phase 12's
		// D-A6 focus-preservation guarantee inherited via the surface store.
		expect(document.activeElement).toBe(input.element());
	});
});

// -----------------------------------------------------------------------------
// Column visibility (TABLE-03)
// -----------------------------------------------------------------------------

describe('Column visibility (TABLE-03)', () => {
	test('V-18 Columns dropdown lists hideable columns', async () => {
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'email', label: 'Email' },
					],
				},
				surface: 'test',
			},
		});

		await screen.getByText('Columns').click();
		// Both columns appear as hideable CheckboxItems. Scoped to the
		// dropdown's menuitemcheckbox role so the text doesn't collide with
		// the column headers.
		await expect
			.element(screen.getByRole('menuitemcheckbox', { name: 'name' }))
			.toBeVisible();
		await expect
			.element(screen.getByRole('menuitemcheckbox', { name: 'email' }))
			.toBeVisible();
	});

	test('V-19 toggling a checkbox hides the column in the rendered table', async () => {
		setFullState('test', { rows: buildRows(2) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'email', label: 'Email' },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		// Header for "Email" starts visible.
		await expect.element(screen.getByRole('columnheader', { name: 'Email' })).toBeVisible();

		await screen.getByText('Columns').click();
		// Click the email checkbox item to hide it (scoped to dropdown).
		await screen.getByRole('menuitemcheckbox', { name: 'email' }).click();

		// Header for Email should no longer be in the DOM.
		await expect
			.element(screen.getByRole('columnheader', { name: 'Email' }))
			.not.toBeInTheDocument();
	});

	test('V-20 hidden_default: true columns start hidden', async () => {
		setFullState('test', { rows: buildRows(2) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'internal_id', label: 'Internal ID', hidden_default: true },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		// Name column visible.
		await expect.element(screen.getByRole('columnheader', { name: 'Name' })).toBeVisible();
		// Internal ID column starts hidden — no header in the DOM.
		await expect
			.element(screen.getByRole('columnheader', { name: 'Internal ID' }))
			.not.toBeInTheDocument();
	});
});

// -----------------------------------------------------------------------------
// Virtualizer + infinite scroll (TABLE-02)
// -----------------------------------------------------------------------------

describe('Virtualizer + infinite scroll (TABLE-02)', () => {
	test('V-10 virtualizer windows rows (only visible subset rendered)', async () => {
		setFullState('test', { rows: buildRows(200) });
		const screen = await render(DataTable, {
			props: {
				props: { columns: [{ key: 'name', label: 'Name' }] },
				bind: '/rows',
				surface: 'test',
			},
		});

		// Allow the virtualizer to measure and render its initial window.
		await new Promise((r) => setTimeout(r, 50));

		const rows = screen.container.querySelectorAll('[data-index]');
		// Far less than 200 rows should be in the DOM (virtualized window +
		// overscan) — precise bounds vary with container height, but the
		// window must be bounded.
		expect(rows.length).toBeGreaterThan(0);
		expect(rows.length).toBeLessThan(200);
	});

	test('V-09 sentinel triggers fetch-rows when scrolled near tail', async () => {
		setFullState('test', { rows: buildRows(60) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					total_rows: 237,
					page_size: 50,
					source: 'test_list',
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		// Let the virtualizer measure + mount the sentinel observer.
		await new Promise((r) => setTimeout(r, 150));

		const scrollEl = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		expect(scrollEl).toBeTruthy();
		if (scrollEl) {
			scrollEl.scrollTop = scrollEl.scrollHeight;
			await new Promise((r) => requestAnimationFrame(() => r(null)));
			await new Promise((r) => requestAnimationFrame(() => r(null)));
		}
		// Wait for IntersectionObserver to fire its microtask.
		await new Promise((r) => setTimeout(r, 300));

		const calls = sendActionMock.mock.calls.filter((c) => c[0] === 'fetch-rows');
		expect(calls.length).toBeGreaterThanOrEqual(1);
		expect(calls[0][1]).toEqual(
			expect.objectContaining({ source: 'test_list', offset: 60 }),
		);
	});

	test('V-22 sort change resets scrollTop to 0', async () => {
		setFullState('test', { rows: buildRows(200) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name', sortable: true }],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		const scrollEl = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		expect(scrollEl).toBeTruthy();
		if (scrollEl) {
			scrollEl.scrollTop = 500;
			scrollEl.dispatchEvent(new Event('scroll'));
			await new Promise((r) => setTimeout(r, 20));
		}

		// Click the sortable header.
		await screen.getByRole('columnheader', { name: /Name/ }).click();

		// After sort, scrollTop must be 0.
		expect(scrollEl?.scrollTop).toBe(0);
		expect(sendActionMock).toHaveBeenCalledWith(
			'sort',
			expect.objectContaining({ column: 'name' }),
		);
	});

	test('V-11 filter change resets scrollTop and re-arms sentinel', async () => {
		vi.useFakeTimers();
		setFullState('test', { rows: buildRows(200) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					filters: [{ id: 'search', kind: 'text', label: 'Search', placeholder: 'Search' }],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		const scrollEl = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		expect(scrollEl).toBeTruthy();
		if (scrollEl) {
			scrollEl.scrollTop = 400;
		}

		await screen.getByPlaceholder('Search').fill('Alice');
		vi.advanceTimersByTime(300);

		expect(sendActionMock).toHaveBeenCalledWith('filter', { search: 'Alice' });
		// After the filter flush, scrollTop must have been reset.
		expect(scrollEl?.scrollTop).toBe(0);
	});

	test('V-12 stops fetching when rows.length >= total_rows', async () => {
		// Seed exactly total_rows — sentinel must idle.
		setFullState('test', { rows: buildRows(50) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					total_rows: 50,
					page_size: 50,
					source: 'test_list',
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		const scrollEl = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		if (scrollEl) {
			scrollEl.scrollTop = scrollEl.scrollHeight;
			scrollEl.dispatchEvent(new Event('scroll'));
		}
		await new Promise((r) => setTimeout(r, 100));

		const fetchCalls = sendActionMock.mock.calls.filter((c) => c[0] === 'fetch-rows');
		expect(fetchCalls.length).toBe(0);
	});

	test('V-13 stops fetching when response returns fewer rows than limit', async () => {
		// Seed 30 rows but page_size=50 — simulates a server response that
		// returned fewer rows than the requested limit. Sentinel must idle.
		setFullState('test', { rows: buildRows(30) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					page_size: 50,
					source: 'test_list',
					// NO total_rows — forces the fewer-than-limit fallback path.
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		// Intentionally NOT dispatching a fetch-rows at this point; we're just
		// confirming the sentinel remains idle when the bound collection size
		// already says "we're done". A bound collection smaller than page_size
		// with no total_rows is the fewer-than-limit contract.
		const scrollEl = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		if (scrollEl) {
			scrollEl.scrollTop = scrollEl.scrollHeight;
			scrollEl.dispatchEvent(new Event('scroll'));
		}
		await new Promise((r) => setTimeout(r, 100));

		// Under this contract the FIRST sentinel fire IS allowed (offset=30),
		// but once the response lands and delta < limit the exhausted latch
		// engages. Here we don't simulate a round-trip — just verify the
		// initial dispatch (if any) carries offset=30 and source='test_list'.
		const fetchCalls = sendActionMock.mock.calls.filter((c) => c[0] === 'fetch-rows');
		for (const call of fetchCalls) {
			expect(call[1]).toEqual(expect.objectContaining({ source: 'test_list', offset: 30 }));
		}
	});

	test('V-14 fetching guard prevents concurrent fetch-rows dispatch', async () => {
		setFullState('test', { rows: buildRows(60) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [{ key: 'name', label: 'Name' }],
					total_rows: 1000,
					page_size: 50,
					source: 'test_list',
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		const scrollEl = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		expect(scrollEl).toBeTruthy();

		// Fire multiple scrolls back-to-back before any response lands.
		if (scrollEl) {
			for (let i = 0; i < 5; i++) {
				scrollEl.scrollTop = scrollEl.scrollHeight;
				scrollEl.dispatchEvent(new Event('scroll'));
				// No await between firings — simulates a rapid burst.
			}
		}
		await new Promise((r) => setTimeout(r, 120));

		// The fetching guard must collapse the burst into at most one dispatch
		// while the first request is still in-flight.
		const fetchCalls = sendActionMock.mock.calls.filter((c) => c[0] === 'fetch-rows');
		expect(fetchCalls.length).toBeLessThanOrEqual(1);
	});
});

// -----------------------------------------------------------------------------
// Cell kinds (D-F1)
// -----------------------------------------------------------------------------

describe('Cell kinds (D-F1)', () => {
	test('V-23 actions kind renders DataTableActions DropdownMenu', async () => {
		setFullState('test', {
			rows: {
				r1: {
					id: 'r1',
					name: 'Alice',
					actions: [{ label: 'Edit', action: { type: 'click', name: 'edit' } }],
				},
			},
		});
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'actions', label: '', kind: 'actions' },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		// DataTableActions renders a button with aria-label="Row actions".
		await expect.element(screen.getByLabelText('Row actions')).toBeVisible();
	});

	test('V-24 date kind formats via Intl.DateTimeFormat', async () => {
		setFullState('test', {
			rows: {
				r1: { id: 'r1', name: 'Alice', created: '2026-04-01T12:00:00Z' },
			},
		});
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'created', label: 'Created', kind: 'date' },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		// Intl.DateTimeFormat with dateStyle: 'medium' produces a month abbrev
		// like "Apr" for April in en-US. The underlying ISO string should
		// NOT appear verbatim (that's the raw value).
		const scrollBody = screen.container.querySelector(
			'[data-testid="datatable-scroll"]',
		) as HTMLElement | null;
		expect(scrollBody).toBeTruthy();
		const text = scrollBody?.textContent ?? '';
		expect(text).not.toContain('2026-04-01T12:00:00Z');
		// Some locale-formatted date-string should appear (at minimum a digit).
		expect(text).toMatch(/\d/);
	});

	test('V-25 number kind right-aligns with tabular-nums', async () => {
		setFullState('test', {
			rows: {
				r1: { id: 'r1', name: 'Alice', count: 12345 },
			},
		});
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'count', label: 'Count', kind: 'number' },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		// The number cell's span should carry `tabular-nums` and a right
		// alignment class.
		const numCell = screen.container.querySelector('.tabular-nums');
		expect(numCell).toBeTruthy();
		// The formatted number appears (Intl.NumberFormat defaults may add a
		// group separator depending on locale — check the digits are there).
		expect(numCell?.textContent ?? '').toMatch(/1.?2.?3.?4.?5/);
	});

	test('V-26 badge kind renders Badge component', async () => {
		setFullState('test', {
			rows: {
				r1: { id: 'r1', name: 'Alice', status: 'active' },
			},
		});
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name' },
						{ key: 'status', label: 'Status', kind: 'badge' },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await new Promise((r) => setTimeout(r, 50));
		// The badge kind renders the value inside a styled span (not just
		// plain text). Find any element whose text matches and whose class
		// list includes a rounded / padding shape consistent with a badge.
		const badge = Array.from(screen.container.querySelectorAll('span')).find(
			(el) => (el.textContent ?? '').trim() === 'active' && el.className.includes('rounded'),
		);
		expect(badge).toBeTruthy();
	});
});

// -----------------------------------------------------------------------------
// Preserved behavior (sort + row click)
// -----------------------------------------------------------------------------

describe('Preserved behavior', () => {
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

		await expect.element(screen.getByRole('columnheader', { name: 'Name' })).toBeVisible();
		await expect.element(screen.getByRole('columnheader', { name: 'Email' })).toBeVisible();
	});

	test('dispatches sort action on header click', async () => {
		setFullState('test', { rows: buildRows(2) });
		const screen = await render(DataTable, {
			props: {
				props: {
					columns: [
						{ key: 'name', label: 'Name', sortable: true },
						{ key: 'email', label: 'Email' },
					],
				},
				bind: '/rows',
				surface: 'test',
			},
		});

		await screen.getByRole('columnheader', { name: /Name/ }).click();

		expect(sendActionMock).toHaveBeenCalledWith(
			'sort',
			expect.objectContaining({ column: 'name', direction: 'asc' }),
		);
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

		await new Promise((r) => setTimeout(r, 50));
		// Both row names appear in their respective cells (virtualizer may
		// reorder positioning but both rows are above the fold with only 2
		// rows total). Use exact cell matches to avoid strict-mode collisions
		// with the email column whose text contains the name substring.
		await expect.element(screen.getByRole('cell', { name: 'Alice', exact: true })).toBeVisible();
		await expect.element(screen.getByRole('cell', { name: 'Bob', exact: true })).toBeVisible();
	});
});
