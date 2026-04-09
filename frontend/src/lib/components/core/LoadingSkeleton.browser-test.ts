import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import LoadingSkeleton from './LoadingSkeleton.svelte';

test('renders skeleton elements without bg-gray classes', async () => {
	const screen = await render(LoadingSkeleton, {
		props: { lines: 3 },
	});

	// Should not contain any bg-gray classes (Pitfall 6 fix)
	const html = screen.baseElement.innerHTML;
	expect(html).not.toContain('bg-gray');

	// Should render skeleton elements
	const skeletons = screen.baseElement.querySelectorAll('[data-slot="skeleton"]');
	expect(skeletons.length).toBe(3);
});

test('renders correct number of lines', async () => {
	const screen = await render(LoadingSkeleton, {
		props: { lines: 5 },
	});

	const skeletons = screen.baseElement.querySelectorAll('[data-slot="skeleton"]');
	expect(skeletons.length).toBe(5);
});
