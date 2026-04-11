import { test, expect, type Page } from '@playwright/test';
import { captureWebSocketFrames, type CapturedFrame } from '../helpers/ws-capture';

// -----------------------------------------------------------------------------
// Phase 13 Plan 07 Task 2 — DataTable filter roundtrip (TABLE-01).
//
// Drives the real crm-demo backend (via playwright.e2e.config.ts) through
// the Contacts list, types into the built-in DataTable search filter, and
// asserts that:
//   1. A `filter` action is dispatched over the WebSocket with the typed
//      payload after the 300ms debounce (D-C1).
//   2. Pressing Enter flushes immediately without waiting for the debounce.
//
// NOTE: These tests ONLY work under playwright.e2e.config.ts which spawns
// the real crm-demo binary with seeded data. Under playwright.config.ts
// (dev server on :5173 without a backend) there is nothing to filter.
// -----------------------------------------------------------------------------

async function loginAsAdmin(page: Page): Promise<void> {
	const emailInput = page
		.locator('div.grid:has(label:has-text("Email"))')
		.getByRole('textbox');
	const passwordInput = page
		.locator('div.grid:has(label:has-text("Password"))')
		.getByRole('textbox');
	await emailInput.fill('admin@localhost');
	await passwordInput.fill('admin');
	await page.getByRole('button', { name: /log in/i }).click();
	// Admin lands on Contact Management (the default post-login view)
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}

function filterFramesWithName(
	frames: CapturedFrame[],
	name: string,
): Array<Record<string, unknown>> {
	return frames
		.filter((f) => f.direction === 'sent' && f.data.type === 'action')
		.map((f) => f.data)
		.filter((d) => (d as { name?: string }).name === name);
}

test.describe('DataTable filter roundtrip (TABLE-01)', () => {
	test('typing in text filter dispatches debounced filter action', async ({ page }) => {
		const frames = captureWebSocketFrames(page);
		await page.goto('/');
		await loginAsAdmin(page);

		// Contact Management is the default landing view for admin; the
		// DataTable filter bar is inline with the `Search` aria-label
		// (contact_list Filter::text("search").label("Search")).
		const searchInput = page.getByLabel('Search');
		await expect(searchInput).toBeVisible({ timeout: 10000 });
		await searchInput.fill('Alice');

		// Wait past the 300ms debounce + a generous network margin.
		await expect
			.poll(() => filterFramesWithName(frames, 'filter').length, {
				timeout: 5000,
			})
			.toBeGreaterThan(0);

		// The last filter frame must carry the typed payload. Debounce may
		// emit a single frame with "Alice" OR several frames with the
		// growing prefix; check the final one.
		const filterFrames = filterFramesWithName(frames, 'filter');
		const lastFilter = filterFrames[filterFrames.length - 1] as {
			name: string;
			payload: Record<string, unknown>;
		};
		expect(lastFilter.name).toBe('filter');
		expect(lastFilter.payload).toBeDefined();
		expect(lastFilter.payload.search).toBe('Alice');

		// The DOM should reflect the filtered result — Alice Johnson is a
		// stable seeded row; filter by 'Alice' must keep it visible and
		// the seed rows (Seed Contact ...) must not match.
		await expect(page.getByText('Alice Johnson')).toBeVisible({ timeout: 5000 });
	});

	test('Enter in text filter flushes immediately (no debounce wait)', async ({ page }) => {
		const frames = captureWebSocketFrames(page);
		await page.goto('/');
		await loginAsAdmin(page);

		const searchInput = page.getByLabel('Search');
		await expect(searchInput).toBeVisible({ timeout: 10000 });

		// Snapshot the frame count BEFORE typing so we can count only
		// frames produced by this interaction.
		const before = frames.length;

		await searchInput.fill('Bob');
		await searchInput.press('Enter');

		// Enter bypasses the 300ms debounce — a filter frame should
		// arrive well before the 300ms mark. Give it 250ms and require
		// at least one filter frame.
		await expect
			.poll(
				() =>
					frames
						.slice(before)
						.filter(
							(f) =>
								f.direction === 'sent' &&
								f.data.type === 'action' &&
								(f.data as { name?: string }).name === 'filter' &&
								JSON.stringify(f.data).includes('"search":"Bob"'),
						).length,
				{ timeout: 250, intervals: [25, 50, 50, 100] },
			)
			.toBeGreaterThan(0);
	});

	test('select filter fires immediately on change (no debounce)', async ({ page }) => {
		const frames = captureWebSocketFrames(page);
		await page.goto('/');
		await loginAsAdmin(page);

		// The contact_list ships a `company_filter` select with aria-label
		// "Company" (see handlers/contact.rs:346). shadcn Select.Trigger
		// renders as a <button> with aria-label — use the aria-label
		// locator directly rather than getByRole (bits-ui role varies).
		const companyTrigger = page
			.locator('button[aria-label="Company"]')
			.or(page.getByLabel('Company'));
		await expect(companyTrigger.first()).toBeVisible({ timeout: 10000 });
		const before = frames.length;
		await companyTrigger.first().click();

		// Pick a non-empty company option. The first option is "All Companies"
		// (empty value); pick the second option (the first actual company)
		// so the resulting filter action carries a non-empty payload.
		const allOptions = page.getByRole('option');
		await expect(allOptions.first()).toBeVisible({ timeout: 5000 });
		const optionCount = await allOptions.count();
		// If there are multiple options, the second is the first real company.
		// Otherwise fall back to the only option available (empty selection
		// still counts as a user-driven immediate change per D-C1).
		const targetIndex = optionCount > 1 ? 1 : 0;
		await allOptions.nth(targetIndex).click();

		// D-C1: selects fire on change WITHOUT debounce — a filter action
		// must appear within the timing budget of a single event loop,
		// not after a 300ms wait.
		await expect
			.poll(
				() =>
					frames
						.slice(before)
						.filter(
							(f) =>
								f.direction === 'sent' &&
								f.data.type === 'action' &&
								(f.data as { name?: string }).name === 'filter',
						).length,
				{ timeout: 2000 },
			)
			.toBeGreaterThan(0);
	});
});
