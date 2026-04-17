import { test, expect, type Page } from '@playwright/test';

// -----------------------------------------------------------------------------
// Phase 14 Plan 08 — Contact edit form end-to-end coverage.
//
// Drives the real crm-demo backend (via playwright.e2e.config.ts) through the
// Phase 14 migrated contact form and asserts:
//   1. The three FieldSet legends render (Contact information, Organisation,
//      Notes and preferences) — D-C1 structural groupings are wired.
//   2. The action row exposes `Save contact` (primary) and `Cancel` buttons
//      in the correct left-to-right order — D-D1 Option A.
//   3. The login password field serialises as <input type="password"> — a
//      Phase 13 D-E1 regression guard surviving Phase 14's Field.Field wrap.
//   4. The migrated contact form's Email field renders as <input type="email">
//      (exercises the new .input_type("email") helper end-to-end).
//   5. The Phase 12 country-select node-patch flow still swaps siblings in
//      place without tearing the focused input (D-A6 regression) — the
//      target parent changed from `contact-form` to `organisation-set` in
//      Plan 08 Task 3.
// -----------------------------------------------------------------------------

async function login(page: Page): Promise<void> {
	await page.goto('/');
	// Login form: the labels use `for={fieldId}` after Phase 14's Field.Field
	// rewrite, so `getByLabel` is reliable. The password input type must
	// stay `password` (D-E1) — asserted in its own test below.
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

async function openEditContactForm(page: Page): Promise<void> {
	// Trigger `contact_edit` for the first seeded contact via the E2E hook
	// to avoid depending on DataTable row rendering timing. The seed.rs
	// generator creates at least one contact (Alice Johnson) with id=1.
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
	// The migrated form renders heading text 'Edit Contact'.
	await expect(page.getByRole('heading', { name: 'Edit Contact' })).toBeVisible({
		timeout: 5000,
	});
}

async function openNewContactForm(page: Page): Promise<void> {
	await page.getByRole('button', { name: 'New Contact' }).click();
	await expect(page.getByRole('heading', { name: 'New Contact' })).toBeVisible({
		timeout: 5000,
	});
}

test.describe('Contact edit form (Phase 14 Plan 08)', () => {
	test('renders FieldSet legends and action-row buttons (D-C1, D-D1)', async ({ page }) => {
		await login(page);
		await openEditContactForm(page);

		// Three FieldSet legends — the shadcn Field.Legend primitive
		// renders as a <legend> element inside <fieldset data-slot="field-set">.
		await expect(page.getByText('Contact information')).toBeVisible();
		await expect(page.getByText('Organisation')).toBeVisible();
		await expect(page.getByText('Notes and preferences')).toBeVisible();

		// Action row: Cancel + Save contact, Save is the rightmost primary.
		const saveBtn = page.getByRole('button', { name: 'Save contact' });
		const cancelBtn = page.getByRole('button', { name: 'Cancel' });
		await expect(saveBtn).toBeVisible();
		await expect(cancelBtn).toBeVisible();

		// Bounding-box left edges: Cancel must sit to the LEFT of Save (D-D1).
		const cancelBox = await cancelBtn.boundingBox();
		const saveBox = await saveBtn.boundingBox();
		expect(cancelBox, 'Cancel button must have a layout box').not.toBeNull();
		expect(saveBox, 'Save button must have a layout box').not.toBeNull();
		expect(cancelBox!.x).toBeLessThan(saveBox!.x);
	});

	test('login password field renders as type="password" (D-E1 regression)', async ({ page }) => {
		// Do NOT call login() — we want to inspect the login screen's password
		// input type attribute directly. Explicitly clear any cached cookies.
		await page.context().clearCookies();
		await page.goto('/');
		const passwordInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Password"))')
			.locator('input')
			.first();
		await expect(passwordInput).toBeVisible({ timeout: 10000 });
		await expect(passwordInput).toHaveAttribute('type', 'password');
	});

	test('contact form email field renders as type="email" (new input_type exercise)', async ({
		page,
	}) => {
		await login(page);
		await openEditContactForm(page);

		const emailInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Email"))')
			.locator('input')
			.first();
		await expect(emailInput).toBeVisible();
		await expect(emailInput).toHaveAttribute('type', 'email');
	});

	test('country-select node-patch preserves focus after FieldSet migration (Phase 12 D-A6 regression)', async ({
		page,
	}) => {
		await login(page);
		await openNewContactForm(page);

		// Type into the Name field and park the cursor inside the value
		// so the D-A6 focus-preservation assertion has signal to verify.
		const nameInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Name"))')
			.locator('input')
			.first();
		await nameInput.click();
		await nameInput.fill('Alice');
		await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			if (el && 'setSelectionRange' in el) el.setSelectionRange(3, 3);
		});

		// Dispatch the country-change action directly via the test hook to
		// avoid Select-trigger refocusing (same pattern as
		// node-patch-focus.spec.ts). The handler now targets the
		// `organisation-set` FieldSet at index 2 after the Plan 08 migration.
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
			hook(
				'contact_country_change',
				{ contactForm: { country: 'CH' } },
				'contact-form-country',
			);
		});

		// New Canton field must appear inside the organisation-set FieldSet
		// (SetNode + InsertChild against parent="organisation-set").
		await expect(
			page.locator('div[data-slot="field"]:has(label:has-text("Canton"))'),
		).toBeVisible({ timeout: 5000 });

		// D-A6 assertion: Name input retained focus, value, and cursor
		// position across the sibling node-patch on a different parent.
		const after = await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			if (!el || el.tagName !== 'INPUT') {
				return { focused: false, value: null, cursor: null };
			}
			return {
				focused: true,
				value: el.value,
				cursor: el.selectionStart,
			};
		});
		expect(after.focused).toBe(true);
		expect(after.value).toBe('Alice');
		expect(after.cursor).toBe(3);
	});

	test('textarea and switch primitives render inside the Notes and preferences FieldSet (D-E3, D-E4)', async ({
		page,
	}) => {
		await login(page);
		await openEditContactForm(page);

		// Notes — Textarea primitive, full_width=true, should render a
		// native <textarea> element inside a Field.Field wrapper.
		const notesField = page.locator(
			'div[data-slot="field"]:has(label:has-text("Notes"))',
		);
		await expect(notesField).toBeVisible();
		await expect(notesField.locator('textarea').first()).toBeVisible();

		// Switch — the opt-in control uses bits-ui's `role="switch"`.
		const optInSwitch = page.getByRole('switch', {
			name: /receive marketing emails/i,
		});
		await expect(optInSwitch).toBeVisible();
	});
});
