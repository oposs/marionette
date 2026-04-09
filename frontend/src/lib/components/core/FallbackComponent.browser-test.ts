import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';
import FallbackComponent from './FallbackComponent.svelte';

test('renders component type name in dev mode', async () => {
	const screen = await render(FallbackComponent, {
		props: { nodeType: 'my-widget', props: { foo: 'bar' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Unknown component: my-widget')).toBeVisible();
});

test('uses destructive token classes instead of raw red', async () => {
	const screen = await render(FallbackComponent, {
		props: { nodeType: 'test-type', props: {}, surface: 'test' },
	});

	const html = screen.baseElement.innerHTML;
	expect(html).toContain('border-destructive');
	expect(html).toContain('bg-destructive/10');
	expect(html).toContain('text-destructive');
	expect(html).not.toContain('border-red');
	expect(html).not.toContain('bg-red');
	expect(html).not.toContain('text-red');
});
