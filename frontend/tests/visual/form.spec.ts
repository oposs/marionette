import { test, expect, type Page } from '@playwright/test';

// -----------------------------------------------------------------------------
// Phase 14 Plan 08 — Contact edit form visual snapshot baseline.
//
// Captures two baselines:
//   - Desktop viewport (default Playwright Chromium, 1280x720): FieldSet grid
//     flows in 2 columns with Notes spanning full_width (col-span-full).
//   - Mobile 375px viewport: FieldSet children stack vertically in 1 column
//     per the D-C3 responsive grid contract.
//
// These snapshots replace the pre-Phase-14 `form.png` captured in
// components.spec.ts; after Plan 08 that file still exists but was taken
// against the old flat-form composition. Rebaselining is handled by
// `npx playwright test tests/visual/ --update-snapshots` after a dev
// server is running.
//
// NOTE: visual/ tests run under `playwright.config.ts` which spawns the
// Vite dev server on :5173. In dev-mode the frontend shows a demo login
// screen with seeded contact data, so the form is reachable by clicking
// the DataTable's Edit action or by directly rendering a contact form.
// -----------------------------------------------------------------------------

async function loginDemo(page: Page): Promise<void> {
	await page.goto('/');
	// The dev server's demo mode shows the login form by default.
	const emailInput = page
		.locator('div[data-slot="field"]:has(label:has-text("Email"))')
		.locator('input')
		.first();
	const passwordInput = page
		.locator('div[data-slot="field"]:has(label:has-text("Password"))')
		.locator('input')
		.first();
	await emailInput.fill('admin@localhost');
	await passwordInput.fill('admin');
	await page.getByRole('button', { name: /log in/i }).click();
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}

async function openEditForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((
					name: string,
					payload?: Record<string, unknown>,
					source?: string,
			  ) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('contact_edit', { contact_id: 1 }, 'contact-edit-1');
	});
	await expect(page.getByRole('heading', { name: 'Edit Contact' })).toBeVisible({
		timeout: 5000,
	});
}

test('contact edit form — desktop baseline', async ({ page }) => {
	await loginDemo(page);
	await openEditForm(page);
	// Wait for the Field anatomy to settle before snapshotting.
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('contact-edit-form.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

test('contact edit form — mobile 375px baseline', async ({ page }) => {
	await page.setViewportSize({ width: 375, height: 800 });
	await loginDemo(page);
	await openEditForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('contact-edit-form-mobile.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — Company / User / Interaction edit form baselines.
//
// Six new snapshot cases (3 screens × desktop 1280×720 + mobile 375×800)
// per 15-UI-SPEC §Visual Rebaseline Plan. First-run baselines created via
// `npx playwright test tests/visual/form.spec.ts --update-snapshots`; lock
// on a clean second run.
//
// The contact-edit baselines above must still match after the Plan 15-04
// form_shell refactor (D-B2) — any diff there is a regression.
// -----------------------------------------------------------------------------

async function openCompanyEditForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((
					name: string,
					payload?: Record<string, unknown>,
					source?: string,
			  ) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('company_edit', { company_id: 1 }, 'company-edit-1');
	});
	await expect(page.getByRole('heading', { name: 'Edit Company' })).toBeVisible({
		timeout: 5000,
	});
}

async function openUserEditForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((
					name: string,
					payload?: Record<string, unknown>,
					source?: string,
			  ) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('user_edit', { user_id: 1 }, 'user-edit-1');
	});
	await expect(page.getByRole('heading', { name: 'Edit User' })).toBeVisible({
		timeout: 5000,
	});
}

async function openInteractionForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((
					name: string,
					payload?: Record<string, unknown>,
					source?: string,
			  ) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('interaction_form', { contact_id: 1 }, 'interaction-form-1');
	});
	await expect(page.getByRole('heading', { name: 'Log Interaction' })).toBeVisible({
		timeout: 5000,
	});
}

test('company edit form — desktop baseline', async ({ page }) => {
	await loginDemo(page);
	await openCompanyEditForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('company-edit-form.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

test('company edit form — mobile 375px baseline', async ({ page }) => {
	await page.setViewportSize({ width: 375, height: 800 });
	await loginDemo(page);
	await openCompanyEditForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('company-edit-form-mobile.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

test('user edit form — desktop baseline', async ({ page }) => {
	await loginDemo(page);
	await openUserEditForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('user-edit-form.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

test('user edit form — mobile 375px baseline', async ({ page }) => {
	await page.setViewportSize({ width: 375, height: 800 });
	await loginDemo(page);
	await openUserEditForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('user-edit-form-mobile.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

test('interaction edit form — desktop baseline', async ({ page }) => {
	await loginDemo(page);
	await openInteractionForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('interaction-edit-form.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});

test('interaction edit form — mobile 375px baseline', async ({ page }) => {
	await page.setViewportSize({ width: 375, height: 800 });
	await loginDemo(page);
	await openInteractionForm(page);
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
	await expect(page).toHaveScreenshot('interaction-edit-form-mobile.png', {
		fullPage: true,
		maxDiffPixels: 200,
	});
});
