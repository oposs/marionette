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

test('renders with label text', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Save' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Save')).toBeVisible();
});

test('renders destructive variant for color=red', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Delete', color: 'red' }, surface: 'test' },
	});

	const button = screen.getByRole('button');
	await expect.element(button).toBeVisible();
	// shadcn destructive variant applies destructive styling classes
	const el = button.element() as HTMLButtonElement;
	expect(el.className).toContain('destructive');
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

test('renders disabled state', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Save', disabled: true }, surface: 'test' },
	});

	const button = screen.getByRole('button');
	await expect.element(button).toBeDisabled();
});

test('renders icon when props.icon set', async () => {
	const screen = await render(Button, {
		props: { props: { label: 'Add', icon: 'plus' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Add')).toBeVisible();
	// Icon should render as SVG inside button
	const button = screen.getByRole('button').element() as HTMLButtonElement;
	expect(button.querySelector('svg')).toBeTruthy();
});
