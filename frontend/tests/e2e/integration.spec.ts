import { test, expect } from '@playwright/test';
import { captureWebSocketFrames } from '../helpers/ws-capture';

// -----------------------------------------------------------------------------
// Post-Phase-12 integration smoke tests against the crm-demo backend.
//
// Updated during Plan 12-08 Task 3: the original tests here asserted on
// strings from the pre-CRM demo app (`Welcome to Marionette`, `Click Me`)
// and hard-coded hello version `1.0.0`, which haven't been valid since
// the Plan 12-07 CRM integration and Plan 12-02 protocol version bump
// landed. The assertions are now rewritten against the current CRM
// landing surface.
// -----------------------------------------------------------------------------

test('WebSocket connects and receives hello version 1.1.0', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Wait for at least one received frame
	await expect
		.poll(() => frames.filter((f) => f.direction === 'received').length, {
			timeout: 10000,
		})
		.toBeGreaterThan(0);

	const helloFrame = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'hello',
	);
	expect(helloFrame).toBeDefined();
	// Phase 12 D-A5: protocol bumped to 1.1.0.
	expect(helloFrame!.data.version).toBe('1.1.0');
});

test('navigate action triggers a main-surface render with the login form', async ({
	page,
}) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Unauthenticated users land on the login form (D-B11 pre-auth path).
	// The form's submit button carries the label "Log In".
	await expect(page.getByRole('button', { name: /log in/i })).toBeVisible({
		timeout: 10000,
	});

	// Verify protocol frames: a navigate action was sent and a main-surface
	// render was received.
	const navigateAction = frames.find(
		(f) => f.direction === 'sent' && f.data.type === 'action' && f.data.name === 'navigate',
	);
	expect(navigateAction).toBeDefined();

	const renderMsg = frames.find(
		(f) =>
			f.direction === 'received' &&
			f.data.type === 'render' &&
			(f.data as { surface?: string }).surface === 'main',
	);
	expect(renderMsg).toBeDefined();
});

test('login button sends action and receives AppShell render', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Fill and submit the login form.
	await page
		.locator('div.grid:has(label:has-text("Email"))')
		.getByRole('textbox')
		.fill('admin@localhost');
	await page
		.locator('div.grid:has(label:has-text("Password"))')
		.getByRole('textbox')
		.fill('admin');
	await page.getByRole('button', { name: /log in/i }).click();

	// Post-auth: Contact Management heading appears inside the content
	// sub-surface rendered by handle_navigate's second Render.
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });

	// Verify the login action went out and at least one post-auth Render
	// targeted the `content` sub-surface (Plan 07 retargeting).
	const loginAction = frames.find(
		(f) => f.direction === 'sent' && f.data.type === 'action' && f.data.name === 'login',
	);
	expect(loginAction).toBeDefined();

	const contentRender = frames.find(
		(f) =>
			f.direction === 'received' &&
			f.data.type === 'render' &&
			(f.data as { surface?: string }).surface === 'content',
	);
	expect(contentRender).toBeDefined();
});

test('health endpoint responds', async ({ page }) => {
	const response = await page.request.get('/api/health');
	expect(response.status()).toBe(200);
	expect(await response.text()).toBe('ok');
});

test('SPA fallback serves app for deep routes', async ({ page }) => {
	// Navigate to a deep route that is NOT a real static file
	const response = await page.goto('/some/deep/route');

	// Should not be 404 -- SPA fallback returns index.html
	expect(response).not.toBeNull();
	expect(response!.status()).not.toBe(404);

	// The SPA shell should load (Surface elements from layout)
	await expect(page.locator('[data-surface="main"]')).toBeAttached({ timeout: 10000 });
});
