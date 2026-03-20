import { test, expect } from '@playwright/test';

test('app loads and shows content', async ({ page }) => {
	await page.goto('/');
	// Wait for either WebSocket connection or demo mode (2s timeout + rendering)
	await page.waitForSelector('[data-surface]', { timeout: 5000 });
	await expect(page.locator('[data-surface="main"]')).toBeVisible();
});

test('sidebar renders navigation', async ({ page }) => {
	await page.goto('/');
	await page.waitForSelector('[data-surface="sidebar"]', { timeout: 5000 });
	// In demo mode, sidebar should show nav items after 2s demo timer
	const sidebar = page.locator('[data-surface="sidebar"]');
	await expect(sidebar).toBeVisible();
});

test('main surface renders demo form and table', async ({ page }) => {
	await page.goto('/');
	// Wait for demo mode to populate the main surface
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 5000 });
	// Form elements should be present
	await expect(page.locator('form').getByText('Name')).toBeVisible();
	await expect(page.locator('form').getByText('Email')).toBeVisible();
	await expect(page.getByText('Save Contact')).toBeVisible();
});
