import { test, expect } from '@playwright/test';

test('sidebar visual snapshot', async ({ page }) => {
	await page.goto('/');
	// Wait for demo mode to populate
	await expect(page.getByText('Dashboard')).toBeVisible({ timeout: 5000 });
	const sidebar = page.locator('[data-surface="sidebar"]');
	await expect(sidebar).toHaveScreenshot('sidebar.png', { maxDiffPixels: 100 });
});

test('form components visual snapshot', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByText('Save Contact')).toBeVisible({ timeout: 5000 });
	const form = page.locator('form').first();
	await expect(form).toHaveScreenshot('form.png', { maxDiffPixels: 100 });
});

test('data table visual snapshot', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByText('Alice Johnson')).toBeVisible({ timeout: 5000 });
	// The DataTable is rendered inside a div with overflow
	const tableArea = page.locator('[data-surface="main"] table').first();
	await expect(tableArea).toHaveScreenshot('data-table.png', { maxDiffPixels: 100 });
});
