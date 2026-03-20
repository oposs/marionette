import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import { register, clearRegistry } from '$lib/registry/registry';
import NodeRenderer from '$lib/components/core/NodeRenderer.svelte';
import SideNav from './SideNav.svelte';
import NavItem from './NavItem.svelte';
import NavGroup from './NavGroup.svelte';
import type { ComponentNode } from '$lib/transport/messages';

// Mock sendAction so we can verify navigate dispatches
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	clearRegistry();
	vi.clearAllMocks();
});

test('sidebar renders with nav items as children', async () => {
	register('side-nav', SideNav);
	register('nav-item', NavItem);
	register('nav-group', NavGroup);

	const nodes: Record<string, ComponentNode> = {
		root: { type: 'side-nav', children: ['group1'] },
		group1: { type: 'nav-group', children: ['item1', 'item2'] },
		item1: { type: 'nav-item', props: { label: 'Dashboard', href: '/dashboard' } },
		item2: { type: 'nav-item', props: { label: 'Contacts', href: '/contacts' } },
	};

	const screen = await render(NodeRenderer, {
		props: { nodeId: 'root', nodes, surface: 'test' },
	});

	await expect.element(screen.getByText('Dashboard')).toBeVisible();
	await expect.element(screen.getByText('Contacts')).toBeVisible();
});

test('nav item click dispatches navigate action', async () => {
	register('side-nav', SideNav);
	register('nav-item', NavItem);

	const nodes: Record<string, ComponentNode> = {
		root: { type: 'side-nav', children: ['item1'] },
		item1: { type: 'nav-item', props: { label: 'Settings', href: '/settings' } },
	};

	const screen = await render(NodeRenderer, {
		props: { nodeId: 'root', nodes, surface: 'test' },
	});

	await screen.getByText('Settings').click();

	expect(sendAction).toHaveBeenCalledWith('navigate', { path: '/settings' });
});
