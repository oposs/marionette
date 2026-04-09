import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import Heading from './Heading.svelte';

vi.mock('$lib/store/data.svelte', () => ({
	getData: vi.fn(),
}));

beforeEach(() => {
	vi.clearAllMocks();
});

test('renders correct heading level tag', async () => {
	const screen = await render(Heading, {
		props: { props: { text: 'Title', level: 1 }, surface: 'test' },
	});

	const h1 = screen.baseElement.querySelector('h1');
	expect(h1).toBeTruthy();
	expect(h1!.textContent).toBe('Title');
});

test('applies font-semibold class', async () => {
	const screen = await render(Heading, {
		props: { props: { text: 'Subtitle', level: 3 }, surface: 'test' },
	});

	const h3 = screen.baseElement.querySelector('h3');
	expect(h3).toBeTruthy();
	expect(h3!.classList.contains('font-semibold')).toBe(true);
});
