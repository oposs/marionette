/*
 * Phase 13 CI guards — cheap filesystem assertions that run inside the
 * regular Playwright E2E suite. No browser navigation required; these
 * tests only check disk state so they catch regressions cheaply.
 *
 * Purpose:
 *   Assert that files retired by Phase 13 (per D-A2) are not
 *   re-introduced by a future refactor. If any of these files come back,
 *   this spec fails and surfaces the regression.
 *
 * Note on types: svelte-check runs this file against the frontend
 * tsconfig which does NOT include `@types/node` (Node built-ins would
 * otherwise resolve under Playwright's own runtime). The existing
 * `tests/helpers/schema-validator.ts` file has the same limitation and
 * is logged in `.planning/phases/13-datatable-enhancements/deferred-
 * items.md`. We use `@ts-expect-error` suppressions matching that
 * pattern to keep svelte-check clean on new code.
 */

import { test, expect } from '@playwright/test';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Use import.meta.url to avoid __dirname which isn't typed without @types/node.
const FRONTEND_ROOT = resolve(fileURLToPath(import.meta.url), '..', '..', '..');

test.describe('Phase 13 CI guards', () => {
	test('TableScreen.svelte is retired (D-A2)', () => {
		const p = resolve(
			FRONTEND_ROOT,
			'src/lib/components/screen/TableScreen.svelte',
		);
		expect(existsSync(p)).toBe(false);
	});

	test('TableScreen.browser-test.ts is retired (D-A2)', () => {
		const p = resolve(
			FRONTEND_ROOT,
			'src/lib/components/screen/TableScreen.browser-test.ts',
		);
		expect(existsSync(p)).toBe(false);
	});
});
