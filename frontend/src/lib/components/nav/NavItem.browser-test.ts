import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import NavItem from './NavItem.svelte';

vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

vi.mock('$lib/store/data.svelte', () => ({
	getData: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders button with label text', async () => {
	const screen = await render(NavItem, {
		props: { props: { label: 'Dashboard' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Dashboard')).toBeVisible();
});

test('dispatches action on click', async () => {
	const screen = await render(NavItem, {
		props: { props: { label: 'Settings', href: '/settings' }, surface: 'test' },
	});

	await screen.getByText('Settings').click();

	expect(sendAction).toHaveBeenCalledWith('navigate', { path: '/settings' });
});

test('renders icon when provided', async () => {
	const screen = await render(NavItem, {
		props: { props: { label: 'Search', icon: 'search' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Search')).toBeVisible();
	// Icon should render as an SVG element before the label
	const svg = screen.baseElement.querySelector('svg');
	expect(svg).toBeTruthy();
});
