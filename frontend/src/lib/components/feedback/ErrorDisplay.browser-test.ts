import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';

vi.mock('$lib/store/data.svelte', () => ({
	getData: vi.fn(),
}));

import { getData } from '$lib/store/data.svelte';
import ErrorDisplay from './ErrorDisplay.svelte';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders error messages when bound data has errors', async () => {
	vi.mocked(getData).mockReturnValue([
		{ message: 'Name is required' },
		{ message: 'Email is invalid' },
	]);

	const screen = await render(ErrorDisplay, {
		props: { props: {}, bind: '/errors', surface: 'test' },
	});

	await expect.element(screen.getByText('Name is required')).toBeVisible();
	await expect.element(screen.getByText('Email is invalid')).toBeVisible();
});

test('renders AlertCircle icon', async () => {
	vi.mocked(getData).mockReturnValue([{ message: 'Something went wrong' }]);

	const screen = await render(ErrorDisplay, {
		props: { props: {}, bind: '/errors', surface: 'test' },
	});

	const svg = screen.baseElement.querySelector('svg');
	expect(svg).toBeTruthy();
});
