import { test, expect, type Page } from '@playwright/test';
import { captureWebSocketFrames, type CapturedFrame } from '../helpers/ws-capture';

// -----------------------------------------------------------------------------
// Phase 13 Plan 07 Task 2 — DataTable infinite scroll (TABLE-02).
//
// Loads the contacts list (seeded with 120 contacts by crm-demo seed.rs, with
// page_size=50 on contact_list — see handlers/contact.rs) and scrolls the
// virtualised DataTable to the tail. Asserts that:
//   1. The IntersectionObserver sentinel fires a `fetch-rows` action with
//      source="contact_list" and a non-zero offset (D-H1, D-H3).
//   2. A corresponding patch response arrives applying new rows under
//      the /contacts collection path.
//   3. The fetch-rows action id is echoed in the response patch id
//      (D-H3 correlation convention).
//
// NOTE: These tests ONLY work under playwright.e2e.config.ts which spawns
// the real crm-demo binary. The dev-server config has no backend data.
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
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}

function fetchRowsFrames(frames: CapturedFrame[]): Array<Record<string, unknown>> {
	return frames
		.filter((f) => f.direction === 'sent' && f.data.type === 'action')
		.map((f) => f.data)
		.filter((d) => (d as { name?: string }).name === 'fetch-rows');
}

test.describe('DataTable infinite scroll (TABLE-02)', () => {
	test('scrolling to tail triggers fetch-rows with non-zero offset', async ({ page }) => {
		const frames = captureWebSocketFrames(page);
		await page.goto('/');
		await loginAsAdmin(page);

		// Contacts is the admin default — the DataTable should be visible.
		const scroller = page.locator('[data-testid="datatable-scroll"]');
		await expect(scroller).toBeVisible({ timeout: 10000 });

		// Initial rows must include at least one seeded contact to prove
		// the table is populated before we scroll.
		await expect(page.getByText(/Alice Johnson|Seed Contact 000/).first()).toBeVisible({
			timeout: 10000,
		});

		// Force scroll to the bottom so the sentinel intersects the
		// viewport. The inline style sets overflow-y: auto + height: 400px
		// (see DataTable.svelte ~line 461).
		await scroller.evaluate((el) => {
			(el as HTMLElement).scrollTop = (el as HTMLElement).scrollHeight;
		});

		// Wait for the fetch-rows dispatch.
		await expect
			.poll(() => fetchRowsFrames(frames).length, { timeout: 5000 })
			.toBeGreaterThan(0);

		const fetches = fetchRowsFrames(frames);
		const first = fetches[0] as {
			id: string;
			name: string;
			payload: { source?: string; offset?: number; limit?: number };
		};
		expect(first.name).toBe('fetch-rows');
		expect(first.payload.source).toBe('contact_list');
		expect(first.payload.offset).toBeGreaterThan(0);
		expect(first.payload.limit).toBeDefined();
		expect(first.payload.limit!).toBeGreaterThan(0);
		expect(first.payload.limit!).toBeLessThanOrEqual(100);

		// A patch response on the `content` surface should arrive carrying
		// new rows under the /contacts path.
		await expect
			.poll(
				() =>
					frames.filter(
						(f) =>
							f.direction === 'received' &&
							f.data.type === 'patch' &&
							Array.isArray((f.data as { patch?: unknown[] }).patch) &&
							JSON.stringify((f.data as { patch: unknown[] }).patch).includes('/contacts/'),
					).length,
				{ timeout: 5000 },
			)
			.toBeGreaterThan(0);
	});

	test('fetch-rows action id is echoed into the response patch id (D-H3)', async ({ page }) => {
		const frames = captureWebSocketFrames(page);
		await page.goto('/');
		await loginAsAdmin(page);

		const scroller = page.locator('[data-testid="datatable-scroll"]');
		await expect(scroller).toBeVisible({ timeout: 10000 });
		await expect(page.getByText(/Alice Johnson|Seed Contact 000/).first()).toBeVisible({
			timeout: 10000,
		});

		await scroller.evaluate((el) => {
			(el as HTMLElement).scrollTop = (el as HTMLElement).scrollHeight;
		});

		await expect
			.poll(() => fetchRowsFrames(frames).length, { timeout: 5000 })
			.toBeGreaterThan(0);

		const fetches = fetchRowsFrames(frames);
		const sent = fetches[0] as { id: string; name: string };
		expect(sent.id).toBeDefined();

		// The backend fetch_rows handler echoes ctx.action.id into the
		// PatchMessage.id field (D-H3 correlation).
		await expect
			.poll(
				() =>
					frames.filter(
						(f) =>
							f.direction === 'received' &&
							f.data.type === 'patch' &&
							(f.data as { id?: string }).id === sent.id,
					).length,
				{ timeout: 5000 },
			)
			.toBeGreaterThan(0);
	});
});
