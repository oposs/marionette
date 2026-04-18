import { test, expect, type Page } from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? '.';

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — Interaction form UAT evidence driver.
// -----------------------------------------------------------------------------

const EVIDENCE_DIR = path.resolve(
	cwd,
	'..',
	'.planning/phases/15-crm-migration-validation/15-uat-evidence/interaction-edit',
);
if (!fs.existsSync(EVIDENCE_DIR)) fs.mkdirSync(EVIDENCE_DIR, { recursive: true });
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

async function openInteractionForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((name: string, payload?: Record<string, unknown>, source?: string) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('interaction_form', { contact_id: 1 }, 'interaction-form-1');
	});
	await expect(page.getByRole('heading', { name: 'Log Interaction' })).toBeVisible({
		timeout: 10000,
	});
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
}

test.describe.configure({ mode: 'serial' });

test.describe('Phase 15 Plan 07 — Interaction edit UAT', () => {
	test('UAT-IE-01 Render: desktop + mobile screenshots + assertions.json', async ({
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
		await openInteractionForm(desktopPage);
		await desktopPage.screenshot({ path: artifactPath('desktop.png'), fullPage: true });

		const domShape = await desktopPage.evaluate(() => {
			const legends = Array.from(
				document.querySelectorAll('[data-slot="field-set"] legend'),
			).map((l) => l.textContent?.trim() ?? '');
			const radioItems = document.querySelectorAll('[role="radio"]');
			const radioStates = Array.from(radioItems).map((r) =>
				r.getAttribute('data-state'),
			);
			const notesLabel = Array.from(document.querySelectorAll('label')).find(
				(l) => l.textContent?.trim() === 'Notes',
			);
			const notesWrapper = notesLabel?.closest(
				'[data-slot="field"]',
			) as HTMLElement | null;
			const notesClass = notesWrapper?.className ?? '';
			const notesGridColumn = notesWrapper
				? window.getComputedStyle(notesWrapper).gridColumn
				: '';
			const dateDescFound = Array.from(document.querySelectorAll('p, [data-slot="field-description"]')).find(
				(el) => el.textContent?.trim() === 'Format: YYYY-MM-DD HH:MM (24-hour).',
			);
			// Type radio group should have no per-option descriptions (contrast
			// with user-form locked strings which would also live in the DOM if
			// this were the wrong form).
			const forbiddenOptionDescs = Array.from(document.querySelectorAll('body *'))
				.map((el) => el.textContent?.trim())
				.filter(
					(t): t is string =>
						!!t &&
						[
							'Receive updates by email.',
							'Text messages to your phone.',
							'A human will call you.',
						].includes(t),
				);
			return {
				legends,
				radio_items: radioItems.length,
				radio_states: radioStates,
				notes_class: notesClass,
				notes_grid_column: notesGridColumn,
				notes_full_width:
					notesClass.includes('col-span-full') || /1\s*\/\s*-1/.test(notesGridColumn),
				date_description_found: !!dateDescFound,
				forbidden_user_form_descs_present: forbiddenOptionDescs.length > 0,
			};
		});
		await desktopCtx.close();

		const mobileCtx = await browser.newContext({ viewport: { width: 375, height: 800 } });
		const mobilePage = await mobileCtx.newPage();
		await login(mobilePage);
		await openInteractionForm(mobilePage);
		await mobilePage.screenshot({ path: artifactPath('mobile.png'), fullPage: true });
		await mobileCtx.close();

		fs.writeFileSync(
			artifactPath('assertions.json'),
			JSON.stringify(
				{
					screen: 'interaction-edit',
					viewport: { desktop: [1280, 720], mobile: [375, 800] },
					fieldset_legends: domShape.legends,
					radio_group: {
						option_count: domShape.radio_items,
						data_states: domShape.radio_states,
						no_per_option_descriptions: !domShape.forbidden_user_form_descs_present,
					},
					textarea_full_width: domShape.notes_full_width,
					descriptions_present: {
						date: domShape.date_description_found
							? 'Format: YYYY-MM-DD HH:MM (24-hour).'
							: null,
					},
					console_errors: consoleErrors.length,
					passed:
						domShape.legends.includes('Interaction') &&
						domShape.radio_items === 3 &&
						domShape.notes_full_width &&
						domShape.date_description_found &&
						!domShape.forbidden_user_form_descs_present,
				},
				null,
				2,
			),
		);

		fs.writeFileSync(
			artifactPath('console.log'),
			[
				'Phase 15 Plan 07 — Interaction edit UAT console capture',
				`desktop console_errors: ${consoleErrors.length}`,
				...consoleErrors.map((e) => `[error] ${e}`),
			].join('\n'),
		);

		expect(domShape.legends).toContain('Interaction');
		expect(domShape.radio_items).toBe(3);
		expect(domShape.notes_full_width).toBe(true);
		expect(domShape.date_description_found).toBe(true);
	});

	test('UAT-IE-02 Validation: empty Subject → Field.Error', async ({ page }) => {
		await login(page);
		await openInteractionForm(page);
		const subjectInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Subject"))')
			.locator('input')
			.first();
		await subjectInput.fill('');
		await page.getByRole('button', { name: 'Save interaction' }).click();
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
					scenario: 'submit with empty Subject',
					expected_error_path: '/_errors/interactionForm/subject',
					observed_errors: errorTexts,
					passed: errorTexts.some((t) => /required/i.test(t)),
				},
				null,
				2,
			),
		);
	});

	test('UAT-IE-03 RadioGroup click-through: data-state migrates on click', async ({
		page,
	}) => {
		await login(page);
		await openInteractionForm(page);
		const radioItems = page.getByRole('radio');
		await expect(radioItems).toHaveCount(3);
		// Default "call" (index 0) → click email (1) → click meeting (2).
		await radioItems.nth(1).click();
		await expect(radioItems.nth(1)).toHaveAttribute('data-state', 'checked');
		await radioItems.nth(2).click();
		await expect(radioItems.nth(2)).toHaveAttribute('data-state', 'checked');
		const states = await radioItems.evaluateAll((els) =>
			els.map((e) => e.getAttribute('data-state')),
		);
		fs.writeFileSync(
			artifactPath('radiogroup.json'),
			JSON.stringify(
				{
					scenario: 'click each interaction type option',
					final_states: states,
					passed: states[2] === 'checked' && states[0] === 'unchecked',
				},
				null,
				2,
			),
		);
	});
});
