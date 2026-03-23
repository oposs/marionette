import { test, expect } from '@playwright/test';
import { captureWebSocketFrames } from '../helpers/ws-capture';

test('WebSocket connects and receives hello', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Wait for at least one received frame
	await expect.poll(() => frames.filter((f) => f.direction === 'received').length, {
		timeout: 10000,
	}).toBeGreaterThan(0);

	const helloFrame = frames.find((f) => f.direction === 'received' && f.data.type === 'hello');
	expect(helloFrame).toBeDefined();
	expect(helloFrame!.data.version).toBe('1.0.0');
});

test('navigate action triggers render with components', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Wait for the heading to appear (proves render was received and rendered)
	await expect(page.getByText('Welcome to Marionette')).toBeVisible({ timeout: 10000 });

	// Verify protocol frames
	const navigateAction = frames.find(
		(f) => f.direction === 'sent' && f.data.type === 'action' && f.data.name === 'navigate',
	);
	expect(navigateAction).toBeDefined();

	const renderMsg = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'render' && f.data.surface === 'main',
	);
	expect(renderMsg).toBeDefined();
});

test('button click sends action and receives patch', async ({ page }) => {
	const frames = captureWebSocketFrames(page);
	await page.goto('/');

	// Wait for the button to appear
	await expect(page.getByText('Click Me')).toBeVisible({ timeout: 10000 });

	// Click the button
	await page.getByText('Click Me').click();

	// Wait for the patch to be applied -- "Button was clicked!" should appear
	await expect(page.getByText('Button was clicked!')).toBeVisible({ timeout: 5000 });

	// Verify protocol frames
	const clickAction = frames.find(
		(f) => f.direction === 'sent' && f.data.type === 'action' && f.data.name === 'demo_click',
	);
	expect(clickAction).toBeDefined();

	const patchMsg = frames.find(
		(f) => f.direction === 'received' && f.data.type === 'patch',
	);
	expect(patchMsg).toBeDefined();
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
