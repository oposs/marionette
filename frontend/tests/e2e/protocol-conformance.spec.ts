import { test, expect, type Page } from '@playwright/test';
import { captureWebSocketFrames } from '../helpers/ws-capture';
import { createValidator } from '../helpers/schema-validator';

// -----------------------------------------------------------------------------
// Phase 12 Plan 08 Task 3 — protocol conformance E2E.
//
// Validates that live WebSocket frames from the crm-demo backend match the
// updated OpenSDUI schemas (node-patching additions + PatchMessage.surface
// + HelloMessage version bump to 1.1.0). The new node-op coverage is driven
// by the country-change flow added in Plan 08 Task 1, which emits a
// PatchMessage containing Set + RemoveChild + DeleteNode + SetNode +
// InsertChild ops on the `content` surface.
//
// Phase 13 Plan 07 Task 3 — extends the spec to validate the two new Phase 13
// WebSocket traffic patterns against the same ActionMessage / PatchMessage
// schemas in spec/schemas/:
//   1. `filter` action payload (D-C3 flat values map)
//   2. `fetch-rows` action + its response patch (D-H1 / D-H3)
// -----------------------------------------------------------------------------

async function login(page: Page): Promise<void> {
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

test('hello message conforms to schema and reports version 1.1.0', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	await expect
		.poll(
			() =>
				frames.filter(
					(f) => f.direction === 'received' && f.data.type === 'hello',
				).length,
			{ timeout: 10000 },
		)
		.toBeGreaterThan(0);

	// The server hello — the client also sends its own hello frame in the
	// opposite direction, so filter on direction to avoid picking up the
	// client's copy by mistake.
	const helloFrame = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'hello',
	)!;
	const validator = createValidator();
	const valid = validator.validateHello(helloFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
	// Phase 12 protocol version bump gate (Plan 02 + Plan 03 + D-A5).
	expect(helloFrame.data.version).toBe('1.1.0');
});

test('render message conforms to schema', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	await expect
		.poll(
			() =>
				frames.filter((f) => f.direction === 'received' && f.data.type === 'render').length,
			{ timeout: 10000 },
		)
		.toBeGreaterThan(0);

	const renderFrame = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'render',
	)!;
	const validator = createValidator();
	const valid = validator.validateRender(renderFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
});

test('action message conforms to schema', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	await expect
		.poll(
			() =>
				frames.filter((f) => f.direction === 'sent' && f.data.type === 'action').length,
			{ timeout: 10000 },
		)
		.toBeGreaterThan(0);

	const actionFrame = frames.find(
		(f) => f.direction === 'sent' && f.data.type === 'action',
	)!;
	const validator = createValidator();
	const valid = validator.validateAction(actionFrame.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);
});

test('patch message with node tree ops conforms to schema', async ({ page }) => {
	// Drive the Plan 08 country-change flow which emits a PatchMessage
	// on the `content` surface with a mix of Set + RemoveChild +
	// DeleteNode + SetNode + InsertChild ops. Also captures a second
	// PatchMessage on the `toasts` sub-surface (D-B15).
	const frames = captureWebSocketFrames(page);
	await page.goto('/');
	await login(page);

	await page.getByRole('button', { name: 'New Contact' }).click();
	await expect(page.getByRole('heading', { name: 'New Contact' })).toBeVisible({
		timeout: 5000,
	});

	// Trigger the country-change flow via the E2E test hook — avoids
	// timing hazards with shadcn Select focus.
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((name: string, payload?: Record<string, unknown>, source?: string) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction test hook not exposed');
		hook(
			'contact_country_change',
			{ contactForm: { country: 'CH' } },
			'contact-form-country',
		);
	});

	// Wait for at least one patch frame from the country-change response.
	await expect
		.poll(
			() =>
				frames.filter(
					(f) =>
						f.direction === 'received' &&
						f.data.type === 'patch' &&
						Array.isArray((f.data as { patch?: unknown[] }).patch) &&
						((f.data as { patch: unknown[] }).patch as Array<{ op: string }>).some(
							(op) => op.op !== 'set',
						),
				).length,
			{ timeout: 10000 },
		)
		.toBeGreaterThan(0);

	// Find the content-surface patch (the demo's primary payload).
	const contentPatch = frames.find(
		(f) =>
			f.direction === 'received' &&
			f.data.type === 'patch' &&
			(f.data as { surface?: string }).surface === 'content' &&
			((f.data as { patch: Array<{ op: string }> }).patch as Array<{ op: string }>).some(
				(op) => ['set-node', 'insert-child', 'delete-node', 'remove-child'].includes(op.op),
			),
	);
	expect(contentPatch, 'expected a content-surface PatchMessage with node ops').toBeDefined();
	expect((contentPatch!.data as { surface: string }).surface).toBe('content');

	// Validate against the updated spec/schemas/message.yaml PatchMessage.
	const validator = createValidator();
	const valid = validator.validatePatch(contentPatch!.data);
	expect(valid, `Schema errors: ${validator.getErrors()}`).toBe(true);

	// Sanity-check the op distribution: the demo patch must contain
	// all five new PatchOperation variants at least once across the
	// two PatchMessages it emits (content + toasts).
	const allPatchFrames = frames.filter(
		(f) => f.direction === 'received' && f.data.type === 'patch',
	);
	const allOps = new Set<string>();
	for (const frame of allPatchFrames) {
		const patch = (frame.data as { patch?: Array<{ op: string }> }).patch ?? [];
		for (const op of patch) allOps.add(op.op);
	}
	expect(allOps.has('set')).toBe(true);
	expect(allOps.has('set-node')).toBe(true);
	expect(allOps.has('insert-child')).toBe(true);
	expect(allOps.has('delete-node')).toBe(true);
	expect(allOps.has('remove-child')).toBe(true);

	// Validate each toasts-surface PatchMessage as well (D-B15 gate).
	const toastsPatch = allPatchFrames.find(
		(f) => (f.data as { surface?: string }).surface === 'toasts',
	);
	expect(toastsPatch, 'expected a toasts-surface PatchMessage').toBeDefined();
	const validToasts = validator.validatePatch(toastsPatch!.data);
	expect(validToasts, `toasts schema errors: ${validator.getErrors()}`).toBe(true);
});

test('filter action payload conforms to ActionMessage schema (Phase 13)', async ({ page }) => {
	// Drive the Phase 13 DataTable filter bar: type into the Search
	// input on the contacts list; DataTable debounces 300ms then dispatches
	// `sendAction('filter', { search: 'Acme' })`. Validate the sent frame
	// against the ActionMessage schema in spec/schemas/message.yaml.
	const frames = captureWebSocketFrames(page);
	await page.goto('/');
	await login(page);

	const searchInput = page.getByLabel('Search');
	await expect(searchInput).toBeVisible({ timeout: 10000 });
	await searchInput.fill('Acme');

	await expect
		.poll(
			() =>
				frames.filter(
					(f) =>
						f.direction === 'sent' &&
						f.data.type === 'action' &&
						(f.data as { name?: string }).name === 'filter',
				).length,
			{ timeout: 5000 },
		)
		.toBeGreaterThan(0);

	const filterFrame = frames.find(
		(f) =>
			f.direction === 'sent' &&
			f.data.type === 'action' &&
			(f.data as { name?: string }).name === 'filter',
	)!;
	const validator = createValidator();
	const valid = validator.validateAction(filterFrame.data);
	expect(valid, `filter ActionMessage schema errors: ${validator.getErrors()}`).toBe(true);

	// Sanity-check the D-C3 flat values map shape.
	const msg = filterFrame.data as {
		name: string;
		payload: Record<string, unknown>;
	};
	expect(msg.name).toBe('filter');
	expect(msg.payload).toBeDefined();
	expect(msg.payload.search).toBe('Acme');
});

test('fetch-rows action + response patch conform to schemas (Phase 13)', async ({ page }) => {
	// Drive the Phase 13 infinite-scroll sentinel: navigate to contacts,
	// scroll the virtualised DataTable to its tail, and validate both
	// the sent `fetch-rows` ActionMessage and the received PatchMessage
	// response (which echoes the action id per D-H3) against their
	// respective schemas.
	const frames = captureWebSocketFrames(page);
	await page.goto('/');
	await login(page);

	const scroller = page.locator('[data-testid="datatable-scroll"]');
	await expect(scroller).toBeVisible({ timeout: 10000 });
	// Wait for initial rows to settle so the sentinel is mounted.
	await expect(page.getByText(/Alice Johnson|Seed Contact 000/).first()).toBeVisible({
		timeout: 10000,
	});

	await scroller.evaluate((el) => {
		(el as HTMLElement).scrollTop = (el as HTMLElement).scrollHeight;
	});

	// Wait for the fetch-rows dispatch.
	await expect
		.poll(
			() =>
				frames.filter(
					(f) =>
						f.direction === 'sent' &&
						f.data.type === 'action' &&
						(f.data as { name?: string }).name === 'fetch-rows',
				).length,
			{ timeout: 5000 },
		)
		.toBeGreaterThan(0);

	const fetchFrame = frames.find(
		(f) =>
			f.direction === 'sent' &&
			f.data.type === 'action' &&
			(f.data as { name?: string }).name === 'fetch-rows',
	)!;
	const validator = createValidator();
	const fetchValid = validator.validateAction(fetchFrame.data);
	expect(
		fetchValid,
		`fetch-rows ActionMessage schema errors: ${validator.getErrors()}`,
	).toBe(true);

	// D-H1: payload must include source, offset, limit.
	const fetchMsg = fetchFrame.data as {
		id: string;
		name: string;
		payload: { source?: string; offset?: number; limit?: number };
	};
	expect(fetchMsg.name).toBe('fetch-rows');
	expect(fetchMsg.payload.source).toBe('contact_list');
	expect(fetchMsg.payload.offset).toBeGreaterThan(0);
	expect(fetchMsg.payload.limit).toBeGreaterThan(0);

	// D-H3: the response PatchMessage echoes the action id.
	await expect
		.poll(
			() =>
				frames.filter(
					(f) =>
						f.direction === 'received' &&
						f.data.type === 'patch' &&
						(f.data as { id?: string }).id === fetchMsg.id,
				).length,
			{ timeout: 5000 },
		)
		.toBeGreaterThan(0);

	const patchFrame = frames.find(
		(f) =>
			f.direction === 'received' &&
			f.data.type === 'patch' &&
			(f.data as { id?: string }).id === fetchMsg.id,
	)!;
	const patchValid = validator.validatePatch(patchFrame.data);
	expect(
		patchValid,
		`fetch-rows PatchMessage schema errors: ${validator.getErrors()}`,
	).toBe(true);
});
