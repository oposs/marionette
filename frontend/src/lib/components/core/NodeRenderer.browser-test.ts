import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import NodeRenderer from './NodeRenderer.svelte';
import { register, clearRegistry } from '$lib/registry/registry';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import type { ComponentNode } from '$lib/transport/messages';

// Import real registered components for testing
import Heading from '$lib/components/layout/Heading.svelte';
import Text from '$lib/components/layout/Text.svelte';
import Container from '$lib/components/layout/Container.svelte';

beforeEach(() => {
	clearRegistry();
	resetStore('test');
});

test('renders a single node from adjacency list', async () => {
	register('heading', Heading);

	const nodes: Record<string, ComponentNode> = {
		root: { type: 'heading', props: { text: 'Hello World', level: 1 } },
	};

	const screen = await render(NodeRenderer, {
		props: { nodeId: 'root', nodes, surface: 'test' },
	});

	await expect.element(screen.getByText('Hello World')).toBeVisible();
});

test('renders nested children recursively', async () => {
	register('container', Container);
	register('text', Text);

	const nodes: Record<string, ComponentNode> = {
		root: { type: 'container', children: ['child1', 'child2'] },
		child1: { type: 'text', props: { text: 'First child' } },
		child2: { type: 'text', props: { text: 'Second child' } },
	};

	const screen = await render(NodeRenderer, {
		props: { nodeId: 'root', nodes, surface: 'test' },
	});

	await expect.element(screen.getByText('First child')).toBeVisible();
	await expect.element(screen.getByText('Second child')).toBeVisible();
});

test('unknown type shows fallback in dev mode', async () => {
	// Do not register 'unknown-widget' -- should trigger FallbackComponent
	const nodes: Record<string, ComponentNode> = {
		root: { type: 'unknown-widget', props: { foo: 'bar' } },
	};

	const screen = await render(NodeRenderer, {
		props: { nodeId: 'root', nodes, surface: 'test' },
	});

	await expect.element(screen.getByText('Unknown component: unknown-widget')).toBeVisible();
});

test('respects visible binding (hidden when false)', async () => {
	register('text', Text);
	setFullState('test', { show: false });

	const nodes: Record<string, ComponentNode> = {
		root: { type: 'text', props: { text: 'Conditional' }, visible: '/show' },
	};

	const screen = await render(NodeRenderer, {
		props: { nodeId: 'root', nodes, surface: 'test' },
	});

	// When visible path resolves to false, the node should not be rendered
	const el = screen.baseElement.querySelector('[data-surface]') ?? screen.baseElement;
	expect(el.textContent).not.toContain('Conditional');
});
