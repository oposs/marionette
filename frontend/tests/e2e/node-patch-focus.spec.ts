import { test, expect, type Page } from '@playwright/test';

// -----------------------------------------------------------------------------
// Phase 12 Plan 08 — country-select node-patch + focus preservation (D-A6)
// and D-B15 toast lifecycle end-to-end tests.
//
// These tests drive a real WebSocket session against the crm-demo backend
// on :3001. They prove three things:
//   1. Sibling node-patch operations (SetNode / InsertChild / DeleteNode /
//      RemoveChild) mutate the contact form tree in place WITHOUT tearing
//      down a focused, mid-edit text input. The Name field retains focus,
//      cursor position, and value across the country-change flow.
//   2. Switching country twice (CH → US) swaps Canton out and State in
//      via the same node-patch pipeline — different node IDs, same
//      guarantee.
//   3. D-B15 toast lifecycle: a country change inserts a dismissable
//      toast into the `toasts` sub-surface root; clicking the toast
//      triggers `dismiss_toast` which removes it via delete-node.
// -----------------------------------------------------------------------------

// Log in as the seeded admin (seed.rs defaults: admin@localhost / admin).
// Both the login form and the CRM use TextInputs without explicit label/input
// association, so we locate inputs by their parent grid wrapper that holds
// the label text.
async function login(page: Page): Promise<void> {
	await page.goto('/');
	// The login form renders `<div class="grid..."><Label>Email</Label><Input></div>`
	// with no `for` attribute, so `getByLabel` is unreliable; locate via the
	// wrapper div that contains the label text.
	//
	// NB: `TextInput.svelte` reads `props.type` for the input type, while the
	// backend builder serializes `input_type` from its `input_type` field, so
	// the login password field actually renders as `type="text"` — a
	// pre-existing bug outside Plan 08's scope. Both inputs therefore match
	// `role="textbox"`, so we locate the password input via its wrapper.
	const emailInput = page
		.locator('div.grid:has(label:has-text("Email"))')
		.getByRole('textbox');
	const passwordInput = page
		.locator('div.grid:has(label:has-text("Password"))')
		.getByRole('textbox');
	await emailInput.fill('admin@localhost');
	await passwordInput.fill('admin');
	await page.getByRole('button', { name: /log in/i }).click();
	// Landing screen is the Contact Management table.
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
}

// Navigate into the "New Contact" form from the contact list screen.
async function openNewContactForm(page: Page): Promise<void> {
	await page.getByRole('button', { name: 'New Contact' }).click();
	// The form's heading becomes "New Contact" once rendered.
	await expect(page.getByRole('heading', { name: 'New Contact' })).toBeVisible({
		timeout: 5000,
	});
}

// Locate the form's Name input by its parent grid wrapper.
function nameInputLocator(page: Page) {
	return page
		.locator('div.grid:has(label:has-text("Name"))')
		.getByRole('textbox')
		.first();
}

// Click the shadcn Select trigger for the given label, then click the
// option whose visible text matches `optionLabel`. shadcn Select is NOT
// a native <select> — Playwright's selectOption does not apply.
async function selectShadcnOption(
	page: Page,
	labelText: string,
	optionLabel: string,
): Promise<void> {
	// Select wrapper: div.grid gap-2 > Label > Select.Trigger (button)
	const trigger = page
		.locator(`div.grid:has(label:has-text("${labelText}"))`)
		.locator('[data-slot="select-trigger"]')
		.first();
	await trigger.click();
	// shadcn Select.Content renders the options as `[data-slot="select-item"]`
	// inside a Portal — match by visible text.
	await page.getByRole('option', { name: optionLabel }).click();
}

test.describe('Phase 12: node patch + focus preservation end-to-end', () => {
	test('country-select change swaps sibling fields and preserves focus on Name', async ({
		page,
	}) => {
		await login(page);
		await openNewContactForm(page);

		// Type into the Name field and put the cursor at position 3.
		const nameField = nameInputLocator(page);
		await nameField.click();
		await nameField.fill('Hello');

		// Move cursor to index 3. Playwright doesn't expose selectionStart
		// directly; drive it through the DOM on the focused element.
		await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			if (el && 'setSelectionRange' in el) {
				el.setSelectionRange(3, 3);
			}
		});

		// Sanity: the focused element is the Name input at cursor 3.
		const before = await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			return {
				value: el?.value ?? null,
				cursor: el?.selectionStart ?? null,
				tag: el?.tagName ?? null,
			};
		});
		expect(before.tag).toBe('INPUT');
		expect(before.value).toBe('Hello');
		expect(before.cursor).toBe(3);

		// D-A6 canonical proof: trigger the server-side patch WITHOUT
		// moving keyboard focus off the Name input. Clicking the shadcn
		// Select trigger would naturally re-focus the trigger button
		// (native browser behavior — unrelated to the patch itself), so
		// we instead dispatch the `contact_country_change` action
		// directly via the E2E test hook exposed from `init.ts`. This
		// isolates the test's focus-preservation assertion to the patch
		// application step only, which is exactly what D-A6 promises.
		await page.evaluate(() => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const hook = (window as any).__mrnSendAction as
				| ((
						name: string,
						payload?: Record<string, unknown>,
						source?: string,
				  ) => void)
				| undefined;
			if (!hook) {
				throw new Error('__mrnSendAction test hook not exposed');
			}
			hook(
				'contact_country_change',
				{
					contactForm: {
						country: 'CH',
					},
				},
				'contact-form-country',
			);
		});

		// The new Canton field appears via SetNode + InsertChild.
		await expect(
			page.locator('div.grid:has(label:has-text("Canton"))'),
		).toBeVisible({ timeout: 5000 });

		// CRITICAL D-A6 ASSERTION: Name still focused, cursor at 3,
		// value unchanged — sibling patching did not tear the input.
		const after = await page.evaluate(() => {
			const el = document.activeElement as HTMLInputElement | null;
			if (!el || el.tagName !== 'INPUT') {
				return { focused: false, value: null, cursor: null, tag: el?.tagName ?? null };
			}
			return {
				focused: true,
				value: el.value,
				cursor: el.selectionStart,
				tag: el.tagName,
			};
		});
		expect(after.focused).toBe(true);
		expect(after.value).toBe('Hello');
		expect(after.cursor).toBe(3);
	});

	test('switching country from Switzerland to United States swaps Canton -> State', async ({
		page,
	}) => {
		await login(page);
		await openNewContactForm(page);

		await selectShadcnOption(page, 'Country', 'Switzerland');
		await expect(
			page.locator('div.grid:has(label:has-text("Canton"))'),
		).toBeVisible({ timeout: 5000 });

		await selectShadcnOption(page, 'Country', 'United States');
		await expect(
			page.locator('div.grid:has(label:has-text("State"))'),
		).toBeVisible({ timeout: 5000 });
		// Canton must be gone (the RemoveChild + DeleteNode ops ran).
		await expect(
			page.locator('div.grid:has(label:has-text("Canton"))'),
		).toHaveCount(0);
	});

	test('D-B15 toast lifecycle: country change inserts a toast; clicking dismisses it', async ({
		page,
	}) => {
		await login(page);
		await openNewContactForm(page);

		// Trigger the country change. Backend emits TWO PatchMessages:
		//   1. content: mixed ops swapping Canton in
		//   2. toasts: InsertChild + SetNode inserting a dismissable toast
		await selectShadcnOption(page, 'Country', 'Switzerland');

		// D-B15 insert-child proven: the toast node renders its label
		// "Country set to Switzerland" inside the toasts sub-surface.
		const toast = page.getByRole('button', { name: 'Country set to Switzerland' });
		await expect(toast).toBeVisible({ timeout: 5000 });

		// Clicking the toast dispatches `dismiss_toast`, which emits a
		// toasts-surface PatchMessage with RemoveChild + DeleteNode ops.
		await toast.click();

		// D-B15 delete-node proven: the toast disappears from the DOM.
		await expect(toast).toHaveCount(0, { timeout: 5000 });
	});
});
