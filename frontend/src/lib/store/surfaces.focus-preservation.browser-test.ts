import { render } from 'vitest-browser-svelte';
import { expect, test, beforeEach } from 'vitest';
import { tick } from 'svelte';
import Surface from '$lib/components/core/Surface.svelte';
import {
	setSurfaceTree,
	setNode,
	clearSurfaceTree,
} from '$lib/store/surfaces.svelte';
import { setFullState, resetStore } from '$lib/store/data.svelte';
import { registerDefaults } from '$lib/registry/defaults';

const SURFACE = 'fptest';

beforeEach(() => {
	resetStore(SURFACE);
	clearSurfaceTree(SURFACE);
	registerDefaults();
});

test('setNode on sibling preserves focus and cursor on focused input', async () => {
	// Arrange: surface with a container holding two text inputs
	setFullState(SURFACE, { a: '', b: '' });
	setSurfaceTree(SURFACE, 'root', {
		root: { type: 'container', children: ['field-a', 'field-b'] },
		'field-a': { type: 'text-input', bind: '/a', props: { label: 'A' } },
		'field-b': { type: 'text-input', bind: '/b', props: { label: 'B' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	// Locate the two inputs by their labels' rendered DOM order
	const inputs = screen.baseElement.querySelectorAll('input');
	expect(inputs.length).toBeGreaterThanOrEqual(2);
	const inputA = inputs[0] as HTMLInputElement;

	// Focus field-a, type "hello", move cursor to position 3
	inputA.focus();
	inputA.value = 'hello';
	inputA.dispatchEvent(new Event('input', { bubbles: true }));
	inputA.setSelectionRange(3, 3);

	expect(document.activeElement).toBe(inputA);
	expect(inputA.selectionStart).toBe(3);

	// Act: patch field-b to change its label (NOT touching field-a)
	setNode(SURFACE, 'field-b', {
		type: 'text-input',
		bind: '/b',
		props: { label: 'B (changed)' },
	});
	await tick();

	// Assert: field-a retains focus and cursor exactly where the user left it
	expect(document.activeElement).toBe(inputA);
	expect(inputA.selectionStart).toBe(3);
	expect(inputA.selectionEnd).toBe(3);
	expect(inputA.value).toBe('hello');

	// Sanity: field-b's new label is visible in the DOM
	const allLabels = Array.from(screen.baseElement.querySelectorAll('label')).map((l) =>
		l.textContent?.trim()
	);
	expect(allLabels.some((l) => l?.includes('B (changed)'))).toBe(true);
});

test('setNode on focused node does replace it (not a focus-preservation guarantee)', async () => {
	// This is the negative control per RESEARCH Pitfall 5 — we do NOT claim
	// that patching the focused node preserves focus. Document explicitly.
	setFullState(SURFACE, { a: '' });
	setSurfaceTree(SURFACE, 'root', {
		root: { type: 'container', children: ['only-field'] },
		'only-field': { type: 'text-input', bind: '/a', props: { label: 'Only' } },
	});

	const screen = await render(Surface, { props: { name: SURFACE } });
	await tick();

	const inputOnly = screen.baseElement.querySelector('input') as HTMLInputElement;
	inputOnly.focus();
	expect(document.activeElement).toBe(inputOnly);

	setNode(SURFACE, 'only-field', {
		type: 'text-input',
		bind: '/a',
		props: { label: 'Changed' },
	});
	await tick();

	// D-A6 does NOT promise focus preservation when the focused node itself
	// is replaced — only sibling patches. Instead of a no-op assertion, this
	// test documents that the patch was actually applied by asserting the
	// new label appears in the DOM. Focus state after the replacement is
	// explicitly unspecified and not asserted here.
	const label = screen.baseElement.querySelector('label');
	expect(label?.textContent).toContain('Changed');
});
