import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './tests/e2e',
	fullyParallel: true,
	retries: 0,
	use: {
		baseURL: 'http://localhost:3001',
		trace: 'on-first-retry',
	},
	projects: [
		{ name: 'chromium', use: { ...devices['Desktop Chrome'] } },
	],
	expect: {
		toHaveScreenshot: { maxDiffPixels: 100 },
	},
	snapshotDir: './tests/__snapshots__',
	webServer: {
		command: 'cd .. && make build && cd backend && cargo run -p crm-demo',
		port: 3001,
		reuseExistingServer: !process.env.CI,
		timeout: 120000, // cargo build can be slow first time
	},
});
