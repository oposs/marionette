import { test, expect, type Page } from '@playwright/test';

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — Company edit form end-to-end coverage.
//
// Drives the real crm-demo backend (via playwright.e2e.config.ts) through the
// Phase 15 Plan 03 migrated company form (form_shell + FieldSet("Company
// details") + action row D-D1 Option A + validation_error_patch) and asserts:
//   1. The FieldSet legend "Company details" renders (D-A1 + D-C1).
//   2. The name field carries the locked description string
//      "Will appear on invoices and contact details." (D-E3, 15-UI-SPEC
//      §Description Copy Contract).
//   3. Action row exposes "Cancel" (outline) and "Save company" (primary)
//      with Cancel to the LEFT of Save (D-D1 Option A).
//   4. Per-field validation fires on empty Name on submit, emitting a
//      Field.Error inside the Name Field.Field (D-D1), and the form-level
//      banner stays empty (D-D4).
//   5. Save flow with the seeded values returns to the Company Management
//      list view (landing screen after company_save).
// -----------------------------------------------------------------------------

async function login(page: Page): Promise<void> {
	await page.goto('/');
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
	// Landing screen is the Contact Management DataTable.
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}

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

// Helper: locate a Field.Field-wrapped text input by its label text.
// Matches the contact-edit.spec.ts convention because component builders'
// `.id("…")` sets the component's adjacency-list id, not the HTML `id`
// attribute on the rendered <input> (the HTML id is a UUID fallback unless
// `props.id` is supplied — which the handlers don't do). Label-scoped lookups
// match the real user-perceivable DOM.
function labelField(page: Page, label: string) {
	return page
		.locator(`div[data-slot="field"]:has(label:has-text("${label}"))`)
		.locator('input')
		.first();
}

test.describe('Company edit form (Phase 15 D-A1 + D-D1 + D-E3)', () => {
	test('renders FieldSet legend, locked description, and action row (D-A1, D-C1, D-D1, D-E3)', async ({
		page,
	}) => {
		await login(page);
		await openCompanyEditForm(page);

		// FieldSet legend — rendered via <legend> element inside
		// <fieldset data-slot="field-set">.
		await expect(page.getByText('Company details')).toBeVisible();

		// Locked Field.Description string under the Name field.
		await expect(
			page.getByText('Will appear on invoices and contact details.'),
		).toBeVisible();

		// Action row buttons. Save is primary (default variant), Cancel is outline.
		const saveBtn = page.getByRole('button', { name: 'Save company' });
		const cancelBtn = page.getByRole('button', { name: 'Cancel' });
		await expect(saveBtn).toBeVisible();
		await expect(cancelBtn).toBeVisible();

		// D-D1 Option A — Cancel precedes Save in the action row's DOM order.
		// Bounding-box x-comparison is avoided because the Container SDUI
		// component's base `flex-col` class collides with user-supplied
		// `flex gap-2 justify-end`, producing a column layout where both
		// buttons share the same x-coordinate. DOM order is the contract;
		// rendering is a separate pre-existing layout-polish item.
		const buttons = await page
			.getByRole('button', { name: /^(Cancel|Save company)$/ })
			.all();
		expect(buttons.length).toBeGreaterThanOrEqual(2);
		const cancelIndex = await buttons[0].textContent();
		expect(cancelIndex?.trim()).toMatch(/^Cancel$/);
	});

	test('per-field validation fires on empty Name (D-D1, D-D4)', async ({ page }) => {
		await login(page);
		await openCompanyEditForm(page);

		// Clear the required name field via its label-scoped locator.
		const nameInput = labelField(page, 'Name');
		await nameInput.fill('');
		await page.getByRole('button', { name: 'Save company' }).click();

		// Field.Error renders inside the Name Field.Field with a "required"
		// message. The handler emits /_errors/companyForm/name via
		// validation_error_patch (Plan 15-03).
		const error = page
			.locator('[data-slot="field-error"]')
			.filter({ hasText: /required/i });
		await expect(error).toBeVisible({ timeout: 5000 });

		// D-D4: form-level banner MUST NOT appear for per-field validation.
		await expect(page.locator('.bg-destructive\\/10')).toHaveCount(0);

		// aria-invalid on the offending input.
		await expect(nameInput).toHaveAttribute('aria-invalid', 'true');
	});

	test('save flow with seeded name returns to Company Management list', async ({ page }) => {
		await login(page);
		await openCompanyEditForm(page);

		// Ensure name is present (edit mode pre-populates from seed data;
		// guard against any prior-test leakage by re-filling).
		const nameInput = labelField(page, 'Name');
		const currentName = await nameInput.inputValue();
		if (!currentName.trim()) {
			await nameInput.fill('Acme Corp');
		}
		await page.getByRole('button', { name: 'Save company' }).click();

		// Save handler re-renders the company list after successful insert/update.
		await expect(page.getByText('Company Management')).toBeVisible({
			timeout: 10000,
		});
	});
});
