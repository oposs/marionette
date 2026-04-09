import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';

vi.mock('$lib/transport/websocket.svelte', () => ({
	isConnected: vi.fn(() => false),
}));

import { isConnected } from '$lib/transport/websocket.svelte';
import ConnectionBanner from './ConnectionBanner.svelte';

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(isConnected).mockReturnValue(false);
});

test('shows banner text when disconnected', async () => {
	const screen = await render(ConnectionBanner);

	await expect.element(screen.getByText('Connection lost. Reconnecting...')).toBeVisible();
});

test('uses destructive background styling', async () => {
	const screen = await render(ConnectionBanner);

	const banner = screen.baseElement.querySelector('.bg-destructive');
	expect(banner).toBeTruthy();
});
