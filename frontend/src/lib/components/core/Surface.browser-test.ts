import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import Surface from './Surface.svelte';
import { register, clearRegistry } from '$lib/registry/registry';
import { setSurfaceTree, clearSurfaceTree } from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import Text from '$lib/components/layout/Text.svelte';
import type { ComponentNode } from '$lib/transport/messages';

beforeEach(() => {
	clearRegistry();
	clearSurfaceTree('test-surface');
	resetStore('test-surface');
});

test('renders loading skeleton when no tree is set', async () => {
	const screen = await render(Surface, {
		props: { name: 'test-surface' },
	});

	// LoadingSkeleton renders shadcn Skeleton components
	const container = screen.baseElement.querySelector('[data-surface="test-surface"]');
	expect(container).toBeTruthy();
	// Should show skeleton elements when no tree set
	const skeletons = container!.querySelectorAll('[data-slot="skeleton"]');
	expect(skeletons.length).toBeGreaterThan(0);
});

test('renders component tree from surface state', async () => {
	register('text', Text);

	const nodes: Record<string, ComponentNode> = {
		root: { type: 'text', props: { text: 'Surface content' } },
	};
	setSurfaceTree('test-surface', 'root', nodes);
	setFullState('test-surface', {});

	const screen = await render(Surface, {
		props: { name: 'test-surface' },
	});

	await expect.element(screen.getByText('Surface content')).toBeVisible();
});

test('has data-surface attribute for testing and E2E', async () => {
	const screen = await render(Surface, {
		props: { name: 'my-surface' },
	});

	const el = screen.baseElement.querySelector('[data-surface="my-surface"]');
	expect(el).toBeTruthy();
});
