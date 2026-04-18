import { test, expect, type Page } from '@playwright/test';

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — Interaction form end-to-end coverage.
//
// Drives the real crm-demo backend (via playwright.e2e.config.ts) through the
// Phase 15 Plan 04 migrated interaction form and asserts:
//   1. FieldSet legend "Interaction" renders (D-A1 + D-C1).
//   2. RadioGroup "Type" replaces the Phase 14 Select — 3 options stacked
//      vertically, NO per-option descriptions (D-E1 + 15-UI-SPEC §RadioGroup
//      Production Contract).
//   3. RadioGroup state migration: clicking each option toggles data-state.
//   4. The date field carries the locked description string
//      "Format: YYYY-MM-DD HH:MM (24-hour)." (D-E3).
//   5. The notes field is a Textarea with full_width=true — its Field.Field
//      wrapper carries `col-span-full` (15-UI-SPEC §Textarea full_width
//      Contract).
//   6. Action row exposes "Cancel" + "Save interaction" buttons.
//   7. Per-field validation fires on empty Subject (D-D1), no form-level
//      banner.
//
// Note: the interaction form is opened via the `interaction_form` action
// with a `{contact_id}` payload (the form creates a new interaction; the
// heading reads "Log Interaction" per the Plan 04 decision to match the
// existing in-page "Log Interaction" button vocabulary).
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
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 10000 });
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

// Label-scoped field locator — component builders' `.id(...)` sets the
// adjacency-list id, not the HTML id attribute on the rendered input.
function labelField(page: Page, label: string) {
	return page
		.locator(`div[data-slot="field"]:has(label:has-text("${label}"))`)
		.locator('input')
		.first();
}

test.describe('Interaction form (Phase 15 D-A1 + D-D1 + D-E1 + D-E3)', () => {
	test('renders FieldSet "Interaction" legend, date description, and action row (D-A1, D-C1, D-D1, D-E3)', async ({
		page,
	}) => {
		await login(page);
		await openInteractionForm(page);

		// FieldSet legend — the only "Interaction" text rendered as the
		// FieldSet legend inside <fieldset data-slot="field-set">.
		const legend = page
			.locator('[data-slot="field-set"] legend')
			.filter({ hasText: 'Interaction' });
		await expect(legend).toBeVisible();

		// Locked Field.Description on the Date field.
		await expect(page.getByText('Format: YYYY-MM-DD HH:MM (24-hour).')).toBeVisible();

		// Action row: Save interaction + Cancel.
		await expect(page.getByRole('button', { name: 'Save interaction' })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();
	});

	test('RadioGroup "Type" (not Select) shows 3 options, no per-option descriptions (D-E1)', async ({
		page,
	}) => {
		await login(page);
		await openInteractionForm(page);

		// RadioGroup items (role="radio"). There are 3 on this screen
		// (call/email/meeting). This is the ONLY RadioGroup on the
		// interaction form — the Type radio replaces the Phase 14 Select.
		const radioItems = page.getByRole('radio');
		await expect(radioItems).toHaveCount(3);

		// Anti-pattern: if Type were still a Select, this page would contain
		// a <button role="combobox"> for the type dropdown. Assert no such
		// combobox exists (beyond any pre-existing unrelated ones — scope to
		// the form). There should be no combobox with accessible name "Type".
		await expect(
			page.getByRole('combobox', { name: /type/i }),
		).toHaveCount(0);

		// No per-option descriptions for interaction type — Call/Email/Meeting
		// labels are self-explanatory per 15-UI-SPEC §RadioGroup Production
		// Contract. Ensure the user-form-only locked description strings are
		// NOT present on this screen.
		await expect(page.getByText('Receive updates by email.')).toHaveCount(0);
		await expect(page.getByText('Text messages to your phone.')).toHaveCount(0);
		await expect(page.getByText('A human will call you.')).toHaveCount(0);
	});

	test('RadioGroup Type data-state migrates on click (D-E1)', async ({ page }) => {
		await login(page);
		await openInteractionForm(page);

		const radioItems = page.getByRole('radio');
		await expect(radioItems).toHaveCount(3);

		// Form defaults to "call" (index 0) per handle_interaction_form's
		// form_data.
		await expect(radioItems.nth(0)).toHaveAttribute('data-state', 'checked');

		// Click Email option (index 1 — options order [call, email, meeting]).
		await radioItems.nth(1).click();
		await expect(radioItems.nth(1)).toHaveAttribute('data-state', 'checked');
		await expect(radioItems.nth(0)).toHaveAttribute('data-state', 'unchecked');

		// Click Meeting option (index 2).
		await radioItems.nth(2).click();
		await expect(radioItems.nth(2)).toHaveAttribute('data-state', 'checked');
		await expect(radioItems.nth(1)).toHaveAttribute('data-state', 'unchecked');
	});

	test('Textarea notes field has full_width (col-span-full) on desktop (15-UI-SPEC §Textarea)', async ({
		page,
	}) => {
		await login(page);
		await openInteractionForm(page);

		// The notes Textarea is scoped by its label. The Field.Field wrapper
		// carries `col-span-full` when full_width=true.
		const notesWrapperInfo = await page.evaluate(() => {
			const labels = Array.from(document.querySelectorAll('label'));
			const notesLabel = labels.find(
				(l) => l.textContent?.trim() === 'Notes',
			);
			if (!notesLabel) return null;
			const wrapper = notesLabel.closest(
				'[data-slot="field"]',
			) as HTMLElement | null;
			if (!wrapper) return null;
			return {
				class: wrapper.className,
				gridColumn: window.getComputedStyle(wrapper).gridColumn,
			};
		});
		expect(notesWrapperInfo).not.toBeNull();
		// Either the raw class contains col-span-full, or the computed
		// grid-column is "1 / -1" (Tailwind's col-span-full expansion).
		const classOrComputed =
			(notesWrapperInfo!.class ?? '').includes('col-span-full') ||
			/1\s*\/\s*-1/.test(notesWrapperInfo!.gridColumn ?? '');
		expect(classOrComputed).toBe(true);
	});

	test('per-field validation fires on empty Subject (D-D1, D-D4)', async ({ page }) => {
		await login(page);
		await openInteractionForm(page);

		// Subject is required; clear and submit.
		const subjectInput = labelField(page, 'Subject');
		await subjectInput.fill('');
		await page.getByRole('button', { name: 'Save interaction' }).click();

		const error = page
			.locator('[data-slot="field-error"]')
			.filter({ hasText: /required/i });
		await expect(error).toBeVisible({ timeout: 5000 });

		// D-D4: no form-level banner.
		await expect(page.locator('.bg-destructive\\/10')).toHaveCount(0);
	});
});
