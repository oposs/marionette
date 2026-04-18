import { defineConfig, devices } from '@playwright/test';

// Standalone Playwright config for the Phase 14 Plan 08 UAT driver. Does NOT
// spawn its own webServer — assumes `make dev` is already running on
// :5173 (Vite) + :3001 (crm-demo backend). Run from frontend/:
//
//   npx playwright test \
//     --config ../.planning/phases/14-formscreen-enhancements/14-uat-evidence/playwright.uat.config.ts \
//     --reporter=line
export default defineConfig({
	testDir: './',
	fullyParallel: false,
	workers: 1,
	retries: 0,
	use: {
		baseURL: 'http://localhost:5173',
		trace: 'retain-on-failure',
	},
	projects: [
		{ name: 'chromium', use: { ...devices['Desktop Chrome'] } },
	],
	expect: {
		toHaveScreenshot: { maxDiffPixels: 100 },
	},
});
