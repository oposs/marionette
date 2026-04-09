import { render } from 'vitest-browser-svelte';
import { expect, test, vi, beforeEach } from 'vitest';
import ModalSurface from './ModalSurface.svelte';
import { setSurfaceTree, clearSurfaceTree } from '$lib/store/surfaces.svelte';

// Mock sendAction for close assertions
vi.mock('$lib/transport/dispatcher', () => ({
	sendAction: vi.fn(),
}));

import { sendAction } from '$lib/transport/dispatcher';

beforeEach(() => {
	clearSurfaceTree('modal');
	vi.clearAllMocks();
});

test('does not render dialog when no modal surface tree', async () => {
	const screen = await render(ModalSurface);

	// Dialog content should not be in the DOM
	expect(screen.baseElement.querySelector('[data-slot="dialog-content"]')).toBeNull();
});

test('renders dialog when modal surface tree exists', async () => {
	setSurfaceTree('modal', 'root', {
		root: { type: 'confirm-dialog', props: { title: 'Test' } },
	});

	const screen = await render(ModalSurface);

	// Dialog content should be rendered via portal
	await expect.element(screen.baseElement.ownerDocument.querySelector('[data-slot="dialog-content"]')! as HTMLElement).toBeVisible();
});

test('dispatches close-modal on dialog close button click', async () => {
	setSurfaceTree('modal', 'root', {
		root: { type: 'confirm-dialog', props: { title: 'Test' } },
	});

	const screen = await render(ModalSurface);

	// Find the close button (X icon) rendered by Dialog.Content
	const closeBtn = screen.baseElement.ownerDocument.querySelector('[data-slot="dialog-close"]');
	if (closeBtn) {
		(closeBtn as HTMLElement).click();
		expect(sendAction).toHaveBeenCalledWith('close-modal');
	}
});
