import { test, expect } from '@playwright/test';

test('full page layout snapshot', async ({ page }) => {
	await page.goto('/');
	// Wait for demo mode content to be fully rendered
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 5000 });
	await expect(page).toHaveScreenshot('full-page.png', {
		maxDiffPixels: 200,
		fullPage: true,
	});
});
