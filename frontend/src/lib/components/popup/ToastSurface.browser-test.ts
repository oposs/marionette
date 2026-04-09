import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach, afterEach } from 'vitest';
import ToastSurface from './ToastSurface.svelte';

beforeEach(() => {
	vi.useFakeTimers();
});

afterEach(() => {
	vi.useRealTimers();
});

test('renders empty when no toasts', async () => {
	const screen = await render(ToastSurface);

	// Container exists but has no toast cards
	const cards = screen.baseElement.querySelectorAll('[aria-label="Dismiss"]');
	expect(cards.length).toBe(0);
});

test('renders toast after addToast', async () => {
	const result = await render(ToastSurface);
	const component = result.component as unknown as { addToast: (event: { name: string; hint?: Record<string, unknown> }) => void };

	component.addToast({
		name: 'test-toast',
		hint: { message: 'Hello World', severity: 'success', duration: 5000 },
	});

	// Wait for reactivity
	await vi.advanceTimersByTimeAsync(50);

	await expect.element(result.getByText('Hello World')).toBeVisible();
});

test('removes toast on dismiss click', async () => {
	// Use real timers for this test since Svelte transitions use requestAnimationFrame
	vi.useRealTimers();

	const result = await render(ToastSurface);
	const component = result.component as unknown as { addToast: (event: { name: string; hint?: Record<string, unknown> }) => void };

	component.addToast({
		name: 'test-toast',
		hint: { message: 'Dismiss me', severity: 'info', duration: 60000 },
	});

	// Wait for reactivity and intro transition
	await new Promise((r) => setTimeout(r, 300));

	await expect.element(result.getByText('Dismiss me')).toBeVisible();

	// Click dismiss button
	await result.getByLabelText('Dismiss').click();

	// Wait for outro transition to complete (fly duration 200ms + buffer)
	await new Promise((r) => setTimeout(r, 400));

	// The toast text should no longer be in the DOM
	expect(result.baseElement.textContent).not.toContain('Dismiss me');
});
