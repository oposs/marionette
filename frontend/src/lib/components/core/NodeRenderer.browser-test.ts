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
import TextInput from '$lib/components/form/TextInput.svelte';

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

// -----------------------------------------------------------------------------
// D-E2 unmount-race regression (Plan 14-01, Task 3)
//
// When a parent patch removes a bound leaf node (e.g., TextInput) while its
// handleBlur is still in-flight, the Svelte compiler's generated accessor for
// `bind={node.bind}` can read `node.bind` on a now-undefined `node` —
// `TypeError: Cannot read properties of undefined (reading 'bind')`.
//
// Caught by ErrorBoundary so the user sees nothing, but it spams the console
// and leaks tree-structure info on every blur-during-patch event.
//
// Fix (per 14-RESEARCH.md Pitfall 2, 14-CONTEXT.md D-E2 preference b): destructure
// node.props/bind/action via `{@const}` *inside* the `{#if node}` branch so the
// generated accessor reads a local const, never the live object.
// -----------------------------------------------------------------------------

describe_unmount_race: {
	test('does not throw TypeError when a bound node is removed while rendered', async () => {
		register('text-input', TextInput);

		const capturedErrors: ErrorEvent[] = [];
		const errorHandler = (e: ErrorEvent) => capturedErrors.push(e);
		window.addEventListener('error', errorHandler);

		try {
			const initial: Record<string, ComponentNode> = {
				root: { type: 'text-input', props: { label: 'X' }, bind: '/x' },
			};

			const screen = await render(NodeRenderer, {
				props: { nodeId: 'root', nodes: initial, surface: 'test' },
			});

			// Mutate the nodes map: remove the bound node. This simulates a
			// set-children / delete-node patch arriving while the renderer still
			// holds the old nodes reference (which is exactly what happens when a
			// parent re-renders while a child's handleBlur is in-flight).
			const patched: Record<string, ComponentNode> = {};

			await screen.rerender({ nodeId: 'root', nodes: patched, surface: 'test' });

			// Give any queued microtasks / error events a chance to fire.
			await new Promise((r) => setTimeout(r, 50));

			const typeErrors = capturedErrors.filter((e) =>
				/Cannot read properties of undefined \(reading 'bind'\)/.test(e.message ?? '')
			);
			expect(typeErrors.length).toBe(0);
		} finally {
			window.removeEventListener('error', errorHandler);
		}
	});

	test('moved destructure into guarded branch (structural contract)', async () => {
		// The behavioral test above is necessary but not sufficient — a future
		// refactor could reintroduce the race in a way that ErrorBoundary catches
		// before our handler fires. This structural assertion pins the fix: the
		// destructure must live inside {#if node} via {@const}.
		const src = await import('./NodeRenderer.svelte?raw').then((m) => m.default as string);
		expect(src).toMatch(/\{@const\s+nodeBind\s*=\s*node\.bind\s*\}/);
		expect(src).toMatch(/\{@const\s+nodeProps\s*=\s*node\.props\s*\?\?\s*\{\}\s*\}/);
		expect(src).toMatch(/\{@const\s+nodeAction\s*=\s*node\.action\s*\}/);
	});
}
