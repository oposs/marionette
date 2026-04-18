import { test, expect, type Page } from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? '.';

// -----------------------------------------------------------------------------
// Phase 15 Plan 07 — Inline contact tag-add UAT evidence driver.
//
// Captures the inline tag-add scenarios per 15-UI-SPEC §UAT Evidence
// Contract §Scope (smoke-level: render + submit validation).
// -----------------------------------------------------------------------------

const EVIDENCE_DIR = path.resolve(
	cwd,
	'..',
	'.planning/phases/15-crm-migration-validation/15-uat-evidence/contact-tag-add',
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

async function openEditContactForm(page: Page): Promise<void> {
	await page.evaluate(() => {
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const hook = (window as any).__mrnSendAction as
			| ((name: string, payload?: Record<string, unknown>, source?: string) => void)
			| undefined;
		if (!hook) throw new Error('__mrnSendAction hook missing');
		hook('contact_edit', { contact_id: 1 }, 'contact-edit-1');
	});
	await expect(page.getByRole('heading', { name: 'Edit Contact' })).toBeVisible({
		timeout: 10000,
	});
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
}

test.describe.configure({ mode: 'serial' });

test.describe('Phase 15 Plan 07 — Contact inline tag-add UAT', () => {
	test('UAT-TAG-01 Render + Submit: inline tag-add submits + renders tag', async ({
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
		await openEditContactForm(desktopPage);

		await desktopPage.screenshot({ path: artifactPath('desktop.png'), fullPage: true });

		const tagValue = `uat-t${Date.now()}`;
		const tagInput = desktopPage
			.locator('div[data-slot="field"]:has(label:has-text("Add tag"))')
			.locator('input')
			.first();
		await tagInput.fill(tagValue);
		await desktopPage.getByRole('button', { name: '+ Add tag' }).click();
		await expect(
			desktopPage.getByRole('button', { name: new RegExp(`^${tagValue}`) }),
		).toBeVisible({ timeout: 5000 });

		const domShape = await desktopPage.evaluate(() => {
			const addTagBtn = Array.from(document.querySelectorAll('button')).find(
				(b) => b.textContent?.trim() === '+ Add tag',
			);
			const wrapper = addTagBtn?.closest('form, [class*="flex"]');
			const wrapperClass = wrapper?.className ?? '';
			return {
				add_tag_button_present: !!addTagBtn,
				wrapper_flex_class_match: /flex.*gap-2.*items-end/.test(wrapperClass),
			};
		});
		await desktopCtx.close();

		const mobileCtx = await browser.newContext({ viewport: { width: 375, height: 800 } });
		const mobilePage = await mobileCtx.newPage();
		await login(mobilePage);
		await openEditContactForm(mobilePage);
		await mobilePage.screenshot({ path: artifactPath('mobile.png'), fullPage: true });
		await mobileCtx.close();

		fs.writeFileSync(
			artifactPath('assertions.json'),
			JSON.stringify(
				{
					screen: 'contact-tag-add',
					viewport: { desktop: [1280, 720], mobile: [375, 800] },
					button_label: '+ Add tag',
					submitted_tag: tagValue,
					observed: domShape,
					console_errors: consoleErrors.length,
					passed: domShape.add_tag_button_present,
				},
				null,
				2,
			),
		);

		fs.writeFileSync(
			artifactPath('console.log'),
			[
				'Phase 15 Plan 07 — Contact tag-add UAT console capture',
				`desktop console_errors: ${consoleErrors.length}`,
				...consoleErrors.map((e) => `[error] ${e}`),
			].join('\n'),
		);
	});

	test('UAT-TAG-02 Validation: empty submit emits /_errors/tagForm/name Field.Error', async ({
		page,
	}) => {
		await login(page);
		await openEditContactForm(page);
		const tagInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Add tag"))')
			.locator('input')
			.first();
		await tagInput.fill('');
		await page.getByRole('button', { name: '+ Add tag' }).click();
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
					scenario: 'empty tag submit',
					expected_error_path: '/_errors/tagForm/name',
					observed_errors: errorTexts,
					passed: errorTexts.some((t) => /required/i.test(t)),
				},
				null,
				2,
			),
		);
	});
});
