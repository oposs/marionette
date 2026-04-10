import { test, expect, type Page } from '@playwright/test';

// -----------------------------------------------------------------------------
// Phase 12 Plan 08 Task 3 — AppShell nav end-to-end.
//
// Drives a real WebSocket session and verifies:
//   1. Login -> AppShell renders with sidebar/header/main/footer landmarks,
//      the footer shows the Plan 12-07 "Marionette v1.1 · Protocol 1.1.0"
//      + connection-status wiring, and clicking a sidebar nav item swaps
//      the `content` sub-surface without tearing down the shell.
//   2. The Sidebar.Trigger (shadcn mobile hamburger) is present and
//      visible at a narrow viewport.
// -----------------------------------------------------------------------------

async function login(page: Page): Promise<void> {
	await page.goto('/');
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

test.describe('Phase 12: AppShell nav end-to-end', () => {
	test('login -> AppShell renders with landmarks; nav updates content sub-surface', async ({
		page,
	}) => {
		await login(page);

		// AppShell landmarks. The `<header>` and `<footer>` live inside
		// the shadcn `Sidebar.Inset` wrapper, so ARIA downgrades them
		// from the `banner` / `contentinfo` landmarks to generic
		// section elements. Address them by tag directly.
		await expect(page.locator('header').first()).toBeVisible({ timeout: 5000 });
		await expect(page.locator('footer').first()).toBeVisible();

		// Footer content — D-B6 version literal + connection status +
		// legal. All three are rendered as Heading nodes by Plan 07's
		// handle_navigate.
		await expect(
			page.getByText('Marionette v1.1 · Protocol 1.1.0'),
		).toBeVisible();
		await expect(page.getByText('© 2026 Marionette')).toBeVisible();

		// Header title — D-B5 app title literal.
		await expect(page.getByRole('heading', { name: 'Marionette CRM' }))
			.toBeVisible();

		// Sidebar nav items. Plan 07 wires Home / Contacts / Companies /
		// Users / Audit Log via NavItem components — the admin role sees
		// all 5. Each NavItem renders as a shadcn Button with the label
		// text. Use role-based matchers with an exact name so the
		// sidebar's "Contacts" button does not collide with the
		// "Contact Management" heading.
		const contactsNav = page.getByRole('button', { name: 'Contacts', exact: true });
		const companiesNav = page.getByRole('button', { name: 'Companies', exact: true });
		await expect(contactsNav).toBeVisible();
		await expect(companiesNav).toBeVisible();

		// Navigate to Companies — the content sub-surface swaps to the
		// Company Management screen while the shell landmarks persist.
		await companiesNav.click();
		await expect(page.getByText('Company Management')).toBeVisible({
			timeout: 5000,
		});

		// Shell still visible — not remounted — so the footer literal
		// is still present.
		await expect(
			page.getByText('Marionette v1.1 · Protocol 1.1.0'),
		).toBeVisible();

		// Navigate back to Contacts.
		await contactsNav.click();
		await expect(page.getByText('Contact Management')).toBeVisible({
			timeout: 5000,
		});
	});

	test('Sidebar.Trigger (mobile hamburger) is present in the header', async ({
		page,
	}) => {
		await login(page);

		// Shrink to mobile viewport — shadcn Sidebar switches to the
		// offcanvas sheet mode and the trigger becomes the primary
		// way to open the sidebar.
		await page.setViewportSize({ width: 375, height: 700 });

		// The shadcn Sidebar.Trigger renders as a button with
		// `data-sidebar="trigger"` — Plan 06 confirmed this attribute
		// is part of the shadcn primitive's DOM surface.
		const trigger = page.locator('button[data-sidebar="trigger"]');
		await expect(trigger.first()).toBeVisible({ timeout: 5000 });
	});
});
