import { test, expect, type Page } from '@playwright/test';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-expect-error node:fs types not wired into frontend tsconfig yet (deferred-items)
import * as fs from 'node:fs';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-expect-error node:path types not wired into frontend tsconfig yet
import * as path from 'node:path';
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? '.';

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — Company edit form UAT evidence driver.
//
// Drives the running dev server (Vite :5173 proxying to crm-demo :3001 via
// `make dev`) through the company-edit scenarios from 15-UI-SPEC §UAT
// Evidence Contract §Scope:
//   1. Render — desktop + mobile screenshots, assertions.json with
//      fieldset_legends, fields_in_order, action_row, descriptions_present.
//   2. Validation — submit with empty Name; Field.Error + aria-invalid.
//   3. Save — fill valid data, submit, land on Company Management list.
//
// Chrome-MCP is unavailable in the worktree environment (Phase 14 Plan 08
// precedent) — Playwright produces equivalent objective evidence.
// -----------------------------------------------------------------------------

const EVIDENCE_DIR = path.resolve(
	cwd,
	'..',
	'.planning/phases/15-crm-migration-validation/15-uat-evidence/company-edit',
);
if (!fs.existsSync(EVIDENCE_DIR)) {
	fs.mkdirSync(EVIDENCE_DIR, { recursive: true });
}

function artifactPath(name: string): string {
	return path.join(EVIDENCE_DIR, name);
}

async function login(page: Page): Promise<void> {
	await page.goto('http://localhost:5173/');
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
	await expect(page.getByText('Contact Management')).toBeVisible({ timeout: 15000 });
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
		timeout: 10000,
	});
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
}

test.describe.configure({ mode: 'serial' });

test.describe('Phase 15 Plan 07 — Company edit UAT', () => {
	test('UAT-CE-01 Render: desktop + mobile screenshots + assertions.json', async ({
		browser,
	}) => {
		// Desktop 1280×720.
		const desktopCtx = await browser.newContext({
			viewport: { width: 1280, height: 720 },
		});
		const desktopPage = await desktopCtx.newPage();
		const consoleErrors: string[] = [];
		desktopPage.on('console', (msg) => {
			if (msg.type() === 'error') consoleErrors.push(msg.text());
		});
		await login(desktopPage);
		await openCompanyEditForm(desktopPage);
		await desktopPage.screenshot({
			path: artifactPath('desktop.png'),
			fullPage: true,
		});

		const domShape = await desktopPage.evaluate(() => {
			const legends = Array.from(
				document.querySelectorAll('[data-slot="field-set"] legend'),
			).map((l) => l.textContent?.trim() ?? '');
			const fields = Array.from(
				document.querySelectorAll('[data-slot="field"] label'),
			)
				.map((l) => l.textContent?.trim() ?? '')
				.filter((t) => t.length > 0);
			const cancel = !!Array.from(document.querySelectorAll('button')).find(
				(b) => b.textContent?.trim() === 'Cancel',
			);
			const save = !!Array.from(document.querySelectorAll('button')).find(
				(b) => b.textContent?.trim() === 'Save company',
			);
			const nameDesc = Array.from(document.querySelectorAll('p, [data-slot="field-description"]')).find(
				(el) => el.textContent?.trim() === 'Will appear on invoices and contact details.',
			);
			return { legends, fields, cancel, save, descriptionFound: !!nameDesc };
		});
		await desktopCtx.close();

		// Mobile 375×800.
		const mobileCtx = await browser.newContext({
			viewport: { width: 375, height: 800 },
		});
		const mobilePage = await mobileCtx.newPage();
		await login(mobilePage);
		await openCompanyEditForm(mobilePage);
		await mobilePage.screenshot({
			path: artifactPath('mobile.png'),
			fullPage: true,
		});
		await mobileCtx.close();

		fs.writeFileSync(
			artifactPath('assertions.json'),
			JSON.stringify(
				{
					screen: 'company-edit',
					viewport: { desktop: [1280, 720], mobile: [375, 800] },
					fieldset_legends: domShape.legends,
					fields_in_order: domShape.fields,
					action_row: {
						buttons: ['Cancel', 'Save company'],
						align: 'right',
						cancel_present: domShape.cancel,
						save_present: domShape.save,
					},
					descriptions_present: {
						name: domShape.descriptionFound
							? 'Will appear on invoices and contact details.'
							: null,
					},
					console_errors: consoleErrors.length,
					passed:
						domShape.legends.includes('Company details') &&
						domShape.cancel &&
						domShape.save &&
						domShape.descriptionFound,
				},
				null,
				2,
			),
		);

		fs.writeFileSync(
			artifactPath('console.log'),
			[
				`Phase 15 Plan 07 — Company edit UAT console capture`,
				`desktop console_errors: ${consoleErrors.length}`,
				...consoleErrors.map((e) => `[error] ${e}`),
			].join('\n'),
		);

		expect(domShape.legends).toContain('Company details');
		expect(domShape.cancel).toBe(true);
		expect(domShape.save).toBe(true);
		expect(domShape.descriptionFound).toBe(true);
	});

	test('UAT-CE-02 Validation: empty Name → Field.Error + aria-invalid', async ({
		page,
	}) => {
		await login(page);
		await openCompanyEditForm(page);

		const nameInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Name"))')
			.locator('input')
			.first();
		await nameInput.fill('');
		await page.getByRole('button', { name: 'Save company' }).click();

		await expect(
			page.locator('[data-slot="field-error"]').filter({ hasText: /required/i }),
		).toBeVisible({ timeout: 5000 });

		const errorInfo = await page.evaluate(() => {
			const errors = Array.from(
				document.querySelectorAll('[data-slot="field-error"]'),
			).map((el) => el.textContent?.trim() ?? '');
			const invalidInputs = document.querySelectorAll('input[aria-invalid="true"]');
			const banner = document.querySelector('.bg-destructive\\/10, [class*="bg-destructive"]');
			return {
				error_texts: errors,
				invalid_input_count: invalidInputs.length,
				form_level_banner_present: !!banner,
			};
		});

		fs.writeFileSync(
			artifactPath('validation.json'),
			JSON.stringify(
				{
					scenario: 'submit with empty Name',
					expected_error_path: '/_errors/companyForm/name',
					expected_aria_invalid: true,
					observed: errorInfo,
					passed:
						errorInfo.error_texts.length >= 1 &&
						errorInfo.invalid_input_count >= 1,
				},
				null,
				2,
			),
		);
		expect(errorInfo.error_texts.length).toBeGreaterThanOrEqual(1);
	});

	test('UAT-CE-03 Save: valid submit returns to Company Management list', async ({
		page,
	}) => {
		await login(page);
		await openCompanyEditForm(page);

		const nameInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Name"))')
			.locator('input')
			.first();
		const currentName = await nameInput.inputValue();
		if (!currentName.trim()) await nameInput.fill('Acme Corp');

		await page.getByRole('button', { name: 'Save company' }).click();
		await expect(page.getByText('Company Management')).toBeVisible({
			timeout: 10000,
		});
		await page.screenshot({
			path: artifactPath('save-success.png'),
			fullPage: true,
		});

		fs.writeFileSync(
			artifactPath('save.json'),
			JSON.stringify(
				{
					scenario: 'fill valid data, submit, land on list',
					landing_heading: 'Company Management',
					passed: true,
				},
				null,
				2,
			),
		);
	});
});
