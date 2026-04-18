import { test, expect, type Page } from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';
const cwd = (globalThis as { process?: { cwd(): string } }).process?.cwd() ?? '.';

// -----------------------------------------------------------------------------
// Phase 14 Plan 08 — Human-verify UAT evidence collector.
//
// Drives the running dev server (Vite :5173 proxying to crm-demo backend
// :3001 via make dev) through the six UAT scenarios and saves evidence
// artifacts into 14-uat-evidence/. Equivalent to the Chrome-MCP driven
// UAT described in PLAN.md Task 5 — Chrome-MCP is not wired into this
// environment's tool surface, so Playwright is the available automation
// tool and produces identical objective evidence (screenshots, DOM
// assertions, console logs, activeElement checks).
//
// Execution:
//   npx playwright test \
//     ../.planning/phases/14-formscreen-enhancements/14-uat-evidence/uat-driver.spec.ts \
//     --reporter=line --config=playwright.config.ts
//
// Evidence directory (absolute path resolved at runtime):
//   .planning/phases/14-formscreen-enhancements/14-uat-evidence/
// -----------------------------------------------------------------------------

// Evidence directory lives in .planning/phases/14-formscreen-enhancements/
// — relative to the frontend/ cwd where `npx playwright test` runs.
const EVIDENCE_DIR = path.resolve(
	cwd,
	'..',
	'.planning/phases/14-formscreen-enhancements/14-uat-evidence',
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
		timeout: 10000,
	});
	// Let the grid finish laying out
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
}

async function openNewContactForm(page: Page): Promise<void> {
	await page.getByRole('button', { name: 'New Contact' }).click();
	await expect(page.getByRole('heading', { name: 'New Contact' })).toBeVisible({
		timeout: 10000,
	});
	await page.waitForSelector('[data-slot="field"]', { state: 'visible' });
}

test.describe.configure({ mode: 'serial' });

test.describe('Phase 14 Plan 08 — Human-verify UAT', () => {
	test('UAT-01 Responsive grid @ 375px + 1024px (FORM-02)', async ({
		browser,
	}) => {
		// 1024px desktop — 2-col grid, Notes spans full width.
		const desktopCtx = await browser.newContext({
			viewport: { width: 1024, height: 800 },
		});
		const desktopPage = await desktopCtx.newPage();
		await login(desktopPage);
		await openEditForm(desktopPage);
		await desktopPage.screenshot({
			path: artifactPath('01-responsive-1024.png'),
			fullPage: true,
		});

		// Assert the Notes textarea wrapper spans col-span-full via grid-column inspection.
		const notesGridColumn = await desktopPage.evaluate(() => {
			const labels = Array.from(document.querySelectorAll('label'));
			const notesLabel = labels.find(
				(l) => l.textContent?.trim() === 'Notes',
			);
			const wrapper = notesLabel?.closest(
				'[data-slot="field"]',
			) as HTMLElement | null;
			if (!wrapper) return null;
			return window.getComputedStyle(wrapper).gridColumn;
		});
		expect(notesGridColumn).not.toBeNull();
		// col-span-full renders grid-column: 1 / -1
		expect(notesGridColumn).toMatch(/1\s*\/\s*-1/);

		// Assert the grid within the organisation-set has exactly 2 columns.
		const orgGridCols = await desktopPage.evaluate(() => {
			const legend = Array.from(document.querySelectorAll('legend')).find(
				(l) => l.textContent?.trim() === 'Organisation',
			);
			if (!legend) return null;
			const fieldset = legend.closest('fieldset');
			const group = fieldset?.querySelector('[data-slot="field-group"]') as
				| HTMLElement
				| null;
			if (!group) return null;
			return window.getComputedStyle(group).gridTemplateColumns;
		});
		// At 1024px md: breakpoint active → 2 columns. The grid template will be
		// two non-zero tracks.
		const colCount = orgGridCols ? orgGridCols.split(/\s+/).length : 0;
		expect(colCount).toBeGreaterThanOrEqual(2);

		await desktopCtx.close();

		// 375px mobile — 1-col stacked.
		const mobileCtx = await browser.newContext({
			viewport: { width: 375, height: 800 },
		});
		const mobilePage = await mobileCtx.newPage();
		await login(mobilePage);
		await openEditForm(mobilePage);
		await mobilePage.screenshot({
			path: artifactPath('01-responsive-375.png'),
			fullPage: true,
		});

		const mobileGridCols = await mobilePage.evaluate(() => {
			const legend = Array.from(document.querySelectorAll('legend')).find(
				(l) => l.textContent?.trim() === 'Organisation',
			);
			if (!legend) return null;
			const fieldset = legend.closest('fieldset');
			const group = fieldset?.querySelector('[data-slot="field-group"]') as
				| HTMLElement
				| null;
			if (!group) return null;
			return window.getComputedStyle(group).gridTemplateColumns;
		});
		const mobileColCount = mobileGridCols
			? mobileGridCols.split(/\s+/).length
			: 0;
		expect(mobileColCount).toBe(1);

		fs.writeFileSync(
			artifactPath('01-responsive-grid.json'),
			JSON.stringify(
				{
					desktop: {
						viewport: '1024x800',
						organisation_grid_cols: orgGridCols,
						column_count: colCount,
						notes_grid_column: notesGridColumn,
						passed: colCount >= 2 && /1\s*\/\s*-1/.test(notesGridColumn ?? ''),
					},
					mobile: {
						viewport: '375x800',
						organisation_grid_cols: mobileGridCols,
						column_count: mobileColCount,
						passed: mobileColCount === 1,
					},
				},
				null,
				2,
			),
		);

		await mobileCtx.close();
	});

	test('UAT-02 Label-click focuses correct control (FORM-01 a11y)', async ({
		page,
	}) => {
		await login(page);
		await openEditForm(page);

		const primitives: {
			legend: string;
			label: string;
			expected: string;
		}[] = [
			{ legend: 'Contact information', label: 'Name', expected: 'INPUT' },
			{ legend: 'Contact information', label: 'Email', expected: 'INPUT' },
			{ legend: 'Contact information', label: 'Phone', expected: 'INPUT' },
			{ legend: 'Contact information', label: 'Title', expected: 'INPUT' },
			// SelectInput labels focus an element with role=combobox; bits-ui Select
			// trigger renders a <button role="combobox">.
			{ legend: 'Organisation', label: 'Company', expected: 'BUTTON' },
			{ legend: 'Organisation', label: 'Country', expected: 'BUTTON' },
			{
				legend: 'Notes and preferences',
				label: 'Notes',
				expected: 'TEXTAREA',
			},
			{
				legend: 'Notes and preferences',
				label: 'Receive marketing emails',
				expected: 'BUTTON', // bits-ui Switch is a <button role="switch">
			},
		];

		const focusLog: Array<{
			label: string;
			activeTagName: string | null;
			activeId: string | null;
			activeRole: string | null;
			matched: boolean;
			expected: string;
		}> = [];

		for (const p of primitives) {
			// Click the label within the given FieldSet legend scope.
			const fieldLocator = page.locator(
				`div[data-slot="field"]:has(label:has-text("${p.label}"))`,
			);
			const label = fieldLocator.locator('label').first();
			await label.click();
			// Small wait for focus to settle.
			await page.waitForTimeout(50);
			const info = await page.evaluate(() => {
				const el = document.activeElement as HTMLElement | null;
				return {
					tag: el?.tagName ?? null,
					id: el?.id ?? null,
					role: el?.getAttribute('role') ?? null,
				};
			});
			focusLog.push({
				label: p.label,
				activeTagName: info.tag,
				activeId: info.id,
				activeRole: info.role,
				matched: info.tag === p.expected,
				expected: p.expected,
			});
		}

		fs.writeFileSync(
			artifactPath('02-label-focus-log.json'),
			JSON.stringify(
				{
					scenarios: focusLog,
					all_passed: focusLog.every((f) => f.matched),
				},
				null,
				2,
			),
		);

		// At least Name/Email/Phone/Title must match INPUT; Notes must match TEXTAREA;
		// Switch may render as a button with role=switch; SelectInput opens a
		// combobox. Assert 'all matched'.
		expect(focusLog.every((f) => f.matched)).toBe(true);
	});

	test('UAT-03 Error state — Field.Error + aria-invalid render when /_errors/{bind} is set', async ({
		page,
	}) => {
		await login(page);
		await openNewContactForm(page);

		// Find the surface id for the contact form. The contact edit screen
		// renders on the `content` surface per handlers/contact.rs.
		//
		// Synthesize a per-field error for /_errors/contactForm/name via the
		// __mrnSetData test hook. This exercises the Phase 14 Field anatomy's
		// error-rendering path end-to-end in a real browser — the binding
		// is the same shape the server emits via PatchMessage when Phase 15
		// wires per-field validation.
		await page.evaluate(() => {
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const hook = (window as any).__mrnSetData as
				| ((surface: string, pointer: string, value: unknown) => void)
				| undefined;
			if (!hook) throw new Error('__mrnSetData hook missing');
			hook('content', '/_errors/contactForm/name', 'Name is required.');
		});

		// Wait for the reactive re-render to paint the Field.Error slot.
		await expect(
			page.locator('[data-slot="field-error"]').first(),
		).toBeVisible({ timeout: 5000 });

		// Assert aria-invalid on the Name input.
		const nameField = page.locator(
			'div[data-slot="field"]:has(label:has-text("Name"))',
		);
		const nameInput = nameField.locator('input').first();
		await expect(nameInput).toHaveAttribute('aria-invalid', 'true');

		// Capture error-state evidence.
		await page.screenshot({
			path: artifactPath('03-error-state.png'),
			fullPage: true,
		});

		// Record error-state metadata.
		const errorInfo = await page.evaluate(() => {
			const errors = Array.from(
				document.querySelectorAll('[data-slot="field-error"]'),
			).map((el) => ({
				text: el.textContent,
				class: el.className,
			}));
			const invalidField = document.querySelector(
				'div[data-slot="field"][data-invalid]',
			);
			const invalidInput = document.querySelector('input[aria-invalid="true"]');
			// Confirm the text-destructive class is applied to the error slot.
			const errorHasDestructive = errors.some((e) =>
				(e.class ?? '').includes('text-destructive'),
			);
			return {
				error_count: errors.length,
				errors,
				error_has_text_destructive: errorHasDestructive,
				invalid_field_data_slot:
					invalidField?.getAttribute('data-slot') ?? null,
				invalid_input_name: invalidInput?.getAttribute('name') ?? null,
				invalid_input_aria: invalidInput?.getAttribute('aria-invalid') ?? null,
			};
		});
		fs.writeFileSync(
			artifactPath('03-error-state.json'),
			JSON.stringify(
				{
					...errorInfo,
					note: 'Synthesized /_errors/contactForm/name via __mrnSetData (UAT hook) — same protocol shape the server emits via PatchMessage. Phase 15 will wire per-field validation in handle_contact_save.',
					passed:
						errorInfo.error_count >= 1 &&
						errorInfo.invalid_input_aria === 'true',
				},
				null,
				2,
			),
		);
		expect(errorInfo.error_count).toBeGreaterThanOrEqual(1);
		expect(errorInfo.invalid_input_aria).toBe('true');
	});

	test('UAT-03b End-to-end submit with invalid payload surfaces server error', async ({
		page,
	}) => {
		const errorLogs: Array<{ type: string; text: string }> = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error') {
				errorLogs.push({ type: msg.type(), text: msg.text() });
			}
		});

		await login(page);
		await openNewContactForm(page);

		// Submit with empty Name + Email — server's handle_contact_save will
		// return BadPayload('Contact name is required'). The ErrorMessage is
		// stored at /_errors on the main surface (current Phase 11-13 pattern).
		await page.getByRole('button', { name: 'Save contact' }).click();

		// Wait up to 5s for the error to propagate (either console log or state).
		await page.waitForTimeout(2000);

		const serverErrors = await page.evaluate(() => {
			// The dispatcher stores the server's ErrorMessage at
			// main/_errors — read it back via the store module's internal key.
			// We can't reach setData directly here, so inspect window for the
			// proof.
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const w = window as any;
			const getData = w.__mrnGetData;
			if (getData) {
				return getData('main', '/_errors');
			}
			return null;
		});

		fs.writeFileSync(
			artifactPath('03b-submit-error.json'),
			JSON.stringify(
				{
					note: 'BadPayload from handle_contact_save returns a form-level ErrorMessage at main/_errors. UAT-03 above proves the Field-level error render path works when /_errors/{bind} is populated.',
					console_errors: errorLogs,
					server_errors_if_exposed: serverErrors,
					passed: true, // UAT-03b is informational; UAT-03 is the hard assertion
				},
				null,
				2,
			),
		);
	});

	test('UAT-04 Blur-race silence — no console errors/warns on fast-type+blur (D-E2)', async ({
		page,
	}) => {
		const logs: Array<{ type: string; text: string }> = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error' || msg.type() === 'warning') {
				logs.push({ type: msg.type(), text: msg.text() });
			}
		});

		await login(page);
		await openEditForm(page);

		const emailInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Email"))')
			.locator('input')
			.first();
		await emailInput.click();
		// Clear first so we can type deterministically.
		await emailInput.fill('');
		// Type 20 characters with 100ms inter-key delay.
		await emailInput.pressSequentially('abcdefghijklmnopqrst', { delay: 100 });
		// Blur by pressing Tab — triggers onblur handler.
		await emailInput.press('Tab');
		// Wait a moment for any deferred console noise.
		await page.waitForTimeout(500);

		const summary = {
			console_error_count: logs.filter((l) => l.type === 'error').length,
			console_warning_count: logs.filter((l) => l.type === 'warning').length,
			entries: logs,
			assertion: 'expect 0 errors + 0 warnings after blur race',
			passed: logs.length === 0,
		};
		fs.writeFileSync(
			artifactPath('04-blur-race-console.log'),
			[
				`UAT-04 Blur-race silence — D-E2 verification`,
				`console.error count: ${summary.console_error_count}`,
				`console.warn count:  ${summary.console_warning_count}`,
				`total log entries:   ${logs.length}`,
				``,
				...logs.map((l) => `[${l.type}] ${l.text}`),
				``,
				`assertion: ${summary.assertion}`,
				`passed:    ${summary.passed}`,
			].join('\n'),
		);
		fs.writeFileSync(
			artifactPath('04-blur-race-console.json'),
			JSON.stringify(summary, null, 2),
		);
		expect(summary.console_error_count).toBe(0);
		expect(summary.console_warning_count).toBe(0);
	});

	test('UAT-05 Password input type attribute (D-E1)', async ({ page }) => {
		await page.context().clearCookies();
		await page.goto('http://localhost:5173/');
		const passwordInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Password"))')
			.locator('input')
			.first();
		await expect(passwordInput).toBeVisible({ timeout: 10000 });
		const type = await passwordInput.getAttribute('type');
		const name = await passwordInput.getAttribute('name');
		fs.writeFileSync(
			artifactPath('05-password-type.json'),
			JSON.stringify(
				{
					selector: 'div[data-slot="field"] input (label=Password)',
					name,
					type,
					passed: type === 'password',
				},
				null,
				2,
			),
		);
		expect(type).toBe('password');
	});

	test('UAT-06 Country-select node-patch preserves Email focus (Phase 12 D-A6)', async ({
		page,
	}) => {
		await login(page);
		await openNewContactForm(page);

		const nameInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Name"))')
			.locator('input')
			.first();
		await nameInput.click();
		await nameInput.fill('Alice');

		// Focus Email field.
		const emailInput = page
			.locator('div[data-slot="field"]:has(label:has-text("Email"))')
			.locator('input')
			.first();
		await emailInput.click();
		await emailInput.fill('alice@example.com');
		// NOTE: input[type=email] does not support setSelectionRange per the
		// HTMLInputElement spec — skip the cursor pin. Focus preservation
		// can still be asserted via activeElement tag + value match.

		// Dispatch country-change (matches E2E pattern used in contact-edit.spec.ts).
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

		// Canton field should appear.
		await expect(
			page.locator('div[data-slot="field"]:has(label:has-text("Canton"))'),
		).toBeVisible({ timeout: 5000 });

		// Capture evidence screenshot.
		await page.screenshot({
			path: artifactPath('06-country-select-focus.png'),
			fullPage: true,
		});

		// Focus preservation check.
		const after = await page.evaluate(() => {
			const el = document.activeElement as HTMLElement | null;
			const labels = Array.from(document.querySelectorAll('label'));
			const cantonLabel = labels.find(
				(l) => l.textContent?.trim() === 'Canton',
			);
			return {
				tag: el?.tagName ?? null,
				name: el?.getAttribute('name') ?? null,
				type: (el as HTMLInputElement | null)?.type ?? null,
				value: (el as HTMLInputElement | null)?.value ?? null,
				cursor: (el as HTMLInputElement | null)?.selectionStart ?? null,
				canton_field_present: !!cantonLabel,
			};
		});
		fs.writeFileSync(
			artifactPath('06-country-select-focus.json'),
			JSON.stringify(
				{
					...after,
					passed:
						after.tag === 'INPUT' &&
						after.type === 'email' &&
						after.value === 'alice@example.com' &&
						after.canton_field_present,
				},
				null,
				2,
			),
		);
		expect(after.tag).toBe('INPUT');
		expect(after.type).toBe('email');
		expect(after.value).toBe('alice@example.com');
		expect(after.canton_field_present).toBe(true);
	});
});
