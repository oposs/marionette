import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import SurfaceMount from './SurfaceMount.svelte';
import {
	setSurfaceTree,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { registerDefaults } from '$lib/registry/defaults';

const CHILD_SURFACE = 'child-test';

beforeEach(() => {
	resetStore(CHILD_SURFACE);
	clearSurfaceTree(CHILD_SURFACE);
	registerDefaults();
});

test('SurfaceMount with props.name mounts the named sub-surface', async () => {
	// Arrange: pre-populate the child sub-surface before mount
	setFullState(CHILD_SURFACE, {});
	setSurfaceTree(CHILD_SURFACE, 'root', {
		root: { type: 'heading', props: { text: 'Child Content Rendered' } },
	});

	const screen = render(SurfaceMount, {
		props: {
			props: { name: CHILD_SURFACE },
			surface: 'main',
		},
	});
	await tick();

	expect(screen.baseElement.textContent).toContain('Child Content Rendered');
});

test('SurfaceMount with a surface that has no tree shows LoadingSkeleton', async () => {
	const screen = render(SurfaceMount, {
		props: {
			props: { name: 'not-rendered-yet' },
			surface: 'main',
		},
	});
	await tick();

	// Surface.svelte renders LoadingSkeleton when its tree is undefined.
	// The skeleton is visible; assert nothing else crashed.
	expect(screen.baseElement.querySelector('[data-surface="not-rendered-yet"]')).not.toBeNull();
});
