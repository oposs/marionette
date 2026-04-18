import { test, expect, type Page } from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? '.';

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — User edit form UAT evidence driver.
//
// Captures the user-edit scenarios from 15-UI-SPEC §UAT Evidence Contract:
//   1. Render — desktop + mobile screenshots, assertions.json recording
//      2 FieldSet legends, RadioGroup options + per-option descriptions.
//   2. Validation — empty Name → Field.Error.
//   3. Save — fill valid data, submit, land on user list.
//   4. RadioGroup click-through — data-state migration.
// -----------------------------------------------------------------------------

const EVIDENCE_DIR = path.resolve(
	cwd,
	'..',
	'.planning/phases/15-crm-migration-validation/15-uat-evidence/user-edit',
);
if (!fs.existsSync(EVIDENCE_DIR)) {
	fs.mkdirSync(EVIDENCE_DIR, { recursive: true });
}
const artifactPath = (name: string) => path.join(EVIDENCE_DIR, name);

async function login(page: Page): Promise<void> {
	await page.goto('http://localhost:5173/');
	await page
		.locator('div[data-slot="field"]:has(label:has-text("Email"))')
		.locator('input')
		.first()
		.fill('admin@localhost');
	await page
		.locator('div[data-slot="field"]:has(label:has-text("Password"))')
		.locator('input')
		.first()
		.fill('admin');
	await page.getByRole('button', { name: /log in/i }).click();
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 15000 });
}

async function openUserEditForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((name: string, payload?: Record<string, unknown>, source?: string) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('user_edit', { user_id: 1 }, 'user-edit-1');
	});
	await expect(page.getByRole('heading', { name: 'Edit User' })).toBeVisible({
		timeout: 10000,
	});
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
}

test.describe.configure({ mode: 'serial' });

test.describe('Phase 15 Plan 07 — User edit UAT', () => {
	test('UAT-UE-01 Render: desktop + mobile screenshots + assertions.json', async ({
		browser,
	}) => {
		const desktopCtx = await browser.newContext({
			viewport: { width: 1280, height: 720 },
		});
		const desktopPage = await desktopCtx.newPage();
		const consoleErrors: string[] = [];
		desktopPage.on('console', (msg) => {
			if (msg.type() === 'error') consoleErrors.push(msg.text());
		});
		await login(desktopPage);
		await openUserEditForm(desktopPage);
		await desktopPage.screenshot({ path: artifactPath('desktop.png'), fullPage: true });

		const domShape = await desktopPage.evaluate(() => {
			const legends = Array.from(
				document.querySelectorAll('[data-slot="field-set"] legend'),
			).map((l) => l.textContent?.trim() ?? '');
			const separators = document.querySelectorAll(
				'[data-slot="field-separator"], hr, [data-slot="separator"]',
			).length;
			const fields = Array.from(
				document.querySelectorAll('[data-slot="field"] label'),
			)
				.map((l) => l.textContent?.trim() ?? '')
				.filter((t) => t.length > 0);
			const radioItems = document.querySelectorAll('[role="radio"]');
			const radioStates = Array.from(radioItems).map((r) =>
				r.getAttribute('data-state'),
			);
			const emailDescFound = Array.from(document.querySelectorAll('p, [data-slot="field-description"]')).find(
				(el) => el.textContent?.trim() === 'Used for password resets and notifications.',
			);
			const optionDescs = Array.from(document.querySelectorAll('body *'))
				.map((el) => el.textContent?.trim())
				.filter((t): t is string =>
					!!t &&
					['Receive updates by email.', 'Text messages to your phone.', 'A human will call you.'].includes(t),
				);
			return {
				legends,
				separators,
				fields,
				radio_items: radioItems.length,
				radio_states: radioStates,
				email_description_found: !!emailDescFound,
				per_option_descriptions_found: Array.from(new Set(optionDescs)),
			};
		});
		await desktopCtx.close();

		const mobileCtx = await browser.newContext({ viewport: { width: 375, height: 800 } });
		const mobilePage = await mobileCtx.newPage();
		await login(mobilePage);
		await openUserEditForm(mobilePage);
		await mobilePage.screenshot({ path: artifactPath('mobile.png'), fullPage: true });
		await mobileCtx.close();

		fs.writeFileSync(
			artifactPath('assertions.json'),
			JSON.stringify(
				{
					screen: 'user-edit',
					viewport: { desktop: [1280, 720], mobile: [375, 800] },
					fieldset_legends: domShape.legends,
					field_separator_count: domShape.separators,
					fields_in_order: domShape.fields,
					radio_group: {
						option_count: domShape.radio_items,
						data_states: domShape.radio_states,
						per_option_descriptions: domShape.per_option_descriptions_found,
					},
					descriptions_present: {
						email: domShape.email_description_found
							? 'Used for password resets and notifications.'
							: null,
					},
					console_errors: consoleErrors.length,
					passed:
						domShape.legends.includes('Account') &&
						domShape.legends.includes('Permissions') &&
						domShape.radio_items >= 3 &&
						domShape.email_description_found &&
						domShape.per_option_descriptions_found.length === 3,
				},
				null,
				2,
			),
		);

		fs.writeFileSync(
			artifactPath('console.log'),
			[
				'Phase 15 Plan 07 — User edit UAT console capture',
				`desktop console_errors: ${consoleErrors.length}`,
				...consoleErrors.map((e) => `[error] ${e}`),
			].join('\n'),
		);

		expect(domShape.legends).toEqual(
			expect.arrayContaining(['Account', 'Permissions']),
		);
		expect(domShape.radio_items).toBeGreaterThanOrEqual(3);
		expect(domShape.email_description_found).toBe(true);
	});

	test('UAT-UE-02 Validation: empty Name → Field.Error', async ({ page }) => {
		await login(page);
		await openUserEditForm(page);
		const nameInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Name"))')
			.locator('input')
			.first();
		await nameInput.fill('');
		await page.getByRole('button', { name: 'Save user' }).click();
		await expect(
			page.locator('[data-slot="field-error"]').filter({ hasText: /required/i }),
		).toBeVisible({ timeout: 5000 });
		const errorTexts = await page
			.locator('[data-slot="field-error"]')
			.allTextContents();
		fs.writeFileSync(
			artifactPath('validation.json'),
			JSON.stringify(
				{
					scenario: 'submit with empty Name',
					expected_error_path: '/_errors/userForm/name',
					observed_errors: errorTexts,
					passed: errorTexts.some((t) => /required/i.test(t)),
				},
				null,
				2,
			),
		);
	});

	test('UAT-UE-03 Save: valid submit returns to user list', async ({ page }) => {
		await login(page);
		await openUserEditForm(page);
		// Name is pre-populated from seed. Just click save.
		await page.getByRole('button', { name: 'Save user' }).click();
		// Save handler re-renders User Management list.
		await expect(page.getByText(/User Management|Users/i)).toBeVisible({ timeout: 10000 });
		await page.screenshot({ path: artifactPath('save-success.png'), fullPage: true });
		fs.writeFileSync(
			artifactPath('save.json'),
			JSON.stringify({ scenario: 'save user returns to list', passed: true }, null, 2),
		);
	});

	test('UAT-UE-04 RadioGroup click-through: data-state migrates on click', async ({
		page,
	}) => {
		await login(page);
		await openUserEditForm(page);
		const radioItems = page.getByRole('radio');
		await expect(radioItems).toHaveCount(3);
		// Form defaults preferred_contact_method to "email" (index 0).
		await radioItems.nth(1).click();
		await expect(radioItems.nth(1)).toHaveAttribute('data-state', 'checked');
		const stateAfterSms = await radioItems.evaluateAll((els) =>
			els.map((e) => e.getAttribute('data-state')),
		);
		await radioItems.nth(2).click();
		await expect(radioItems.nth(2)).toHaveAttribute('data-state', 'checked');
		const stateAfterPhone = await radioItems.evaluateAll((els) =>
			els.map((e) => e.getAttribute('data-state')),
		);
		fs.writeFileSync(
			artifactPath('radiogroup.json'),
			JSON.stringify(
				{
					scenario: 'click each RadioGroup option',
					observed: { after_sms: stateAfterSms, after_phone: stateAfterPhone },
					passed:
						stateAfterSms[1] === 'checked' && stateAfterPhone[2] === 'checked',
				},
				null,
				2,
			),
		);
	});
});
