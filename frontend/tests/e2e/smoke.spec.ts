import { test, expect } from '@playwright/test';

test('app loads and shows content', async ({ page }) => {
	await page.goto('/');
	// Wait for the app shell to render with at least one surface
	await page.waitForSelector('[data-surface]', { timeout: 10000 });
	await expect(page.locator('[data-surface="main"]')).toBeVisible();
});
