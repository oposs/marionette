import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import DataTableActions from './DataTableActions.svelte';

// Mock the dispatcher so tests can assert the outgoing sendAction call.
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders DropdownMenu trigger even for empty items', async () => {
	const screen = await render(DataTableActions, { props: { items: [] } });
	await expect.element(screen.getByLabelText('Row actions')).toBeVisible();
});

test('renders one menu item per action after trigger click', async () => {
	const items = [
		{ label: 'Edit', action: { type: 'click', name: 'contact_edit', payload: { id: 42 } } },
		{ label: 'Delete', action: { type: 'click', name: 'contact_delete', payload: { id: 42 } } },
	];
	const screen = await render(DataTableActions, { props: { items } });
	await screen.getByLabelText('Row actions').click();
	await expect.element(screen.getByText('Edit')).toBeVisible();
	await expect.element(screen.getByText('Delete')).toBeVisible();
});

test('dispatches sendAction on item click with name + payload + target', async () => {
	const items = [
		{
			label: 'Delete',
			action: {
				type: 'click',
				name: 'contact_delete',
				payload: { contact_id: 7 },
				target: 'modal',
			},
		},
	];
	const screen = await render(DataTableActions, { props: { items } });
	await screen.getByLabelText('Row actions').click();
	await screen.getByText('Delete').click();
	expect(sendAction).toHaveBeenCalledWith('contact_delete', { contact_id: 7 }, 'modal');
});

test('falls back to action.type when action.name is missing', async () => {
	const items = [{ label: 'Raw', action: { type: 'custom_action' } }];
	const screen = await render(DataTableActions, { props: { items } });
	await screen.getByLabelText('Row actions').click();
	await screen.getByText('Raw').click();
	expect(sendAction).toHaveBeenCalledWith('custom_action', undefined, undefined);
});

test('escapes malicious labels via text interpolation (XSS mitigation)', async () => {
	const evil = '<script>window.__pwned = true</script>';
	const items = [{ label: evil, action: { type: 'click', name: 'noop' } }];
	const screen = await render(DataTableActions, { props: { items } });
	await screen.getByLabelText('Row actions').click();

	// The literal text must appear, escaped — getByText matches the raw string.
	await expect.element(screen.getByText(evil, { exact: true })).toBeVisible();

	// No injected <script> element should have executed. Walk every <script>
	// currently in the DOM and make sure none of them contain the payload.
	const scripts = document.querySelectorAll('script');
	for (const s of Array.from(scripts)) {
		expect(s.textContent ?? '').not.toContain('__pwned');
	}
	// And the global pollution must not have happened.
	expect((window as unknown as { __pwned?: boolean }).__pwned).toBeUndefined();
});
