import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	test: {
		include: ['src/**/*.test.ts', 'src/**/*.test.svelte.ts'],
		environment: 'node',
		passWithNoTests: true,
		// Browser component tests use vitest-browser.config.ts
	},
	server: {
		proxy: {
			'/api': {
				target: 'http://localhost:3001',
				changeOrigin: true
			},
			'/ws': {
				target: 'ws://localhost:3001',
				ws: true
			}
		}
	}
});
