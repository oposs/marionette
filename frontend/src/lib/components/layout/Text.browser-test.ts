import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import Text from './Text.svelte';

vi.mock('$lib/store/data.svelte', () => ({
	getData: vi.fn(),
}));

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders paragraph text', async () => {
	const screen = await render(Text, {
		props: { props: { text: 'Hello world' }, surface: 'test' },
	});

	await expect.element(screen.getByText('Hello world')).toBeVisible();
	const p = screen.baseElement.querySelector('p');
	expect(p).toBeTruthy();
});

test('renders muted variant with muted-foreground class', async () => {
	const screen = await render(Text, {
		props: { props: { text: 'Muted text', muted: true }, surface: 'test' },
	});

	const p = screen.baseElement.querySelector('p');
	expect(p).toBeTruthy();
	expect(p!.classList.contains('text-muted-foreground')).toBe(true);
});
