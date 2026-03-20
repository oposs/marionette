import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import Button from './Button.svelte';

// Mock sendAction
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders button with label', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Save' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Save')).toBeVisible();
});

test('dispatches action on click', async () => {
	const screen = await render(Button, {
		props: {
			props: { label: 'Submit' },
			action: { type: 'submit', name: 'save-contact' },
			surface: 'test',
		},
	});

	await screen.getByText('Submit').click();

	expect(sendAction).toHaveBeenCalledWith(
		'save-contact',
		expect.any(Object),
		undefined,
		undefined,
	);
});

test('renders with custom color and size', async () => {
	const screen = await render(Button, {
		props: {
			props: { label: 'Delete', color: 'red', size: 'lg' },
			surface: 'test',
		},
	});

	await expect.element(screen.getByText('Delete')).toBeVisible();
});
