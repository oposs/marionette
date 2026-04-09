import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import NavGroup from './NavGroup.svelte';

test('renders group label when provided', async () => {
	const screen = await render(NavGroup, {
		props: { props: { label: 'Main Menu' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Main Menu')).toBeVisible();
});

test('renders without label when not provided', async () => {
	const screen = await render(NavGroup, {
		props: { props: {}, surface: 'test' },
	});

	// Container div should exist but no label paragraph
	const container = screen.baseElement.querySelector('.mt-2');
	expect(container).toBeTruthy();
	const label = container!.querySelector('p');
	expect(label).toBeFalsy();
});
