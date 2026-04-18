/*
 * Phase 13/14/15 CI guards — cheap filesystem assertions that run inside
 * the regular Playwright E2E suite. No browser navigation required; these
 * tests only check disk state (existsSync + git grep) so they catch
 * regressions cheaply.
 *
 * Purpose:
 *   Assert that files retired by Phase 13 (D-A2) and Phase 14 (D-A1) are
 *   not re-introduced by a future refactor, and that Flowbite vocabulary
 *   does not leak back into runtime paths (Phase 15 D-F1). If any of
 *   these conditions regress, this spec fails and surfaces the issue.
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
import { execSync } from 'node:child_process';

// Use import.meta.url to avoid __dirname which isn't typed without @types/node.
const FRONTEND_ROOT = resolve(fileURLToPath(import.meta.url), '..', '..', '..');
// Repo root is one level above the frontend package — used by the Flowbite
// residue grep below so git grep can scan backend/crates, spec/, and the
// top-level doc files.
const REPO_ROOT = resolve(FRONTEND_ROOT, '..');

test.describe('Phase 13/14/15 CI guards', () => {
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

	test('FormScreen.svelte is retired (Phase 14 D-A1)', () => {
		const p = resolve(
			FRONTEND_ROOT,
			'src/lib/components/screen/FormScreen.svelte',
		);
		expect(existsSync(p)).toBe(false);
	});

	test('FormScreen.browser-test.ts is retired (Phase 14 D-A1)', () => {
		const p = resolve(
			FRONTEND_ROOT,
			'src/lib/components/screen/FormScreen.browser-test.ts',
		);
		expect(existsSync(p)).toBe(false);
	});

	test('No Flowbite residue in runtime code (Phase 15 D-F1)', () => {
		// git grep -Iil 'flowbite' → case-insensitive match, text files only,
		// name-only output. Respects .gitignore (skips node_modules/, target/,
		// __snapshots__/, etc.). -I skips binary files automatically.
		let matches: string[] = [];
		try {
			const out = execSync(
				"git grep -Iil 'flowbite' -- " +
					"'frontend/src/**' " +
					"'backend/crates/**' " +
					"'spec/**' " +
					'CONCEPT.md ' +
					'TOOLING.md',
				{ cwd: REPO_ROOT, encoding: 'utf8' },
			);
			matches = out.trim().split('\n').filter(Boolean);
		} catch (e: unknown) {
			// git grep exits with code 1 when there are NO matches — that's the
			// success case for this test. Any other non-zero exit is a genuine
			// error.
			const err = e as { status?: number; stdout?: string };
			if (err.status === 1) {
				matches = [];
			} else {
				throw e;
			}
		}
		expect(
			matches,
			`Flowbite residue found in:\n${matches.join('\n')}`,
		).toHaveLength(0);
	});
});
